use super::encode::quoted;
use crate::{CloneEvidence, CloneFamily, CloneLocation, ClonePair, CloneReport, CloneStatistics};
use std::fmt::Write;

pub const JSON_SCHEMA: &str = "https://weavatrix.com/schemas/clone-report/v1";
pub const JSON_SCHEMA_DOCUMENT: &str = include_str!("../../schemas/clone-report-v1.schema.json");

#[must_use]
pub fn to_json(report: &CloneReport) -> String {
    let mut output = String::new();
    output.push_str("{\"schema\":");
    quoted(&mut output, JSON_SCHEMA);
    output.push_str(",\"version\":1,\"pairs\":[");
    let mut pairs = report.pairs.iter().collect::<Vec<_>>();
    pairs.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    join(&mut output, pairs, pair);
    output.push_str("],\"families\":[");
    let mut families = report.families.iter().collect::<Vec<_>>();
    families.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    join(&mut output, families, family);
    output.push_str("],\"statistics\":");
    statistics(&mut output, &report.statistics);
    output.push('}');
    output
}

fn pair(output: &mut String, pair: &ClonePair) {
    output.push_str("{\"id\":");
    quoted(output, &pair.id);
    output.push_str(",\"kind\":");
    quoted(
        output,
        match pair.kind {
            crate::CloneKind::Type1 => "type1",
            crate::CloneKind::Type2 => "type2",
            crate::CloneKind::Type3 => "type3",
        },
    );
    let _ = write!(
        output,
        ",\"similarityPermille\":{},\"left\":",
        pair.similarity.permille()
    );
    location(output, &pair.left);
    output.push_str(",\"right\":");
    location(output, &pair.right);
    output.push_str(",\"evidence\":");
    evidence(output, &pair.evidence);
    output.push('}');
}

fn location(output: &mut String, location: &CloneLocation) {
    output.push_str("{\"fragmentId\":");
    quoted(output, &location.fragment_id);
    output.push_str(",\"path\":");
    quoted(output, &location.path);
    let span = location.span;
    let _ = write!(
        output,
        ",\"span\":{{\"startByte\":{},\"endByte\":{},\"startLine\":{},\"endLine\":{}}}}}",
        span.start_byte, span.end_byte, span.start_line, span.end_line
    );
}

fn evidence(output: &mut String, value: &CloneEvidence) {
    let _ = write!(
        output,
        concat!(
            "{{\"strictEqual\":{},\"renamedEqual\":{},",
            "\"sharedFingerprints\":{},\"fingerprintJaccardPermille\":{},",
            "\"fingerprintContainmentPermille\":{},\"editDistance\":{},",
            "\"comparedTokens\":{}}}"
        ),
        value.strict_equal,
        value.renamed_equal,
        value.shared_fingerprints,
        value.fingerprint_jaccard.permille(),
        value.fingerprint_containment.permille(),
        value.edit_distance,
        value.compared_tokens
    );
}

fn family(output: &mut String, family: &CloneFamily) {
    output.push_str("{\"id\":");
    quoted(output, &family.id);
    output.push_str(",\"members\":[");
    let mut members = family.members.iter().collect::<Vec<_>>();
    members.sort_unstable();
    join(output, members, location);
    output.push_str("],\"pairIds\":[");
    let mut pair_ids = family.pair_ids.iter().collect::<Vec<_>>();
    pair_ids.sort_unstable();
    join(output, pair_ids, |output, id| quoted(output, id));
    output.push_str("]}");
}

fn statistics(output: &mut String, value: &CloneStatistics) {
    let _ = write!(
        output,
        concat!(
            "{{\"sourceFiles\":{},\"sourceTokens\":{},\"inputFragments\":{},",
            "\"analyzedFragments\":{},\"skippedSmallFragments\":{},\"tokens\":{},",
            "\"fingerprints\":{},\"candidatePairs\":{},\"exactBlockCandidates\":{},",
            "\"verifiedPairs\":{},\"suppressedBuckets\":{},",
            "\"suppressedExactBuckets\":{}}}"
        ),
        value.source_files,
        value.source_tokens,
        value.input_fragments,
        value.analyzed_fragments,
        value.skipped_small_fragments,
        value.tokens,
        value.fingerprints,
        value.candidate_pairs,
        value.exact_block_candidates,
        value.verified_pairs,
        value.suppressed_buckets,
        value.suppressed_exact_buckets
    );
}

fn join<T>(
    output: &mut String,
    values: impl IntoIterator<Item = T>,
    mut render: impl FnMut(&mut String, T),
) {
    let mut first = true;
    for value in values {
        if !first {
            output.push(',');
        }
        first = false;
        render(output, value);
    }
}
