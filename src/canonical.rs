use crate::ClonePair;

pub(crate) fn suppress_contained(mut pairs: Vec<ClonePair>) -> Vec<ClonePair> {
    pairs.sort_by(compare_for_suppression);
    let mut result = Vec::new();
    let mut kept = Vec::<ClonePair>::new();
    let mut current_paths = None::<(String, String)>;
    for candidate in pairs {
        let same_group = current_paths.as_ref().is_some_and(|(left, right)| {
            left == &candidate.left.path && right == &candidate.right.path
        });
        if !same_group {
            result.append(&mut kept);
            current_paths = Some((candidate.left.path.clone(), candidate.right.path.clone()));
        }
        if !kept.iter().any(|existing| dominates(existing, &candidate)) {
            kept.push(candidate);
        }
    }
    result.append(&mut kept);
    result.sort_by(|left, right| {
        (&left.left, &left.right, left.kind).cmp(&(&right.left, &right.right, right.kind))
    });
    result
}

fn compare_for_suppression(left: &ClonePair, right: &ClonePair) -> std::cmp::Ordering {
    (&left.left.path, &left.right.path)
        .cmp(&(&right.left.path, &right.right.path))
        .then_with(|| covered_bytes(right).cmp(&covered_bytes(left)))
        .then_with(|| {
            right
                .evidence
                .compared_tokens
                .cmp(&left.evidence.compared_tokens)
        })
        .then_with(|| right.similarity.cmp(&left.similarity))
        .then_with(|| left.id.cmp(&right.id))
}

const fn covered_bytes(pair: &ClonePair) -> usize {
    pair.left
        .span
        .end_byte
        .saturating_sub(pair.left.span.start_byte)
        .saturating_add(
            pair.right
                .span
                .end_byte
                .saturating_sub(pair.right.span.start_byte),
        )
}

fn dominates(existing: &ClonePair, candidate: &ClonePair) -> bool {
    existing.similarity >= candidate.similarity
        && contains(existing.left.span, candidate.left.span)
        && contains(existing.right.span, candidate.right.span)
}

const fn contains(outer: crate::SourceSpan, inner: crate::SourceSpan) -> bool {
    outer.start_byte <= inner.start_byte && outer.end_byte >= inner.end_byte
}

#[cfg(test)]
mod tests {
    use super::suppress_contained;
    use crate::{CloneEvidence, CloneKind, CloneLocation, ClonePair, Similarity, SourceSpan};

    #[test]
    fn larger_equivalent_pair_suppresses_its_windows() {
        let whole = pair("whole", 0, 100, 100);
        let window = pair("window", 10, 50, 40);
        assert_eq!(suppress_contained(vec![window, whole]).len(), 1);
    }

    #[test]
    fn perfect_structural_pair_suppresses_nested_exact_evidence() {
        let mut whole = pair("whole", 0, 100, 100);
        whole.kind = CloneKind::Type2;
        let window = pair("window", 10, 50, 40);
        assert_eq!(suppress_contained(vec![window, whole]).len(), 1);
    }

    fn pair(id: &str, start: usize, end: usize, tokens: usize) -> ClonePair {
        let location = |path: &str| CloneLocation {
            fragment_id: format!("{path}:{id}"),
            path: path.to_owned(),
            span: SourceSpan {
                start_byte: start,
                end_byte: end,
                start_line: 1,
                end_line: 10,
            },
        };
        ClonePair {
            id: id.to_owned(),
            left: location("a.rs"),
            right: location("b.rs"),
            kind: CloneKind::Type1,
            similarity: Similarity::PERFECT,
            evidence: CloneEvidence {
                strict_equal: true,
                renamed_equal: true,
                shared_fingerprints: 1,
                fingerprint_jaccard: Similarity::PERFECT,
                fingerprint_containment: Similarity::PERFECT,
                edit_distance: 0,
                compared_tokens: tokens,
            },
        }
    }
}
