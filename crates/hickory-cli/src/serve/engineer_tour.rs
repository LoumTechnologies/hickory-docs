//! Explicit isolated practice repository; no implicit edits to the open project.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{Json, extract::State};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
const MARKER: &str = ".hickory-engineer-tour.json";
const MEETING: &str = include_str!("../../../../examples/ai-engineer-meeting.vtt");
fn base() -> anyhow::Result<PathBuf> {
    Ok(hickory_workspace::state_root()?.join("engineer-tours"))
}
fn seed(base: &Path) -> anyhow::Result<PathBuf> {
    let id = hick_context_store::fresh_id();
    let root = base.join(&id);
    std::fs::create_dir_all(&root)?;
    let source = include_str!("../../../../examples/ai-engineer-deadline.md").replace(
        "var allowed = Math.max(0, budget - total);",
        "var allowed = budget;",
    );
    std::fs::write(root.join("deadline-spec.md"), source)?;
    std::fs::write(
        root.join("deadline.js"),
        include_str!("../../../../examples/deadline.js").replace(
            "var allowed = Math.max(0, budget - total);",
            "var allowed = budget;",
        ),
    )?;
    std::fs::write(root.join("meeting.vtt"), MEETING)?;
    std::fs::write(
        root.join("AGENTS.md"),
        "# Practice instructions\n\nThe request deadline is a total budget. Retain the independent short-attempt and empty-sequence checks.\n",
    )?;
    std::fs::write(
        root.join("hotspot.md"),
        include_str!("../../../../examples/python-profile/hotspot.md"),
    )?;
    std::fs::write(
        root.join(MARKER),
        serde_json::to_vec_pretty(
            &json!({"version":1,"id":id,"seed":"Recorded meeting/proposal fixtures; subsequent actions and evidence are real product operations."}),
        )?,
    )?;
    std::fs::write(root.join(".gitignore"), "sessions/\n.hick-cache/\n.hick/\n")?;
    std::fs::write(
        root.join("README.md"),
        "# Isolated AI engineer practice\n\nThe meeting and original proposal are transparent seeded examples. Import meeting.vtt with the normal importer, open deadline-spec.md, and use the AI engineer tour. Real model requests require your configured provider and explicit Send action. Profile hotspot.md using the real confined Python runtime. This repository is separate from your original project.\n",
    )?;
    for args in [
        vec!["init", "--initial-branch=master"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Hickory recorded fixture generator",
            "-c",
            "user.email=fixture@hickory.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            "Seed transparent AI engineer practice fixtures",
        ],
    ] {
        let result = std::process::Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()?;
        anyhow::ensure!(
            result.status.success(),
            "Practice Git setup failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    Ok(root)
}
fn failure(error: impl std::fmt::Display) -> ApiError {
    ApiError::unprocessable(error.to_string())
}
pub async fn create(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let parent = base().map_err(|e| ApiError::unprocessable(e.to_string()))?;
    std::fs::create_dir_all(&parent).map_err(failure)?;
    if parent
        .canonicalize()
        .map_err(failure)?
        .starts_with(state.index.root().canonicalize().map_err(failure)?)
    {
        return Err(ApiError::bad_request(
            "Practice repository storage must be outside the current workspace",
        ));
    }
    let root = tokio::task::spawn_blocking(move || seed(&parent))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    let opened = state.open_folder(&root, super::shell::OpenWhere::NewWindow);
    Ok(Json(
        json!({"path":root,"opened":opened.is_ok(),"open_error":opened.err().map(|e|e.to_string()),"coverage":"Created a separate Git repository. The current workspace was not edited."}),
    ))
}
fn delivered_observations(source: &str) -> Vec<Value> {
    fn annotations(value: &Value, found: &mut Vec<String>) {
        match value {
            Value::Object(object) => {
                if object
                    .get("payload")
                    .and_then(|p| p.get("kind"))
                    .and_then(Value::as_str)
                    .is_some_and(|kind| matches!(kind, "paused" | "breakpoint_bound" | "profile"))
                    && object.get("scope").is_some_and(Value::is_object)
                {
                    if let Some(id) = object.get("evidence").and_then(Value::as_str) {
                        found.push(id.into());
                    }
                }
                for child in object.values() {
                    annotations(child, found);
                }
            }
            Value::Array(array) => {
                for child in array {
                    annotations(child, found);
                }
            }
            _ => (),
        }
    }
    let mut confirmed = std::collections::BTreeSet::new();
    let mut manifests = vec![];
    for line in source.lines() {
        for kind in ["python-context-delivery", "python-context-manifest"] {
            if let Some(raw) = line
                .strip_prefix(&format!("<hick:context kind=\"{kind}\">"))
                .and_then(|s| s.strip_suffix("</hick:context>"))
            {
                if let Ok(value) = serde_json::from_str::<Value>(raw) {
                    if kind == "python-context-delivery" && value["state"] == "provider_confirmed" {
                        if let Some(id) = value["request"].as_str() {
                            confirmed.insert(id.to_string());
                        }
                    }
                    if kind == "python-context-manifest" {
                        manifests.push(value);
                    }
                }
            }
        }
    }
    manifests
        .into_iter()
        .filter_map(|manifest| {
            let request = &manifest["request"];
            if !confirmed.contains(request["id"].as_str()?) {
                return None;
            }
            let mut evidence = vec![];
            for part in request["parts"].as_array()? {
                if part["role"] != "user" {
                    continue;
                }
                if let Some(raw) = part["text"]
                    .as_str()
                    .and_then(|t| t.strip_prefix("Editor context (user content):\n"))
                {
                    if let Ok(context) = serde_json::from_str::<Value>(raw) {
                        annotations(&context, &mut evidence);
                    }
                }
            }
            (!evidence.is_empty()).then(
                || json!({"request":request["id"],"branch":request["branch"],"evidence":evidence}),
            )
        })
        .collect()
}
fn session_observations(root: &Path) -> Vec<Value> {
    session_evidence(root, delivered_observations)
}
fn confirmed_requests(source: &str) -> Vec<Value> {
    let mut confirmed = std::collections::BTreeSet::new();
    let mut requests = vec![];
    for line in source.lines() {
        for kind in ["python-context-delivery", "python-context-manifest"] {
            if let Some(raw) = line
                .strip_prefix(&format!("<hick:context kind=\"{kind}\">"))
                .and_then(|s| s.strip_suffix("</hick:context>"))
                && let Ok(value) = serde_json::from_str::<Value>(raw)
            {
                if kind == "python-context-delivery" && value["state"] == "provider_confirmed" {
                    if let Some(id) = value["request"].as_str() {
                        confirmed.insert(id.to_owned());
                    }
                }
                if kind == "python-context-manifest" {
                    requests.push(value["request"].clone());
                }
            }
        }
    }
    requests.into_iter().filter_map(|request| confirmed.contains(request["id"].as_str()?).then(|| json!({"request":request["id"],"branch":request["branch"],"state":"provider_confirmed","coverage":"Actual Python request delivery; not evidence of model understanding"}))).collect()
}
fn reviewed_edits(source: &str) -> Vec<Value> {
    let mut branch = Value::Null;
    let mut approved: Option<Value> = None;
    let mut edits = vec![];
    for line in source.lines() {
        for kind in [
            "python-context-manifest",
            "python-edit-review",
            "python-reviewed-replacement",
            "python-tool-result",
        ] {
            let Some(raw) = line
                .strip_prefix(&format!("<hick:context kind=\"{kind}\">"))
                .and_then(|s| s.strip_suffix("</hick:context>"))
            else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<Value>(raw) else {
                continue;
            };
            match kind {
                "python-context-manifest" => branch = value["request"]["branch"].clone(),
                "python-edit-review" => {
                    approved = (value["approved"] == true)
                        .then(|| json!({"proposal":value["proposal"],"branch":branch}))
                }
                "python-reviewed-replacement" => {
                    if let Some(review) = approved.as_mut() {
                        // The human can replace the preview before the host executes it.
                        // Retain those effective bytes, rather than the model's proposal.
                        review["proposal"]["after"] = value["arguments"]["input"].clone();
                    }
                }
                "python-tool-result" => {
                    if let Some(review) = approved.take() {
                        let proposal = &review["proposal"];
                        if value["name"] == proposal["tool"]
                            && matches!(value["name"].as_str(), Some("edit_doc" | "edit_output"))
                            && value["result"].is_object()
                            && value["result"]["isError"] != true
                            && value["error"].is_null()
                            && let Some(after) = proposal["after"].as_str()
                        {
                            edits.push(json!({"surface":proposal["surface"],"tool":proposal["tool"],"branch":review["branch"],"source_hash":hex::encode(Sha256::digest(after.as_bytes())),"scope":"Host-observed successful tool after independent approval; compare current bytes separately"}));
                        }
                    }
                }
                _ => (),
            }
        }
    }
    edits
}
fn session_evidence(root: &Path, extract: fn(&str) -> Vec<Value>) -> Vec<Value> {
    let mut result = vec![];
    let sessions = root.join("sessions");
    if sessions
        .canonicalize()
        .is_ok_and(|path| !path.starts_with(root))
    {
        return result;
    }
    if let Ok(entries) = std::fs::read_dir(sessions) {
        for entry in entries.flatten().take(100) {
            if entry.path().extension().is_some_and(|e| e == "md")
                && entry
                    .path()
                    .canonicalize()
                    .is_ok_and(|p| p.starts_with(root))
                && entry
                    .metadata()
                    .is_ok_and(|m| m.is_file() && m.len() < 8 * 1024 * 1024)
            {
                if let Ok(text) = std::fs::read_to_string(entry.path()) {
                    result.extend(extract(&text));
                }
            }
        }
    }
    result
}
pub async fn status(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let root = state.index.root().canonicalize().map_err(failure)?;
    let parent = base().map_err(|e| ApiError::unprocessable(e.to_string()))?;
    if !root.starts_with(&parent) || !root.join(MARKER).is_file() {
        return Ok(Json(json!({"active":false})));
    }
    let files = hick_context_store::Files::new(&root, 8 * 1024 * 1024).map_err(failure)?;
    let marker: Value = serde_json::from_str(&files.read(MARKER).map_err(failure)?)
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    if marker["version"] != 1 || marker["id"].as_str() != root.file_name().and_then(|p| p.to_str())
    {
        return Err(ApiError::bad_request("Invalid practice repository marker"));
    }
    let source = files.read("deadline-spec.md").map_err(failure)?;
    let profile_source = files.read("hotspot.md").map_err(failure)?;
    let doc_id = state.index.add("deadline-spec.md");
    let executions: Vec<_> = state.runs.lock().unwrap().iter().filter(|(_, run)| run.document == doc_id).map(|(id,run)| json!({"id":id,"status":run.status,"source_hash":run.source_hash,"requested_source_hash":run.requested_source_hash,"error":run.error,"started_at":run.started_at,"blocks":run.blocks})).collect();
    let seed_source = include_str!("../../../../examples/ai-engineer-deadline.md").replace(
        "var allowed = Math.max(0, budget - total);",
        "var allowed = budget;",
    );
    let seed_source_hash = hex::encode(Sha256::digest(seed_source.as_bytes()));
    let debug_evidence = super::ui_actions::retained_debug_witnesses(&state, &seed_source_hash);
    let agent_edits: Vec<_> = session_evidence(&root, reviewed_edits)
        .into_iter()
        .filter(|edit| {
            edit["surface"]
                .as_str()
                .is_some_and(|surface| Path::new(surface) == root.join("deadline-spec.md"))
        })
        .collect();
    Ok(Json(
        json!({"active":true,"agent_requests":session_evidence(&root, confirmed_requests),"agent_edits":agent_edits,"executions":executions,"debug_evidence":debug_evidence,"seed_source_hash":seed_source_hash,"delivered_observations":session_observations(&root),"id":marker["id"],"path":root,"source":source,"source_hash":hex::encode(Sha256::digest(source.as_bytes())),"profile_source":profile_source,"meeting":MEETING,"doc_id":doc_id,"profile_doc_id":state.index.add("hotspot.md"),"seed":marker["seed"]}),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn correction_witness_requires_success_and_hashes_the_reviewed_replacement() {
        let record = |kind: &str, value: Value| {
            format!("<hick:context kind=\"{kind}\">{value}</hick:context>\n")
        };
        let approved = record("python-context-manifest", json!({"request":{"branch":"b"}}))
            + &record(
                "python-edit-review",
                json!({"approved":true,"proposal":{"tool":"edit_doc","surface":"/practice/deadline-spec.md","after":"model bytes"}}),
            );
        assert!(reviewed_edits(&approved).is_empty());
        let replaced = approved
            + &record(
                "python-reviewed-replacement",
                json!({"arguments":{"input":"human replacement"}}),
            );
        let failed = replaced.clone()
            + &record(
                "python-tool-result",
                json!({"name":"edit_doc","result":{"isError":true}}),
            );
        assert!(reviewed_edits(&failed).is_empty());
        let successful = replaced
            + &record(
                "python-tool-result",
                json!({"name":"edit_doc","result":{"content":[]}}),
            );
        let edits = reviewed_edits(&successful);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0]["branch"], "b");
        assert_eq!(
            edits[0]["source_hash"],
            hex::encode(Sha256::digest(b"human replacement"))
        );
    }
    #[test]
    fn coach_delegation_requires_a_correlated_provider_delivery() {
        let manifest = "<hick:context kind=\"python-context-manifest\">{\"request\":{\"id\":\"request-actual\",\"branch\":\"branch-actual\"}}</hick:context>\n";
        assert!(confirmed_requests(manifest).is_empty());
        let wrong = format!(
            "{manifest}<hick:context kind=\"python-context-delivery\">{{\"request\":\"another\",\"state\":\"provider_confirmed\"}}</hick:context>\n"
        );
        assert!(confirmed_requests(&wrong).is_empty());
        let actual = wrong.replace("another", "request-actual");
        assert_eq!(confirmed_requests(&actual)[0]["branch"], "branch-actual");
        assert!(confirmed_requests(&actual.replace("provider_confirmed", "cancelled")).is_empty());
    }
    // Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
    #[test]
    fn practice_is_a_separate_git_repository_with_a_transparent_counterexample() {
        let temporary = tempfile::tempdir().unwrap();
        let original = temporary.path().join("original");
        std::fs::create_dir(&original).unwrap();
        std::fs::write(original.join("untouched.md"), "human bytes").unwrap();
        let practice = seed(&temporary.path().join("practice")).unwrap();
        assert!(!practice.starts_with(&original));
        assert!(practice.join(".git").is_dir());
        assert!(
            std::fs::read_to_string(practice.join("deadline-spec.md"))
                .unwrap()
                .contains("var allowed = budget;")
        );
        assert!(
            std::fs::read_to_string(practice.join("hotspot.md"))
                .unwrap()
                .contains("for value in range(n)")
        );
        assert_eq!(
            std::fs::read_to_string(original.join("untouched.md")).unwrap(),
            "human bytes"
        );
        assert!(!original.join(".git").exists());
    }
    // Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
    #[test]
    fn observation_delivery_requires_exact_structured_context_and_a_confirmed_receipt() {
        let context = json!({"editors":{"projection":{"observations":[{"reading":{"annotations":[{"evidence":"profile-real","scope":{"run":"r"},"payload":{"kind":"profile"}}]}}]}}});
        let manifest = json!({"request":{"id":"request","branch":"branch","parts":[{"role":"user","text":format!("Editor context (user content):\n{context}")}]}});
        let record =
            format!("<hick:context kind=\"python-context-manifest\">{manifest}</hick:context>\n");
        assert!(delivered_observations(&record).is_empty());
        let confirmed = record
            + "<hick:context kind=\"python-context-delivery\">{\"request\":\"request\",\"state\":\"provider_confirmed\"}</hick:context>\n";
        let evidence = delivered_observations(&confirmed);
        assert_eq!(evidence[0]["evidence"], json!(["profile-real"]));
        let forged = confirmed.replace("Editor context (user content):", "Agent says context:");
        assert!(delivered_observations(&forged).is_empty());
    }
    // Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
    #[cfg(unix)]
    #[test]
    fn practice_observation_scan_does_not_follow_external_session_directories() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let context = json!({"reading":{"annotations":[{"evidence":"private","scope":{"run":"r"},"payload":{"kind":"profile"}}]}});
        let manifest = json!({"request":{"id":"request","branch":"branch","parts":[{"role":"user","text":format!("Editor context (user content):\n{context}")}]}});
        let source = format!(
            "<hick:context kind=\"python-context-manifest\">{manifest}</hick:context>\n<hick:context kind=\"python-context-delivery\">{{\"request\":\"request\",\"state\":\"provider_confirmed\"}}</hick:context>\n"
        );
        assert_eq!(delivered_observations(&source).len(), 1);
        std::fs::write(outside.path().join("private.md"), source).unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("sessions")).unwrap();
        assert!(session_observations(root.path()).is_empty());
    }
}
