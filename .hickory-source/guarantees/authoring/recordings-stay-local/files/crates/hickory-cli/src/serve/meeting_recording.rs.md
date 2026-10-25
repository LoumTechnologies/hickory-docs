<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/meeting_recording.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Meeting recordings remain recoverable local evidence](../../../../../contract.md)

Captured SHA-256: `181b030284c49b20c9c7d7f64140fc03d2b88700e07a8fab62d15cbb11ba492d`.

<book:entry path="crates/hickory-cli/src/serve/meeting_recording.rs">
//! Local retained audio. Transcription requires an explicit configured adapter.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Path, State},
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{io::Write, path::PathBuf};
static AUDIO_GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());
const MAX_AUDIO: u64 = 256 * 1024 * 1024;
#[derive(Clone, Serialize, Deserialize)]
struct Recording {
    id: String,
    mime: String,
    bytes: u64,
    sha256: String,
    complete: bool,
    created: String,
}
#[derive(Default, PartialEq, Serialize, Deserialize)]
pub struct Adapter {
    mode: String,
    command: String,
    #[serde(default)]
    args: Vec<String>,
    endpoint: String,
    model: String,
}
#[derive(Deserialize)]
pub struct Action {
    #[serde(default)]
    id: String,
    #[serde(default)]
    mime: String,
    #[serde(default)]
    data: String,
    #[serde(default)]
    offset: u64,
    #[serde(default)]
    complete: bool,
    #[serde(default)]
    remote_consent: bool,
    #[serde(default)]
    expected_adapter: Option<Adapter>,
}
fn directory(state: &LocalState) -> anyhow::Result<PathBuf> {
    let path = hickory_workspace::WorkspaceStore::for_project(state.index.root())?
        .dir()
        .join("recordings");
    // Even a custom state override must not place recordings in this Git tree.
    let root = state.index.root().canonicalize()?;
    std::fs::create_dir_all(&path)?;
    anyhow::ensure!(
        !path.canonicalize()?.starts_with(root),
        "Recording storage must be outside the workspace; change HICKORY_STATE_DIR"
    );
    Ok(path)
}
fn identity(id: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        id.len() == 36 && id.bytes().all(|c| c.is_ascii_hexdigit() || c == b'-'),
        "Invalid recording identity"
    );
    Ok(())
}
fn error(e: impl std::fmt::Display) -> ApiError {
    ApiError::unprocessable(e.to_string())
}
fn load(dir: &std::path::Path, id: &str) -> anyhow::Result<Recording> {
    identity(id)?;
    Ok(serde_json::from_slice(&std::fs::read(
        dir.join(format!("{id}.json")),
    )?)?)
}
fn save(dir: &std::path::Path, record: &Recording) -> anyhow::Result<()> {
    super::store::write_atomic(
        &dir.join(format!("{}.json", record.id)),
        &serde_json::to_vec_pretty(record)?,
    )
}
fn config(state: &LocalState) -> anyhow::Result<Adapter> {
    let path = super::acp::config::directory(state).join("transcription.json");
    match std::fs::read(path) {
        Ok(raw) => Ok(serde_json::from_slice(&raw)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Adapter::default()),
        Err(e) => Err(e.into()),
    }
}
pub async fn catalogue(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let dir = directory(&state).map_err(error)?;
    let adapter = config(&state).map_err(error)?;
    let mut recordings = vec![];
    for entry in std::fs::read_dir(&dir).map_err(error)?.flatten() {
        if entry.path().extension().is_some_and(|e| e == "json") {
            if let Ok(record) =
                serde_json::from_slice::<Recording>(&std::fs::read(entry.path()).map_err(error)?)
            {
                recordings.push(record);
            }
        }
    }
    Ok(Json(
        json!({"directory":dir,"max_bytes":MAX_AUDIO,"recordings":recordings,"adapter":adapter,
        "capabilities":{"microphone":"Browser permission and MediaRecorder required","system_audio":"unavailable"},
        "retention":"Retained locally until explicit deletion; no audio is committed or uploaded automatically."}),
    ))
}
pub async fn configure(
    State(state): State<LocalState>,
    Json(body): Json<Adapter>,
) -> ApiResult<Json<Value>> {
    if !matches!(body.mode.as_str(), "local" | "remote") {
        return Err(ApiError::bad_request(
            "Select local or remote transcription",
        ));
    }
    if body.mode == "local" && !std::path::Path::new(&body.command).is_absolute() {
        return Err(ApiError::bad_request(
            "Choose an absolute local adapter executable",
        ));
    }
    if body.mode == "remote" && !body.endpoint.starts_with("https://") {
        return Err(ApiError::bad_request("Remote adapters require HTTPS"));
    }
    let dir = super::acp::config::directory(&state);
    std::fs::create_dir_all(&dir).map_err(error)?;
    super::store::write_atomic(
        &dir.join("transcription.json"),
        &serde_json::to_vec_pretty(&body).map_err(error)?,
    )
    .map_err(error)?;
    catalogue(State(state)).await
}
fn append(dir: &std::path::Path, body: Action) -> anyhow::Result<Recording> {
    let _guard = AUDIO_GATE.lock().unwrap();
    let mut record = if body.id.is_empty() {
        anyhow::ensure!(
            matches!(
                body.mime.as_str(),
                "audio/webm" | "audio/ogg" | "audio/mp4" | "audio/wav"
            ),
            "Unsupported capture MIME type"
        );
        Recording {
            id: hick_context_store::fresh_id(),
            mime: body.mime,
            bytes: 0,
            sha256: hex::encode(Sha256::digest([])),
            complete: false,
            created: format!(
                "unix:{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
        }
    } else {
        load(dir, &body.id)?
    };
    anyhow::ensure!(!record.complete, "Recording already stopped");
    anyhow::ensure!(
        record.bytes == body.offset,
        "Recording offset changed; refresh before retrying"
    );
    let bytes = base64::engine::general_purpose::STANDARD.decode(body.data)?;
    anyhow::ensure!(
        bytes.len() <= 1024 * 1024 && record.bytes + bytes.len() as u64 <= MAX_AUDIO,
        "Audio limit reached; stop and retain this recording"
    );
    save(dir, &record)?;
    let path = dir.join(format!("{}.audio", record.id));
    // An interrupted append is recoverable from audio bytes; metadata is rebuilt.
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    anyhow::ensure!(
        file.metadata()?.len() == body.offset,
        "Interrupted append; recover retained audio before continuing"
    );
    file.write_all(&bytes)?;
    file.sync_data()?;
    record.bytes += bytes.len() as u64;
    record.sha256 = if body.complete {
        hex::encode(Sha256::digest(std::fs::read(path)?))
    } else {
        String::new()
    };
    record.complete = body.complete;
    save(dir, &record)?;
    Ok(record)
}
pub async fn capture(
    State(state): State<LocalState>,
    Json(body): Json<Action>,
) -> ApiResult<Json<Value>> {
    let dir = directory(&state).map_err(error)?;
    Ok(Json(
        serde_json::to_value(
            tokio::task::spawn_blocking(move || append(&dir, body))
                .await
                .map_err(error)?
                .map_err(error)?,
        )
        .map_err(error)?,
    ))
}
pub async fn audio(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<axum::response::Response> {
    use axum::response::IntoResponse;
    let dir = directory(&state).map_err(error)?;
    let record = load(&dir, &id).map_err(error)?;
    let bytes = std::fs::read(dir.join(format!("{id}.audio"))).map_err(error)?;
    Ok((
        [
            (axum::http::header::CONTENT_TYPE, record.mime),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{id}.audio\""),
            ),
        ],
        bytes,
    )
        .into_response())
}
pub async fn remove(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let dir = directory(&state).map_err(error)?;
    identity(&id).map_err(error)?;
    if JOBS.lock().unwrap().contains_key(&id) {
        return Err(ApiError::conflict(
            "Cancel transcription before deleting retained evidence",
        ));
    }
    let _guard = AUDIO_GATE.lock().unwrap();
    if !load(&dir, &id).map_err(error)?.complete {
        return Err(ApiError::conflict(
            "Stop or recover capture before deleting audio",
        ));
    }
    // Explicit deletion removes audio and local transcription evidence, never the imported note.
    for suffix in ["audio", "json", "transcript.json"] {
        let path = dir.join(format!("{id}.{suffix}"));
        if path.exists() {
            std::fs::remove_file(path).map_err(error)?;
        }
    }
    for entry in std::fs::read_dir(&dir).map_err(error)?.flatten() {
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with(&format!("{id}-"))
            && entry.file_name().to_string_lossy().ends_with(".json")
        {
            std::fs::remove_file(entry.path()).map_err(error)?;
        }
    }
    Ok(Json(json!({"deleted":id})))
}
static JOBS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::BTreeMap<String, tokio::sync::watch::Sender<bool>>>,
> = std::sync::LazyLock::new(Default::default);
struct Job(String);
impl Drop for Job {
    fn drop(&mut self) {
        JOBS.lock().unwrap().remove(&self.0);
    }
}
pub async fn cancel(Path(id): Path<String>) -> ApiResult<Json<Value>> {
    if let Some(job) = JOBS.lock().unwrap().get(&id) {
        let _ = job.send(true);
    }
    Ok(Json(
        json!({"cancelled":id,"coverage":"Local work cancelled; a remote provider may already have processed or billed the upload."}),
    ))
}
pub async fn recover(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let dir = directory(&state).map_err(error)?;
    let _guard = AUDIO_GATE.lock().unwrap();
    let mut record = load(&dir, &id).map_err(error)?;
    let bytes = std::fs::read(dir.join(format!("{id}.audio"))).map_err(error)?;
    if bytes.len() as u64 > MAX_AUDIO {
        return Err(ApiError::unprocessable("Retained audio exceeds limit"));
    }
    record.bytes = bytes.len() as u64;
    record.sha256 = hex::encode(Sha256::digest(bytes));
    record.complete = true;
    save(&dir, &record).map_err(error)?;
    Ok(Json(serde_json::to_value(record).map_err(error)?))
}
pub async fn transcribe(
    State(state): State<LocalState>,
    Json(body): Json<Action>,
) -> ApiResult<Json<Value>> {
    let dir = directory(&state).map_err(error)?;
    let record = load(&dir, &body.id).map_err(error)?;
    if !record.complete || record.bytes == 0 {
        return Err(ApiError::bad_request(
            "Stop capture and retain nonempty audio first",
        ));
    }
    let adapter = config(&state).map_err(error)?;
    if body.expected_adapter.as_ref() != Some(&adapter) {
        return Err(ApiError::conflict(
            "Transcription configuration changed; inspect saved settings again",
        ));
    }
    let audio = dir.join(format!("{}.audio", record.id));
    let (send, mut cancelled) = tokio::sync::watch::channel(false);
    {
        let mut jobs = JOBS.lock().unwrap();
        if jobs.contains_key(&body.id) {
            return Err(ApiError::conflict("Transcription already running"));
        }
        jobs.insert(body.id.clone(), send);
    }
    let _job = Job(body.id.clone());
    let work = async {
        let text = match adapter.mode.as_str() {
            "local" => {
                let mut command = tokio::process::Command::new(&adapter.command);
                command.args(adapter.args.iter().map(|arg| {
                    arg.replace("{audio}", &audio.to_string_lossy())
                        .replace("{model}", &adapter.model)
                }));
                command
                    .kill_on_drop(true)
                    .stdin(std::process::Stdio::null());
                use tokio::io::AsyncReadExt;
                command
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::null());
                let mut child = command.spawn().map_err(error)?;
                let mut bytes = Vec::new();
                child
                    .stdout
                    .take()
                    .ok_or_else(|| error("Missing adapter stdout"))?
                    .take(1024 * 1024 + 1)
                    .read_to_end(&mut bytes)
                    .await
                    .map_err(error)?;
                if bytes.len() > 1024 * 1024 {
                    return Err(error("Adapter transcript exceeds 1 MB; audio retained"));
                }
                if !child.wait().await.map_err(error)?.success() {
                    return Err(error(
                        "Local transcription failed; retained audio is available for retry",
                    ));
                }
                String::from_utf8(bytes).map_err(error)?
            }
            "remote" => {
                if !adapter.endpoint.starts_with("https://") {
                    return Err(ApiError::bad_request("Remote adapters require HTTPS"));
                }
                if !body.remote_consent {
                    return Err(ApiError::bad_request(
                        "Confirm upload and possible provider charges for this recording",
                    ));
                }
                let bytes = std::fs::read(audio).map_err(error)?;
                let response = reqwest::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .build()
                    .map_err(error)?
                    .post(&adapter.endpoint)
                    .header("Content-Type", &record.mime)
                    .header("X-Hickory-Model", &adapter.model)
                    .body(bytes)
                    .send()
                    .await
                    .map_err(error)?
                    .error_for_status()
                    .map_err(error)?;
                use futures::StreamExt;
                let mut stream = response.bytes_stream();
                let mut bytes = Vec::new();
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk.map_err(error)?;
                    if bytes.len() + chunk.len() > 1024 * 1024 {
                        return Err(ApiError::unprocessable(
                            "Transcript exceeds 1 MB; retained audio available for retry",
                        ));
                    }
                    bytes.extend_from_slice(&chunk);
                }
                String::from_utf8(bytes).map_err(error)?
            }
            _ => {
                return Err(ApiError::bad_request(
                    "Configure a local or remote transcription adapter first",
                ));
            }
        };
        Ok::<String, ApiError>(text)
    };
    let text = tokio::select! {
        result=tokio::time::timeout(std::time::Duration::from_secs(600),work) => result.map_err(error)??,
        _=cancelled.changed() => return Err(ApiError::unprocessable("Transcription cancelled; audio retained for retry. Remote processing/charges may already have occurred")),
    };
    if text.len() > 1024 * 1024 || hick_transcript::detect(&text).is_none() {
        return Err(ApiError::unprocessable(
            "Adapter must return UTF-8 VTT/SRT/SBV or speaker-labelled text under 1 MB; audio retained for retry",
        ));
    }
    let revision = hex::encode(Sha256::digest(text.as_bytes()));
    let evidence = json!({"recording":record,"transcript_sha256":revision,"transcript":text,"provider":adapter,"speaker_policy":"Unknown unless supplied by provider or confirmed; corrections are separate note edits","created":format!("unix:{}",std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())});
    // A retry never overwrites earlier transcript evidence.
    super::store::write_atomic(
        &dir.join(format!("{}-{}.transcript.json", body.id, revision)),
        &serde_json::to_vec_pretty(&evidence).map_err(error)?,
    )
    .map_err(error)?;
    Ok(Json(evidence))
}
#[derive(Deserialize)]
pub struct Link {
    doc_id: String,
    transcript_sha256: String,
    correction: String,
}
pub async fn link(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<Link>,
) -> ApiResult<Json<Value>> {
    let dir = directory(&state).map_err(error)?;
    let recording = load(&dir, &id).map_err(error)?;
    if body.transcript_sha256.len() != 64
        || !body
            .transcript_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
        || body.correction.len() > 1024 * 1024
    {
        return Err(ApiError::bad_request("Invalid transcript revision"));
    }
    let original = dir.join(format!("{id}-{}.transcript.json", body.transcript_sha256));
    let provider: Value =
        serde_json::from_slice(&std::fs::read(original).map_err(error)?).map_err(error)?;
    let correction = hex::encode(Sha256::digest(body.correction.as_bytes()));
    let evidence = json!({"recording":recording,"transcript_sha256":body.transcript_sha256,"correction_sha256":correction,"correction":body.correction,"provider":provider["provider"],"doc_id":body.doc_id});
    let note = state
        .index
        .absolute(&body.doc_id)
        .ok_or_else(|| ApiError::not_found("Imported note unavailable"))?;
    super::store::write_atomic(
        &dir.join(format!("{id}-{correction}.correction.json")),
        &serde_json::to_vec_pretty(&evidence).map_err(error)?,
    )
    .map_err(error)?;
    // Only fixed hashes and validated IDs enter the authored note; provider output
    // remains inert transcript data handled by the existing importer.
    let citation = format!(
        "\n\nRecording evidence: local audio `{id}` (SHA-256 `{}`); transcript `{}`; reviewed text `{correction}`. Audio retained separately in local application data.\n",
        recording.sha256, body.transcript_sha256
    );
    let room = state
        .rooms
        .get_or_create(&body.doc_id)
        .await
        .map_err(error)?;
    let before = room.text().await;
    let after = format!("{before}{citation}");
    if !state
        .rooms
        .replace_source_if_current(&body.doc_id, &before, &after, || {
            super::store::write_atomic(&note, after.as_bytes())
        })
        .await
        .map_err(error)?
    {
        return Err(ApiError::conflict(
            "Imported note changed; audio evidence retained, retry linking",
        ));
    }
    Ok(Json(evidence))
}
pub fn routes() -> axum::Router<LocalState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/meetings/recordings", get(catalogue).post(capture))
        .route("/meetings/transcription", post(configure))
        .route("/meetings/transcribe", post(transcribe))
        .route("/meetings/recordings/{id}", get(audio).delete(remove))
        .route("/meetings/recordings/{id}/cancel", post(cancel))
        .route("/meetings/recordings/{id}/recover", post(recover))
        .route("/meetings/recordings/{id}/link", post(link))
}
#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/authoring/recordings-stay-local.md
    #[test]
    fn retained_audio_offsets_hashes_limits_and_stopped_state() {
        let dir = tempfile::tempdir().unwrap();
        let action = |id: String, offset, complete| Action {
            id,
            mime: "audio/wav".into(),
            data: base64::engine::general_purpose::STANDARD.encode(b"RIFF audio fixture"),
            offset,
            complete,
            remote_consent: false,
            expected_adapter: None,
        };
        let first = append(dir.path(), action(String::new(), 0, false)).unwrap();
        assert_eq!(first.bytes, 18);
        assert!(append(dir.path(), action(first.id.clone(), 0, false)).is_err());
        let stopped = append(dir.path(), action(first.id.clone(), 18, true)).unwrap();
        assert_eq!(
            stopped.sha256,
            hex::encode(Sha256::digest(b"RIFF audio fixtureRIFF audio fixture"))
        );
        assert!(append(dir.path(), action(first.id, 36, false)).is_err());
        assert!(identity("../../escape").is_err());
    }
}
</book:entry>

</book:doc>
