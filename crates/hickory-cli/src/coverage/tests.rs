// Guarantee: docs/guarantees/verification/coverage-retains-its-measured-scope.md
use super::*;
use serde_json::json;
fn project() -> (tempfile::TempDir, PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().canonicalize().unwrap();
    std::fs::write(root.join("app.py"), "print('hello')\n").unwrap();
    (t, root)
}
fn req(value: Value) -> Request {
    serde_json::from_value(value).unwrap()
}
#[tokio::test]
async fn aggregate_import_is_unknown_and_history_is_durable() {
    let (_temp, root) = project();
    let result=dispatch(&root,req(json!({"action":"import","path":".","format":"lcov","raw":"SF:app.py\nDA:1,1\nend_of_record\n","labels":["happy-path"]}))).await.unwrap();
    assert_eq!(result["source_state"], "unknown");
    assert_eq!(result["evidence"]["attribution"], "unavailable");
    assert!(
        dispatch(
            &root,
            req(json!({"action":"compare","id":result["id"],"other":result["id"]}))
        )
        .await
        .is_err()
    );
    let all = dispatch(&root, req(json!({"action":"list"})))
        .await
        .unwrap();
    assert_eq!(all["total"], 1);
}
#[tokio::test]
async fn browser_manual_baseline_checkpoint_stop_and_verdict_are_separate() {
    let (_temp, root) = project();
    let coverage = |count| {
        json!({"app.py":{"path":"app.py","statementMap":{"0":{"start":{"line":1}}},"s":{"0":count},"f":{},"branchMap":{},"b":{}}}).to_string()
    };
    let session=dispatch(&root,req(json!({"action":"manual-start","path":".","raw":coverage(8),"labels":["adversarial","boundary"]}))).await.unwrap();
    let checkpoint = dispatch(
        &root,
        req(
            json!({"action":"checkpoint","id":session["id"],"format":"istanbul","raw":coverage(9)}),
        ),
    )
    .await
    .unwrap();
    assert_eq!(
        checkpoint["evidence"]["report"]["files"]["app.py"]["lines"]["1"],
        1
    );
    assert_eq!(checkpoint["evidence"]["attribution"], "group");
    assert_eq!(
        checkpoint["evidence"]["labels"],
        json!(["adversarial", "boundary"])
    );
    assert!(checkpoint["manual_verdict"].is_null());
    let stopped = dispatch(&root, req(json!({"action":"stop","id":session["id"]})))
        .await
        .unwrap();
    assert_eq!(stopped["status"], "completed");
    let verdict=dispatch(&root,req(json!({"action":"annotate","id":session["id"],"verdict":"failed","notes":"Rejected input displayed the wrong error"}))).await.unwrap();
    assert_eq!(verdict["manual_verdict"], "failed");
    assert_eq!(
        verdict["evidence"]["report"]["files"]["app.py"]["lines"]["1"],
        1
    );
    std::fs::write(root.join("app.py"), "print('changed')\n").unwrap();
    let stale = dispatch(&root, req(json!({"action":"get","id":session["id"]})))
        .await
        .unwrap();
    assert_eq!(stale["source_state"], "historical");
}
#[test]
fn browser_counter_resets_are_not_coverage() {
    let map =
        |n| json!({"a":{"statementMap":{},"branchMap":{},"s":{"0":n},"f":{},"b":{}}}).to_string();
    assert!(runner::browser_delta(&map(2), &map(1)).is_err());
    assert!(runner::browser_delta(&map(2), "{}").is_err());
}
#[test]
fn selectors_are_argv_and_cannot_inject_runner_options() {
    let (_temp, root) = project();
    let mut r = store::new_record(
        &root,
        &root,
        &req(json!({"action":"run","language":"python"})),
    )
    .unwrap();
    assert!(commands::automated(&root, &root, &mut r, &["--collect-only".into()]).is_err());
    let p = commands::automated(
        &root,
        &root,
        &mut r,
        &["test_app.py::test_rejection".into()],
    )
    .unwrap();
    assert_eq!(p.argv.last().unwrap(), "test_app.py::test_rejection");
    assert!(p.argv.iter().any(|a| a == "--cov-branch"));
}
#[test]
fn python_junit_retains_failure_identity_and_source() {
    let (_temp, root) = project();
    let raw = r#"<testsuite><testcase classname="app.TestApp" name="test_bad[x]" file="app.py" line="4"><failure>expected rejection</failure></testcase><testcase classname="app" name="test_skip"><skipped>no account</skipped></testcase></testsuite>"#;
    let tests = results::parse("python", raw, "", &Config::default(), &root).unwrap();
    assert_eq!(tests[0].id, "app.py::TestApp::test_bad[x]");
    assert_eq!(tests[0].status, "failed");
    assert_eq!(tests[0].line, Some(5));
    assert!(tests[0].output.contains("expected rejection"));
    assert_eq!(tests[1].status, "skipped");
}
#[test]
fn csharp_trx_and_rust_results_keep_runner_test_identities() {
    let raw = r#"<TestRun xmlns="http://microsoft.com/schemas/VisualStudio/TeamTest/2010"><TestDefinitions><UnitTest id="abc"><TestMethod className="App.Tests, App" name="Rejects"/></UnitTest></TestDefinitions><Results><UnitTestResult testId="abc" outcome="Failed"><Output><ErrorInfo><Message>wrong error</Message></ErrorInfo></Output></UnitTestResult></Results></TestRun>"#;
    let tests = results::parse("csharp", raw, "", &Config::default(), Path::new(".")).unwrap();
    assert_eq!(tests[0].id, "App.Tests.Rejects");
    assert_eq!(tests[0].status, "failed");
    assert!(tests[0].output.contains("wrong error"));
    let tests=results::parse("rust","","test tests::rejects ... FAILED\ntest tests::accepts ... ok\ntest tests::needs_account ... ignored",&Config::default(),Path::new(".")).unwrap();
    assert_eq!(tests.len(), 3);
    assert_eq!(tests[2].status, "skipped");
}
#[test]
fn javascript_results_and_discovery_preserve_full_names() {
    let raw=json!({"testResults":[{"name":"app.test.ts","assertionResults":[{"fullName":"validation rejects invalid","status":"failed","failureMessages":["expected error"],"location":{"line":12}}]}]}).to_string();
    let tests = results::parse("typescript", &raw, "", &Config::default(), Path::new(".")).unwrap();
    assert_eq!(tests[0].id, "validation rejects invalid");
    assert_eq!(tests[0].line, Some(12));
    let tests = results::discover(
        "typescript",
        r#"[{"name":"rejects invalid","fullName":"validation rejects invalid","file":"app.test.ts"}]"#,
        &Config::default(),
    );
    assert_eq!(tests[0].id, "validation rejects invalid");
}
#[test]
fn paths_and_storage_refuse_escape_and_symlinks() {
    let (_temp, root) = project();
    assert!(safe_path(&root, "../other").is_err());
    assert!(safe_path(&root, "/etc/passwd").is_err());
    assert!(store::load(&root, "../record").is_err());
    #[cfg(unix)]
    {
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.join("outside")).unwrap();
        assert!(safe_path(&root, "outside").is_err());
        std::os::unix::fs::symlink(outside.path(), root.join(".hick-cache")).unwrap();
        assert!(store::list(&root).is_err());
    }
}
#[tokio::test]
async fn imported_sources_cannot_escape_the_workspace() {
    let (_temp, root) = project();
    let outside = tempfile::NamedTempFile::new().unwrap();
    let raw = format!("SF:{}\nDA:1,1\nend_of_record\n", outside.path().display());
    assert!(
        dispatch(
            &root,
            req(json!({"action":"import","path":".","format":"lcov","raw":raw}))
        )
        .await
        .is_err()
    );
}
#[test]
fn generated_coverage_maps_only_matching_bytes_to_document() {
    let (_temp, root) = project();
    let source = "# App\n\n<hick:file path=\"app.py\">\nprint('hello')\n</hick:file>\n";
    std::fs::write(root.join("app.md"), source).unwrap();
    let mut record = store::new_record(
        &root,
        &root,
        &req(json!({"action":"run","language":"python"})),
    )
    .unwrap();
    record.evidence = Some(
        evidence(
            &root,
            &record,
            "lcov",
            "SF:app.py\nDA:1,1\nend_of_record\n",
            "group",
            vec![],
        )
        .unwrap(),
    );
    let map = mapping::capture(&root, &record).unwrap();
    assert_eq!(map["app.py"][&1]["path"], "app.md");
    assert_eq!(map["app.py"][&1]["line"], 4);
    std::fs::write(root.join("app.py"), "different\n").unwrap();
    assert!(mapping::capture(&root, &record).unwrap().is_empty());
}

