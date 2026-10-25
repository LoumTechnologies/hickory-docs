<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/meeting_import.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Meeting import retains original evidence without invoking a model](../../../../../contract.md)

Captured SHA-256: `ddf3925a25e58fbbb1c9a1e56c9348ef36acfb6e98b7fd6bc17ede7603c9b74e`.

<book:entry path="crates/hickory-cli/src/serve/meeting_import.rs">
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
</book:entry>

</book:doc>
