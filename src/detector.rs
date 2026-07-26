use crate::canonical::suppress_contained;
use crate::cluster::{families_for_pairs, pair_id};
use crate::fingerprint::winnow;
use crate::index::candidates;
use crate::token::{Interner, Tokenized, tokenize};
use crate::verify::{Verifier, evidence};
use crate::{
    CloneConfig, CloneError, CloneLocation, ClonePair, CloneReport, CloneStatistics, DetectionMode,
    Result, SourceFragment,
};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, Default)]
pub struct CloneDetector {
    config: CloneConfig,
}

impl CloneDetector {
    /// Creates a detector after validating all safety bounds.
    ///
    /// # Errors
    ///
    /// Returns an error for inconsistent thresholds or zero limits.
    pub fn new(config: CloneConfig) -> Result<Self> {
        Ok(Self {
            config: config.validate()?,
        })
    }

    #[must_use]
    pub const fn config(&self) -> CloneConfig {
        self.config
    }

    /// Detects deterministic Type-1, Type-2, and bounded near-miss Type-3
    /// clones over caller-provided fragments.
    ///
    /// # Errors
    ///
    /// Rejects malformed or duplicate fragments and configured capacity
    /// limits without returning partial output.
    #[allow(clippy::too_many_lines)]
    pub fn detect(&self, fragments: &[SourceFragment]) -> Result<CloneReport> {
        if fragments.len() > self.config.max_fragments {
            return Err(CloneError::CapacityExceeded {
                resource: "input fragments",
                limit: self.config.max_fragments,
            });
        }
        validate_fragments(fragments)?;
        let mut order = (0..fragments.len()).collect::<Vec<_>>();
        order.sort_unstable_by(|left, right| {
            let left = &fragments[*left];
            let right = &fragments[*right];
            (&left.path, left.span, &left.id).cmp(&(&right.path, right.span, &right.id))
        });

        let mut interner = Interner::default();
        let mut prepared = Vec::with_capacity(fragments.len());
        let mut statistics = CloneStatistics {
            input_fragments: fragments.len(),
            ..CloneStatistics::default()
        };
        for index in order {
            let fragment = &fragments[index];
            let tokens = tokenize(
                &fragment.text,
                fragment.language,
                self.config,
                &mut interner,
            )?;
            if tokens.strict.len() < self.config.min_tokens {
                statistics.skipped_small_fragments += 1;
                continue;
            }
            statistics.tokens = statistics.tokens.saturating_add(tokens.strict.len());
            let fingerprint_tokens = if self.config.mode == DetectionMode::Exact {
                &tokens.strict
            } else {
                &tokens.renamed
            };
            let fingerprints = winnow(
                fingerprint_tokens,
                self.config.k_gram,
                self.config.winnowing_window,
            );
            statistics.fingerprints = statistics.fingerprints.saturating_add(fingerprints.len());
            prepared.push(Prepared {
                source_index: index,
                tokens,
                fingerprints,
            });
        }
        statistics.analyzed_fragments = prepared.len();
        let fingerprint_sets = prepared
            .iter()
            .map(|item| item.fingerprints.clone())
            .collect::<Vec<_>>();
        let index = candidates(&fingerprint_sets, self.config)?;
        statistics.candidate_pairs = index.candidates.len();
        statistics.suppressed_buckets = index.suppressed_buckets;

        let locations = prepared
            .iter()
            .map(|item| CloneLocation::from_fragment(&fragments[item.source_index]))
            .collect::<Vec<_>>();
        let mut pairs = Vec::new();
        let mut verifier = Verifier::default();
        for candidate in index.candidates {
            let left = &prepared[candidate.left];
            let right = &prepared[candidate.right];
            let left_fragment = &fragments[left.source_index];
            let right_fragment = &fragments[right.source_index];
            if !self.config.compare_overlapping_fragments
                && left_fragment.path == right_fragment.path
                && left_fragment.span.overlaps(right_fragment.span)
            {
                continue;
            }
            let Some(match_result) = verifier.verify(
                &left.tokens.strict,
                &right.tokens.strict,
                &left.tokens.renamed,
                &right.tokens.renamed,
                self.config,
            ) else {
                continue;
            };
            let left_location = &locations[candidate.left];
            let right_location = &locations[candidate.right];
            let id = pair_id(left_location, right_location);
            pairs.push(ClonePair {
                id,
                left: left_location.clone(),
                right: right_location.clone(),
                kind: match_result.kind,
                similarity: match_result.similarity,
                evidence: evidence(
                    &match_result,
                    candidate.shared,
                    candidate.jaccard,
                    candidate.containment,
                    left.tokens.renamed.len().max(right.tokens.renamed.len()),
                ),
            });
        }
        let pairs = suppress_contained(pairs);
        statistics.verified_pairs = pairs.len();
        Ok(CloneReport {
            families: families_for_pairs(&pairs),
            pairs,
            statistics,
        })
    }
}

struct Prepared {
    source_index: usize,
    tokens: Tokenized,
    fingerprints: Vec<u64>,
}

fn validate_fragments(fragments: &[SourceFragment]) -> Result<()> {
    let mut ids = HashSet::<&str>::with_capacity(fragments.len());
    for fragment in fragments {
        fragment.validate()?;
        if !ids.insert(&fragment.id) {
            return Err(CloneError::DuplicateFragment(fragment.id.clone()));
        }
    }
    Ok(())
}
