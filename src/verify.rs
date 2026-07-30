use crate::config::CloneConfig;
use crate::model::{CloneEvidence, CloneKind, DetectionMode, Similarity};

#[derive(Debug)]
pub(crate) struct Verified {
    pub kind: CloneKind,
    pub similarity: Similarity,
    pub edit_distance: usize,
}

#[derive(Debug, Default)]
pub(crate) struct Verifier {
    previous: Vec<usize>,
    current: Vec<usize>,
}

impl Verifier {
    pub fn verify(
        &mut self,
        strict_left: &[u32],
        strict_right: &[u32],
        renamed_left: &[u32],
        renamed_right: &[u32],
        config: CloneConfig,
    ) -> Option<Verified> {
        if strict_left == strict_right {
            return Some(Verified {
                kind: CloneKind::Type1,
                similarity: Similarity::PERFECT,
                edit_distance: 0,
            });
        }
        if config.mode == DetectionMode::Exact {
            return None;
        }
        if renamed_left == renamed_right {
            return Some(Verified {
                kind: CloneKind::Type2,
                similarity: Similarity::PERFECT,
                edit_distance: 0,
            });
        }
        if config.mode == DetectionMode::Renamed {
            return None;
        }
        let compared = renamed_left.len().max(renamed_right.len());
        let allowed =
            compared.saturating_mul(usize::from(1_000 - config.min_similarity.permille())) / 1_000;
        let (left, right) = trim_equal_edges(renamed_left, renamed_right);
        let distance = self.bounded_levenshtein(left, right, allowed)?;
        let similarity = Similarity::from_ratio(compared.saturating_sub(distance), compared);
        (similarity >= config.min_similarity).then_some(Verified {
            kind: CloneKind::Type3,
            similarity,
            edit_distance: distance,
        })
    }

    fn bounded_levenshtein(
        &mut self,
        left: &[u32],
        right: &[u32],
        max_distance: usize,
    ) -> Option<usize> {
        if left.len().abs_diff(right.len()) > max_distance {
            return None;
        }
        if left.is_empty() || right.is_empty() {
            return Some(left.len().max(right.len()));
        }
        let (rows, columns) = if left.len() >= right.len() {
            (left, right)
        } else {
            (right, left)
        };
        let unreachable = max_distance.saturating_add(1);
        self.previous.resize(columns.len() + 1, unreachable);
        self.current.resize(columns.len() + 1, unreachable);
        self.previous.fill(unreachable);
        for (index, value) in self
            .previous
            .iter_mut()
            .enumerate()
            .take(max_distance.min(columns.len()) + 1)
        {
            *value = index;
        }
        for (row_index, row) in rows.iter().enumerate() {
            let row_number = row_index + 1;
            self.current.fill(unreachable);
            if row_number <= max_distance {
                self.current[0] = row_number;
            }
            let start = row_number.saturating_sub(max_distance).max(1);
            let end = row_number.saturating_add(max_distance).min(columns.len());
            let mut row_minimum = unreachable;
            for column_number in start..=end {
                let substitution = self.previous[column_number - 1]
                    + usize::from(*row != columns[column_number - 1]);
                let deletion = self.previous[column_number].saturating_add(1);
                let insertion = self.current[column_number - 1].saturating_add(1);
                let value = substitution.min(deletion).min(insertion);
                self.current[column_number] = value;
                row_minimum = row_minimum.min(value);
            }
            if row_minimum > max_distance {
                return None;
            }
            std::mem::swap(&mut self.previous, &mut self.current);
        }
        (self.previous[columns.len()] <= max_distance).then_some(self.previous[columns.len()])
    }
}

pub(crate) fn evidence(
    verified: &Verified,
    shared: usize,
    jaccard: Similarity,
    containment: Similarity,
    compared_tokens: usize,
) -> CloneEvidence {
    CloneEvidence {
        strict_equal: verified.kind == CloneKind::Type1,
        renamed_equal: verified.kind <= CloneKind::Type2,
        shared_fingerprints: shared,
        fingerprint_jaccard: jaccard,
        fingerprint_containment: containment,
        edit_distance: verified.edit_distance,
        compared_tokens,
    }
}

fn trim_equal_edges<'a>(left: &'a [u32], right: &'a [u32]) -> (&'a [u32], &'a [u32]) {
    let prefix = left
        .iter()
        .zip(right)
        .take_while(|(left, right)| left == right)
        .count();
    let left = &left[prefix..];
    let right = &right[prefix..];
    let suffix = left
        .iter()
        .rev()
        .zip(right.iter().rev())
        .take_while(|(left, right)| left == right)
        .count();
    (
        &left[..left.len().saturating_sub(suffix)],
        &right[..right.len().saturating_sub(suffix)],
    )
}

#[cfg(test)]
mod tests {
    use crate::verify::Verifier;

    #[test]
    fn bounded_distance_matches_expected_edits() {
        let mut verifier = Verifier::default();
        assert_eq!(
            verifier.bounded_levenshtein(&[1, 2, 3], &[1, 4, 3], 1),
            Some(1)
        );
        assert_eq!(
            verifier.bounded_levenshtein(&[1, 2, 3], &[1, 3], 1),
            Some(1)
        );
        assert_eq!(verifier.bounded_levenshtein(&[1, 2], &[], 2), Some(2));
        assert_eq!(
            verifier.bounded_levenshtein(&[1, 2, 3], &[4, 5, 6], 2),
            None
        );
        assert_eq!(verifier.bounded_levenshtein(&[], &[], 0), Some(0));
    }
}
