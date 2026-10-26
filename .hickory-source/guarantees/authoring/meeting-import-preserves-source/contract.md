<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Meeting import retains original evidence without invoking a model

For: Authors editing executable documents and the files those documents produce.

Given an open local workspace, the meeting import control accepts UTF-8 VTT,
SRT, SBV, Markdown and text transcripts up to 1 MB. Import invokes the existing
offline transcript parser and ingestion engine. It writes the source into the
configured inbox, retains ingested originals, and returns the existing note for
identical bytes. Both legacy Markdown and current Hickory note fingerprints
participate in deduplication. Changed bytes create a distinct note.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/meeting-import-preserves-source.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-835cdf37fe72"></a>
## `crates/hickory-cli/src/serve/meeting_import.rs`

Related contracts: [Meeting import retains original evidence without invoking a model](contract.md).

<a id="f-835cdf37fe72-Import"></a>
### `Import`

Explicit offline transcript upload; reuses ingest's original preservation.

<storycode:copy id="f-835cdf37fe72-Import">
//! Explicit offline transcript upload; reuses ingest's original preservation.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{Json, extract::State};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::Digest;
use std::{io::Write, path::Path};

#[derive(Deserialize)]
pub struct Import {
    name: String,
    text: String,
}

</storycode:copy>
<a id="f-835cdf37fe72-import"></a>
### `import`

<storycode:copy id="f-835cdf37fe72-import">
fn import(root: &Path, body: Import) -> anyhow::Result<Value> {
    anyhow::ensure!(
        body.text.len() <= 1024 * 1024,
        "Choose a transcript smaller than 1 MB"
    );
    anyhow::ensure!(!body.text.trim().is_empty(), "The transcript is empty");
    anyhow::ensure!(
        !body.name.contains(['/', '\\', '\n', '\r']) && !body.name.starts_with('.'),
        "Choose a plain transcript filename"
    );
    let extension = Path::new(&body.name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    anyhow::ensure!(
        ["txt", "md", "vtt", "srt", "sbv"].contains(&extension.as_str()),
        "Import VTT, SRT, SBV or text; audio/video must be transcribed first"
    );
    let format = hick_transcript::detect(&body.text)
        .ok_or_else(|| anyhow::anyhow!("No transcript utterances found"))?;
    let utterances = hick_transcript::parse(&body.text, format);
    anyhow::ensure!(!utterances.is_empty(), "No transcript utterances found");
    // Validate raw-block safety before writing anything.
    crate::ingest::note_for(&body.text, Path::new(&body.name), None, None)?;
    let config = crate::ingest::InboxConfig::from_env()?;
    let canonical_root = root.canonicalize()?;
    let inbox = config.inbox(root);
    // Existing ancestors must stay inside the workspace; refuse symlink escapes
    // before creating directories. Retained sources receive the same check.
    for target in [&inbox, &config.ingested(root)] {
        let mut ancestor = target.as_path();
        while !ancestor.exists() {
            ancestor = ancestor
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Invalid inbox"))?;
        }
        anyhow::ensure!(
            ancestor.canonicalize()?.starts_with(&canonical_root),
            "Inbox resolves outside the workspace"
        );
    }
    std::fs::create_dir_all(&inbox)?;
    // Isolate each upload while preserving the selected original basename.
    // Keep duplicate sources in the inbox rather than deleting input evidence.
    let staging = tempfile::Builder::new()
        .prefix("meeting-upload-")
        .tempdir_in(&inbox)?
        .keep();
    let source = staging.join(&body.name);
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&source)?
        .write_all(body.text.as_bytes())?;
    let result = crate::ingest::ingest_one(&source, root, &config)?;
    let (note, duplicate, retained) = match result {
        crate::ingest::Outcome::Ingested { note, moved_to, .. } => (note, false, moved_to),
        crate::ingest::Outcome::AlreadyIngested { note, .. } => (note, true, source),
        other => anyhow::bail!("{other}"),
    };
    if std::fs::read_dir(&staging)?.next().is_none() {
        std::fs::remove_dir(&staging)?;
    }
    let relative = |path: &Path| {
        path.strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    };
    Ok(
        json!({"path":relative(&note), "duplicate":duplicate, "source":relative(&retained),
        "source_sha256":hex::encode(sha2::Sha256::digest(body.text.as_bytes())),
        "format":format.name(), "utterances":utterances.iter().enumerate().map(|(index, u)| json!({
            "id":format!("u{}", index + 1), "speaker":u.speaker, "at":u.at,
            "text":u.text, "start_utf8":u.start, "end_utf8":u.end,
            "line":body.text[..u.start].bytes().filter(|b| *b == b'\n').count() + 1
        })).collect::<Vec<_>>(), "coverage":"Parsed source evidence; no model called; summaries require separate refresh"}),
    )
}

