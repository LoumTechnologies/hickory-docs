//! One captured A/B boundary for every dock harness, including native ACP writes.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Path, State},
};
use hick_lang::{AgentProjectCapture, ProjectInventory, ProjectSnapshot, ProjectTree};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct CapturedTurn {
    pub root: PathBuf,
    pub entry: String,
    pub documents: BTreeMap<String, String>,
    pub inventory: ProjectInventory,
    pub starting: ProjectSnapshot,
    pub completed: Option<ProjectSnapshot>,
    pub ambiguous: BTreeSet<String>,
    pub error: Option<String>,
    pub response: Option<String>,
    #[serde(default)]
    pub artifact: Option<crate::changeset::Artifact>,
    pub session: PathBuf,
    #[serde(default)]
    rooms: BTreeMap<String, String>,
}

pub async fn begin(
    state: &LocalState,
    doc: &str,
    session: PathBuf,
) -> anyhow::Result<Option<CapturedTurn>> {
    let Some(document) = state.index.absolute(doc) else {
        return Ok(None);
    };
    let source = if let Some(room) = state.rooms.get(doc).await {
        room.text().await
    } else {
        state
            .read_source(doc)
            .map_err(|e| anyhow::anyhow!("{}", e.detail()))?
    };
    if hick_lang::parse(&source)?.find_tags("project").is_empty() {
        return Ok(None);
    }
    let root = document
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Project needs a directory"))?
        .canonicalize()?;
    let entry = document
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Project needs a UTF-8 entry"))?
        .to_owned();
    let mut live = BTreeMap::new();
    let mut rooms = BTreeMap::new();
    for (id, _) in state.index.entries() {
        if let Some(room) = state.rooms.get(&id).await {
            let text = room.text().await;
            rooms.insert(id.clone(), text.clone());
            if let Some(path) = state.index.absolute(&id).and_then(|p| {
                p.strip_prefix(&root)
                    .ok()
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
            }) {
                live.insert(path, text);
            }
        }
    }
    let documents = hick_lang::load_project_sources(&entry, &|name| {
        if name == entry {
            return Ok(source.clone());
        }
        if let Some(source) = live.get(name) {
            return Ok(source.clone());
        }
        let path = root.join(name);
        if !std::fs::symlink_metadata(&path)
            .map_err(|e| e.to_string())?
            .is_file()
            || !path
                .canonicalize()
                .map_err(|e| e.to_string())?
                .starts_with(&root)
        {
            return Err(format!("Unconfined Markdown source: {name}"));
        }
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    })
    .map_err(anyhow::Error::msg)?;
    let project = hick_lang::evaluate_project(&entry, &documents, &ProjectTree::default())
        .map_err(anyhow::Error::msg)?;
    let starting = crate::project_revision::directory(&root, Some(&project.inventory))?;
    Ok(Some(CapturedTurn {
        root,
        entry,
        documents,
        inventory: project.inventory,
        starting: ProjectSnapshot::capture(&starting),
        completed: None,
        ambiguous: BTreeSet::new(),
        error: None,
        response: None,
        artifact: None,
        session,
        rooms,
    }))
}

pub async fn finish(
    state: &LocalState,
    turn: &str,
    mut capture: CapturedTurn,
    explanation: Option<&str>,
) {
    let result = async {
        let completed =
            crate::project_revision::directory(&capture.root, Some(&capture.inventory))?;
        // Native filesystem writes have no exclusive writer identity. Do not
        // silently call an external editor's write the agent's successful act.
        let starting = capture.starting.restore().map_err(anyhow::Error::msg)?;
        capture
            .ambiguous
            .extend(starting.compare(&completed).into_iter().map(|d| d.path));
        for (id, source) in &capture.rooms {
            if let Some(room) = state.rooms.get(id).await
                && room.text().await != *source
            {
                capture.ambiguous.insert(format!(
                    "editor:{}",
                    state.index.path_of(id).unwrap_or(id.clone())
                ));
            }
        }
        if capture
            .artifact
            .as_ref()
            .is_some_and(|a| a.expected != crate::changeset::fingerprint(&completed))
        {
            capture.artifact = None;
            capture.response = None;
        }
        capture.completed = Some(ProjectSnapshot::capture(&completed));
        if explanation.is_none() {
            anyhow::bail!("Turn did not complete successfully; its actual writes remain captured.");
        }
        Ok::<_, anyhow::Error>(())
    }
    .await;
    capture.error = result.err().map(|e| format!("{e:#}"));
    let record = super::acp::record::Record::existing(capture.session.clone());
    if let Err(e) = record.context(
        "literate-project-capture",
        &json!({"turn":turn,"capture":capture}),
    ) {
        capture.error = Some(format!("Project capture could not be recorded: {e:#}"));
    }
    state
        .agent
        .projects
        .lock()
        .unwrap()
        .insert(turn.into(), capture);
}

