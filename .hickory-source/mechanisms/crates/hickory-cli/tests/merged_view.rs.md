<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/merged_view.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-87756f3e699e"></a>
## `crates/hickory-cli/tests/merged_view.rs`

Related contracts: [A Write Through The Merged View Reports Per Target, And Undo Reaches All Of Them](../../../../guarantees/collaboration/a-merged-view-write-reports-per-target/contract.md), [The Merged View Is A Lens Over Several Worktrees, Never A Document](../../../../guarantees/collaboration/the-merged-view-is-a-lens/contract.md), [Comparison edit permissions do not establish replay evidence](../../../../guarantees/literate-projects/comparison-permissions-and-evidence-are-independent/contract.md).

[Read the test `git` beside its requirement](../../../../guarantees/collaboration/the-merged-view-is-a-lens/contract.md#f-87756f3e699e-git).
<a id="f-87756f3e699e-Session"></a>
### `Session`

<storycode:copy id="f-87756f3e699e-Session">
struct Session {
    base: String,
    root: PathBuf,
    _dir: tempfile::TempDir,
}

</storycode:copy>
<a id="f-87756f3e699e-DOC"></a>
### `DOC`

<storycode:copy id="f-87756f3e699e-DOC">
const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:file path="note.txt">shared head
</hick:file>
</hick:doc>
"##;

</storycode:copy>
<a id="f-87756f3e699e-start"></a>
### `start`

<storycode:copy id="f-87756f3e699e-start">
async fn start() -> Session {
    // One tempdir holding BOTH worktrees, so they are siblings that vanish
    // together and never collide with another test's.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap().join("repo");
    std::fs::create_dir_all(&root).unwrap();
    git(&root, &["init", "-q", "-b", "master"]);
    git(&root, &["config", "user.email", "t@example.com"]);
    git(&root, &["config", "user.name", "T"]);
    git(&root, &["config", "commit.gpgsign", "false"]);
    std::fs::write(root.join("demo.md"), DOC).unwrap();
    std::fs::write(root.join("shared.txt"), "head\nmaster only\ntail\n").unwrap();
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "--no-verify", "-m", "base"]);

    // A second worktree on its own branch — "across branches" means across
    // worktrees, because a branch that is not checked out cannot be written
    // to without going behind the working tree.
    let side = root.parent().unwrap().join("side-tree");
    assert!(!side.exists());
    let side_str = side.to_string_lossy().to_string();
    git(&root, &["worktree", "add", "-q", "-b", "side", &side_str]);
    std::fs::write(side.join("shared.txt"), "head\nside only\ntail\n").unwrap();

    let prepared = prepare(ServeOptions {
        target: root.join("demo.md"),
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

</storycode:copy>
<a id="f-87756f3e699e-get"></a>
### `get`

<storycode:copy id="f-87756f3e699e-get">
async fn get(session: &Session, path: &str) -> Value {
    reqwest::get(format!("{}{path}", session.base))
        .await
        .unwrap()
        .json()
        .await
        .unwrap_or(Value::Null)
}

</storycode:copy>
<a id="f-87756f3e699e-both_worktrees_are_offered_as_sources"></a>
### `both_worktrees_are_offered_as_sources`

<storycode:copy id="f-87756f3e699e-both_worktrees_are_offered_as_sources">
#[tokio::test]
async fn both_worktrees_are_offered_as_sources() {
    let session = start().await;
    let answer = get(&session, "/api/worktrees").await;
    assert_eq!(answer["repository"], Value::Bool(true));
    let names: Vec<&str> = answer["worktrees"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"side-tree"), "{answer}");
    assert_eq!(names.len(), 2, "{answer}");
    // Branches are named, because which branch a worktree is on is the thing
    // a person is choosing between.
    let branches: Vec<&str> = answer["worktrees"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|w| w["branch"].as_str())
        .collect();
    assert!(branches.contains(&"side"), "{answer}");
}

</storycode:copy>
<a id="f-87756f3e699e-agreed_regions_appear_once_and_differences_appear_as_variants"></a>
### `agreed_regions_appear_once_and_differences_appear_as_variants`

<storycode:copy id="f-87756f3e699e-agreed_regions_appear_once_and_differences_appear_as_variants">
#[tokio::test]
async fn agreed_regions_appear_once_and_differences_appear_as_variants() {
    let session = start().await;
    let answer = get(&session, "/api/merged?path=shared.txt").await;
    let regions = answer["regions"].as_array().unwrap();

    let shared: String = regions
        .iter()
        .filter(|r| r["kind"] == "shared")
        .map(|r| r["text"].as_str().unwrap())
        .collect();
    assert_eq!(shared, "head\ntail\n", "{answer}");

    let variants: Vec<&Value> = regions.iter().filter(|r| r["kind"] == "variant").collect();
    assert_eq!(variants.len(), 1, "{answer}");
    let by = &variants[0]["by_source"];
    let sides: Vec<&str> = by
        .as_object()
        .unwrap()
        .values()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(sides.contains(&"master only\n"), "{answer}");
    assert!(sides.contains(&"side only\n"), "{answer}");

    // It is a LENS, not a document: there is no save path, and the view says
    // so rather than leaving it to be discovered.
    assert_eq!(answer["read_only"], Value::Bool(true));
}

</storycode:copy>
<a id="f-87756f3e699e-a_worktree_without_the_file_is_named_rather_than_dropped"></a>
### `a_worktree_without_the_file_is_named_rather_than_dropped`

<storycode:copy id="f-87756f3e699e-a_worktree_without_the_file_is_named_rather_than_dropped">
#[tokio::test]
async fn a_worktree_without_the_file_is_named_rather_than_dropped() {
    // "This branch does not have this file yet" is an answer somebody opened
    // the view to get.
    let session = start().await;
    std::fs::write(session.root.join("only-here.txt"), "a\nb\n").unwrap();
    let answer = get(&session, "/api/merged?path=only-here.txt").await;
    let missing: Vec<&str> = answer["missing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m.as_str().unwrap())
        .collect();
    assert_eq!(missing, vec!["side-tree"], "{answer}");
    // And the one source that has it is entirely shared with itself.
    assert_eq!(answer["variants"], Value::from(0), "{answer}");
}

</storycode:copy>
<a id="f-87756f3e699e-one_target_can_be_asked_for_by_name"></a>
### `one_target_can_be_asked_for_by_name`

<storycode:copy id="f-87756f3e699e-one_target_can_be_asked_for_by_name">
#[tokio::test]
async fn one_target_can_be_asked_for_by_name() {
    let session = start().await;
    let answer = get(&session, "/api/merged?path=shared.txt&targets=side-tree").await;
    assert_eq!(answer["sources"].as_array().unwrap().len(), 1, "{answer}");
    assert_eq!(answer["variants"], Value::from(0), "{answer}");
}

</storycode:copy>
<a id="f-87756f3e699e-a_path_that_escapes_the_repository_is_refused"></a>
### `a_path_that_escapes_the_repository_is_refused`

<storycode:copy id="f-87756f3e699e-a_path_that_escapes_the_repository_is_refused">
#[tokio::test]
async fn a_path_that_escapes_the_repository_is_refused() {
    let session = start().await;
    let status = reqwest::get(format!("{}/api/merged?path=../etc/passwd", session.base))
        .await
        .unwrap()
        .status();
    assert_eq!(status.as_u16(), 400);
}

</storycode:copy>
<a id="f-87756f3e699e-post"></a>
### `post`

<storycode:copy id="f-87756f3e699e-post">
// ---------------------------------------------------------------------------
// Writing through the view
// ---------------------------------------------------------------------------

async fn post(session: &Session, path: &str, mut body: Value) -> (u16, Value) {
    if path == "/api/merged/write" {
        if body.get("writable").is_none() {
            body["writable"] = body["targets"].clone();
        }
        if body.get("expected").is_none() {
            let names = body["targets"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(",");
            let view = get(
                session,
                &format!(
                    "/api/merged?path={}&targets={names}",
                    body["path"].as_str().unwrap()
                ),
            )
            .await;
            body["expected"] = view["snapshot"].clone();
        }
    }
    let resp = reqwest::Client::new()
        .post(format!("{}{path}", session.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

</storycode:copy>
<a id="f-87756f3e699e-side_path"></a>
### `side_path`

<storycode:copy id="f-87756f3e699e-side_path">
fn side_path(session: &Session) -> PathBuf {
    session.root.parent().unwrap().join("side-tree")
}

</storycode:copy>
<a id="f-87756f3e699e-region_indices"></a>
### `region_indices`

The index of the first shared / first variant region in the live view.

<storycode:copy id="f-87756f3e699e-region_indices">
/// The index of the first shared / first variant region in the live view.
async fn region_indices(session: &Session) -> (usize, usize) {
    let view = get(session, "/api/merged?path=shared.txt").await;
    let regions = view["regions"].as_array().unwrap();
    let shared = regions.iter().position(|r| r["kind"] == "shared").unwrap();
    let variant = regions.iter().position(|r| r["kind"] == "variant").unwrap();
    (shared, variant)
}

</storycode:copy>
<a id="f-87756f3e699e-a_shared_edit_writes_every_target_in_one_keystroke"></a>
### `a_shared_edit_writes_every_target_in_one_keystroke`

<storycode:copy id="f-87756f3e699e-a_shared_edit_writes_every_target_in_one_keystroke">
#[tokio::test]
async fn a_shared_edit_writes_every_target_in_one_keystroke() {
    let session = start().await;
    let (shared, _) = region_indices(&session).await;
    let (status, answer) = post(
        &session,
        "/api/merged/write",
        serde_json::json!({
            "path": "shared.txt",
            "targets": ["repo", "side-tree"],
            "region": shared,
            "text": "HEAD\n",
            "route": "shared",
        }),
    )
    .await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(answer["landed"], Value::from(2), "{answer}");
    assert_eq!(answer["partial"], Value::Bool(false), "{answer}");

    assert_eq!(
        std::fs::read_to_string(session.root.join("shared.txt")).unwrap(),
        "HEAD\nmaster only\ntail\n"
    );
    assert_eq!(
        std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap(),
        "HEAD\nside only\ntail\n"
    );
}

</storycode:copy>
<a id="f-87756f3e699e-just_here_writes_one_target_and_leaves_the_other_alone"></a>
### `just_here_writes_one_target_and_leaves_the_other_alone`

<storycode:copy id="f-87756f3e699e-just_here_writes_one_target_and_leaves_the_other_alone">
#[tokio::test]
async fn just_here_writes_one_target_and_leaves_the_other_alone() {
    // The gesture no longer wraps text into a conditional in a file: it means
    // "write this to one target only", which is easier to implement and
    // easier to explain.
    let session = start().await;
    let (shared, _) = region_indices(&session).await;
    let before_side = std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap();

    let (status, answer) = post(
        &session,
        "/api/merged/write",
        serde_json::json!({
            "path": "shared.txt",
            "targets": ["repo", "side-tree"],
            "region": shared,
            "text": "HEAD\n",
            "route": "just-here",
            "source": "repo",
        }),
    )
    .await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(answer["landed"], Value::from(1), "{answer}");
    assert_eq!(
        std::fs::read_to_string(session.root.join("shared.txt")).unwrap(),
        "HEAD\nmaster only\ntail\n"
    );
    assert_eq!(
        std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap(),
        before_side
    );
}

</storycode:copy>
<a id="f-87756f3e699e-a_target_not_opened_for_writing_is_never_written"></a>
### `a_target_not_opened_for_writing_is_never_written`

<storycode:copy id="f-87756f3e699e-a_target_not_opened_for_writing_is_never_written">
#[tokio::test]
async fn a_target_not_opened_for_writing_is_never_written() {
    // Read-only is the DEFAULT for anything not explicitly opened for
    // writing: a long-lived release branch in the view must not silently
    // receive shared edits.
    let session = start().await;
    let (shared, _) = region_indices(&session).await;
    let before_side = std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap();

    let (status, answer) = post(
        &session,
        "/api/merged/write",
        serde_json::json!({
            "path": "shared.txt",
            "targets": ["repo"],
            "region": shared,
            "text": "HEAD\n",
            "route": "shared",
        }),
    )
    .await;
    assert_eq!(status, 200, "{answer}");
    let named: Vec<&str> = answer["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["source"].as_str().unwrap())
        .collect();
    assert_eq!(named, vec!["repo"], "{answer}");
    assert_eq!(
        std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap(),
        before_side
    );
}

</storycode:copy>
<a id="f-87756f3e699e-undo_puts_every_target_back_including_ones_the_write_did_not_reach"></a>
### `undo_puts_every_target_back_including_ones_the_write_did_not_reach`

<storycode:copy id="f-87756f3e699e-undo_puts_every_target_back_including_ones_the_write_did_not_reach">
#[tokio::test]
async fn undo_puts_every_target_back_including_ones_the_write_did_not_reach() {
    // A half-applied edit that is then undone in three of four places is a
    // state somebody will reach on the first day.
    let session = start().await;
    let (shared, _) = region_indices(&session).await;
    let before_main = std::fs::read_to_string(session.root.join("shared.txt")).unwrap();
    let before_side = std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap();

    let (_, written) = post(
        &session,
        "/api/merged/write",
        serde_json::json!({
            "path": "shared.txt",
            "targets": ["repo", "side-tree"],
            "region": shared,
            "text": "CHANGED\n",
            "route": "shared",
        }),
    )
    .await;
    assert_eq!(written["landed"], Value::from(2));

    let (status, undone) = post(
        &session,
        "/api/merged/undo",
        serde_json::json!({ "path": "shared.txt", "undo": written["undo"] }),
    )
    .await;
    assert_eq!(status, 200, "{undone}");
    assert_eq!(undone["restored"], Value::from(2), "{undone}");
    assert_eq!(
        std::fs::read_to_string(session.root.join("shared.txt")).unwrap(),
        before_main
    );
    assert_eq!(
        std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap(),
        before_side
    );
}

</storycode:copy>
[Read the test `displayed_snapshots_permissions_and_undo_never_overwrite_newer_target_edits` beside its requirement](../../../../guarantees/literate-projects/comparison-permissions-and-evidence-are-independent/contract.md#f-87756f3e699e-displayed_snapshots_permissions_and_undo_never_overwrite_newer_target_edits).
<a id="f-87756f3e699e-editing_a_variant_touches_only_that_side"></a>
### `editing_a_variant_touches_only_that_side`

<storycode:copy id="f-87756f3e699e-editing_a_variant_touches_only_that_side">
#[tokio::test]
async fn editing_a_variant_touches_only_that_side() {
    let session = start().await;
    let (_, variant) = region_indices(&session).await;
    let (status, answer) = post(
        &session,
        "/api/merged/write",
        serde_json::json!({
            "path": "shared.txt",
            "targets": ["repo", "side-tree"],
            "region": variant,
            "text": "master changed\n",
            "route": "just-here",
            "source": "repo",
        }),
    )
    .await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(
        std::fs::read_to_string(session.root.join("shared.txt")).unwrap(),
        "head\nmaster changed\ntail\n"
    );
    assert_eq!(
        std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap(),
        "head\nside only\ntail\n"
    );
}

</storycode:copy>
<a id="f-87756f3e699e-a_write_routed_at_a_source_that_is_not_here_is_refused"></a>
### `a_write_routed_at_a_source_that_is_not_here_is_refused`

<storycode:copy id="f-87756f3e699e-a_write_routed_at_a_source_that_is_not_here_is_refused">
#[tokio::test]
async fn a_write_routed_at_a_source_that_is_not_here_is_refused() {
    let session = start().await;
    let (shared, _) = region_indices(&session).await;
    let (status, answer) = post(
        &session,
        "/api/merged/write",
        serde_json::json!({
            "path": "shared.txt",
            "targets": ["repo", "side-tree"],
            "region": shared,
            "text": "x\n",
            "route": "just-here",
            "source": "ghost",
        }),
    )
    .await;
    assert_eq!(status, 422, "{answer}");
}

</storycode:copy>
<a id="f-87756f3e699e-versioned_selection_preserves_empty_literal_names_and_exact_file_paths"></a>
### `versioned_selection_preserves_empty_literal_names_and_exact_file_paths`

<storycode:copy id="f-87756f3e699e-versioned_selection_preserves_empty_literal_names_and_exact_file_paths">
#[tokio::test]
async fn versioned_selection_preserves_empty_literal_names_and_exact_file_paths() {
    let session = start().await;
    let literal = session.root.parent().unwrap().join("comma,tree");
    git(
        &session.root,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "literal-name",
            literal.to_str().unwrap(),
        ],
    );
    let filename = " exact name.txt ";
    std::fs::write(session.root.join(filename), "spaced repo\n").unwrap();
    std::fs::write(session.root.join(filename.trim()), "wrong trimmed file\n").unwrap();
    std::fs::write(literal.join(filename), "spaced comma\n").unwrap();
    let client = reqwest::Client::new();
    let endpoint = format!("{}/api/v1/merged", session.base);
    let response = client
        .get(&endpoint)
        .query(&[("path", filename), ("targets", r#"["comma,tree"]"#)])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let answer: Value = response.json().await.unwrap();
    assert_eq!(answer["path"], filename);
    assert_eq!(answer["sources"].as_array().unwrap().len(), 1);
    assert_eq!(answer["sources"][0]["name"], "comma,tree");
    assert_eq!(answer["snapshot"]["comma,tree"], "spaced comma\n");
    let empty: Value = client
        .get(&endpoint)
        .query(&[("path", filename), ("targets", "[]")])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(empty["sources"], serde_json::json!([]));
    assert_eq!(empty["regions"], serde_json::json!([]));
    assert_eq!(empty["snapshot"], serde_json::json!({}));
    let all: Value = client
        .get(&endpoint)
        .query(&[("path", filename)])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(all["sources"].as_array().unwrap().len(), 3);
    assert_eq!(all["snapshot"]["repo"], "spaced repo\n");
    let refused = client
        .get(&endpoint)
        .query(&[("path", filename), ("targets", "repo,comma,tree")])
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 400);
    assert_eq!(
        std::fs::read_to_string(literal.join(filename)).unwrap(),
        "spaced comma\n"
    );
    assert_eq!(
        std::fs::read_to_string(session.root.join(filename)).unwrap(),
        "spaced repo\n"
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/merged_view.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/merged_view.rs"><storycode:paste select="#f-87756f3e699e-git"/><storycode:paste select="#f-87756f3e699e-Session"/><storycode:paste select="#f-87756f3e699e-DOC"/><storycode:paste select="#f-87756f3e699e-start"/><storycode:paste select="#f-87756f3e699e-get"/><storycode:paste select="#f-87756f3e699e-both_worktrees_are_offered_as_sources"/><storycode:paste select="#f-87756f3e699e-agreed_regions_appear_once_and_differences_appear_as_variants"/><storycode:paste select="#f-87756f3e699e-a_worktree_without_the_file_is_named_rather_than_dropped"/><storycode:paste select="#f-87756f3e699e-one_target_can_be_asked_for_by_name"/><storycode:paste select="#f-87756f3e699e-a_path_that_escapes_the_repository_is_refused"/><storycode:paste select="#f-87756f3e699e-post"/><storycode:paste select="#f-87756f3e699e-side_path"/><storycode:paste select="#f-87756f3e699e-region_indices"/><storycode:paste select="#f-87756f3e699e-a_shared_edit_writes_every_target_in_one_keystroke"/><storycode:paste select="#f-87756f3e699e-just_here_writes_one_target_and_leaves_the_other_alone"/><storycode:paste select="#f-87756f3e699e-a_target_not_opened_for_writing_is_never_written"/><storycode:paste select="#f-87756f3e699e-undo_puts_every_target_back_including_ones_the_write_did_not_reach"/><storycode:paste select="#f-87756f3e699e-displayed_snapshots_permissions_and_undo_never_overwrite_newer_target_edits"/><storycode:paste select="#f-87756f3e699e-editing_a_variant_touches_only_that_side"/><storycode:paste select="#f-87756f3e699e-a_write_routed_at_a_source_that_is_not_here_is_refused"/><storycode:paste select="#f-87756f3e699e-versioned_selection_preserves_empty_literal_names_and_exact_file_paths"/></storycode:file>

</storycode:doc>
