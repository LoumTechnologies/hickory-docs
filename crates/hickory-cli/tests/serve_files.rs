//! `GET /api/files` — the folder tree the app's file pane renders.
//!
//! Driven over a real socket, serve_local.rs style: the claim is about what
//! the wire says, not about a handler's internals.

use std::path::PathBuf;

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::Value;

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="demo.md">
# Demo
</hick:doc>
"##;

struct Session {
    base: String,
    root: PathBuf,
    _dir: tempfile::TempDir,
}

async fn start() -> Session {
    start_view(false).await
}

async fn start_view(single_file: bool) -> Session {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();

    // A tree with everything the endpoint promises to handle: nesting, a
    // gitignored file and folder, hidden files, the caches, and names whose
    // order only comes out right if directories sort before files and both
    // sort case-insensitively.
    std::fs::write(root.join("notes.md"), DOC).unwrap();
    std::fs::write(root.join("README.md"), "hello").unwrap();
    std::fs::write(root.join("zeta.txt"), "z").unwrap();
    std::fs::create_dir_all(root.join("src/deep")).unwrap();
    std::fs::write(root.join("src/lib.rs"), "// code").unwrap();
    std::fs::write(root.join("src/deep/inner.md"), DOC).unwrap();
    std::fs::create_dir(root.join("Tools")).unwrap();
    std::fs::write(root.join("Tools/run.sh"), "#!/bin/sh").unwrap();
    // Excluded: gitignored (even though this is not a git repository yet),
    // hidden, and the two directories the walker never enters.
    std::fs::write(root.join(".gitignore"), "ignored.txt\ndist/\n").unwrap();
    std::fs::write(root.join("ignored.txt"), "secret").unwrap();
    std::fs::create_dir(root.join("dist")).unwrap();
    std::fs::write(root.join("dist/out.js"), "built").unwrap();
    std::fs::write(root.join(".hidden.txt"), "dot").unwrap();
    std::fs::create_dir(root.join(".hick-cache")).unwrap();
    std::fs::write(root.join(".hick-cache/state.json"), "{}").unwrap();
    std::fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
    std::fs::write(root.join("node_modules/pkg/index.js"), "x").unwrap();

    let prepared = prepare(ServeOptions {
        target: if single_file { root.join("notes.md") } else { root.clone() },
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("session prepares");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    Session {
        base: format!("http://127.0.0.1:{port}"),
        root,
        _dir: dir,
    }
}

async fn get(session: &Session, path: &str) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .get(format!("{}{path}", session.base))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap())
}

fn names(nodes: &Value) -> Vec<String> {
    nodes
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["name"].as_str().unwrap().to_string())
        .collect()
}

fn child<'a>(nodes: &'a Value, name: &str) -> &'a Value {
    nodes
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["name"] == name)
        .unwrap_or_else(|| panic!("no node named {name} in {nodes}"))
}

#[tokio::test]
async fn file_tree_lists_the_root_gitignore_aware_and_sorted() {
    let session = start().await;
    let (status, body) = get(&session, "/api/files").await;
    assert_eq!(status, 200, "{body}");

    // The root is the served folder's name.
    let folder = session.root.file_name().unwrap().to_string_lossy();
    assert_eq!(body["root"], folder.as_ref());
    assert_eq!(body["truncated"], false);

    // Where the folder actually is, and how this machine spells a path
    // inside it: what the tree's context menu copies as an absolute path.
    // Guards docs/guarantees/authoring/a-tree-row-opens-in-the-platform.md.
    assert_eq!(body["root_path"], session.root.to_string_lossy().as_ref());
    assert_eq!(body["separator"], std::path::MAIN_SEPARATOR_STR);
    assert!(
        body["file_manager"].as_str().is_some_and(|n| !n.is_empty()),
        "the menu needs a name for this desktop's file manager: {body}"
    );

    // Directories first, then files, both case-insensitive alphabetical —
    // and nothing gitignored, hidden, cached, or vendored.
    assert_eq!(
        names(&body["tree"]),
        ["src", "Tools", "notes.md", "README.md", "zeta.txt"]
    );

    // Nesting: src holds a directory and a file, root-relative paths with
    // forward slashes all the way down.
    let src = child(&body["tree"], "src");
    assert_eq!(src["dir"], true);
    assert_eq!(names(&src["children"]), ["deep", "lib.rs"]);
    let lib = child(&src["children"], "lib.rs");
    assert_eq!(lib["path"], "src/lib.rs");
    assert_eq!(lib["dir"], false);

    // doc_id on Markdown documents only.
    let notes = child(&body["tree"], "notes.md");
    assert!(notes["doc_id"].is_string(), "{notes}");
    let readme = child(&body["tree"], "README.md");
    assert!(readme["doc_id"].is_string(), "{readme}");
    let sh = child(&child(&body["tree"], "Tools")["children"], "run.sh");
    assert!(sh.get("doc_id").is_none(), "{sh}");
}

