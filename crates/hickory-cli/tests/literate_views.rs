//! Guarantees: authoring/a-literate-view-writes-through-to-ordinary-source.md
//! and editing/a-comparison-keeps-current-code-editable.md; git/bisect-keeps-candidates-isolated.md.
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

struct App {
    root: PathBuf,
    base: String,
    _dir: tempfile::TempDir,
}
fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
async fn serve(root: &Path) -> String {
    let prepared = prepare(ServeOptions {
        target: root.into(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    format!("http://127.0.0.1:{port}/api")
}
async fn app() -> App {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.email", "tests@example.invalid"]);
    git(&root, &["config", "user.name", "Test"]);
    std::fs::write(root.join("code.py"), "first = 'é'\nsecond = 2\n").unwrap();
    std::fs::write(root.join("other.txt"), "no trailing newline").unwrap();
    std::fs::write(
        root.join("story.md"),
        "# Persistent\n\n<hick:file path=\"out.py\">\nvalue = 1\n</hick:file>\n",
    )
    .unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "initial"]);
    let base = serve(&root).await;
    App {
        root,
        base,
        _dir: dir,
    }
}
async fn request(base: &str, method: &str, route: &str, body: Value) -> (u16, Value) {
    let response = reqwest::Client::new()
        .request(method.parse().unwrap(), format!("{base}{route}"))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = response.status().as_u16();
    let text = response.text().await.unwrap();
    (
        status,
        serde_json::from_str(&text).unwrap_or_else(|_| json!({"text":text})),
    )
}
async fn ok(base: &str, method: &str, route: &str, body: Value) -> Value {
    let (status, v) = request(base, method, route, body).await;
    assert_eq!(status, 200, "{route}: {v}");
    v
}
async fn create(a: &App) -> Value {
    ok(
        &a.base,
        "POST",
        "/representations",
        json!({"backing":{"kind":"files","paths":["code.py","other.txt"]}}),
    )
    .await
}
fn endpoint(v: &Value) -> String {
    format!("/representations/{}", v["id"].as_str().unwrap())
}

#[tokio::test]
async fn disposable_arrangement_exact_edits_conflicts_refresh_and_personal_persistence() {
    let a = app().await;
    let before = git(&a.root, &["status", "--porcelain"]);
    let v = create(&a).await;
    let ep = endpoint(&v);
    assert_eq!(git(&a.root, &["status", "--porcelain"]), before);
    let code = std::fs::read_to_string(a.root.join("code.py")).unwrap();
    let split = code.find("second").unwrap();
    let v=ok(&a.base,"POST",&format!("{ep}/arrange"),json!({"revision":v["revision"],"sections":[
        {"path":"code.py","from":split,"to":code.len(),"heading":"Second first","explanation":"A test explanation."},
        {"path":"other.txt","from":0,"to":19,"heading":"Other"},
        {"path":"code.py","from":0,"to":split,"heading":"First second"}
    ]})).await;
    assert!(
        v["source"].as_str().unwrap().find("second =").unwrap()
            < v["source"].as_str().unwrap().find("first =").unwrap()
    );
    assert_eq!(git(&a.root, &["status", "--porcelain"]), before);
    let source = v["source"]
        .as_str()
        .unwrap()
        .replace("second = 2", "second = 3");
    let edited = ok(
        &a.base,
        "PUT",
        &ep,
        json!({"revision":v["revision"],"source":source}),
    )
    .await;
    assert!(
        std::fs::read_to_string(a.root.join("code.py"))
            .unwrap()
            .contains("second = 3")
    );
    assert_eq!(edited["explanation_stale"], true);
    assert_eq!(
        request(
            &a.base,
            "PUT",
            &ep,
            json!({"revision":v["revision"],"source":v["source"]})
        )
        .await
        .0,
        409
    );
    std::fs::write(
        a.root.join("code.py"),
        code.replace("second = 2", "second = 4"),
    )
    .unwrap();
    assert_eq!(
        request(
            &a.base,
            "PUT",
            &ep,
            json!({"revision":edited["revision"],"source":edited["source"]})
        )
        .await
        .0,
        409
    );
    let refreshed = ok(&a.base, "POST", &format!("{ep}/refresh"), json!({})).await;
    assert!(refreshed["source"].as_str().unwrap().contains("second = 4"));
    assert!(
        refreshed["source"]
            .as_str()
            .unwrap()
            .contains("A test explanation.")
    );
    let kept = ok(&a.base, "POST", &format!("{ep}/keep"), json!({})).await;
    let path = Path::new(kept["path"].as_str().unwrap());
    assert!(!path.starts_with(&a.root));
    assert_eq!(path.extension().unwrap(), "md");
    let restarted = serve(&a.root).await;
    assert_eq!(
        ok(&restarted, "GET", &ep, json!({})).await["source"],
        refreshed["source"]
    );
    ok(&restarted, "DELETE", &ep, json!({})).await;
    assert!(!path.exists());
}

#[tokio::test]
async fn comparisons_read_git_without_checkout_and_persistent_edits_write_the_document() {
    let a = app().await;
    let v = create(&a).await;
    let ep = endpoint(&v);
    let source = v["source"]
        .as_str()
        .unwrap()
        .replace("second = 2", "second = 9");
    let v = ok(
        &a.base,
        "PUT",
        &ep,
        json!({"revision":v["revision"],"source":source}),
    )
    .await;
    let status = git(&a.root, &["status", "--porcelain"]);
    let head = git(&a.root, &["rev-parse", "HEAD"]);
    let diff = ok(
        &a.base,
        "GET",
        &format!("{ep}/compare?base=HEAD"),
        json!({}),
    )
    .await;
    assert!(diff["source"].as_str().unwrap().contains("second = 2"));
    assert_eq!(diff["target_source"], v["source"]);
    assert_eq!(diff["editable"], true);
    assert_eq!(
        ok(
            &a.base,
            "GET",
            &format!("{ep}/compare?base=HEAD&target=INDEX"),
            json!({})
        )
        .await["editable"],
        false
    );
    assert_eq!(git(&a.root, &["status", "--porcelain"]), status);
    assert_eq!(git(&a.root, &["rev-parse", "HEAD"]), head);
    let docs = ok(&a.base, "GET", "/projects/local/docs", json!({})).await;
    let doc = docs
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["path"] == "story.md")
        .unwrap();
    let v = ok(
        &a.base,
        "POST",
        "/representations",
        json!({"backing":{"kind":"document","doc_id":doc["id"]}}),
    )
    .await;
    ok(&a.base,"PUT",&endpoint(&v),json!({"revision":v["revision"],"source":v["source"].as_str().unwrap().replace("value = 1","value = 2")})).await;
    assert!(
        std::fs::read_to_string(a.root.join("story.md"))
            .unwrap()
            .contains("value = 2")
    );
}

