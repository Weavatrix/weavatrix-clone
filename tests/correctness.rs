use weavatrix_clone::{
    CloneConfig, CloneDetector, CloneError, CloneKind, DetectionMode, Language, Similarity,
    SourceFragment, SourceSpan,
};

#[test]
fn classifies_exact_renamed_and_near_miss_clones() {
    let exact = function("exact:a", "a.rs", "value", "result", "");
    let exact_copy = function("exact:b", "b.rs", "value", "result", "");
    let renamed = function("renamed", "c.rs", "input", "output", "");
    let near_miss = function(
        "near",
        "d.rs",
        "item",
        "answer",
        "let audit = answer.is_positive();",
    );

    let report = CloneDetector::default()
        .detect(&[exact, exact_copy, renamed, near_miss])
        .unwrap();

    assert!(
        report
            .pairs
            .iter()
            .any(|pair| pair.kind == CloneKind::Type1)
    );
    assert!(
        report
            .pairs
            .iter()
            .any(|pair| pair.kind == CloneKind::Type2)
    );
    assert!(
        report
            .pairs
            .iter()
            .any(|pair| pair.kind == CloneKind::Type3)
    );
    assert_eq!(report.statistics.verified_pairs, report.pairs.len());
    assert_eq!(report.families.len(), 1);
}

#[test]
fn exact_mode_excludes_identifier_and_near_miss_matches() {
    let exact = function("exact:a", "a.rs", "value", "result", "");
    let exact_copy = function("exact:b", "b.rs", "value", "result", "");
    let renamed = function("renamed", "c.rs", "input", "output", "");
    let detector = CloneDetector::new(CloneConfig {
        mode: DetectionMode::Exact,
        ..CloneConfig::default()
    })
    .unwrap();
    let report = detector.detect(&[exact, exact_copy, renamed]).unwrap();
    assert_eq!(report.pairs.len(), 1);
    assert_eq!(report.pairs[0].kind, CloneKind::Type1);
}

#[test]
fn preserves_behavior_defining_numbers_and_rejects_shape_only_matches() {
    let left = numeric_table("left", "left.rs", 0);
    let right = numeric_table("right", "right.rs", 10_000);
    let report = CloneDetector::default().detect(&[left, right]).unwrap();
    assert!(report.pairs.is_empty());
}

#[test]
fn output_is_stable_under_input_reordering() {
    let fragments = vec![
        function("a", "z.rs", "value", "result", ""),
        function("b", "a.rs", "item", "answer", ""),
        function("c", "m.rs", "source", "target", ""),
    ];
    let forward = CloneDetector::default().detect(&fragments).unwrap();
    let reverse = CloneDetector::default()
        .detect(&fragments.into_iter().rev().collect::<Vec<_>>())
        .unwrap();
    assert_eq!(forward, reverse);
}

#[test]
fn overlapping_fragments_are_suppressed_by_default() {
    let text = body("value", "result", "");
    let left = SourceFragment::new(
        "left",
        "same.rs",
        Language::Rust,
        SourceSpan {
            start_byte: 0,
            end_byte: text.len(),
            start_line: 1,
            end_line: 10,
        },
        &text,
    )
    .unwrap();
    let right = SourceFragment::new(
        "right",
        "same.rs",
        Language::Rust,
        SourceSpan {
            start_byte: 10,
            end_byte: text.len() + 10,
            start_line: 2,
            end_line: 11,
        },
        &text,
    )
    .unwrap();
    assert!(
        CloneDetector::default()
            .detect(&[left.clone(), right.clone()])
            .unwrap()
            .pairs
            .is_empty()
    );
    let detector = CloneDetector::new(CloneConfig {
        compare_overlapping_fragments: true,
        ..CloneConfig::default()
    })
    .unwrap();
    assert_eq!(detector.detect(&[left, right]).unwrap().pairs.len(), 1);
}

#[test]
fn invalid_configuration_and_duplicate_ids_are_explicit() {
    assert!(matches!(
        CloneDetector::new(CloneConfig {
            candidate_similarity: Similarity::from_permille(900),
            min_similarity: Similarity::from_permille(800),
            ..CloneConfig::default()
        }),
        Err(CloneError::InvalidConfig { .. })
    ));
    let fragment = function("same", "a.rs", "value", "result", "");
    assert!(matches!(
        CloneDetector::default().detect(&[fragment.clone(), fragment]),
        Err(CloneError::DuplicateFragment(_))
    ));
}

fn function(id: &str, path: &str, input: &str, output: &str, extra: &str) -> SourceFragment {
    let text = body(input, output, extra);
    SourceFragment::new(
        id,
        path,
        Language::Rust,
        SourceSpan {
            start_byte: 0,
            end_byte: text.len(),
            start_line: 1,
            end_line: 10,
        },
        text,
    )
    .unwrap()
}

fn body(input: &str, output: &str, extra: &str) -> String {
    format!(
        "fn calculate({input}: i64) -> i64 {{\n\
         let baseline = {input} * 31;\n\
         let adjusted = baseline + 7;\n\
         let {output} = if adjusted > 100 {{ adjusted / 2 }} else {{ adjusted * 2 }};\n\
         {extra}\n\
         let checked = {output}.saturating_add(baseline);\n\
         let bounded = checked.clamp(0, 10000);\n\
         bounded + adjusted + baseline\n\
         }}\n"
    )
}

fn numeric_table(id: &str, path: &str, base: usize) -> SourceFragment {
    let rows = (0..30)
        .map(|index| format!("let value_{index} = input + {};", base + index))
        .collect::<Vec<_>>()
        .join("\n");
    let text = format!("fn table(input: usize) -> usize {{\n{rows}\ninput\n}}\n");
    SourceFragment::new(
        id,
        path,
        Language::Rust,
        SourceSpan {
            start_byte: 0,
            end_byte: text.len(),
            start_line: 1,
            end_line: 33,
        },
        text,
    )
    .unwrap()
}
