// Guarantee: docs/guarantees/verification/coverage-retains-its-measured-scope.md
use hick_coverage::*;
use std::collections::BTreeMap;
fn evidence(raw: &str) -> Evidence {
    Evidence {
        identity: Identity {
            collector: "test".into(),
            version: "1".into(),
            configuration: "branch".into(),
            scope: "app".into(),
            source_hashes: BTreeMap::from([("a.rs".into(), "abc".into())]),
        },
        report: parse("lcov", raw).unwrap(),
        labels: vec!["happy-path".into(), "boundary".into()],
        tests: vec![],
        attribution: "group".into(),
    }
}
#[test]
fn comparison_uses_sets_and_keeps_missed_executable_lines() {
    let a = evidence("SF:a.rs\nDA:1,3\nDA:2,1\nDA:3,0\nDA:4,0\nend_of_record\n");
    let b = evidence("SF:a.rs\nDA:1,2\nDA:2,0\nDA:3,1\nDA:4,0\nend_of_record\n");
    let c = compare(&a, &b, "line").unwrap();
    assert_eq!(c.denominator, 4);
    assert_eq!(c.shared.into_iter().collect::<Vec<_>>(), ["a.rs:1"]);
    assert_eq!(c.left_only.into_iter().collect::<Vec<_>>(), ["a.rs:2"]);
    assert_eq!(c.right_only.into_iter().collect::<Vec<_>>(), ["a.rs:3"]);
    assert_eq!(c.missed.into_iter().collect::<Vec<_>>(), ["a.rs:4"]);
}
#[test]
fn incompatible_or_unknown_sources_and_denominators_cannot_compare() {
    let a = evidence("SF:a.rs\nDA:1,1\nend_of_record\n");
    let mut b = a.clone();
    b.identity.version = "2".into();
    assert!(compare(&a, &b, "line").is_err());
    b = a.clone();
    b.identity.source_hashes.clear();
    assert!(compare(&b, &b, "line").is_err());
    b = a.clone();
    b.report.files.get_mut("a.rs").unwrap().lines.insert(2, 0);
    assert!(compare(&a, &b, "line").is_err());
    assert!(compare(&a, &a, "branch").is_err());
}
#[test]
fn lcov_retains_unknown_branch_counts_and_merges_duplicate_records() {
    let r = parse("lcov", "SF:a.ts\nDA:1,2\nBRDA:1,0,0,-\nend_of_record\nSF:a.ts\nDA:1,3\nBRDA:1,0,0,1\nend_of_record\n").unwrap();
    assert_eq!(r.files["a.ts"].lines[&1], 5);
    let b = &r.files["a.ts"].branches.as_ref().unwrap()[0];
    assert!(b.covered);
    assert_eq!(b.count, None);
}
#[test]
fn python_exclusions_and_arcs_remain_distinct() {
    let raw = r#"{"meta":{"branch_coverage":true},"files":{"app.py":{"executed_lines":[1],"missing_lines":[2,3],"excluded_lines":[3],"executed_branches":[[1,2]],"missing_branches":[[1,-1]]}}}"#;
    let r = parse("coverage-py", raw).unwrap();
    let f = &r.files["app.py"];
    assert_eq!(f.lines.len(), 2);
    assert_eq!(f.excluded, [3]);
    let branches = f.branches.as_ref().unwrap();
    assert_eq!(branches.len(), 2);
    assert_eq!(branches[1].id, "1:-1");
    assert!(!branches[1].covered);
    assert_eq!(r.raw, raw);
}
#[test]
fn cobertura_does_not_double_count_method_tables_or_invent_branches() {
    let r = parse("cobertura", r#"<coverage><packages><package><classes><class filename="App.cs"><methods><method><lines><line number="1" hits="80"/></lines></method></methods><lines><line number="1" hits="2" branch="true" condition-coverage="50% (1/2)"/><line number="2" hits="0"/></lines></class></classes></package></packages></coverage>"#).unwrap();
    assert_eq!(r.files["App.cs"].lines[&1], 2);
    assert_eq!(r.files["App.cs"].branches, None);
    assert!(r.raw.contains("50% (1/2)"));
}
#[test]
fn browser_istanbul_statement_and_branch_counters_are_preserved() {
    let r = parse("istanbul", r#"{"app.ts":{"path":"app.ts","statementMap":{"0":{"start":{"line":1}},"1":{"start":{"line":1}}},"s":{"0":0,"1":4},"branchMap":{"0":{"line":1,"locations":[{"start":{"line":1}},{"start":{"line":2}}]}},"b":{"0":[4,0]}}}"#).unwrap();
    let f = &r.files["app.ts"];
    assert_eq!(f.lines[&1], 4);
    assert_eq!(f.branches.as_ref().unwrap()[1].count, Some(0));
}
#[test]
fn malformed_reports_are_not_zero_coverage() {
    for (format, raw) in [
        ("lcov", "SF:a\nDA:0,1\nend_of_record"),
        ("lcov", "SF:a\nDA:1,1"),
        ("coverage-py", "{}"),
        ("cobertura", "<coverage/>"),
        ("istanbul", "{}"),
        ("other", ""),
    ] {
        assert!(parse(format, raw).is_err(), "{format}: {raw}");
    }
}
