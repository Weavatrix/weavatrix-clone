use crate::args::OutputKind;
use std::{fmt::Write, time::Duration};
use weavatrix_clone::{CloneKind, CloneReport, output};

pub(crate) fn render(
    report: &CloneReport,
    output_kind: OutputKind,
    summary_only: bool,
    elapsed: Duration,
) -> Result<String, String> {
    match output_kind {
        OutputKind::Text => Ok(text(report, summary_only, elapsed)),
        OutputKind::Json => Ok(format!("{}\n", output::to_json(report))),
        OutputKind::Sarif => Ok(format!("{}\n", output::to_sarif(report))),
        OutputKind::BigCloneEval => {
            output::to_bigcloneeval(report).map_err(|error| error.to_string())
        }
    }
}

fn text(report: &CloneReport, summary_only: bool, elapsed: Duration) -> String {
    let mut output = String::new();
    if !summary_only {
        for pair in &report.pairs {
            let _ = writeln!(
                output,
                "{}:{}-{} ~ {}:{}-{} type={:?} similarity={:.1}%",
                pair.left.path,
                pair.left.span.start_line,
                pair.left.span.end_line,
                pair.right.path,
                pair.right.span.start_line,
                pair.right.span.end_line,
                pair.kind,
                pair.similarity.percent()
            );
        }
    }
    let type1 = count(report, CloneKind::Type1);
    let type2 = count(report, CloneKind::Type2);
    let type3 = count(report, CloneKind::Type3);
    let _ = writeln!(
        output,
        concat!(
            "files={} fragments={} candidates={} pairs={} families={} ",
            "type1={} type2={} type3={} elapsed_ms={:.3}"
        ),
        report.statistics.source_files,
        report.statistics.analyzed_fragments,
        report.statistics.candidate_pairs,
        report.pairs.len(),
        report.families.len(),
        type1,
        type2,
        type3,
        elapsed.as_secs_f64() * 1_000.0
    );
    output
}

fn count(report: &CloneReport, kind: CloneKind) -> usize {
    report.pairs.iter().filter(|pair| pair.kind == kind).count()
}
