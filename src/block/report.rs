use super::detect::RegionKey;
use super::{BlockSource, BlockTokens};
use crate::cluster::pair_id;
use crate::model::{CloneEvidence, CloneKind, CloneLocation, ClonePair, Similarity, SourceSpan};

pub(super) fn region_pair(
    region: RegionKey,
    sources: &[BlockSource],
    tokens: &[BlockTokens],
    minimum: usize,
) -> ClonePair {
    let left = location(
        &sources[region.left_source],
        &tokens[region.left_source],
        region.left_start,
        region.length,
    );
    let right = location(
        &sources[region.right_source],
        &tokens[region.right_source],
        region.right_start,
        region.length,
    );
    ClonePair {
        id: pair_id(&left, &right),
        left,
        right,
        kind: CloneKind::Type1,
        similarity: Similarity::PERFECT,
        evidence: CloneEvidence {
            strict_equal: true,
            renamed_equal: true,
            shared_fingerprints: region.length.saturating_sub(minimum).saturating_add(1),
            fingerprint_jaccard: Similarity::PERFECT,
            fingerprint_containment: Similarity::PERFECT,
            edit_distance: 0,
            compared_tokens: region.length,
        },
    }
}

pub(super) fn line_count(span: SourceSpan) -> usize {
    usize::try_from(
        span.end_line
            .saturating_sub(span.start_line)
            .saturating_add(1),
    )
    .unwrap_or(usize::MAX)
}

fn location(
    source: &BlockSource,
    tokens: &BlockTokens,
    start: usize,
    length: usize,
) -> CloneLocation {
    let first = tokens.positions[start];
    let last = tokens.positions[start + length - 1];
    CloneLocation {
        fragment_id: format!("{}#tokens:{start}-{}", source.path, start + length),
        path: source.path.clone(),
        span: SourceSpan {
            start_byte: first.start_byte,
            end_byte: last.end_byte,
            start_line: first.start_line,
            end_line: last.end_line,
        },
    }
}
