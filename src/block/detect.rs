use super::report::{line_count, region_pair};
use super::{BlockSource, BlockTokens};
use crate::config::CloneConfig;
use crate::error::{CloneError, Result};
use crate::fast_hash::{FastMap, FastSet, SeededBuildHasher};
use crate::fingerprint::rolling_hashes;
use crate::model::ClonePair;

pub(super) struct WindowDetection {
    pub pairs: Vec<ClonePair>,
    pub candidates: usize,
    pub suppressed_buckets: usize,
}

#[derive(Debug, Clone, Copy)]
struct Occurrence {
    source: u32,
    position: u32,
}

pub(super) fn detect_windows(
    sources: &[BlockSource],
    tokenized: &[BlockTokens],
    config: CloneConfig,
    min_lines: usize,
) -> Result<WindowDetection> {
    let capacity = tokenized
        .iter()
        .map(|tokens| tokens.strict.len().saturating_sub(config.min_tokens))
        .sum();
    let mut state = WindowState {
        store: FastMap::with_capacity_and_hasher(capacity, SeededBuildHasher::random()),
        seen: FastSet::with_hasher(SeededBuildHasher::random()),
        suppressed: FastSet::with_hasher(SeededBuildHasher::random()),
        pairs: Vec::new(),
        candidates: 0,
    };
    for source in 0..tokenized.len() {
        state.detect_source(source, tokenized, sources, config, min_lines)?;
    }
    Ok(WindowDetection {
        pairs: state.pairs,
        candidates: state.candidates,
        suppressed_buckets: state.suppressed.len(),
    })
}

struct WindowState {
    store: FastMap<u64, OccurrenceBucket>,
    seen: FastSet<RegionKey>,
    suppressed: FastSet<u64>,
    pairs: Vec<ClonePair>,
    candidates: usize,
}

impl WindowState {
    fn detect_source(
        &mut self,
        source: usize,
        tokenized: &[BlockTokens],
        sources: &[BlockSource],
        config: CloneConfig,
        min_lines: usize,
    ) -> Result<()> {
        let source_id = u32::try_from(source).map_err(|_| CloneError::CapacityExceeded {
            resource: "block source index",
            limit: u32::MAX as usize,
        })?;
        let mut open = None::<OpenRegion>;
        for (position, hash) in
            rolling_hashes(&tokenized[source].strict, config.min_tokens).enumerate()
        {
            let occurrence = Occurrence {
                source: source_id,
                position: u32::try_from(position).map_err(|_| CloneError::CapacityExceeded {
                    resource: "block token position",
                    limit: u32::MAX as usize,
                })?,
            };
            let representative = self.store.get(&hash).and_then(|bucket| {
                bucket.find(|candidate| {
                    window_equal(candidate, occurrence, tokenized, sources, config.min_tokens)
                })
            });
            let Some(representative) = representative else {
                self.flush(open.take(), sources, tokenized, config, min_lines);
                match self.store.entry(hash) {
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(OccurrenceBucket::new(occurrence));
                    }
                    std::collections::hash_map::Entry::Occupied(mut entry) => {
                        if entry.get().len() < config.max_bucket_size {
                            entry.get_mut().push(occurrence);
                        } else {
                            self.suppressed.insert(hash);
                        }
                    }
                }
                continue;
            };
            self.candidates = self.candidates.saturating_add(1);
            if self.candidates > config.max_candidates {
                return Err(CloneError::CapacityExceeded {
                    resource: "block candidates",
                    limit: config.max_candidates,
                });
            }
            if open
                .as_mut()
                .is_some_and(|region| region.extend(representative, occurrence))
            {
                continue;
            }
            self.flush(open.take(), sources, tokenized, config, min_lines);
            open = OpenRegion::new(representative, occurrence, config.min_tokens);
        }
        self.flush(open, sources, tokenized, config, min_lines);
        Ok(())
    }

    fn flush(
        &mut self,
        open: Option<OpenRegion>,
        sources: &[BlockSource],
        tokenized: &[BlockTokens],
        config: CloneConfig,
        min_lines: usize,
    ) {
        let Some(open) = open else {
            return;
        };
        let region = open.region;
        if !config.compare_overlapping_fragments
            && region.left_source == region.right_source
            && region.left_start < region.right_start + region.length
            && region.right_start < region.left_start + region.length
        {
            return;
        }
        if !self.seen.insert(region) {
            return;
        }
        let pair = region_pair(region, sources, tokenized, config.min_tokens);
        if line_count(pair.left.span) >= min_lines && line_count(pair.right.span) >= min_lines {
            self.pairs.push(pair);
        }
    }
}

