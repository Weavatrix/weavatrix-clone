use crate::{CloneError, CloneKind, CloneLocation, CloneReport, Result, Similarity};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OracleLocation {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
}

impl OracleLocation {
    #[must_use]
    pub fn new(path: impl Into<String>, start_line: u32, end_line: u32) -> Self {
        Self {
            path: path.into().replace('\\', "/"),
            start_line,
            end_line,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OraclePair {
    pub id: String,
    pub left: OracleLocation,
    pub right: OracleLocation,
    pub expected: bool,
    pub kind: Option<CloneKind>,
}

impl OraclePair {
    #[must_use]
    pub fn positive(
        id: impl Into<String>,
        kind: CloneKind,
        left: OracleLocation,
        right: OracleLocation,
    ) -> Self {
        Self {
            id: id.into(),
            left,
            right,
            expected: true,
            kind: Some(kind),
        }
    }

    #[must_use]
    pub fn negative(id: impl Into<String>, left: OracleLocation, right: OracleLocation) -> Self {
        Self {
            id: id.into(),
            left,
            right,
            expected: false,
            kind: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AccuracyCounts {
    pub true_positives: usize,
    pub false_positives: usize,
    pub true_negatives: usize,
    pub false_negatives: usize,
}

impl AccuracyCounts {
    #[must_use]
    pub fn precision(self) -> Similarity {
        ratio(
            self.true_positives,
            self.true_positives.saturating_add(self.false_positives),
        )
    }

    #[must_use]
    pub fn recall(self) -> Similarity {
        ratio(
            self.true_positives,
            self.true_positives.saturating_add(self.false_negatives),
        )
    }

    #[must_use]
    pub fn f1(self) -> Similarity {
        let precision = usize::from(self.precision().permille());
        let recall = usize::from(self.recall().permille());
        if precision + recall == 0 {
            return Similarity::from_permille(0);
        }
        Similarity::from_permille(
            u16::try_from(2 * precision * recall / (precision + recall)).unwrap_or(1_000),
        )
    }

    fn record(&mut self, expected: bool, detected: bool) {
        match (expected, detected) {
            (true, true) => self.true_positives += 1,
            (false, true) => self.false_positives += 1,
            (false, false) => self.true_negatives += 1,
            (true, false) => self.false_negatives += 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AccuracyReport {
    pub overall: AccuracyCounts,
    pub type1: AccuracyCounts,
    pub type2: AccuracyCounts,
    pub type3: AccuracyCounts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccuracyGate {
    pub coverage: Similarity,
    pub min_precision: Similarity,
    pub min_recall: Similarity,
}

impl Default for AccuracyGate {
    fn default() -> Self {
        Self {
            coverage: Similarity::from_permille(700),
            min_precision: Similarity::PERFECT,
            min_recall: Similarity::PERFECT,
        }
    }
}

impl AccuracyGate {
    #[must_use]
    pub fn evaluate(self, report: &CloneReport, oracle: &[OraclePair]) -> AccuracyReport {
        let mut accuracy = AccuracyReport::default();
        let mut by_paths = HashMap::<(&str, &str), Vec<&crate::ClonePair>>::new();
        for candidate in &report.pairs {
            by_paths
                .entry(path_key(&candidate.left.path, &candidate.right.path))
                .or_default()
                .push(candidate);
        }
        for expected in oracle {
            let detected = by_paths
                .get(&path_key(&expected.left.path, &expected.right.path))
                .into_iter()
                .flatten()
                .any(|candidate| pair_matches(candidate, expected, self.coverage));
            accuracy.overall.record(expected.expected, detected);
            let kind_counts = match expected.kind {
                Some(CloneKind::Type1) => Some(&mut accuracy.type1),
                Some(CloneKind::Type2) => Some(&mut accuracy.type2),
                Some(CloneKind::Type3) => Some(&mut accuracy.type3),
                None => None,
            };
            if let Some(counts) = kind_counts {
                counts.record(expected.expected, detected);
            }
        }
        accuracy
    }

    /// Checks precision and recall over explicitly labeled oracle relations.
    ///
    /// # Errors
    ///
    /// Returns an accuracy error when either configured threshold is missed.
    pub fn check(self, report: &CloneReport, oracle: &[OraclePair]) -> Result<AccuracyReport> {
        let accuracy = self.evaluate(report, oracle);
        require(
            "precision",
            accuracy.overall.precision(),
            self.min_precision,
        )?;
        require("recall", accuracy.overall.recall(), self.min_recall)?;
        Ok(accuracy)
    }
}

fn path_key<'a>(left: &'a str, right: &'a str) -> (&'a str, &'a str) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

fn pair_matches(
    candidate: &crate::ClonePair,
    expected: &OraclePair,
    threshold: Similarity,
) -> bool {
    (covers(&candidate.left, &expected.left, threshold)
        && covers(&candidate.right, &expected.right, threshold))
        || (covers(&candidate.left, &expected.right, threshold)
            && covers(&candidate.right, &expected.left, threshold))
}

fn covers(candidate: &CloneLocation, expected: &OracleLocation, threshold: Similarity) -> bool {
    if candidate.path.replace('\\', "/") != expected.path || expected.start_line > expected.end_line
    {
        return false;
    }
    let start = candidate.span.start_line.max(expected.start_line);
    let end = candidate.span.end_line.min(expected.end_line);
    let intersection = end
        .saturating_sub(start)
        .saturating_add(u32::from(end >= start));
    let expected_lines = expected
        .end_line
        .saturating_sub(expected.start_line)
        .saturating_add(1);
    u64::from(intersection) * 1_000 >= u64::from(expected_lines) * u64::from(threshold.permille())
}

fn ratio(numerator: usize, denominator: usize) -> Similarity {
    if denominator == 0 {
        Similarity::PERFECT
    } else {
        Similarity::from_ratio(numerator, denominator)
    }
}

fn require(metric: &'static str, actual: Similarity, required: Similarity) -> Result<()> {
    if actual < required {
        return Err(CloneError::AccuracyGate {
            metric,
            actual: actual.permille(),
            required: required.permille(),
        });
    }
    Ok(())
}