fn captured(state: &LocalState, doc: &str, turn: &str) -> ApiResult<CapturedTurn> {
    let record = state
        .agent
        .turns
        .lock()
        .unwrap()
        .get(doc)
        .and_then(|ts| ts.iter().find(|t| t.id == turn))
        .cloned()
        .ok_or_else(|| ApiError::not_found("No turn in this conversation."))?;
    if let Some(capture) = state.agent.projects.lock().unwrap().get(turn).cloned() {
        return Ok(capture);
    }
    let session = state.index.root().join(record.session);
    let source = std::fs::read_to_string(&session)
        .map_err(|_| ApiError::not_found("This turn has no declared-project capture."))?;
    let parsed = hick_lang::parse(&source).map_err(|e| ApiError::unprocessable(e.to_string()))?;
    let value = parsed
        .all_tags()
        .into_iter()
        .filter(|t| {
            t.name == "context"
                && matches!(
                    t.get_attribute("kind"),
                    Some("literate-project-capture" | "literate-project-capture-error")
                )
        })
        .filter_map(|t| serde_json::from_str::<Value>(&t.text_content()).ok())
        .rfind(|v| v["turn"] == turn)
        .ok_or_else(|| {
            ApiError::not_found(
                "This turn has no complete project capture. Start a turn on a declared project.",
            )
        })?;
    if let Some(error) = value["error"].as_str() {
        return Err(ApiError::unprocessable(format!(
            "Incomplete project capture: {error}"
        )));
    }
    let capture = serde_json::from_value(value["capture"].clone())
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    Ok(capture)
}

pub async fn snapshot(
    State(state): State<LocalState>,
    Path((doc, turn)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    let capture = captured(&state, &doc, &turn)?;
    Ok(Json(
        json!({"entry":capture.entry,"ambiguous":capture.ambiguous,"error":capture.error,"response":capture.response,"artifact":capture.artifact,"starting":capture.starting,"completed":capture.completed}),
    ))
}

#[derive(Deserialize)]
pub struct Finalize {
    #[serde(default)]
    pub resolve: Vec<String>,
}
pub async fn finalize(
    State(state): State<LocalState>,
    Path((doc, turn)): Path<(String, String)>,
    Json(body): Json<Finalize>,
) -> ApiResult<Json<Value>> {
    let mut capture = captured(&state, &doc, &turn)?;
    if let Some(error) = &capture.error {
        return Err(ApiError::unprocessable(format!(
            "Incomplete capture: {error}"
        )));
    }
    for path in body.resolve {
        if !capture.ambiguous.remove(&path) {
            return Err(ApiError::bad_request(format!(
                "No unresolved captured change: {path}"
            )));
        }
    }
    let mut boundary = AgentProjectCapture::new(
        capture
            .starting
            .restore()
            .map_err(ApiError::unprocessable)?,
    );
    for path in &capture.ambiguous {
        boundary.concurrent_write(path);
    }
    let completed = capture
        .completed
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("Actual completed output state is unavailable."))?
        .restore()
        .map_err(ApiError::unprocessable)?;
    let explanation = state
        .agent
        .turns
        .lock()
        .unwrap()
        .get(&doc)
        .and_then(|ts| ts.iter().find(|t| t.id == turn))
        .and_then(|t| t.answer.clone())
        .ok_or_else(|| ApiError::unprocessable("No successful final answer."))?;
    let response = if let Some(artifact) = &capture.artifact {
        if !capture.ambiguous.is_empty() {
            return Err(ApiError::conflict(
                "Resolve concurrent writes before attributing the transition.",
            ));
        }
        let differences = crate::changeset::verify_sources(
            &capture
                .starting
                .restore()
                .map_err(ApiError::unprocessable)?,
            &completed,
            &artifact.entry,
            &artifact.sources,
        )
        .map_err(|e| ApiError::conflict(e.to_string()))?;
        if !differences.is_empty() {
            return Err(ApiError::conflict(
                "Submitted artifact does not reproduce the captured completed tree.",
            ));
        }
        artifact
            .sources
            .get(&artifact.entry)
            .cloned()
            .ok_or_else(|| ApiError::unprocessable("Artifact entry is missing."))?
    } else {
        boundary
            .finalize(&completed, &explanation)
            .map_err(ApiError::conflict)?
    };
    capture.response = Some(response.clone());
    super::acp::record::Record::existing(capture.session.clone())
        .context(
            "literate-project-capture",
            &json!({"turn":turn,"capture":capture,"resolution":"user-declared"}),
        )
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    state.agent.projects.lock().unwrap().insert(turn, capture);
    Ok(Json(json!({"response":response,"verified":true})))
}

pub(super) fn completed_capture(
    state: &LocalState,
    doc: &str,
    turn: &str,
) -> ApiResult<CapturedTurn> {
    let capture = captured(state, doc, turn)?;
    if capture.response.is_none() || capture.error.is_some() || !capture.ambiguous.is_empty() {
        return Err(ApiError::conflict(
            "Finalize the captured transition and resolve concurrent changes before incorporation.",
        ));
    }
    Ok(capture)
}
