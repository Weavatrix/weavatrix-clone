use weavatrix_clone::{
    CloneEvidence, CloneFamily, CloneKind, CloneLocation, ClonePair, CloneReport, CloneStatistics,
    Similarity, SourceSpan, output,
};

#[test]
fn json_is_stable_sorted_and_escaped() {
    let first = pair("b", "src/a b.rs", "src/\"b\".rs", CloneKind::Type2);
    let second = pair("a", "src/c.rs", "src/d.rs", CloneKind::Type1);
    let left = CloneReport {
        pairs: vec![first.clone(), second.clone()],
        families: vec![CloneFamily {
            id: "family:z".to_owned(),
            members: vec![second.right.clone(), second.left.clone()],
            pair_ids: vec!["b".to_owned(), "a".to_owned()],
        }],
        statistics: CloneStatistics {
            source_files: 4,
            verified_pairs: 2,
            ..CloneStatistics::default()
        },
    };
    let right = CloneReport {
        pairs: vec![second, first],
        families: left.families.clone(),
        statistics: left.statistics.clone(),
    };
    let encoded = output::to_json(&left);
    assert_eq!(encoded, output::to_json(&right));
    assert!(encoded.starts_with("{\"schema\":\"https://weavatrix.com/"));
    assert!(encoded.find("\"id\":\"a\"") < encoded.find("\"id\":\"b\""));
    assert!(encoded.contains("src/\\\"b\\\".rs"));
    assert!(encoded.contains("\"sourceFiles\":4"));
    assert!(encoded.contains("\"pairIds\":[\"a\",\"b\"]"));
    assert!(output::JSON_SCHEMA_DOCUMENT.contains(output::JSON_SCHEMA));
    assert!(encoded.ends_with('}'));
}

#[test]
fn sarif_carries_rules_locations_and_stable_fingerprints() {
    let report = CloneReport {
        pairs: vec![
            pair("pair:one", "src/a.rs", "src/b.rs", CloneKind::Type1),
            pair("pair:two", "src/c.rs", "src/d.rs", CloneKind::Type2),
            pair(
                "pair:stable",
                "src/a b.rs",
                "src/right.rs",
                CloneKind::Type3,
            ),
        ],
        ..CloneReport::default()
    };
    let encoded = output::to_sarif(&report);
    assert!(encoded.contains("\"version\":\"2.1.0\""));
    assert!(encoded.contains("\"ruleId\":\"WEAVATRIX.CLONE.TYPE1\""));
    assert!(encoded.contains("\"ruleId\":\"WEAVATRIX.CLONE.TYPE2\""));
    assert!(encoded.contains("\"ruleId\":\"WEAVATRIX.CLONE.TYPE3\""));
    assert!(encoded.contains("\"uri\":\"src/a%20b.rs\""));
    assert!(encoded.contains("\"relatedLocations\":["));
    assert!(encoded.contains("\"weavatrixClonePairId/v1\":\"pair:stable\""));
}

#[test]
fn bigcloneeval_uses_the_official_eight_columns() {
    let report = CloneReport {
        pairs: vec![pair(
            "pair",
            "2/selected/102353.java",
            "2/default/356923.java",
            CloneKind::Type1,
        )],
        ..CloneReport::default()
    };
    assert_eq!(
        output::to_bigcloneeval(&report).unwrap(),
        "selected,102353.java,2,8,default,356923.java,2,8\n"
    );
    let invalid = CloneReport {
        pairs: vec![pair("pair", "src/a.java", "src/b.java", CloneKind::Type1)],
        ..CloneReport::default()
    };
    assert!(output::to_bigcloneeval(&invalid).is_err());
}

fn pair(id: &str, left: &str, right: &str, kind: CloneKind) -> ClonePair {
    ClonePair {
        id: id.to_owned(),
        left: location(left),
        right: location(right),
        kind,
        similarity: Similarity::from_permille(875),
        evidence: CloneEvidence {
            strict_equal: kind == CloneKind::Type1,
            renamed_equal: kind != CloneKind::Type3,
            shared_fingerprints: 7,
            fingerprint_jaccard: Similarity::from_permille(800),
            fingerprint_containment: Similarity::from_permille(900),
            edit_distance: 2,
            compared_tokens: 42,
        },
    }
}

fn location(path: &str) -> CloneLocation {
    CloneLocation {
        fragment_id: format!("{path}#fragment"),
        path: path.to_owned(),
        span: SourceSpan {
            start_byte: 4,
            end_byte: 80,
            start_line: 2,
            end_line: 8,
        },
    }
}
