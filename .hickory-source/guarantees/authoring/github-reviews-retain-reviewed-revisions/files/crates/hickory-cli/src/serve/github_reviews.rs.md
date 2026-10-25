<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/github_reviews.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [GitHub reviews retain reviewed revisions and external identity](../../../../../contract.md)

Captured SHA-256: `7ef53d3f07757993ef1c690cda94cc8d6bf0a8fc64e554f6e622140d124174ae`.

<book:entry path="crates/hickory-cli/src/serve/github_reviews.rs">
//! Read-only external evidence. No reply, resolve or publish operation is provided.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    github::{gh_json, valid_repository},
};
use axum::{
    Json,
    extract::{Query, State},
};
use hick_observe::reading::{EvidenceOrigin, LocationRef, ProviderAnnotation};
use hick_observe::{Location, Precision, Source};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    path::{Component, Path},
};

#[derive(Deserialize)]
pub struct Selection {
    repository: String,
    number: u64,
    document: String,
}
const COMMENT_FIELDS: &str = "id body createdAt updatedAt url author{login} commit{oid} originalCommit{oid} diffHunk path line originalLine startLine originalStartLine";
fn graphql(root: &Path, query: &str) -> Result<Value, String> {
    let raw = gh_json(root, &["api", "graphql", "-f", &format!("query={query}")])?;
    if raw.get("errors").is_some() {
        return Err(format!("GitHub GraphQL: {}", raw["errors"]));
    }
    Ok(raw["data"].clone())
}
/// Cursor traversal is bounded and refuses cursor loops; incomplete capture is explicit.
fn connection(
    mut fetch: impl FnMut(Option<&str>) -> Result<Value, String>,
) -> Result<(Vec<Value>, bool), String> {
    let mut nodes = vec![];
    let mut cursor: Option<String> = None;
    let mut seen = HashSet::new();
    let mut bytes = 0;
    for _ in 0..20 {
        let value = fetch(cursor.as_deref())?;
        bytes += serde_json::to_vec(&value).map_err(|e| e.to_string())?.len();
        if bytes > 8 * 1024 * 1024 {
            return Ok((nodes, true));
        }
        nodes.extend(
            value["nodes"]
                .as_array()
                .ok_or("GitHub connection missing nodes")?
                .iter()
                .cloned(),
        );
        if !value["pageInfo"]["hasNextPage"]
            .as_bool()
            .ok_or("Missing pagination state")?
        {
            return Ok((nodes, false));
        }
        let next = value["pageInfo"]["endCursor"]
            .as_str()
            .ok_or("Missing next cursor")?
            .to_owned();
        if !seen.insert(next.clone()) {
            return Err("GitHub repeated a pagination cursor".into());
        }
        cursor = Some(next);
    }
    Ok((nodes, true))
}
fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap()
}
fn after(cursor: Option<&str>) -> String {
    cursor
        .map(|v| format!(",after:{}", quoted(v)))
        .unwrap_or_default()
}
fn revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn safe_path(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}
fn encoded_path(path: &str) -> String {
    path.as_bytes()
        .iter()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-._~/".contains(byte) {
                (*byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}
fn archived(root: &Path, repository: &str, commit: &str, path: &str) -> Result<Vec<u8>, String> {
    if !revision(commit) || !safe_path(path) {
        return Err("Unavailable reviewed revision/path".into());
    }
    // Read exact reviewed bytes remotely, never a moving branch and never upload local source.
    let output = std::process::Command::new("gh")
        .current_dir(root)
        .args([
            "api",
            "--method",
            "GET",
            "-H",
            "Accept: application/vnd.github.raw+json",
            &format!(
                "repos/{repository}/contents/{}?ref={commit}",
                encoded_path(path)
            ),
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    if output.stdout.len() > 1024 * 1024 {
        return Err("Reviewed source exceeds 1 MiB".into());
    }
    Ok(output.stdout)
}
fn byte_lines(bytes: &[u8], start: u32, end: u32) -> Option<std::ops::Range<usize>> {
    if start == 0 || end < start {
        return None;
    }
    let lines: Vec<_> = bytes.split_inclusive(|b| *b == b'\n').collect();
    if end as usize > lines.len() {
        return None;
    }
    let from = lines.iter().take(start as usize - 1).map(|l| l.len()).sum();
    let to = lines.iter().take(end as usize).map(|l| l.len()).sum();
    Some(from..to)
}
/// Exact whole-file equality plus existing generated maps; never nearest-line/quote guessing.
fn map_anchor(
    document: &str,
    current: &Source,
    reviewed_path: &str,
    current_path: &str,
    reviewed: &[u8],
    start: u32,
    end: u32,
) -> Option<LocationRef> {
    let source = std::str::from_utf8(&current.bytes).ok()?;
    let range = if current_path == document {
        if reviewed != current.bytes {
            return None;
        }
        byte_lines(&current.bytes, start, end)?
    } else {
        let state = hick_lsp::document::HickDocumentState::from_source(source).ok()?;
        let matches: Vec<_> = state
            .virtual_files
            .iter()
            .filter(|file| file.path == current_path)
            .collect();
        if matches.len() != 1 || matches[0].written_content().as_bytes() != reviewed {
            return None;
        }
        let scratch = tempfile::tempdir().ok()?;
        let scratch_root = scratch.path().canonicalize().ok()?;
        let mapping =
            hick_dap::Mapping::for_state(Path::new(document), &state, &scratch_root).ok()?;
        let mapped: Vec<_> = (start.checked_sub(1)?..end)
            .map(|line| mapping.to_document(&scratch_root.join(current_path), line))
            .collect::<Option<_>>()?;
        if mapped.windows(2).any(|pair| pair[1] != pair[0] + 1) {
            return None;
        }
        byte_lines(
            &current.bytes,
            mapped.first()?.checked_add(1)?,
            mapped.last()?.checked_add(1)?,
        )?
    };
    let _ = reviewed_path;
    Some(LocationRef {
        id: format!(
            "{}:{}:bytes:{}:{}",
            current.id, current.revision, range.start, range.end
        ),
        revision: current.revision.clone(),
        line: current.bytes[..range.start]
            .iter()
            .filter(|b| **b == b'\n')
            .count() as u32,
        location: Location {
            source: current.id.clone(),
            block: None,
            bytes: Some(range),
            precision: Precision::Exact,
        },
    })
}
fn renamed(root: &Path, commit: &str, old: &str) -> Option<String> {
    if !revision(commit) {
        return None;
    }
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(["diff", "--name-status", "-M", commit, "HEAD", "--"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let matches: Vec<_> = std::str::from_utf8(&output.stdout)
        .ok()?
        .lines()
        .filter_map(|line| {
            let parts: Vec<_> = line.split('\t').collect();
            (parts.len() == 3 && parts[0].starts_with('R') && parts[1] == old)
                .then(|| parts[2].to_owned())
        })
        .collect();
    (matches.len() == 1).then(|| matches[0].clone())
}
fn load(root: &Path, selection: &Selection) -> Result<Value, String> {
    if !valid_repository(&selection.repository)
        || selection.number == 0
        || !safe_path(&selection.document)
    {
        return Err("Select owner/repository, positive PR number and workspace document".into());
    }
    let path = root
        .join(&selection.document)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !path.starts_with(root.canonicalize().map_err(|e| e.to_string())?) {
        return Err("Document escapes workspace".into());
    }
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    if bytes.len() > 1024 * 1024 {
        return Err("Document exceeds 1 MiB".into());
    }
    let source = Source::capture("document".into(), selection.document.clone(), bytes);
    let (owner, repo) = selection.repository.split_once('/').unwrap();
    let pr = format!(
        "repository(owner:{},name:{}){{pullRequest(number:{})",
        quoted(owner),
        quoted(repo),
        selection.number
    );
    let raw = graphql(
        root,
        &format!("query{{{pr}{{headRefOid baseRefOid url}}}}}}"),
    )?;
    let info = &raw["repository"]["pullRequest"];
    let head = info["headRefOid"]
        .as_str()
        .ok_or("PR unavailable or permission denied")?;
    let base = info["baseRefOid"].as_str().ok_or("PR base unavailable")?;
    let comparison = gh_json(
        root,
        &[
            "api",
            &format!("repos/{}/compare/{base}...{head}", selection.repository),
        ],
    )?;
    let diff_base = comparison["merge_base_commit"]["sha"]
        .as_str()
        .ok_or("Diff merge base unavailable")?;
    let (mut threads, mut incomplete) = connection(|cursor| {
        let result = graphql(
            root,
            &format!(
                "query{{{pr}{{reviewThreads(first:100{}){{nodes{{id path diffSide startDiffSide line startLine originalLine originalStartLine isOutdated isResolved}}pageInfo{{hasNextPage endCursor}}}}}}}}}}",
                after(cursor)
            ),
        )?;
        Ok(result["repository"]["pullRequest"]["reviewThreads"].clone())
    })?;
    let local_repo = std::process::Command::new("git")
        .current_dir(root)
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()
        .and_then(|o| super::github::github_repository(String::from_utf8_lossy(&o.stdout).trim()));
    let mut annotations = vec![];
    let mut capture_bytes = source.bytes.len();
    let mut unavailable = vec![];
    for thread in &mut threads {
        let id = thread["id"]
            .as_str()
            .ok_or("Thread identity unavailable")?
            .to_owned();
        let (comments, partial) = connection(|cursor| {
            let result = graphql(
                root,
                &format!(
                    "query{{node(id:{}){{... on PullRequestReviewThread{{comments(first:100{}){{nodes{{{COMMENT_FIELDS}}}pageInfo{{hasNextPage endCursor}}}}}}}}}}",
                    quoted(&id),
                    after(cursor)
                ),
            )?;
            Ok(result["node"]["comments"].clone())
        })?;
        incomplete |= partial;
        let first = comments.first().cloned().unwrap_or(Value::Null);
        thread["comments"] = json!(comments);
        let side = thread["diffSide"].as_str().unwrap_or("UNKNOWN");
        let original_head = first["originalCommit"]["oid"].as_str().unwrap_or("unknown");
        let outdated = thread["isOutdated"].as_bool().unwrap_or(true);
        let current_review = original_head == head && !outdated;
        let commit = if side == "RIGHT" {
            original_head
        } else if current_review {
            diff_base
        } else {
            "unknown"
        };
        let reviewed_path = thread["path"].as_str().unwrap_or("");
        let start = thread["startLine"]
            .as_u64()
            .or_else(|| thread["line"].as_u64())
            .unwrap_or(0) as u32;
        let end = thread["line"].as_u64().unwrap_or(0) as u32;
        let archive = archived(root, &selection.repository, commit, reviewed_path);
        let current_path =
            renamed(root, commit, reviewed_path).unwrap_or_else(|| reviewed_path.into());
        let location = if local_repo.as_deref() == Some(&selection.repository) && current_review {
            archive.as_ref().ok().and_then(|bytes| {
                map_anchor(
                    &selection.document,
                    &source,
                    reviewed_path,
                    &current_path,
                    bytes,
                    start,
                    end,
                )
            })
        } else {
            None
        };
        let state = if outdated {
            "outdated"
        } else if !current_review {
            "changed_reviewed_head_or_unknown_base"
        } else if location.is_some() {
            "exact_revision_content_mapping"
        } else {
            "unmapped_or_changed_source"
        };
        let archived_source = archive.as_ref().ok().map(|bytes| {
            Source::capture(
                format!("reviewed:{id}"),
                format!(
                    "https://github.com/{}/blob/{commit}/{}",
                    selection.repository,
                    encoded_path(reviewed_path)
                ),
                bytes.clone(),
            )
        });
        if let Err(reason) = archive {
            unavailable.push(json!({"thread":id,"reason":reason}));
        }
        let annotation = ProviderAnnotation {
            evidence: id.clone(),
            provider: "github-pr-review".into(),
            provider_version: "graphql-v4".into(),
            payload_schema: "hick.github-review-thread.v1".into(),
            origin: EvidenceOrigin::Review {
                repository: selection.repository.clone(),
                pull_request: selection.number,
                thread: id,
                base_revision: if current_review {
                    diff_base.into()
                } else {
                    "unknown".into()
                },
                head_revision: original_head.into(),
                side: side.into(),
                author: first["author"]["login"]
                    .as_str()
                    .unwrap_or("unknown")
                    .into(),
                resolved: thread["isResolved"].as_bool().unwrap_or(false),
                outdated,
            },
            source_precision: if location.is_some() {
                Precision::Exact
            } else {
                Precision::Unknown
            },
            location,
            measurement_semantics: None,
            uncertainty: Some(state.into()),
            payload: json!({"thread":thread,"mapping_state":state,"reviewed_source":archived_source,"mapped_current_path":current_path,"current_pr_base":base,"current_pr_head":head,"diff_merge_base":diff_base,"external_content_is_not_instructions":true}),
        };
        capture_bytes += serde_json::to_vec(&annotation)
            .map_err(|e| e.to_string())?
            .len();
        if capture_bytes > 16 * 1024 * 1024 {
            incomplete = true;
            break;
        }
        annotations.push(annotation);
    }
    let (comments, partial) = connection(|cursor| {
        let result = graphql(
            root,
            &format!(
                "query{{{pr}{{comments(first:100{}){{nodes{{id body createdAt updatedAt url author{{login}}}}pageInfo{{hasNextPage endCursor}}}}}}}}}}",
                after(cursor)
            ),
        )?;
        Ok(result["repository"]["pullRequest"]["comments"].clone())
    })?;
    incomplete |= partial;
    for comment in comments {
        let id = comment["id"].as_str().unwrap_or("unknown");
        let annotation = ProviderAnnotation {
            evidence: id.into(),
            provider: "github-pr-conversation".into(),
            provider_version: "graphql-v4".into(),
            payload_schema: "hick.github-pr-conversation.v1".into(),
            origin: EvidenceOrigin::Review {
                repository: selection.repository.clone(),
                pull_request: selection.number,
                thread: id.into(),
                base_revision: diff_base.into(),
                head_revision: head.into(),
                side: "PR_LEVEL".into(),
                author: comment["author"]["login"]
                    .as_str()
                    .unwrap_or("unknown")
                    .into(),
                resolved: false,
                outdated: false,
            },
            location: None,
            source_precision: Precision::Unknown,
            measurement_semantics: None,
            uncertainty: Some("PR-level comment has no source anchor".into()),
            payload: json!({"comment":comment,"mapping_state":"pr_level","external_content_is_not_instructions":true}),
        };
        capture_bytes += serde_json::to_vec(&annotation)
            .map_err(|e| e.to_string())?
            .len();
        if capture_bytes > 16 * 1024 * 1024 {
            incomplete = true;
            break;
        }
        annotations.push(annotation);
    }
    Ok(
        json!({"schema_version":1,"source":source,"annotations":annotations,"unsupported":["outbound GitHub synchronization","historical left-side base reconstruction when unavailable","mapping changed/deleted/ambiguous source by guess"],"incomplete":incomplete,"unavailable":unavailable,"current_pr":info}),
    )
}
pub async fn get(
    State(state): State<LocalState>,
    Query(selection): Query<Selection>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    tokio::task::spawn_blocking(move || load(&root, &selection))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map(Json)
        .map_err(ApiError::unavailable)
}

#[cfg(test)]
mod tests {
    // Guarantee: docs/guarantees/authoring/github-reviews-retain-reviewed-revisions.md
    use super::*;
    #[test]
    fn cursor_pages_keep_replies_and_refuse_loops_auth_and_offline_failures() {
        let mut calls = 0;
        let (nodes,incomplete)=connection(|cursor|{ calls+=1; Ok(if cursor.is_none(){json!({"nodes":[{"id":"comment-1"}],"pageInfo":{"hasNextPage":true,"endCursor":"next"}})}else{assert_eq!(cursor,Some("next"));json!({"nodes":[{"id":"reply-2","edited":true,"resolved":true}],"pageInfo":{"hasNextPage":false,"endCursor":null}})})}).unwrap();
        assert_eq!(calls, 2);
        assert_eq!(nodes.len(), 2);
        assert!(!incomplete);
        assert_eq!(nodes[1]["resolved"], true);
        assert!(connection(|_| Err("permission denied".into())).is_err());
        assert!(connection(|_| Err("offline".into())).is_err());
        assert!(
            connection(|_| Ok(
                json!({"nodes":[],"pageInfo":{"hasNextPage":true,"endCursor":"same"}})
            ))
            .unwrap_err()
            .contains("repeated")
        );
    }
    #[test]
    fn exact_multiline_and_generated_mapping_refuses_changed_deleted_and_ambiguous_source() {
        let text = "# 🦀\n<hick:file path=\"app.py\">\ndef f():\n    return 1\n</hick:file>\n";
        let source = Source::capture(
            "document".into(),
            "spec.md".into(),
            text.as_bytes().to_vec(),
        );
        let reviewed = b"def f():\n    return 1\n";
        let location = map_anchor("spec.md", &source, "app.py", "app.py", reviewed, 1, 2).unwrap();
        assert_eq!(location.line, 2);
        assert_eq!(&source.bytes[location.location.bytes.unwrap()], reviewed);
        assert!(
            map_anchor(
                "spec.md",
                &source,
                "app.py",
                "app.py",
                b"def f():\n    return 2\n",
                1,
                2
            )
            .is_none()
        );
        assert!(
            map_anchor(
                "spec.md",
                &source,
                "deleted.py",
                "deleted.py",
                reviewed,
                1,
                2
            )
            .is_none()
        );
        assert!(map_anchor("spec.md", &source, "other.py", "other.py", reviewed, 1, 2).is_none());
        assert!(map_anchor("spec.md", &source, "app.py", "app.py", reviewed, 1, 99).is_none());
        let renamed = Source::capture("document".into(), "new.md".into(), reviewed.to_vec());
        assert!(map_anchor("new.md", &renamed, "old.md", "new.md", reviewed, 1, 2).is_some());
    }
    #[test]
    fn external_left_right_and_pr_level_envelopes_have_no_fabricated_runtime() {
        for side in ["LEFT", "RIGHT", "PR_LEVEL"] {
            let annotation = ProviderAnnotation {
                evidence: "comment-id".into(),
                provider: "github-pr-review".into(),
                provider_version: "graphql-v4".into(),
                payload_schema: "hick.github-review-thread.v1".into(),
                origin: EvidenceOrigin::Review {
                    repository: "org/repo".into(),
                    pull_request: 4,
                    thread: "thread".into(),
                    base_revision: "base".into(),
                    head_revision: "reviewed-head".into(),
                    side: side.into(),
                    author: "reviewer".into(),
                    resolved: true,
                    outdated: true,
                },
                location: None,
                source_precision: Precision::Unknown,
                measurement_semantics: None,
                uncertainty: Some("outdated".into()),
                payload: json!({"originalLine":3,"originalStartLine":2,"diffHunk":"@@ -2,2 +2,2 @@"}),
            };
            let value = serde_json::to_value(&annotation).unwrap();
            assert!(value["origin"].get("run").is_none());
            assert!(value["origin"].get("stop").is_none());
            assert_eq!(value["origin"]["side"], side);
            assert_eq!(
                serde_json::from_value::<ProviderAnnotation>(value).unwrap(),
                annotation
            );
        }
    }
    #[test]
    fn explicit_git_rename_is_used_without_guessing_missing_history() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(root)
                .args([
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "-c",
                    "core.hooksPath=/dev/null",
                ])
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout).unwrap().trim().to_owned()
        };
        git(&["init"]);
        std::fs::write(root.join("old.md"), "same captured source\n").unwrap();
        git(&["add", "old.md"]);
        git(&["commit", "-m", "original"]);
        let original = git(&["rev-parse", "HEAD"]);
        git(&["mv", "old.md", "new.md"]);
        git(&["commit", "-m", "rename"]);
        assert_eq!(
            renamed(root, &original, "old.md").as_deref(),
            Some("new.md")
        );
        assert!(renamed(root, &"f".repeat(40), "old.md").is_none());
        assert!(renamed(root, &original, "unrelated.md").is_none());
    }
}
</book:entry>

</book:doc>
