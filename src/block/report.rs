use super::detect::RegionKey;
use super::{BlockSource, BlockTokens};
use crate::cluster::pair_id;
use crate::model::{CloneEvidence, CloneKind, CloneLocation, ClonePair, Similarity, SourceSpan};
use crate::token::TokenPosition;

/// Shrinks a matched region to the tokens both sites cover on whole lines.
///
/// A token run starts and ends wherever the match does, which is regularly
/// mid-line, and two sites rarely break their lines the same way. Reporting
/// the raw first and last token lines puts text the matcher never compared
/// inside the reported range: the sites then read as different while the
/// evidence calls them identical. Snapping both sites to the same
/// line-aligned sub-run keeps every reported line fully matched.
///
/// Returns `None` for a run that covers no whole line on both sites, which
/// leaves the raw token region as the only available answer.
pub(super) fn snap_to_lines(region: RegionKey, tokens: &[BlockTokens]) -> Option<RegionKey> {
    let left = &tokens.get(region.left_source)?.positions;
    let right = &tokens.get(region.right_source)?.positions;
    let head = (0..region.length).find(|offset| {
        starts_line(left, region.left_start + offset)
            && starts_line(right, region.right_start + offset)
    })?;
    let last = region.length - 1;
    let tail = (0..region.length - head).find(|offset| {
        ends_line(left, region.left_start + last - offset)
            && ends_line(right, region.right_start + last - offset)
    })?;
    let length = region.length.checked_sub(head.saturating_add(tail))?;
    (length > 0).then_some(RegionKey {
        left_start: region.left_start + head,
        right_start: region.right_start + head,
        length,
        ..region
    })
}

fn starts_line(positions: &[TokenPosition], index: usize) -> bool {
    positions
        .get(index)
        .is_some_and(|token| index == 0 || positions[index - 1].end_line < token.start_line)
}

fn ends_line(positions: &[TokenPosition], index: usize) -> bool {
    positions.get(index).is_some_and(|token| {
        positions
            .get(index + 1)
            .is_none_or(|next| next.start_line > token.end_line)
    })
}

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

#[cfg(test)]
mod tests {
    use super::super::BlockTokens;
    use super::{RegionKey, snap_to_lines};
    use crate::token::TokenPosition;

    /// One token per entry, on the given line.
    fn tokens(lines: &[u32]) -> BlockTokens {
        BlockTokens {
            strict: vec![0; lines.len()],
            positions: lines
                .iter()
                .enumerate()
                .map(|(index, line)| TokenPosition {
                    start_byte: index,
                    end_byte: index + 1,
                    start_line: *line,
                    end_line: *line,
                })
                .collect(),
        }
    }

    #[test]
    fn both_sites_lose_the_lines_only_one_of_them_breaks() {
        // The left run opens at the end of a line the right run opens fresh,
        // and closes mid-line where the right run closes its own line.
        let sources = [
            tokens(&[1, 1, 2, 2, 3, 3, 4, 4]),
            tokens(&[1, 2, 2, 3, 3, 4, 4, 4]),
        ];
        let region = RegionKey {
            left_source: 0,
            left_start: 1,
            right_source: 1,
            right_start: 0,
            length: 6,
        };

        let snapped = snap_to_lines(region, &sources).unwrap();

        assert_eq!(snapped.left_start, 2, "left kept a partial opening line");
        assert_eq!(snapped.right_start, 1, "right did not follow the left snap");
        assert_eq!(snapped.length, 4, "a partial closing line survived");
    }

    #[test]
    fn an_already_line_aligned_region_is_unchanged() {
        let sources = [tokens(&[1, 1, 2, 2]), tokens(&[7, 7, 8, 8])];
        let region = RegionKey {
            left_source: 0,
            left_start: 0,
            right_source: 1,
            right_start: 0,
            length: 4,
        };

        assert_eq!(snap_to_lines(region, &sources), Some(region));
    }

    #[test]
    fn a_run_inside_a_single_line_has_no_line_aligned_answer() {
        let sources = [tokens(&[1, 1, 1, 1]), tokens(&[2, 2, 2, 2])];
        let region = RegionKey {
            left_source: 0,
            left_start: 1,
            right_source: 1,
            right_start: 1,
            length: 2,
        };

        assert_eq!(snap_to_lines(region, &sources), None);
    }
}