#[tokio::test]
async fn invalid_and_ambiguous_views_never_publish_code() {
    let a = app().await;
    let v = create(&a).await;
    let ep = endpoint(&v);
    let before = git(&a.root, &["diff"]);
    for source in [
        "<hick:file path=\"../escape\">bad</hick:file>",
        "<hick:exec id=\"run\">touch bad</hick:exec>",
        "plain prose",
    ] {
        let (status, _) = request(
            &a.base,
            "PUT",
            &ep,
            json!({"revision":v["revision"],"source":source}),
        )
        .await;
        assert!(status >= 400);
        assert_eq!(git(&a.root, &["diff"]), before);
    }
    assert!(request(&a.base,"POST",&format!("{ep}/arrange"),json!({"revision":v["revision"],"sections":[{"path":"code.py","from":0,"to":1,"heading":"Incomplete"}]})).await.0>=400);
    assert_eq!(git(&a.root, &["diff"]), before);
}

#[tokio::test]
async fn bisect_finds_regression_without_rewriting_inspected_candidates_or_the_checkout() {
    let a = app().await;
    let good = git(&a.root, &["rev-parse", "HEAD"]);
    let mut commits = vec![good.clone()];
    for n in 1..=6 {
        std::fs::write(a.root.join("code.py"), format!("version = {n}\n")).unwrap();
        git(&a.root, &["add", "code.py"]);
        git(&a.root, &["commit", "-qm", &format!("version {n}")]);
        commits.push(git(&a.root, &["rev-parse", "HEAD"]));
    }
    let template = ok(
        &a.base,
        "POST",
        "/representations",
        json!({"backing":{"kind":"files","paths":["code.py"]}}),
    )
    .await;
    let template=ok(&a.base,"POST",&format!("{}/arrange",endpoint(&template)),json!({"revision":template["revision"],"sections":[{"path":"code.py","from":0,"to":template["files"][0]["content"].as_str().unwrap().len(),"heading":"Version behavior","explanation":"Stable reading across candidates."}]})).await;
    std::fs::write(a.root.join("other.txt"), "original dirty checkout").unwrap();
    git(&a.root, &["add", "other.txt"]);
    let initial = git(&a.root, &["diff", "--cached"]);
    let head = git(&a.root, &["rev-parse", "HEAD"]);
    let mut s = ok(
        &a.base,
        "POST",
        "/git/bisect",
        json!({"good":good,"bad":head}),
    )
    .await;
    let ep = format!("/git/bisect/{}", s["id"].as_str().unwrap());
    let worktree = PathBuf::from(s["worktree"].as_str().unwrap());
    let candidate = s["candidate"].clone();
    let candidate_app = serve(&worktree).await;
    let reading = ok(&candidate_app, "GET", &endpoint(&template), json!({})).await;
    assert!(
        reading["source"]
            .as_str()
            .unwrap()
            .contains("Stable reading across candidates.")
    );
    assert_eq!(
        reading["files"][0]["content"].as_str().unwrap(),
        std::fs::read_to_string(worktree.join("code.py")).unwrap()
    );
    std::fs::write(worktree.join("code.py"), "experiment\n").unwrap();
    assert_eq!(
        request(
            &a.base,
            "POST",
            &format!("{ep}/mark"),
            json!({"candidate":candidate,"verdict":"good"})
        )
        .await
        .0,
        409
    );
    let preserved = ok(&a.base, "POST", &format!("{ep}/restore"), json!({})).await;
    assert!(Path::new(preserved["patch"].as_str().unwrap()).exists());
    assert_eq!(
        std::fs::read_to_string(worktree.join("code.py")).unwrap(),
        "experiment\n"
    );
    s = preserved["session"].clone();
    let old = PathBuf::from(s["worktree"].as_str().unwrap());
    let old_head = git(&old, &["rev-parse", "HEAD"]);
    for _ in 0..10 {
        if !s["outcome"].is_null() {
            break;
        }
        let n = commits
            .iter()
            .position(|c| c == s["candidate"].as_str().unwrap())
            .unwrap();
        s = ok(
            &a.base,
            "POST",
            &format!("{ep}/mark"),
            json!({"candidate":s["candidate"],"verdict":if n>=3 {"bad"} else {"good"}}),
        )
        .await;
    }
    assert_eq!(s["outcome"], "first_bad");
    assert_eq!(s["candidate"], commits[3]);
    assert_eq!(git(&old, &["rev-parse", "HEAD"]), old_head);
    assert_eq!(git(&a.root, &["rev-parse", "HEAD"]), head);
    assert_eq!(git(&a.root, &["diff", "--cached"]), initial);
    let restarted = serve(&a.root).await;
    assert!(
        !ok(&restarted, "GET", "/git/bisect", json!({})).await["sessions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let ended = ok(&restarted, "DELETE", &ep, json!({})).await;
    for path in ended["retained_inspections"].as_array().unwrap() {
        git(
            &a.root,
            &["worktree", "remove", "--force", path.as_str().unwrap()],
        );
    }
}

#[tokio::test]
async fn empty_and_deleted_files_have_exact_empty_current_sides_and_can_be_restored() {
    let a = app().await;
    std::fs::remove_file(a.root.join("code.py")).unwrap();
    let v = ok(
        &a.base,
        "POST",
        "/representations",
        json!({"backing":{"kind":"files","paths":["code.py"]}}),
    )
    .await;
    assert_eq!(v["files"][0]["content"], "");
    assert!(!a.root.join("code.py").exists());
    let ep = endpoint(&v);
    let diff = ok(
        &a.base,
        "GET",
        &format!("{ep}/compare?base=HEAD"),
        json!({}),
    )
    .await;
    assert!(diff["source"].as_str().unwrap().contains("first ="));
    assert!(!a.root.join("code.py").exists());
    ok(
        &a.base,
        "PUT",
        &ep,
        json!({"revision":v["revision"],"source":diff["source"]}),
    )
    .await;
    assert_eq!(git(&a.root, &["diff", "--", "code.py"]), "");
    std::fs::write(a.root.join("code.py"), "\r\n    indented\r\n\r\n").unwrap();
    let v = ok(
        &a.base,
        "POST",
        "/representations",
        json!({"backing":{"kind":"files","paths":["code.py"]}}),
    )
    .await;
    let text = v["files"][0]["content"].as_str().unwrap();
    let arranged=ok(&a.base,"POST",&format!("{}/arrange",endpoint(&v)),json!({"revision":v["revision"],"sections":[{"path":"code.py","from":0,"to":text.len(),"heading":"Leading and trailing blanks"}]})).await;
    assert_eq!(arranged["files"][0]["content"], text);
}

#[tokio::test]
async fn skipped_bisect_candidates_leave_an_honest_ambiguous_result() {
    let a = app().await;
    let good = git(&a.root, &["rev-parse", "HEAD"]);
    for n in 0..3 {
        std::fs::write(a.root.join("code.py"), format!("step = {n}\n")).unwrap();
        git(&a.root, &["add", "code.py"]);
        git(&a.root, &["commit", "-qm", &format!("step {n}")]);
    }
    let mut s = ok(
        &a.base,
        "POST",
        "/git/bisect",
        json!({"good":good,"bad":"HEAD"}),
    )
    .await;
    let ep = format!("/git/bisect/{}", s["id"].as_str().unwrap());
    for _ in 0..5 {
        if !s["outcome"].is_null() {
            break;
        }
        s = ok(
            &a.base,
            "POST",
            &format!("{ep}/mark"),
            json!({"candidate":s["candidate"],"verdict":"skip"}),
        )
        .await;
    }
    assert_eq!(s["outcome"], "ambiguous");
    let ended = ok(&a.base, "DELETE", &ep, json!({})).await;
    for p in ended["retained_inspections"].as_array().unwrap() {
        git(&a.root, &["worktree", "remove", p.as_str().unwrap()]);
    }
}
