#![cfg(feature = "scan")]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use weavatrix_clone::{
    CloneConfig, CloneDetector, CloneKind, DetectionMode, RepositoryCloneDetector,
    RepositoryOptions,
};

#[test]
fn repository_scan_extracts_functions_and_detects_renamed_clones() {
    let root = TempDirectory::new();
    fs::create_dir_all(root.path().join("src")).unwrap();
    fs::write(
        root.path().join("src/a.rs"),
        source("calculate_total", "value", "result"),
    )
    .unwrap();
    fs::write(
        root.path().join("src/b.rs"),
        source("compute_sum", "input", "answer"),
    )
    .unwrap();

    let report = RepositoryCloneDetector::new(CloneDetector::default())
        .detect(root.path())
        .unwrap();

    assert_eq!(report.pairs.len(), 1);
    assert_eq!(report.pairs[0].kind, CloneKind::Type2);
    assert_eq!(report.families.len(), 1);
}

#[test]
fn exact_mode_finds_partial_blocks_and_respects_line_threshold() {
    let root = TempDirectory::new();
    fs::write(
        root.path().join("a.rs"),
        "fn left() {\nlet a = load(1, 2, 3);\nlet b = normalize(a, 4, 5);\nsave(b, 6, 7);\n}\n",
    )
    .unwrap();
    fs::write(
        root.path().join("b.rs"),
        "fn right() {\ntrace(\"before\");\nlet a = load(1, 2, 3);\nlet b = normalize(a, 4, 5);\nsave(b, 6, 7);\ntrace(\"after\");\n}\n",
    )
    .unwrap();
    let detector = CloneDetector::new(CloneConfig {
        mode: DetectionMode::Exact,
        ..CloneConfig::default()
    })
    .unwrap();
    let report = RepositoryCloneDetector::new(detector)
        .options(RepositoryOptions {
            min_fragment_lines: 3,
            ..RepositoryOptions::default()
        })
        .detect(root.path())
        .unwrap();
    assert_eq!(report.pairs.len(), 1);
    assert_eq!(report.pairs[0].kind, CloneKind::Type1);
    assert!(report.pairs[0].left.span.start_line > 1);
    assert_eq!(report.statistics.source_files, 2);
}

#[test]
fn exact_mode_does_not_report_long_single_line_matches() {
    let root = TempDirectory::new();
    let line = "fn compact(){a(1);b(2);c(3);d(4);e(5);f(6);g(7);h(8);i(9);j(10);}\n";
    fs::write(root.path().join("a.rs"), line).unwrap();
    fs::write(root.path().join("b.rs"), line).unwrap();
    let detector = CloneDetector::new(CloneConfig {
        mode: DetectionMode::Exact,
        ..CloneConfig::default()
    })
    .unwrap();
    let report = RepositoryCloneDetector::new(detector)
        .detect(root.path())
        .unwrap();
    assert!(report.pairs.is_empty());
}

#[test]
fn cross_extension_matching_is_explicit() {
    let root = TempDirectory::new();
    let source = source("calculate_total", "value", "result");
    fs::write(root.path().join("a.ts"), &source).unwrap();
    fs::write(root.path().join("b.tsx"), &source).unwrap();
    let detector = CloneDetector::new(CloneConfig {
        mode: DetectionMode::Exact,
        ..CloneConfig::default()
    })
    .unwrap();
    let isolated = RepositoryCloneDetector::new(detector)
        .options(RepositoryOptions::default().with_extensions(["ts", "tsx"]))
        .detect(root.path())
        .unwrap();
    assert!(isolated.pairs.is_empty());
    let cross = RepositoryCloneDetector::new(detector)
        .options(RepositoryOptions {
            cross_extensions: true,
            ..RepositoryOptions::default().with_extensions(["ts", "tsx"])
        })
        .detect(root.path())
        .unwrap();
    assert_eq!(cross.pairs.len(), 1);
}

#[test]
fn frequent_exact_windows_are_clustered_without_suppression() {
    let root = TempDirectory::new();
    for index in 0..130 {
        fs::write(
            root.path().join(format!("{index}.rs")),
            source("calculate_total", "value", "result"),
        )
        .unwrap();
    }
    let detector = CloneDetector::new(CloneConfig {
        mode: DetectionMode::Exact,
        ..CloneConfig::default()
    })
    .unwrap();
    let report = RepositoryCloneDetector::new(detector)
        .detect(root.path())
        .unwrap();
    assert_eq!(report.pairs.len(), 129);
    assert_eq!(report.families.len(), 1);
    assert_eq!(report.statistics.suppressed_exact_buckets, 0);
}

fn source(name: &str, input: &str, output: &str) -> String {
    format!(
        "fn {name}({input}: i64) -> i64 {{\n\
         let baseline = {input} * 31;\n\
         let adjusted = baseline + 7;\n\
         let {output} = adjusted.saturating_mul(2);\n\
         let checked = {output}.saturating_add(baseline);\n\
         let bounded = checked.clamp(0, 10000);\n\
         bounded + adjusted + baseline\n\
         }}\n"
    )
}

struct TempDirectory(PathBuf);

impl TempDirectory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "weavatrix-clone-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