</storycode:copy>
<a id="f-835cdf37fe72-post"></a>
### `post`

<storycode:copy id="f-835cdf37fe72-post">
pub async fn post(
    State(state): State<LocalState>,
    Json(body): Json<Import>,
) -> ApiResult<Json<Value>> {
    if !state.folder_open {
        return Err(ApiError::bad_request(
            "Open a folder before importing a meeting",
        ));
    }
    let root = state.index.root().to_path_buf();
    let mut result = tokio::task::spawn_blocking(move || import(&root, body))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map_err(|e| ApiError::bad_request(format!("{e:#}")))?;
    if let Some(path) = result["path"].as_str() {
        result["doc_id"] = json!(state.index.add(path));
    }
    Ok(Json(result))
}

</storycode:copy>
<a id="f-835cdf37fe72-tests"></a>
### `tests`

<storycode:copy id="f-835cdf37fe72-tests">
#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/authoring/meeting-import-preserves-source.md
    #[cfg(unix)]
    #[test]
    fn meeting_import_refuses_inbox_symlink_escape() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let config = crate::ingest::InboxConfig::from_env().unwrap();
        let inbox = config.inbox(root.path());
        std::fs::create_dir_all(inbox.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(outside.path(), &inbox).unwrap();
        assert!(
            import(
                root.path(),
                Import {
                    name: "meeting.txt".into(),
                    text: "Sam: Hello".into()
                }
            )
            .is_err()
        );
        assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
    }
    // Guarantee: docs/guarantees/authoring/meeting-import-preserves-source.md
    #[test]
    fn meeting_import_offline_formats_duplicates_and_changed_source() {
        let root = tempfile::tempdir().unwrap();
        for (name, text) in [
            (
                "meeting.vtt",
                "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\n<v Sam>Hello\n",
            ),
            ("meeting.srt", "1\n00:00:01,000 --> 00:00:02,000\nHello\n"),
            ("meeting.sbv", "0:00:01.000,0:00:02.000\nHello\n"),
            ("meeting.txt", "Sam: Hello"),
        ] {
            let first = import(
                root.path(),
                Import {
                    name: name.into(),
                    text: text.into(),
                },
            )
            .unwrap();
            let second = import(
                root.path(),
                Import {
                    name: name.into(),
                    text: text.into(),
                },
            )
            .unwrap();
            assert_eq!(first["path"], second["path"]);
            assert_eq!(second["duplicate"], true);
            assert_eq!(
                std::fs::read_to_string(root.path().join(first["source"].as_str().unwrap()))
                    .unwrap(),
                text
            );
            assert!(!first["utterances"].as_array().unwrap().is_empty());
        }
        let changed = import(
            root.path(),
            Import {
                name: "meeting.txt".into(),
                text: "Sam: Changed requirement".into(),
            },
        )
        .unwrap();
        assert_eq!(changed["duplicate"], false);
    }
    // Guarantee: docs/guarantees/authoring/meeting-import-preserves-source.md
    #[test]
    fn meeting_import_refuses_unsafe_and_unsupported_inputs_without_writes() {
        let root = tempfile::tempdir().unwrap();
        for (name, text) in [
            ("../escape.txt", "Sam: hi"),
            ("movie.mp4", "Sam: hi"),
            ("empty.txt", ""),
            ("unsafe.txt", "</hick:transcript>"),
        ] {
            assert!(
                import(
                    root.path(),
                    Import {
                        name: name.into(),
                        text: text.into()
                    }
                )
                .is_err()
            );
        }
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/meeting_import.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/meeting_import.rs"><storycode:paste select="#f-835cdf37fe72-Import"/><storycode:paste select="#f-835cdf37fe72-import"/><storycode:paste select="#f-835cdf37fe72-post"/><storycode:paste select="#f-835cdf37fe72-tests"/></storycode:file>

<a id="output-beef23c9f263"></a>
## `crates/hickory-cli/tests/serve_meeting_import.rs`

Related contracts: [Meeting import retains original evidence without invoking a model](contract.md).

<a id="f-beef23c9f263-local_http_import_preserves_original_deduplicates_and_navigates_exact_utterances"></a>
### `local_http_import_preserves_original_deduplicates_and_navigates_exact_utterances`

<storycode:copy id="f-beef23c9f263-local_http_import_preserves_original_deduplicates_and_navigates_exact_utterances">
//! Guarantee: docs/guarantees/authoring/meeting-import-preserves-source.md
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::{Value, json};

#[tokio::test(flavor = "multi_thread")]
async fn local_http_import_preserves_original_deduplicates_and_navigates_exact_utterances() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let client = reqwest::Client::new();
    let text = "WEBVTT\r\n\r\n00:00:01.000 --> 00:00:02.000\r\n<v Ada>Approve before deleting\r\n";
    let body = json!({"name":"meeting.vtt", "text":text});
    let url = format!("{base}/api/meetings/import");
    let first = client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(first.status(), 200);
    let first: Value = first.json().await.unwrap();
    let document = client
        .get(format!(
            "{base}/api/docs/{}",
            first["doc_id"].as_str().unwrap()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(document.status(), 200);
    assert!(first["path"].as_str().unwrap().ends_with(".md"));
    let document: Value = document.json().await.unwrap();
    assert!(document["source"].as_str().unwrap().contains(text));
    let render = client
        .get(format!(
            "{base}/api/docs/{}/render",
            first["doc_id"].as_str().unwrap()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(render.status(), 200, "{}", render.text().await.unwrap());
    let restored_index = hickory_cli::serve::store::DocIndex::scan(&root).unwrap();
    assert!(
        restored_index
            .entries()
            .iter()
            .any(|(_, path)| path == first["path"].as_str().unwrap())
    );
    assert_eq!(first["source_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(first["utterances"][0]["speaker"], "Ada");
    let at = first["utterances"][0]["start_utf8"].as_u64().unwrap() as usize;
    let end = first["utterances"][0]["end_utf8"].as_u64().unwrap() as usize;
    assert!(text[at..end].contains("Approve before deleting"));
    assert_eq!(
        std::fs::read(root.join(first["source"].as_str().unwrap())).unwrap(),
        text.as_bytes()
    );
    let second: Value = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(second["duplicate"], true);
    assert_eq!(first["path"], second["path"]);
    // An explicit reimport repairs only its matching old-format note.
    let current_path = root.join(first["path"].as_str().unwrap());
    let legacy_path = current_path.with_extension("hick");
    let annotated = format!(
        "{}\nHuman correction retained.\n",
        std::fs::read_to_string(&current_path).unwrap()
    );
    std::fs::rename(&current_path, &legacy_path).unwrap();
    std::fs::write(&legacy_path, &annotated).unwrap();
    let upgraded: Value = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(upgraded["duplicate"], true);
    assert!(upgraded["path"].as_str().unwrap().ends_with(".md"));
    assert_eq!(std::fs::read_to_string(&legacy_path).unwrap(), annotated);
    let upgraded_doc = client
        .get(format!(
            "{base}/api/docs/{}",
            upgraded["doc_id"].as_str().unwrap()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(upgraded_doc.status(), 200);
    assert_eq!(
        upgraded_doc.json::<Value>().await.unwrap()["source"],
        annotated
    );
    let upgraded_render = client
        .get(format!(
            "{base}/api/docs/{}/render",
            upgraded["doc_id"].as_str().unwrap()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(
        upgraded_render.status(),
        200,
        "{}",
        upgraded_render.text().await.unwrap()
    );
    assert!(
        hickory_cli::serve::store::DocIndex::scan(&root)
            .unwrap()
            .entries()
            .iter()
            .any(|(_, path)| path == upgraded["path"].as_str().unwrap())
    );
    let rejected = client
        .post(&url)
        .json(&json!({"name":"recording.wav", "text":"audio"}))
        .send()
        .await
        .unwrap();
    assert_eq!(rejected.status(), 400);
    server.abort();
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/serve_meeting_import.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/serve_meeting_import.rs"><storycode:paste select="#f-beef23c9f263-local_http_import_preserves_original_deduplicates_and_navigates_exact_utterances"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-cli/src/serve/meeting_import.rs](contract.md#output-835cdf37fe72)

### Tests and fixtures

- [apps/web/src/components/MeetingImport.test.tsx](../../agent/ui-action-dispatch/implementation/apps/web/src/components/MeetingImport.test.tsx.md#output-4adb88414524)
- [crates/hickory-cli/tests/serve_meeting_import.rs](contract.md#output-beef23c9f263)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