struct OccurrenceBucket {
    first: Occurrence,
    collisions: Option<Box<Collision>>,
}

struct Collision {
    occurrence: Occurrence,
    next: Option<Box<Self>>,
}

impl OccurrenceBucket {
    const fn new(first: Occurrence) -> Self {
        Self {
            first,
            collisions: None,
        }
    }

    fn len(&self) -> usize {
        let mut count = 1;
        let mut cursor = self.collisions.as_deref();
        while let Some(collision) = cursor {
            count += 1;
            cursor = collision.next.as_deref();
        }
        count
    }

    fn find(&self, predicate: impl Fn(Occurrence) -> bool) -> Option<Occurrence> {
        if predicate(self.first) {
            return Some(self.first);
        }
        let mut cursor = self.collisions.as_deref();
        while let Some(collision) = cursor {
            if predicate(collision.occurrence) {
                return Some(collision.occurrence);
            }
            cursor = collision.next.as_deref();
        }
        None
    }

    fn push(&mut self, occurrence: Occurrence) {
        self.collisions = Some(Box::new(Collision {
            occurrence,
            next: self.collisions.take(),
        }));
    }
}

fn window_equal(
    left: Occurrence,
    right: Occurrence,
    tokens: &[BlockTokens],
    sources: &[BlockSource],
    minimum: usize,
) -> bool {
    let (Ok(left_source), Ok(right_source), Ok(left_start), Ok(right_start)) = (
        usize::try_from(left.source),
        usize::try_from(right.source),
        usize::try_from(left.position),
        usize::try_from(right.position),
    ) else {
        return false;
    };
    let Some(left_hashes) = tokens[left_source]
        .strict
        .get(left_start..left_start.saturating_add(minimum))
    else {
        return false;
    };
    let Some(right_hashes) = tokens[right_source]
        .strict
        .get(right_start..right_start.saturating_add(minimum))
    else {
        return false;
    };
    left_hashes == right_hashes
        && (0..minimum).all(|offset| {
            token_text(
                &sources[left_source],
                &tokens[left_source],
                left_start + offset,
            ) == token_text(
                &sources[right_source],
                &tokens[right_source],
                right_start + offset,
            )
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct RegionKey {
    pub left_source: usize,
    pub left_start: usize,
    pub right_source: usize,
    pub right_start: usize,
    pub length: usize,
}

struct OpenRegion {
    region: RegionKey,
    last_left: usize,
    last_right: usize,
}

impl OpenRegion {
    fn new(left: Occurrence, right: Occurrence, minimum: usize) -> Option<Self> {
        let left_source = usize::try_from(left.source).ok()?;
        let right_source = usize::try_from(right.source).ok()?;
        let left_start = usize::try_from(left.position).ok()?;
        let right_start = usize::try_from(right.position).ok()?;
        Some(Self {
            region: RegionKey {
                left_source,
                left_start,
                right_source,
                right_start,
                length: minimum,
            },
            last_left: left_start,
            last_right: right_start,
        })
    }

    fn extend(&mut self, left: Occurrence, right: Occurrence) -> bool {
        let continuation = usize::try_from(left.source).ok() == Some(self.region.left_source)
            && usize::try_from(right.source).ok() == Some(self.region.right_source)
            && usize::try_from(left.position).ok() == Some(self.last_left + 1)
            && usize::try_from(right.position).ok() == Some(self.last_right + 1);
        if continuation {
            self.region.length += 1;
            self.last_left += 1;
            self.last_right += 1;
        }
        continuation
    }
}

fn token_text<'a>(source: &'a BlockSource, tokens: &BlockTokens, index: usize) -> &'a str {
    let position = tokens.positions[index];
    &source.text[position.start_byte..position.end_byte]
}
