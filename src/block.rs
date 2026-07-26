use crate::fast_hash::stable_bytes;
use crate::token::{TokenPosition, lex};
use crate::{CloneConfig, CloneError, ClonePair, Language, Result};

mod detect;
mod report;

use detect::detect_windows;

pub(crate) struct BlockSource {
    pub path: String,
    pub language: Language,
    pub text: String,
}

#[derive(Default)]
pub(crate) struct BlockDetection {
    pub pairs: Vec<ClonePair>,
    pub tokens: usize,
    pub candidates: usize,
    pub suppressed_buckets: usize,
}

impl BlockDetection {
    pub fn merge(&mut self, mut other: Self) {
        self.pairs.append(&mut other.pairs);
        self.tokens = self.tokens.saturating_add(other.tokens);
        self.candidates = self.candidates.saturating_add(other.candidates);
        self.suppressed_buckets = self
            .suppressed_buckets
            .saturating_add(other.suppressed_buckets);
    }
}

pub(super) struct BlockTokens {
    pub strict: Vec<u64>,
    pub positions: Vec<TokenPosition>,
}

pub(crate) fn detect_blocks(
    sources: &[BlockSource],
    config: CloneConfig,
    min_lines: usize,
    parallelism: usize,
) -> Result<BlockDetection> {
    let tokenized = tokenize_sources(sources, config, parallelism)?;
    let token_count = tokenized.iter().map(|tokens| tokens.strict.len()).sum();
    let windows = detect_windows(sources, &tokenized, config, min_lines)?;
    Ok(BlockDetection {
        pairs: windows.pairs,
        tokens: token_count,
        candidates: windows.candidates,
        suppressed_buckets: windows.suppressed_buckets,
    })
}

fn tokenize_sources(
    sources: &[BlockSource],
    config: CloneConfig,
    parallelism: usize,
) -> Result<Vec<BlockTokens>> {
    let available = std::thread::available_parallelism().map_or(1, usize::from);
    let workers = if parallelism == 0 {
        available
    } else {
        parallelism
    }
    .min(sources.len());
    if workers <= 1 || sources.len() < 32 {
        return sources
            .iter()
            .map(|source| tokenize_source(source, config))
            .collect();
    }
    let chunk_size = sources.len().div_ceil(workers);
    std::thread::scope(|scope| {
        let handles = sources
            .chunks(chunk_size)
            .enumerate()
            .map(|(chunk_index, chunk)| {
                scope.spawn(move || {
                    let mut result = Vec::with_capacity(chunk.len());
                    for source in chunk {
                        result.push(tokenize_source(source, config)?);
                    }
                    Ok::<_, CloneError>((chunk_index, result))
                })
            })
            .collect::<Vec<_>>();
        let mut chunks = Vec::with_capacity(handles.len());
        for handle in handles {
            chunks.push(
                handle
                    .join()
                    .map_err(|_| CloneError::Repository("lexer worker panicked".to_owned()))??,
            );
        }
        chunks.sort_unstable_by_key(|(index, _)| *index);
        Ok(chunks.into_iter().flat_map(|(_, tokens)| tokens).collect())
    })
}

fn tokenize_source(source: &BlockSource, config: CloneConfig) -> Result<BlockTokens> {
    let lexed = lex(&source.text, source.language, config)?;
    let mut strict = Vec::with_capacity(lexed.len());
    let mut positions = Vec::with_capacity(lexed.len());
    for token in lexed {
        strict.push(stable_bytes(
            &source.text.as_bytes()[token.position.start_byte..token.position.end_byte],
        ));
        positions.push(token.position);
    }
    Ok(BlockTokens { strict, positions })
}
