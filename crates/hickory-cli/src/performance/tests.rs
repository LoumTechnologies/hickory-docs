// Guarantee: docs/guarantees/execution/profiles-keep-their-measured-scope.md
use super::*;
fn req(v: Value) -> Request {
    serde_json::from_value(v).unwrap()
}
fn project() -> (tempfile::TempDir, PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().canonicalize().unwrap();
    std::fs::write(root.join("app.py"), "def hot():\n    return 1\nhot()\n").unwrap();
    (t, root)
}
fn report(file: &str) -> String {
    json!({"elapsed_seconds":1,"functions":[{"name":"hot","file":file,"line":1,"total_calls":3,"primitive_calls":3,"self_seconds":0.1,"cumulative_seconds":0.2}]}).to_string()
}
#[tokio::test]
async fn imports_keep_unknown_identity_and_original_evidence() {
    let (_t, root) = project();
    let run = dispatch(
        &root,
        req(json!({"action":"import","path":".","format":"cprofile-json","raw":report("app.py")})),
    )
    .await
    .unwrap();
    assert_eq!(run["source_state"], "unknown");
    assert!(run["sources"].as_object().unwrap().is_empty());
    assert!(!run["report"]["raw"].as_str().unwrap().is_empty());
    assert!(
        dispatch(
            &root,
            req(json!({"action":"compare","id":run["id"],"other":run["id"]}))
        )
        .await
        .is_err()
    );
}
#[tokio::test]
async fn captured_hashes_bind_to_literate_source_and_stale_edits_remain_historical() {
    let (_t, root) = project();
    std::fs::write(
        root.join("app.md"),
        "# App\n<hick:file path=\"app.py\">\ndef hot():\n    return 1\nhot()\n</hick:file>\n",
    )
    .unwrap();
    let identity = Identity {
        collector: "test".into(),
        version: "1".into(),
        configuration: "function-profile".into(),
        scope: "".into(),
        source_hashes: snapshot(&root).unwrap(),
    };
    let run=dispatch(&root,req(json!({"action":"import","path":".","format":"cprofile-json","raw":report("app.py"),"identity":identity,"labels":["manual"]}))).await.unwrap();
    assert_eq!(run["mappings"]["app.py"]["1"]["path"], "app.md");
    assert_eq!(run["mappings"]["app.py"]["1"]["line"], 3);
    assert!(
        run["sources"]["app.md"]
            .as_str()
            .unwrap()
            .contains("def hot")
    );
    let page = dispatch(
        &root,
        req(json!({"action":"locations","id":run["id"],"limit":1})),
    )
    .await
    .unwrap();
    assert_eq!(page["rows"][0]["total_calls"], 3);
    assert!(page["rows"][0]["samples"].is_null());
    std::fs::write(root.join("app.py"), "edited\n").unwrap();
    assert_eq!(
        store::view(&root, run["id"].as_str().unwrap(), true).unwrap()["source_state"],
        "historical"
    );
}
#[test]
fn browser_generated_code_requires_exact_bytes_or_verified_source_map() {
    let (_t, root) = project();
    let request = req(json!({"action":"browser-start"}));
    let mut record = fresh(&root, &root, &request, "manual-browser").unwrap();
    let raw=json!({"nodes":[{"id":1,"callFrame":{"functionName":"hot","url":format!("file://{}",root.join("app.py").display()),"lineNumber":0,"columnNumber":0,"scriptId":"1"}}],"startTime":0,"endTime":1000,"samples":[1],"timeDeltas":[1000]}).to_string();
    record.report = Some(hick_profile::parse("chrome-cpu", &raw).unwrap());
    let bad = BTreeMap::from([(
        "1".into(),
        browser::Script {
            url: format!("file://{}", root.join("app.py").display()),
            source: Some("transformed bytes".into()),
            ..Default::default()
        },
    )]);
    resolve::bind(&root, &mut record, Some(&bad)).unwrap();
    assert!(record.sources.is_empty());
    let good = BTreeMap::from([(
        "1".into(),
        browser::Script {
            url: format!("file://{}", root.join("app.py").display()),
            source: Some(std::fs::read_to_string(root.join("app.py")).unwrap()),
            ..Default::default()
        },
    )]);
    resolve::bind(&root, &mut record, Some(&good)).unwrap();
    assert!(record.sources.contains_key("app.py"));
}
#[tokio::test]
async fn before_after_comparisons_preserve_revision_changes_and_missing_rows() {
    let (_t, root) = project();
    let mut a = fresh(&root, &root, &req(json!({"action":"run"})), "native").unwrap();
    a.identity.collector = "test".into();
    a.identity.version = "1".into();
    a.identity.configuration = "same workload".into();
    a.report = Some(hick_profile::parse("cprofile-json", &report("app.py")).unwrap());
    a.status = "completed".into();
    store::save(&root, &a).unwrap();
    let mut b = a.clone();
    b.id = "a".repeat(32);
    b.identity
        .source_hashes
        .insert("app.py".into(), "different revision".into());
    b.report.as_mut().unwrap().threads[0].rows.clear();
    store::save(&root, &b).unwrap();
    let comparison = dispatch(
        &root,
        req(json!({"action":"compare","id":a.id,"other":b.id})),
    )
    .await
    .unwrap();
    assert_eq!(comparison["same_sources"], false);
    assert!(comparison["rows"][0]["right"].is_null());
    assert!(comparison["rows"][0]["self_delta"].is_null());
    b.identity.configuration = "different sampling".into();
    store::save(&root, &b).unwrap();
    assert!(
        dispatch(
            &root,
            req(json!({"action":"compare","id":a.id,"other":b.id}))
        )
        .await
        .is_err()
    );
}
#[tokio::test]
async fn unsafe_paths_and_remote_debugger_endpoints_are_refused() {
    let (_t, root) = project();
    assert!(
        dispatch(&root, req(json!({"action":"get","id":"../outside"})))
            .await
            .is_err()
    );
    assert!(
        dispatch(&root, req(json!({"action":"inspect","path":"../"})))
            .await
            .is_err()
    );
    assert!(
        browser::connect("http://example.com:9222", None)
            .await
            .is_err()
    );
    #[cfg(unix)]
    {
        let out = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(out.path(), root.join(".hick-cache")).unwrap();
        assert!(store::dir(&root).is_err());
    }
}
#[test]
fn native_profile_report_traversal_and_symlink_escape_are_refused_before_writes() {
    let (_t, root) = project();
    #[cfg(unix)]
    {
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.join("outside")).unwrap();
        let config = json!({"bad":{"argv":["not-run"],"format":"speedscope","report":"outside/new/report.json","collector":"test","version":"1"}});
        std::fs::write(root.join(".hick-profile.json"), config.to_string()).unwrap();
        let request = req(json!({"action":"run","path":".","profile":"bad"}));
        let mut run = fresh(&root, &root, &request, "native").unwrap();
        assert!(native::plan(&root, &root, &request, &mut run).is_err());
        assert!(!outside.path().join("new").exists());
    }
    std::fs::write(root.join(".hick-profile.json"),json!({"bad":{"argv":["not-run"],"format":"speedscope","report":"../outside/report.json","collector":"test","version":"1"}}).to_string()).unwrap();
    let request = req(json!({"action":"run","path":".","profile":"bad"}));
    let mut run = fresh(&root, &root, &request, "native").unwrap();
    assert!(native::plan(&root, &root, &request, &mut run).is_err());
}
