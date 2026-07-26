#![cfg(feature = "scan")]

use weavatrix_clone::{
    AccuracyGate, CloneConfig, CloneDetector, CloneKind, DetectionMode, OracleLocation, OraclePair,
    RepositoryCloneDetector, RepositoryOptions, Similarity,
};

#[test]
fn checked_in_oracle_gates_type1_type2_type3_and_negative_precision() {
    let corpus = format!("{}/benchmarks/corpus/rust", env!("CARGO_MANIFEST_DIR"));
    let detector = CloneDetector::new(CloneConfig {
        mode: DetectionMode::NearMiss,
        min_tokens: 24,
        ..CloneConfig::default()
    })
    .unwrap();
    let report = RepositoryCloneDetector::new(detector)
        .options(RepositoryOptions::default().with_extensions(["rs"]))
        .detect(corpus)
        .unwrap();
    let oracle = [
        positive(
            "exact",
            CloneKind::Type1,
            ("exact_a.rs", 11),
            ("exact_b.rs", 11),
        ),
        positive(
            "renamed",
            CloneKind::Type2,
            ("renamed_a.rs", 14),
            ("renamed_b.rs", 14),
        ),
        positive(
            "near",
            CloneKind::Type3,
            ("near_a.rs", 11),
            ("near_b.rs", 12),
        ),
        OraclePair::negative(
            "numeric-lookalike",
            location("negative_a.rs", 7),
            location("negative_b.rs", 7),
        ),
    ];
    let accuracy = AccuracyGate::default().check(&report, &oracle).unwrap();
    assert_eq!(accuracy.overall.precision(), Similarity::PERFECT);
    assert_eq!(accuracy.overall.recall(), Similarity::PERFECT);
    assert_eq!(accuracy.type1.true_positives, 1);
    assert_eq!(accuracy.type2.true_positives, 1);
    assert_eq!(accuracy.type3.true_positives, 1);
}

#[test]
fn portable_java_oracle_has_only_the_three_intended_relations() {
    let corpus = format!("{}/benchmarks/corpus/java", env!("CARGO_MANIFEST_DIR"));
    let detector = CloneDetector::new(CloneConfig {
        mode: DetectionMode::NearMiss,
        min_tokens: 24,
        ..CloneConfig::default()
    })
    .unwrap();
    let report = RepositoryCloneDetector::new(detector)
        .options(RepositoryOptions::default().with_extensions(["java"]))
        .detect(corpus)
        .unwrap();
    let oracle = [
        positive(
            "java-exact",
            CloneKind::Type1,
            ("ExactA.java", 13),
            ("ExactB.java", 13),
        ),
        positive(
            "java-renamed",
            CloneKind::Type2,
            ("RenamedA.java", 13),
            ("RenamedB.java", 13),
        ),
        positive(
            "java-near",
            CloneKind::Type3,
            ("NearA.java", 12),
            ("NearB.java", 15),
        ),
        OraclePair::negative(
            "java-unrelated",
            location("NegativeA.java", 15),
            location("NegativeB.java", 15),
        ),
    ];
    let accuracy = AccuracyGate::default().check(&report, &oracle).unwrap();
    assert_eq!(accuracy.overall.precision(), Similarity::PERFECT);
    assert_eq!(accuracy.overall.recall(), Similarity::PERFECT);
    assert!(
        report
            .pairs
            .iter()
            .all(|pair| { intended_java_pair(&pair.left.path, &pair.right.path) })
    );
}

#[test]
fn gate_reports_a_missed_threshold() {
    let oracle = [OraclePair::positive(
        "missing",
        CloneKind::Type1,
        location("a.rs", 10),
        location("b.rs", 10),
    )];
    let error = AccuracyGate::default()
        .check(&weavatrix_clone::CloneReport::default(), &oracle)
        .unwrap_err();
    assert!(error.to_string().contains("recall"));
}

fn positive(id: &str, kind: CloneKind, left: (&str, u32), right: (&str, u32)) -> OraclePair {
    OraclePair::positive(
        id,
        kind,
        location(left.0, left.1),
        location(right.0, right.1),
    )
}

fn location(path: &str, end_line: u32) -> OracleLocation {
    OracleLocation::new(path, 1, end_line)
}

fn intended_java_pair(left: &str, right: &str) -> bool {
    let (left, right) = if left <= right {
        (left, right)
    } else {
        (right, left)
    };
    matches!(
        (left, right),
        ("ExactA.java", "ExactB.java")
            | ("NearA.java", "NearB.java")
            | ("RenamedA.java", "RenamedB.java")
    )
}
