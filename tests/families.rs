use weavatrix_clone::{
    CloneEvidence, CloneKind, CloneLocation, ClonePair, Similarity, SourceSpan, families_for_pairs,
};

#[test]
fn filtered_pairs_rebuild_coherent_families_with_stable_ids() {
    let all_pairs = vec![
        pair("pair:ab", "a", "b"),
        pair("pair:bc", "b", "c"),
        pair("pair:cd", "c", "d"),
        pair("pair:de", "d", "e"),
    ];
    let filtered = all_pairs
        .into_iter()
        .filter(|pair| pair.id != "pair:cd")
        .collect::<Vec<_>>();

    let families = families_for_pairs(&filtered);
    assert_eq!(families.len(), 2);
    assert_eq!(
        fragment_ids(&families[0].members),
        ["d", "e"],
        "the removed bridge must not leave a stale connected family"
    );
    assert_eq!(families[0].pair_ids, ["pair:de"]);
    assert_eq!(fragment_ids(&families[1].members), ["a", "b", "c"]);
    assert_eq!(families[1].pair_ids, ["pair:ab", "pair:bc"]);

    let reordered = filtered.into_iter().rev().collect::<Vec<_>>();
    assert_eq!(
        families_for_pairs(&reordered),
        families,
        "family ordering, membership, and IDs must be input-order independent"
    );
}

fn pair(id: &str, left: &str, right: &str) -> ClonePair {
    ClonePair {
        id: id.to_owned(),
        left: location(left),
        right: location(right),
        kind: CloneKind::Type1,
        similarity: Similarity::PERFECT,
        evidence: CloneEvidence {
            strict_equal: true,
            renamed_equal: true,
            shared_fingerprints: 1,
            fingerprint_jaccard: Similarity::PERFECT,
            fingerprint_containment: Similarity::PERFECT,
            edit_distance: 0,
            compared_tokens: 1,
        },
    }
}

fn location(id: &str) -> CloneLocation {
    CloneLocation {
        fragment_id: id.to_owned(),
        path: format!("src/{id}.rs"),
        span: SourceSpan {
            start_byte: 0,
            end_byte: 1,
            start_line: 1,
            end_line: 1,
        },
    }
}

fn fragment_ids(locations: &[CloneLocation]) -> Vec<&str> {
    locations
        .iter()
        .map(|location| location.fragment_id.as_str())
        .collect()
}
