//! Daemon POST serialization and the desktop client proxy must share one write act.
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct Processes {
    client: Option<Child>,
    daemon: Child,
}
impl Drop for Processes {
    fn drop(&mut self) {
        if let Some(client) = &mut self.client {
            let _ = client.kill();
            let _ = client.wait();
        }
        let _ = self.daemon.kill();
        let _ = self.daemon.wait();
    }
}
fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
async fn wait_file(path: &Path, child: &mut Child) {
    let deadline = Instant::now() + Duration::from_secs(120);
    while !path.exists() {
        assert!(
            child.try_wait().unwrap().is_none(),
            "process exited before {}",
            path.display()
        );
        assert!(
            Instant::now() < deadline,
            "startup timed out: {}",
            path.display()
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
async fn post(http: &reqwest::Client, base: &str, path: &str, body: Value) -> (u16, Value) {
    let response = http
        .post(format!("{base}{path}"))
        .json(&body)
        .send()
        .await
        .expect(
            "bounded daemon/proxy request must finish instead of waiting for its own write lock",
        );
    let status = response.status().as_u16();
    (status, response.json().await.unwrap())
}

// A separate process gives the client its own isolated environment without
// mutating the test runner's process environment. It runs the actual client router.
#[test]
#[ignore = "child-process entry point, launched only by the exact daemon test"]
fn merged_proxy_worker() {
    let root = PathBuf::from(std::env::var_os("HICKORY_MERGED_TEST_ROOT").unwrap());
    let ready = PathBuf::from(std::env::var_os("HICKORY_MERGED_TEST_READY").unwrap());
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let connection = hickory_cli::engine::connect(hickory_cli::engine::Attach::new(
            root,
            hickory_cli::ExecutorChoice::Local,
        ))
        .await
        .unwrap();
        let _guard = connection.guard();
        let router = hickory_cli::engine::client_router(connection, None, String::new(), None);
        std::fs::write(ready, format!("http://{}", listener.local_addr().unwrap())).unwrap();
        axum::serve(listener, router.into_make_service())
            .await
            .unwrap();
    });
}

fn seed_guarded_routes(root: &Path) -> String {
    use hickory_agent::{HickSessionLog, SessionEvent, SessionLog};
    let a = hick_lang::ProjectTree::default();
    let mut b = a.clone();
    b.insert(
        "guard.txt",
        hick_lang::ProjectEntry::file(b"captured B\n".to_vec()),
    )
    .unwrap();
    let response = hick_lang::transition_response(&a, &b, "Persisted test capture").unwrap();
    let sources = std::collections::BTreeMap::from([("response.md".to_owned(), response.clone())]);
    let project = hick_lang::evaluate_project("response.md", &sources, &a).unwrap();
    std::fs::write(root.join("guard.txt"), "newer than captured B\n").unwrap();
    std::fs::write(root.join("project.md"), "<hick:project/>\nnewer source\n").unwrap();
    std::fs::write(
        root.join("sample.md"),
        "<hick:file path=\"generated.txt\">generated line\n</hick:file>\n",
    )
    .unwrap();
    let session = root.join("sessions/guard.hick");
    std::fs::create_dir_all(session.parent().unwrap()).unwrap();
    let log = HickSessionLog::create_for(&session, Some(&root.join("note.md"))).unwrap();
    log.record(SessionEvent::UserTurn {
        text: "Synthetic captured boundary",
        turn: "guard-turn",
        parent: None,
        provider: "fixture",
        model: "fixture",
    });
    log.record(SessionEvent::Assistant {
        prose: "Accepted",
        action: None,
        reasoning: None,
    });
    log.record(SessionEvent::End);
    log.check().unwrap();
    drop(log);
    let capture = json!({"turn":"guard-turn","capture":{"workspace":null,"root":root,"entry":"note.md","documents":{},"inventory":project.inventory,"starting":hick_lang::ProjectSnapshot::capture(&a),"completed":hick_lang::ProjectSnapshot::capture(&b),"ambiguous":[],"error":null,"response":response,"artifact":{"entry":"response.md","sources":sources,"expected":"fixture"},"session":session}});
    let text = std::fs::read_to_string(&session).unwrap();
    let context = serde_json::to_string(&capture)
        .unwrap()
        .replace('<', "\\u003c");
    std::fs::write(&session, format!("{}<hick:context kind=\"literate-project-capture\">{context}</hick:context>\n</hick:session>\n",text.trim_end().strip_suffix("</hick:session>").unwrap())).unwrap();
    response
}

#[tokio::test(flavor = "multi_thread")]
async fn daemon_client_proxy_applies_only_selected_worktree_and_undoes_without_deadlock() {
    let temp = tempfile::tempdir().unwrap();
    let parent = temp.path().canonicalize().unwrap();
    let root = parent.join("repo");
    let review = parent.join("review");
    let state = parent.join("state");
    std::fs::create_dir(&root).unwrap();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.name", "Focused QA"]);
    git(&root, &["config", "user.email", "qa@example.com"]);
    git(&root, &["config", "commit.gpgsign", "false"]);
    const ORIGINAL: &[u8] = b"head\nshared text\ntail\n";
    const DOCUMENT: &str =
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">A note</hick:doc>\n";
    std::fs::write(root.join("note.md"), DOCUMENT).unwrap();
    std::fs::write(root.join("shared.txt"), ORIGINAL).unwrap();
    git(&root, &["add", "note.md", "shared.txt"]);
    git(&root, &["commit", "-q", "--no-verify", "-m", "fixture"]);
    git(
        &root,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "review",
            review.to_str().unwrap(),
        ],
    );
    let response_source = seed_guarded_routes(&root);
    let daemon = Command::new(env!("CARGO_BIN_EXE_hick"))
        .arg("__engine")
        .env("HICKORY_STATE_DIR", &state)
        .env("HICKORY_EXECUTOR", "local")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut processes = Processes {
        client: None,
        daemon,
    };
    wait_file(&state.join("engine/endpoint.json"), &mut processes.daemon).await;
    let ready = parent.join("proxy-ready");
    processes.client = Some(
        Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", "merged_proxy_worker", "--nocapture"])
            .env("HICKORY_STATE_DIR", &state)
            .env("HICKORY_EXECUTOR", "local")
            .env("HICKORY_MERGED_TEST_ROOT", &root)
            .env("HICKORY_MERGED_TEST_READY", &ready)
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    wait_file(&ready, processes.client.as_mut().unwrap()).await;
    let base = std::fs::read_to_string(ready).unwrap();
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let view: Value = http
        .get(format!("{base}/api/merged?path=shared.txt"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(view["repository"], true, "{view}");
    let region = view["regions"]
        .as_array()
        .unwrap()
        .iter()
        .position(|r| r["kind"] == "shared")
        .unwrap();
    let body = json!({"path":"shared.txt","targets":["repo","review"],"writable":["review"],"expected":view["snapshot"],"region":region,"text":"review-only edit\n","route":"just-here","source":"review"});
    let non_target_hash = hick_context_protocol::digest(ORIGINAL);
    let (status, applied) = post(&http, &base, "/api/merged/write", body.clone()).await;
    assert_eq!(status, 200, "{applied}");
    assert_eq!(applied["landed"], 1, "{applied}");
    assert_eq!(applied["targets"].as_array().unwrap().len(), 1);
    assert_eq!(applied["targets"][0]["source"], "review");
    assert_eq!(applied["targets"][0]["ok"], true);
    assert_eq!(
        std::fs::read(review.join("shared.txt")).unwrap(),
        b"review-only edit\n"
    );
    assert_eq!(
        hick_context_protocol::digest(&std::fs::read(root.join("shared.txt")).unwrap()),
        non_target_hash
    );
    // Each sibling inline write route must reach its post-lock safety guard,
    // rather than deadlocking under the daemon's already-held write act.
    let project_id =
        hickory_cli::serve::store::DocIndex::empty_shared(&root).id_for_path("project.md");
    let empty = hick_lang::ProjectSnapshot::capture(&hick_lang::ProjectTree::default());
    let (status, refused) = post(&http, &base, &format!("/api/docs/{project_id}/project/apply"), json!({"baseline":{"project.md":"<hick:project/>\n"},"documents":{"project.md":"<hick:project/>\n"},"expected":empty})).await;
    assert_ne!(status, 200, "stale project source must refuse: {refused}");
    assert!(
        refused.to_string().contains("changed"),
        "post-lock publication guard: {refused}"
    );
    let (status, undo) = post(
        &http,
        &base,
        &format!("/api/docs/{project_id}/project/undo"),
        json!({"undo":[{"path":"project.md","before":"<hick:project/>\n","after":"stale source"}]}),
    )
    .await;
    assert_eq!(status, 200, "{undo}");
    assert_eq!(undo["targets"][0]["ok"], false, "{undo}");
    let (status, sample) = post(
        &http,
        &base,
        "/api/samples",
        json!({"path":"generated.txt","from":1,"to":1,"expected_output_hash":"stale"}),
    )
    .await;
    assert_eq!(status, 409, "post-lock selected-output guard: {sample}");
    let doc_id = hickory_cli::serve::store::DocIndex::empty_shared(&root).id_for_path("note.md");
    let turns: Value = http
        .get(format!("{base}/api/docs/{doc_id}/agent/turns"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        turns["turns"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["id"] == "guard-turn"),
        "persisted test turn must hydrate: {turns}"
    );
    let (status, draft) = post(
        &http,
        &base,
        &format!("/api/docs/{doc_id}/agent/projects/guard-turn/draft/apply"),
        json!({"sources":{"response.md":response_source},"context":{"buffers":[]}}),
    )
    .await;
    assert_eq!(status, 409, "{draft}");
    assert!(
        draft
            .to_string()
            .contains("Destinations changed since accepted B"),
        "must reach post-lock B freshness guard: {draft}"
    );

    // A stale displayed comparison cannot be replayed after the target changes.
    let (status, stale) = post(&http, &base, "/api/merged/write", body.clone()).await;
    assert_eq!(status, 409, "{stale}");
    assert_eq!(
        std::fs::read(review.join("shared.txt")).unwrap(),
        b"review-only edit\n"
    );
    let (status, undone) = post(
        &http,
        &base,
        "/api/merged/undo",
        json!({"path":"shared.txt","undo":applied["undo"]}),
    )
    .await;
    assert_eq!(status, 200, "{undone}");
    assert_eq!(undone["restored"], 1, "{undone}");
    assert_eq!(std::fs::read(review.join("shared.txt")).unwrap(), ORIGINAL);
    assert_eq!(std::fs::read(root.join("shared.txt")).unwrap(), ORIGINAL);
    let (_, applied) = post(&http, &base, "/api/merged/write", body).await;
    assert_eq!(applied["landed"], 1, "{applied}");
    std::fs::write(review.join("shared.txt"), b"newer external edit\n").unwrap();
    let (status, stale_undo) = post(
        &http,
        &base,
        "/api/merged/undo",
        json!({"path":"shared.txt","undo":applied["undo"]}),
    )
    .await;
    assert_eq!(status, 200, "{stale_undo}");
    assert_eq!(stale_undo["targets"][0]["ok"], false, "{stale_undo}");
    assert_eq!(
        std::fs::read(review.join("shared.txt")).unwrap(),
        b"newer external edit\n"
    );
    assert_eq!(
        hick_context_protocol::digest(&std::fs::read(root.join("shared.txt")).unwrap()),
        non_target_hash
    );
    assert_eq!(
        std::fs::read_to_string(root.join("note.md")).unwrap(),
        DOCUMENT
    );
}