#[tokio::test]
async fn agent_location_pages_distinguish_reachability_from_counts() {
    let (_temp, root) = project();
    let identity = Identity {
        collector: "coverage.py".into(),
        version: "7".into(),
        configuration: "branches".into(),
        scope: ".".into(),
        source_hashes: snapshot(&root).unwrap(),
    };
    let raw=json!({"meta":{"branch_coverage":false},"files":{"app.py":{"executed_lines":[1],"missing_lines":[2],"excluded_lines":[]}}}).to_string();
    let imported=dispatch(&root,req(json!({"action":"import","path":".","format":"coverage-py","identity":identity,"raw":raw}))).await.unwrap();
    let page = dispatch(
        &root,
        req(json!({"action":"locations","id":imported["id"],"path":"app.py","limit":1})),
    )
    .await
    .unwrap();
    assert_eq!(page["denominator"], 2);
    assert_eq!(page["next"], 1);
    assert_eq!(page["locations"][0]["covered"], true);
    assert!(page["locations"][0]["count"].is_null());
    assert!(page["tests"].is_null());
    assert!(
        dispatch(
            &root,
            req(json!({"action":"locations","id":imported["id"],"metric":"branch"}))
        )
        .await
        .is_err()
    );
}
#[test]
fn a_browser_reload_invalidates_even_larger_counters() {
    let map = |epoch, n| {
        json!({"app.js":{"__hickoryTimeOrigin":epoch,"statementMap":{},"branchMap":{},"s":{"0":n},"f":{},"b":{}}}).to_string()
    };
    assert!(runner::browser_delta(&map(1, 2), &map(2, 3)).is_err());
}
#[tokio::test]
async fn metadata_labels_are_snapshots_not_reinterpreted_from_current_config() {
    let (_temp, root) = project();
    std::fs::write(
        root.join(".hick-tests.json"),
        r#"{"tests":{"app.py::test_x":["happy-path"]}}"#,
    )
    .unwrap();
    let record = store::new_record(
        &root,
        &root,
        &req(json!({"action":"run","language":"python"})),
    )
    .unwrap();
    store::save(&root, &record).unwrap();
    std::fs::write(
        root.join(".hick-tests.json"),
        r#"{"tests":{"app.py::test_x":["adversarial"]}}"#,
    )
    .unwrap();
    let old = store::load(&root, &record.id).unwrap();
    assert_eq!(old.metadata.tests["app.py::test_x"], ["happy-path"]);
    assert_eq!(
        store::view(&root, &record.id).unwrap()["source_state"],
        "historical"
    );
}
#[test]
fn malformed_runner_output_does_not_panic_discovery() {
    assert!(results::discover("typescript", "}\n{\n]\n[", &Config::default()).is_empty());
}
