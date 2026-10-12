//! AI proposes maintained source; equality and the existing review queue gate publication.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    project_publish,
};
use axum::{
    Json,
    extract::{Path, State},
};
use hick_lang::IncorporationCandidate;
use hickory_agent::{Message, Role};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct Proposal {
    /// Optional authored candidate uses the same checker and review as AI output.
    pub documents: Option<BTreeMap<String, String>>,
}
pub async fn status(
    State(state): State<LocalState>,
    Path((doc, turn)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    super::agent_projects::completed_capture(&state, &doc, &turn)?;
    Ok(Json(
        state
            .agent
            .incorporations
            .lock()
            .unwrap()
            .get(&turn)
            .cloned()
            .unwrap_or(json!({"status":"idle"})),
    ))
}
pub async fn propose(
    State(state): State<LocalState>,
    Path((doc, turn)): Path<(String, String)>,
    Json(body): Json<Proposal>,
) -> ApiResult<Json<Value>> {
    let capture = super::agent_projects::completed_capture(&state, &doc, &turn)?;
    if state
        .agent
        .turns
        .lock()
        .unwrap()
        .get(&doc)
        .is_some_and(|ts| ts.iter().any(|t| t.status == "running"))
    {
        return Err(ApiError::conflict(
            "Wait for the active turn before incorporating source.",
        ));
    }
    let completed = capture
        .completed
        .as_ref()
        .ok_or_else(|| ApiError::conflict("Completed tree is unavailable."))?
        .restore()
        .map_err(ApiError::unprocessable)?;
    let current =
        project_publish::documents(&state, &capture.root, capture.documents.keys().cloned())
            .await
            .map_err(ApiError::from)?;
    // Include documents added by the agent's source edits, not only A's closure.
    let mut baseline = current;
    let closure = hick_lang::load_project_sources(&capture.entry, &|name| {
        baseline.get(name).cloned().map(Ok).unwrap_or_else(|| {
            std::fs::read_to_string(
                project_publish::confined(&capture.root, name).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())
        })
    })
    .map_err(ApiError::unprocessable)?;
    baseline.extend(closure);
    let llm = if body.documents.is_none() {
        Some(
            state
                .agent
                .incorporation_client(&state, &doc)
                .map_err(|e| {
                    ApiError::unavailable(format!(
                        "Configure an AI provider for incorporation: {e:#}"
                    ))
                })?,
        )
    } else {
        None
    };
    let backend = state
        .agent
        .turns
        .lock()
        .unwrap()
        .get(&doc)
        .and_then(|ts| ts.iter().find(|t| t.id == turn))
        .map(|t| t.backend.clone())
        .unwrap_or_default();
    let queue = if matches!(backend.as_str(), "builtin" | "python") {
        super::agent_review::queue(&state, &doc).edits.clone()
    } else {
        super::acp::project_review(&state, &doc).await?
    };
    {
        let mut jobs = state.agent.incorporations.lock().unwrap();
        if jobs.get(&turn).is_some_and(|v| {
            matches!(
                v["status"].as_str(),
                Some("generating" | "reviewing" | "applying")
            )
        }) {
            return Err(ApiError::conflict("An incorporation is already pending."));
        }
        jobs.insert(turn.clone(), json!({"status":"generating"}));
    }
    tokio::spawn(async move {
        let result = async {
            let mut candidate = if let Some(documents) = body.documents { documents } else {
                let response = llm.unwrap().complete(vec![
                    Message::new(Role::System,"Rewrite maintained literate Markdown to describe and construct the complete current program from an empty tree. Return only a JSON object mapping source .md paths to their complete text. The entry path must remain. You may reorganize, create and remove source documents. No history, working code or sidecars may supply bytes. Preserve the project's explicit inventory exclusions. Hick namespaces structure only prefixed tags; no entity escaping. UTF-8 hick:file and hick:patch, explicit hex/symlink hick:entry construct files. Patches require after dependencies. Keep the successful explanation, not abandoned steps. Necessary source discovery must be in hick:project sources. Outputs must match the supplied completed byte tree exactly."),
                    Message::new(Role::User,serde_json::to_string(&json!({"entry":capture.entry,"documents":baseline,"response":capture.response,"completed":capture.completed}))?),
                ]).await?;
                serde_json::from_str::<BTreeMap<String,String>>(&response)?
            };
            for path in candidate.keys() { project_publish::confined(&capture.root,path)?; }
            let _verified = IncorporationCandidate::verify(&capture.entry,baseline.clone(),candidate.clone(),completed.clone()).map_err(|e| anyhow::anyhow!("Incorporation equality failed: {e:?}"))?;
            super::acp::record::Record::existing(capture.session.clone()).context("literate-incorporation-candidate",&json!({"turn":turn,"documents":candidate,"source_equality":"verified","runtime_equality":"unrun"}))?;
            state.agent.incorporations.lock().unwrap().insert(turn.clone(),json!({"status":"reviewing","documents":candidate}));
            let paths: std::collections::BTreeSet<_> = baseline.keys().chain(candidate.keys()).cloned().collect();
            let mut approvals = vec![];
            for path in &paths {
                if baseline.get(path) == candidate.get(path) { continue; }
                let id = queue.submit_review(format!("Incorporate verified literate source · {}",if candidate.contains_key(path) { path.as_str() } else { "remove document" }),Some(path.clone()),baseline.get(path).cloned().unwrap_or_default(),candidate.get(path).cloned().unwrap_or_default()).await?;
                if let Some(revised) = queue.replacement(&id) { candidate.insert(path.clone(),revised); }
                approvals.push(id);
            }
            let verified = IncorporationCandidate::verify(&capture.entry,baseline.clone(),candidate.clone(),completed.clone()).map_err(|e| anyhow::anyhow!("Reviewed source no longer constructs B: {e:?}"))?;
            state.agent.incorporations.lock().unwrap().insert(turn.clone(),json!({"status":"applying"}));
            let _guard = state.writes.lock().await;
            let destinations = project_publish::documents(&state,&capture.root,paths).await?;
            let code = crate::project_revision::directory(&capture.root,Some(&capture.inventory))?;
            verified.recheck_destinations(&destinations,&code).map_err(anyhow::Error::msg)?;
            let applied = project_publish::publish(&state,&capture.root,&baseline,&candidate).await?;
            for id in approvals { queue.finished(&id,true); }
            super::acp::record::Record::existing(capture.session).context("literate-incorporation-applied",&json!({"turn":turn,"destinations":applied,"code_written":false}))?;
            Ok::<_,anyhow::Error>(applied)
        }.await;
        state.agent.incorporations.lock().unwrap().insert(
            turn,
            match result {
                Ok(applied) => json!({"status":"applied","destinations":applied}),
                Err(e) => json!({"status":"failed","error":format!("{e:#}")}),
            },
        );
    });
    Ok(Json(json!({"status":"generating"})))
}
