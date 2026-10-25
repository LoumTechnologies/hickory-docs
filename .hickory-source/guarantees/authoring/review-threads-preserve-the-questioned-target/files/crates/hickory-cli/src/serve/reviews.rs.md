<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/reviews.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Review threads preserve the questioned target](../../../../../contract.md)

Captured SHA-256: `07d4814060c93f62d1585777ebeb877eb63c66e8d2cb3260789eb8833068925f`.

<book:entry path="crates/hickory-cli/src/serve/reviews.rs">
//! Local, append-only review events. Separate from editable source and agent sessions.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{io::Write, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewTarget {
    pub kind: String,
    pub path: String,
    pub quote: String,
    pub source_hash: String,
    pub lines: [usize; 2],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationship: Option<Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewEvent {
    /// Declared origin, not authentication. UI agent adapters always set this.
    #[serde(default)]
    pub agent: bool,
    pub scope: Option<String>,
    pub thread: Option<String>,
    pub target: Option<ReviewTarget>,
    pub action: String,
    pub body: String,
}
#[derive(Deserialize, Serialize)]
pub struct ReviewEvent {
    pub id: String,
    pub thread: String,
    pub target: Option<ReviewTarget>,
    pub action: String,
    pub body: String,
    pub actor: String,
    pub recorded_at: String,
}
fn directory(root: &Path) -> anyhow::Result<std::path::PathBuf> {
    let dir = hickory_workspace::WorkspaceStore::for_project(root)?
        .dir()
        .join("reviews");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
fn events(dir: &Path) -> anyhow::Result<Vec<ReviewEvent>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            out.push(serde_json::from_slice::<ReviewEvent>(&std::fs::read(
                path,
            )?)?);
        }
    }
    out.sort_by(|a, b| a.recorded_at.cmp(&b.recorded_at).then(a.id.cmp(&b.id)));
    Ok(out)
}
fn append(dir: &Path, body: NewEvent) -> anyhow::Result<ReviewEvent> {
    anyhow::ensure!(
        !body.agent || !matches!(body.action.as_str(), "resolve" | "reopen"),
        "Agent-originated events cannot declare human resolution"
    );
    anyhow::ensure!(
        !body.body.trim().is_empty(),
        "Write a comment or explain the review action"
    );
    anyhow::ensure!(body.body.len() <= 100_000, "Review text is too long");
    anyhow::ensure!(
        ["comment", "resolve", "reopen", "investigate"].contains(&body.action.as_str()),
        "Unknown review action"
    );
    if let Some(thread) = &body.thread {
        anyhow::ensure!(
            body.target.is_none(),
            "Replies cannot replace the original target"
        );
        anyhow::ensure!(
            events(dir)?.iter().any(|e| &e.thread == thread),
            "Review thread does not exist"
        );
    } else {
        let target = body
            .target
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("A new thread needs a target"))?;
        anyhow::ensure!(
            body.action == "comment",
            "A new thread starts with a comment"
        );
        anyhow::ensure!(
            !target.path.is_empty() && !target.quote.trim().is_empty(),
            "Select text or a relationship first"
        );
        anyhow::ensure!(
            target.lines[0] > 0 && target.lines[1] >= target.lines[0],
            "Invalid target lines"
        );
        anyhow::ensure!(
            target.source_hash.len() == 64
                && target.source_hash.bytes().all(|b| b.is_ascii_hexdigit()),
            "A target needs its source SHA-256"
        );
    }
    let mut file = tempfile::Builder::new()
        .prefix("review-")
        .suffix(".tmp")
        .tempfile_in(dir)?;
    let id = file
        .path()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let event = ReviewEvent {
        thread: body.thread.unwrap_or_else(|| id.clone()),
        id: id.clone(),
        target: body.target,
        action: body.action,
        body: body.body,
        actor: if body.agent {
            "agent-declared"
        } else {
            "user-declared"
        }
        .into(),
        recorded_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
            .to_string(),
    };
    serde_json::to_writer(&mut file, &event)?;
    file.flush()?;
    file.as_file().sync_all()?;
    file.persist_noclobber(dir.join(format!("{id}.json")))?;
    Ok(event)
}
fn current_state(root: &Path, target: &ReviewTarget) -> &'static str {
    let path = root.join(&target.path);
    let Ok(path) = path.canonicalize() else {
        return "unavailable";
    };
    let workspace = hickory_workspace::WorkspaceStore::for_project(root).ok();
    let allowed = root.canonicalize().is_ok_and(|root| path.starts_with(root))
        || workspace.is_some_and(|store| path.starts_with(store.dir()));
    if !allowed {
        return "unavailable";
    }
    let Ok(bytes) = std::fs::read(path) else {
        return "unavailable";
    };
    if format!("{:x}", Sha256::digest(&bytes)) == target.source_hash {
        "unchanged"
    } else if let Ok(text) = std::str::from_utf8(&bytes) {
        if text.matches(&target.quote).count() > 1 {
            "ambiguous"
        } else {
            "changed"
        }
    } else {
        "unavailable"
    }
}
pub async fn list(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let dir = directory(state.index.root())?;
    let events = events(&dir)
        .map_err(|e| ApiError::unprocessable(format!("Cannot read review history: {e}")))?;
    let targets: Vec<Value> = events.iter().filter_map(|event| event.target.as_ref().map(|target| json!({"thread":event.thread,"current_state":current_state(state.index.root(),target)}))).collect();
    // Linkage is a user declaration; response/status come from the actual host
    // turn, never a pass parsed from model prose. Missing turns stay unavailable.
    let mut investigations = Vec::new();
    for event in &events {
        if event.action != "investigate" {
            continue;
        }
        let Ok(link) = serde_json::from_str::<Value>(&event.body) else {
            continue;
        };
        if link["version"] != 1 {
            continue;
        }
        let conversation = link["conversation"].as_str().unwrap_or("");
        let turn_id = link["turn"].as_str();
        let Json(turns) = super::agent::list_turns(
            State(state.clone()),
            axum::extract::Path(conversation.to_string()),
        )
        .await;
        let turn = turns["turns"].as_array().and_then(|turns| {
            turns.iter().find(|turn| {
                turn_id.is_some() && turn["id"].as_str() == turn_id
                    || turn_id.is_none()
                        && link["id"].as_str().is_some_and(|id| {
                            turn["prompt"].as_str().is_some_and(|prompt| {
                                prompt.contains(&format!(
                                    "Review thread: {}; investigation: {id}",
                                    event.thread
                                ))
                            })
                        })
            })
        });
        let status = match turn {
            Some(turn) if turn["status"] == "running" => "running",
            Some(turn) if turn["status"] == "ok" => "responded",
            Some(turn) if turn["status"] == "stopped" => "cancelled",
            Some(_) => "failed",
            None if turn_id.is_some() => "unavailable",
            None => link["state"].as_str().unwrap_or("uncertain"),
        };
        investigations.push(json!({"event":event.id,"thread":event.thread,"link":link,
            "state":status,"response":turn.and_then(|turn| turn.get("answer")),
            "session":turn.and_then(|turn| turn.get("session")),
            "error":turn.and_then(|turn| turn.get("error")).or_else(|| link.get("error")),
            "verification":"Inspect executed receipts in the linked session; response prose does not establish a pass"}));
    }
    Ok(Json(
        json!({"events":events,"targets":targets,"investigations":investigations,"scope":format!("{:x}",Sha256::digest(dir.to_string_lossy().as_bytes()))}),
    ))
}
pub async fn create(
    State(state): State<LocalState>,
    Json(body): Json<NewEvent>,
) -> ApiResult<Json<ReviewEvent>> {
    let dir = directory(state.index.root())?;
    let scope = format!("{:x}", Sha256::digest(dir.to_string_lossy().as_bytes()));
    if body.scope.as_deref() != Some(scope.as_str()) {
        return Err(ApiError::conflict(
            "The workspace changed. Reopen review threads before saving.",
        ));
    }
    let event = append(&dir, body).map_err(|e| ApiError::unprocessable(e.to_string()))?;
    Ok(Json(event))
}

