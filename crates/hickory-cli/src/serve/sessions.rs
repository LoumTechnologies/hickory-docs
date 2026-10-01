//! The conversation lens and current, independently checked output evidence.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;

#[derive(Deserialize)]
pub struct SessionQuery {
    pub path: String,
}

/// `GET /api/sessions/view?path=…` — a session file read back as the
/// conversation it records: turns (with their parent turn, for the tree),
/// and within each turn the steps — reasoning, prose, scripts and their
/// output, tool calls and results, files shown, lines written. The same
/// shape the chat dock renders, so a session opened as a document looks like
/// the chat it was.
pub async fn session_view(
    State(state): State<LocalState>,
    Query(q): Query<SessionQuery>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let rel = std::path::Path::new(&q.path);
    let abs = if rel.is_absolute() {
        rel.to_path_buf()
    } else {
        root.join(rel)
    };
    let canon = abs
        .canonicalize()
        .map_err(|e| ApiError::not_found(format!("no session at {}: {e}", q.path)))?;
    let root_canon = root.canonicalize().unwrap_or(root.clone());
    if !canon.starts_with(&root_canon) {
        return Err(ApiError::not_found(format!(
            "{} is outside this folder",
            q.path
        )));
    }
    let source = std::fs::read_to_string(&canon)
        .map_err(|e| ApiError::not_found(format!("cannot read {}: {e}", q.path)))?;
    if !hick_lang::is_session_source(&source) {
        return Err(ApiError::bad_request(format!(
            "{} is not a hick:session document",
            q.path
        )));
    }
    let view = hickory_agent::session_view::session_view(&source);
    let rel_path = canon
        .strip_prefix(&root_canon)
        .map(|p| p.display().to_string())
        .unwrap_or(q.path.clone());
    // The session as the lens draws it: its blocks, and the provenance each
    // element declares — context for what the model was shown, lineage for
    // what a turn wrote, declared for what the prose points at. See
    // docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.
    let blocks = hick_literate::session_elements::session_blocks(&source, Some(&root_canon));
    let links = hick_literate::session_elements::session_links(&source, Some(&root_canon));
    let mut links = serde_json::to_value(links).unwrap_or_default();
    let evidence = output_evidence(&state, &root_canon, &blocks, &links).await;
    if let Some(links) = links.as_array_mut() {
        links.extend(evidence);
    }
    Ok(Json(
        json!({ "path": rel_path, "source": source, "view": view, "blocks": blocks, "links": links }),
    ))
}

/// Citation is the trigger, never the evidence. Weave without executing and
/// accept only an output whose disk bytes still match and whose provenance
/// names a source inside this workspace. Old sessions are not rewritten.
async fn output_evidence(
    state: &LocalState,
    root: &Path,
    blocks: &[hick_blocks::Block],
    links: &Value,
) -> Vec<Value> {
    let mut evidence = Vec::new();
    let docs: std::collections::BTreeSet<_> = links
        .as_array()
        .into_iter()
        .flatten()
        .filter(|link| link["family"] == "declared")
        .filter_map(|link| link["to"]["path"].as_str())
        .filter(|path| path.ends_with(".hick") || path.ends_with(".md"))
        .collect();
    for doc in docs {
        let cited: Vec<_> = links
            .as_array()
            .into_iter()
            .flatten()
            .filter(|link| link["family"] == "declared" && link["to"]["path"] == doc)
            .collect();
        if cited.is_empty() {
            continue;
        }
        let Ok(path) = root.join(doc).canonicalize() else {
            continue;
        };
        if !path.starts_with(root) {
            continue;
        }
        let id = state.index.id_for_path(doc);
        let run = if state.index.absolute(&id).is_some() {
            state.weave(&id).await.ok()
        } else {
            // Older .hick files can be cited even when startup's .md scan
            // did not index them. This is the same non-executing weave.
            crate::run_doc(
                &path,
                &state.params,
                crate::RunMode::Weave,
                crate::ExecutorChoice::Local,
            )
            .await
            .ok()
        };
        let Some(run) = run else {
            continue;
        };
        if std::fs::read_to_string(&path).ok().as_deref() != Some(run.source.as_str()) {
            continue;
        }
        let dir = Path::new(doc).parent().unwrap_or(Path::new(""));
        for (key, content) in &run.result.files {
            let Some(text) = content.as_text() else {
                continue;
            };
            let output = dir.join(key);
            let Ok(canon) = root.join(&output).canonicalize() else {
                continue;
            };
            if !canon.starts_with(root)
                || std::fs::read(&canon).ok().as_deref() != Some(text.as_bytes())
            {
                continue;
            }
            let Ok(provenance) = crate::output_lineage(&run, key) else {
                continue;
            };
            for citation in &cited {
                let Some(answer) = blocks.iter().find(|block| {
                    block.kind == "session-assistant"
                        && serde_json::to_value(block.span).ok().as_ref() == Some(&citation["span"])
                }) else {
                    continue;
                };
                let body = answer
                    .props
                    .get("body")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let output_name = output.to_string_lossy();
                let named = body
                    .split(|c: char| c.is_whitespace() || "`\"'()<>[];,".contains(c))
                    .any(|token| {
                        token.trim_end_matches(['.', '!', '?']) == output_name || token == key
                    });
                if !named {
                    continue;
                }
                for entry in &provenance {
                    let Some((origin_path, from, to)) = entry.origin.location() else {
                        continue;
                    };
                    let Ok(origin) = root.join(origin_path).canonicalize() else {
                        continue;
                    };
                    let Ok(relative) = origin.strip_prefix(root) else {
                        continue;
                    };
                    // Cross-document origins need their own captured source;
                    // this check currently establishes this producer only.
                    if origin != path {
                        continue;
                    }
                    let source = &run.source;
                    if entry.end <= entry.start || to <= from {
                        continue;
                    }
                    let source_lines =
                        (line_at(source, from), line_at(source, to.saturating_sub(1)));
                    let output_lines = (
                        line_at(text, entry.start),
                        line_at(text, entry.end.saturating_sub(1)),
                    );
                    let proof = json!({
                        "family":"lineage", "span":citation["span"], "lines":citation["lines"],
                        "to":{"path":output_name,"lines":output_lines},
                        "evidence":{"path":relative.to_string_lossy(),"lines":source_lines},
                        "title":format!("Checked current output lineage — {}:{}–{} produces {}:{}–{}. Woven bytes match the file on disk; this is document evidence, not a claim that the conversation wrote it.", relative.display(), source_lines.0, source_lines.1, output_name, output_lines.0, output_lines.1)
                    });
                    if !evidence.contains(&proof) {
                        evidence.push(proof);
                    }
                }
            }
        }
    }
    evidence
}

fn line_at(source: &str, byte: usize) -> usize {
    source.as_bytes()[..byte.min(source.len())]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}
