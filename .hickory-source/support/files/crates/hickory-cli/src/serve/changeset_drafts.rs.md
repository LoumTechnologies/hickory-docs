<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/changeset_drafts.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A final changeset revision applies B to C](../../../../../../guarantees/agent/final-changeset-revisions-apply-b-to-c/contract.md)

Captured SHA-256: `b9180ed9fe30be97d56ebb53adb438a26dbfcd6a05b25843f71d92bbc422d3a7`.

<book:entry path="crates/hickory-cli/src/serve/changeset_drafts.rs">
//! Authored final-result revisions replay A to construct C, but publish only B to C.
use super::{
    LocalState,
    agent_context::EditorContext,
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Path, State},
};
use hick_lang::{EntryKind, ProjectTree};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct Apply {
    pub sources: BTreeMap<String, String>,
    pub context: EditorContext,
}
fn texts(tree: &ProjectTree, paths: &[String]) -> anyhow::Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for path in paths {
        if let Some(entry) = tree.get(path) {
            anyhow::ensure!(
                entry.kind == EntryKind::File,
                "Cannot publish symlink revision {path}; keep its accepted construction"
            );
            result.insert(path.clone(), std::str::from_utf8(&entry.bytes)?.to_owned());
        }
    }
    Ok(result)
}
pub async fn apply(
    State(state): State<LocalState>,
    Path((doc, turn)): Path<(String, String)>,
    Json(body): Json<Apply>,
) -> ApiResult<Json<Value>> {
    let capture = super::agent_projects::captured(&state, &doc, &turn)?;
    if capture.error.is_some() || capture.response.is_none() || !capture.ambiguous.is_empty() {
        return Err(ApiError::conflict(
            "Finalize the original capture and resolve attribution before applying a revision.",
        ));
    }
    if state
        .agent
        .turns
        .lock()
        .unwrap()
        .get(&doc)
        .is_some_and(|ts| ts.iter().any(|t| t.status == "running"))
    {
        return Err(ApiError::conflict(
            "Wait for the active turn before applying a revision.",
        ));
    }
    let artifact = capture.artifact.as_ref().ok_or_else(|| {
        ApiError::conflict("Accepted artifact unavailable; finalize the capture first.")
    })?;
    let a = capture
        .starting
        .restore()
        .map_err(ApiError::unprocessable)?;
    let b = capture
        .completed
        .as_ref()
        .ok_or_else(|| ApiError::conflict("Completed B unavailable"))?
        .restore()
        .map_err(ApiError::unprocessable)?;
    let original = hick_lang::evaluate_project(&artifact.entry, &artifact.sources, &a)
        .map_err(ApiError::unprocessable)?;
    let revised = hick_lang::evaluate_project(&artifact.entry, &body.sources, &a)
        .map_err(ApiError::unprocessable)?;
    if !original.old.compare(&revised.old).is_empty() || !original.new.compare(&b).is_empty() {
        return Err(ApiError::conflict(
            "Captured Old and the accepted result must remain fixed. Discard this draft and reopen the accepted artifact.",
        ));
    }
    let c = revised.new;
    let paths: Vec<_> = b.compare(&c).into_iter().map(|d| d.path).collect();
    let disk: Vec<_> = paths
        .iter()
        .filter(|p| !p.starts_with(hick_lang::EDITOR_BUFFER_PREFIX))
        .cloned()
        .collect();
    // Validate the complete plan before writing anything, including unsupported transitions.
    let before = texts(&b, &disk).map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    let after = texts(&c, &disk).map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    for path in &disk {
        if let Some(next) = c.get(path)
            && next.executable != b.get(path).is_some_and(|e| e.executable)
        {
            return Err(ApiError::conflict(format!(
                "Executable mode revisions are not editable code: {path}"
            )));
        }
    }
    let mut editors = vec![];
    if let Some(scope) = &capture.workspace {
        for (id, path) in &scope.identities {
            if path.is_none() && scope.provenance.get(id).and_then(Option::as_ref).is_none() {
                return Err(ApiError::conflict(
                    "This capture has no untitled buffer instance identity. Start a new turn in the original editor before applying revisions.",
                ));
            }
            let address = format!(
                "{}{}",
                hick_lang::EDITOR_BUFFER_PREFIX,
                id.as_bytes()
                    .iter()
                    .map(|v| format!("{v:02x}"))
                    .collect::<String>()
            );
            if !paths.contains(&address) {
                continue;
            }
            let old = b
                .get(&address)
                .ok_or_else(|| ApiError::conflict("Captured editor is missing"))?;
            let new = c.get(&address).ok_or_else(|| {
                ApiError::conflict(
                    "A revision cannot close an editor. Discard and edit its New code instead.",
                )
            })?;
            if new.kind != EntryKind::File || new.executable {
                return Err(ApiError::conflict(
                    "Editor revisions must remain text buffers",
                ));
            }
            let old = std::str::from_utf8(&old.bytes)
                .map_err(|e| ApiError::unprocessable(e.to_string()))?;
            let new = std::str::from_utf8(&new.bytes)
                .map_err(|e| ApiError::unprocessable(e.to_string()))?;
            editors.push(json!({"id":format!("revision:{turn}:{id}"),"buffer":id,"name":id,"path":path,"oldText":old,"newText":new,"editor":true,"status":"applying"}));
        }
    } else if paths
        .iter()
        .any(|p| p.starts_with(hick_lang::EDITOR_BUFFER_PREFIX))
    {
        return Err(ApiError::conflict(
            "No captured editor identity for this revision",
        ));
    }
    if paths
        .iter()
        .filter(|p| p.starts_with(hick_lang::EDITOR_BUFFER_PREFIX))
        .count()
        != editors.len()
    {
        return Err(ApiError::conflict(
            "Every revised editor address must have its original captured identity. A revision cannot create or rename buffers.",
        ));
    }
    crate::engine::writes::during(&state.writes, async {
        let current = if let Some(scope) = &capture.workspace {
            scope
                .current(&body.context.buffers)
                .map_err(|e| ApiError::conflict(format!("{e:#}")))?
        } else {
            crate::project_revision::directory(&capture.root, Some(&capture.inventory))
                .map_err(ApiError::from)?
        };
        if !current.compare(&b).is_empty() {
            return Err(ApiError::conflict(
                "Destinations changed since accepted B (bytes, inventory, metadata or editor identities). Nothing applied. Preserve newer work; discard this revision or restore B explicitly.",
            ));
        }
        let record = super::acp::record::Record::existing(capture.session.clone());
        record.context("authored-changeset-revision", &json!({"turn":turn,"entry":artifact.entry,"sources":body.sources,"status":"constructed; application requested","runtime":"unrun"})).map_err(|e|ApiError::internal(e.to_string()))?;
        let mut applied: Vec<String> = vec![];
        let mut failure = None;
        for path in &disk {
            // Recheck the full remaining B inventory between targets; substitute only our landed writes.
            let mut expected = b.clone();
            for landed in &applied {
                if let Some(entry) = c.get(landed) {
                    expected
                        .insert(landed, (**entry).clone())
                        .map_err(ApiError::unprocessable)?;
                } else {
                    expected.remove(landed).map_err(ApiError::unprocessable)?;
                }
            }
            let actual = if let Some(scope) = &capture.workspace {
                scope.current(&body.context.buffers)
            } else {
                crate::project_revision::directory(&capture.root, Some(&capture.inventory))
            };
            let result = async {
                anyhow::ensure!(
                    actual?.compare(&expected).is_empty(),
                    "Concurrent destination change; remaining writes refused"
                );
                let old = before
                    .get(path)
                    .map(|t| BTreeMap::from([(path.clone(), t.clone())]))
                    .unwrap_or_default();
                let new = after
                    .get(path)
                    .map(|t| BTreeMap::from([(path.clone(), t.clone())]))
                    .unwrap_or_default();
                super::project_publish::publish_text(
                    &state,
                    &capture.root,
                    &old,
                    &new,
                    Some(&expected),
                )
                .await?;
                Ok::<_, anyhow::Error>(())
            }
            .await;
            if let Err(error) = result {
                failure = Some(format!("{error:#}"));
                break;
            }
            applied.push(path.clone());
        }
        let undo: Vec<_> = applied
            .iter()
            .map(|p| json!({"path":p,"before":before.get(p),"after":after.get(p)}))
            .collect();
        let outcome = json!({"constructed":true,"runtime":"unrun","applied":applied,"error":failure,"editors":if failure.is_none(){editors}else{vec![]},"undo":undo});
        // A separate provenance record; never rewrite the accepted artifact or its evidence.
        let mut outcome = outcome;
        if let Err(error) = record.context("authored-changeset-application", &json!({"turn":turn,"outcome":outcome,"editorApplication":"client acknowledgement required"})) {
            outcome["recordError"] = json!(format!("Application receipt could not be recorded: {error:#}. Keep this receipt."));
        }
        Ok(Json(outcome))
    })
    .await
}

/// Editor transactions are reported by the owning window, separately from disk writes.
pub async fn report(
    State(state): State<LocalState>,
    Path((doc, turn)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    let capture = super::agent_projects::captured(&state, &doc, &turn)?;
    super::acp::record::Record::existing(capture.session)
        .context("authored-changeset-editor-application",&json!({"turn":turn,"clientReport":body,"runtime":"unrun","standing":"client-reported"}))
        .map_err(|e|ApiError::internal(e.to_string()))?;
    Ok(Json(json!({"recorded":true})))
}

pub fn routes() -> axum::Router<LocalState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route(
            "/docs/{id}/agent/projects/{turn}",
            get(super::agent_projects::snapshot).post(super::agent_projects::finalize),
        )
        .route("/docs/{id}/agent/projects/{turn}/draft/apply", post(apply))
        .route(
            "/docs/{id}/agent/projects/{turn}/draft/result",
            post(report),
        )
        .route(
            "/docs/{id}/agent/projects/{turn}/incorporate",
            get(super::project_incorporation::status).post(super::project_incorporation::propose),
        )
}
</book:entry>

</book:doc>
