use super::encode::{quoted, uri};
use crate::model::{CloneKind, CloneLocation, ClonePair, CloneReport};
use std::fmt::Write;

#[must_use]
pub fn to_sarif(report: &CloneReport) -> String {
    let mut output = concat!(
        "{\"$schema\":\"https://json.schemastore.org/sarif-2.1.0.json\",",
        "\"version\":\"2.1.0\",\"runs\":[{\"tool\":{\"driver\":{",
        "\"name\":\"weavatrix-clone\",\"semanticVersion\":\""
    )
    .to_owned();
    output.push_str(env!("CARGO_PKG_VERSION"));
    output.push_str(concat!(
        "\",\"informationUri\":\"https://weavatrix.com\",",
        "\"rules\":[",
        "{\"id\":\"WEAVATRIX.CLONE.TYPE1\",\"name\":\"ExactClone\",",
        "\"shortDescription\":{\"text\":\"Exact token clone\"}},",
        "{\"id\":\"WEAVATRIX.CLONE.TYPE2\",\"name\":\"RenamedClone\",",
        "\"shortDescription\":{\"text\":\"Identifier-renamed clone\"}},",
        "{\"id\":\"WEAVATRIX.CLONE.TYPE3\",\"name\":\"NearMissClone\",",
        "\"shortDescription\":{\"text\":\"Near-miss structural clone\"}}",
        "]}},\"results\":["
    ));
    let mut pairs = report.pairs.iter().collect::<Vec<_>>();
    pairs.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    for (index, pair) in pairs.into_iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        result(&mut output, pair);
    }
    output.push_str("]}]}");
    output
}

fn result(output: &mut String, pair: &ClonePair) {
    let (rule, rule_index) = match pair.kind {
        CloneKind::Type1 => ("WEAVATRIX.CLONE.TYPE1", 0),
        CloneKind::Type2 => ("WEAVATRIX.CLONE.TYPE2", 1),
        CloneKind::Type3 => ("WEAVATRIX.CLONE.TYPE3", 2),
    };
    output.push_str("{\"ruleId\":");
    quoted(output, rule);
    let _ = write!(
        output,
        ",\"ruleIndex\":{rule_index},\"level\":\"warning\",\"message\":{{\"text\":"
    );
    quoted(
        output,
        &format!(
            "{:?} clone with {:.1}% similarity; related location contains the paired region",
            pair.kind,
            pair.similarity.percent()
        ),
    );
    output.push_str("},\"locations\":[");
    location(output, &pair.left, None);
    output.push_str("],\"relatedLocations\":[");
    location(output, &pair.right, Some(1));
    output.push_str("],\"partialFingerprints\":{\"weavatrixClonePairId/v1\":");
    quoted(output, &pair.id);
    let evidence = &pair.evidence;
    let _ = write!(
        output,
        concat!(
            "}},\"properties\":{{\"kind\":\"{}\",\"similarityPermille\":{},",
            "\"strictEqual\":{},\"renamedEqual\":{},\"sharedFingerprints\":{},",
            "\"fingerprintJaccardPermille\":{},",
            "\"fingerprintContainmentPermille\":{},\"editDistance\":{},",
            "\"comparedTokens\":{}}}}}"
        ),
        match pair.kind {
            CloneKind::Type1 => "type1",
            CloneKind::Type2 => "type2",
            CloneKind::Type3 => "type3",
        },
        pair.similarity.permille(),
        evidence.strict_equal,
        evidence.renamed_equal,
        evidence.shared_fingerprints,
        evidence.fingerprint_jaccard.permille(),
        evidence.fingerprint_containment.permille(),
        evidence.edit_distance,
        evidence.compared_tokens
    );
}

fn location(output: &mut String, value: &CloneLocation, id: Option<usize>) {
    output.push('{');
    if let Some(id) = id {
        let _ = write!(
            output,
            "\"id\":{id},\"message\":{{\"text\":\"paired clone\"}},"
        );
    }
    output.push_str("\"physicalLocation\":{\"artifactLocation\":{\"uri\":");
    quoted(output, &uri(&value.path));
    let span = value.span;
    let _ = write!(
        output,
        "}},\"region\":{{\"startLine\":{},\"endLine\":{},\"byteOffset\":{},\"byteLength\":{}}}}}}}",
        span.start_line,
        span.end_line,
        span.start_byte,
        span.end_byte.saturating_sub(span.start_byte)
    );
}