#[tokio::test]
async fn a_markdown_document_found_by_the_tree_is_immediately_openable() {
    let session = start().await;
    let (_, body) = get(&session, "/api/files").await;

    let deep = child(&child(&body["tree"], "src")["children"], "deep");
    let inner = child(&deep["children"], "inner.md");
    assert_eq!(inner["path"], "src/deep/inner.md");
    let id = inner["doc_id"].as_str().unwrap();

    // The id the tree hands out opens the document, even if the startup scan
    // never indexed it.
    let (status, doc) = get(&session, &format!("/api/docs/{id}")).await;
    assert_eq!(status, 200, "{doc}");
    assert_eq!(doc["path"], "src/deep/inner.md");
    assert_eq!(doc["source"], DOC);
}

// Guarantees:
// - docs/guarantees/authoring/adoption-is-byte-exact-or-refused.md
// - docs/guarantees/authoring/a-file-is-ingested-from-the-tree.md
#[tokio::test]
async fn make_literate_creates_an_openable_markdown_document() {
    let session = start().await;
    let original = "print('hello')\n";
    std::fs::write(session.root.join("analysis.py"), original).unwrap();
    let response = reqwest::Client::new()
        .post(format!("{}/api/adopt", session.base))
        .json(&serde_json::json!({ "path": "analysis.py" }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let adopted: Value = response.json().await.unwrap();
    assert_eq!(adopted["doc_path"], "analysis.md");
    assert_eq!(adopted["output_path"], "analysis.py");
    assert_eq!(adopted["created"], true);
    assert!(session.root.join("analysis.md").exists());
    assert!(!session.root.join("analysis.hick").exists());
    assert_eq!(
        std::fs::read_to_string(session.root.join("analysis.py")).unwrap(),
        original
    );

    let id = adopted["doc_id"].as_str().unwrap();
    let (status, doc) = get(&session, &format!("/api/docs/{id}")).await;
    assert_eq!(status, 200);
    assert_eq!(doc["path"], "analysis.md");
    let (status, render) = get(&session, &format!("/api/docs/{id}/render")).await;
    assert_eq!(status, 200, "the new document renders: {render}");
    assert_eq!(
        std::fs::read_to_string(session.root.join("analysis.py")).unwrap(),
        original
    );
}


#[tokio::test]
async fn opening_one_file_does_not_open_its_parent_as_a_folder() {
    let session = start_view(true).await;
    let (status, files) = get(&session, "/api/files").await;
    assert_eq!(status, 200);
    assert_eq!(files["folder_open"], false);
    // The tree remains usable for path resolution, independent of Files visibility.
    let id = files["tree"].as_array().unwrap().iter()
        .find(|node| node["path"] == "notes.md").unwrap()["doc_id"].as_str().unwrap();
    let (status, _) = get(&session, &format!("/api/docs/{id}")).await;
    assert_eq!(status, 200, "the opened document still works without a folder view");
    let (_, folder_files) = get(&start().await, "/api/files").await;
    assert_eq!(folder_files["folder_open"], true);
}