#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/authoring/review-threads-preserve-the-questioned-target.md
    #[test]
    fn agent_comments_are_attributed_and_cannot_declare_human_resolution() {
        let dir = tempfile::tempdir().unwrap();
        let input = |action: &str, thread: Option<String>, target: Option<ReviewTarget>| NewEvent {
            agent: true,
            scope: None,
            thread,
            target,
            action: action.into(),
            body: "Agent observation".into(),
        };
        let target = ReviewTarget {
            kind: "claim".into(),
            path: "spec.md".into(),
            quote: "claim".into(),
            source_hash: "a".repeat(64),
            lines: [1, 1],
            relationship: None,
        };
        let event = append(dir.path(), input("comment", None, Some(target))).unwrap();
        assert_eq!(event.actor, "agent-declared");
        for action in ["resolve", "reopen"] {
            assert!(append(dir.path(), input(action, Some(event.thread.clone()), None)).is_err());
        }
        assert_eq!(events(dir.path()).unwrap().len(), 1);
    }
    // Guarantee: docs/guarantees/authoring/review-threads-preserve-the-questioned-target.md
    #[test]
    fn correction_retains_target_and_cannot_retarget_a_reply() {
        let dir = tempfile::tempdir().unwrap();
        let source = "The deadline covers all retries.";
        let target = ReviewTarget {
            kind: "claim".into(),
            path: "spec.md".into(),
            quote: source.into(),
            source_hash: format!("{:x}", Sha256::digest(source)),
            lines: [1, 1],
            relationship: None,
        };
        let first = append(
            dir.path(),
            NewEvent {
                agent: false,
                scope: None,
                thread: None,
                target: Some(target.clone()),
                action: "comment".into(),
                body: "Only each attempt is bounded".into(),
            },
        )
        .unwrap();
        assert!(
            append(
                dir.path(),
                NewEvent {
                    agent: false,
                    scope: None,
                    thread: Some(first.thread.clone()),
                    target: Some(target),
                    action: "comment".into(),
                    body: "replace target".into()
                }
            )
            .is_err()
        );
        append(
            dir.path(),
            NewEvent {
                agent: false,
                scope: None,
                thread: Some(first.thread.clone()),
                target: None,
                action: "resolve".into(),
                body: "Reviewed the fix".into(),
            },
        )
        .unwrap();
        let saved = events(dir.path()).unwrap();
        assert_eq!(saved.len(), 2);
        assert_eq!(saved[0].target.as_ref().unwrap().quote, source);
        assert_eq!(saved[1].actor, "user-declared");
    }
    #[test]
    fn modified_source_is_not_current_evidence() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("spec.md");
        std::fs::write(&path, "before").unwrap();
        let target = ReviewTarget {
            kind: "claim".into(),
            path: "spec.md".into(),
            quote: "before".into(),
            source_hash: format!("{:x}", Sha256::digest("before")),
            lines: [1, 1],
            relationship: None,
        };
        assert_eq!(current_state(root.path(), &target), "unchanged");
        std::fs::write(path, "after").unwrap();
        assert_eq!(current_state(root.path(), &target), "changed");
    }
}
</book:entry>

</book:doc>
