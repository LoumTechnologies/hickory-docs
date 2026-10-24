use super::*;

/// `POST /api/docs/:id/blocks/:at/:action` — one action on one element,
/// addressed by the byte offset its tag starts at.
///
/// The element answers with what it wants (`hick_blocks::ActionOutcome`)
/// and this handler carries it out with the machinery the older routes
/// already use: a `run` starts a run exactly as `POST /run` does and
/// answers `202` with the run id; an `edit` writes the document as `PUT`
/// does; an `answer` is returned as is. An element that does not know the
/// action is a `404` naming the element; one that refuses is a `422` in its
/// own words.
pub async fn block_action(
    State(state): State<LocalState>,
    Path((id, at, action)): Path<(String, usize, String)>,
    body: Option<Json<Value>>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    use hick_blocks::{ActionError, ActionOutcome};
    let run = state.weave(&id).await?;
    let body = body.map(|Json(v)| v).unwrap_or(Value::Null);
    let expected = body
        .get("expected_source_hash")
        .and_then(Value::as_str)
        .map(str::to_owned);
    if let Some(expected) = &expected {
        let actual = hick_context_protocol::digest(run.doc.source.as_bytes());
        if expected.trim_start_matches("sha256:") != actual.trim_start_matches("sha256:") {
            return Err(ApiError::conflict(
                "Source revision changed before block action; refresh the document. No action ran.",
            ));
        }
    }
    let outcome = crate::block_action(&run, at, &action, body).map_err(|e| match e {
        ActionError::Unknown { .. } => ApiError::not_found(e.to_string()),
        ActionError::Refused(_) => ApiError::unprocessable(e.to_string()),
        ActionError::Failed(_) => ApiError::internal(e.to_string()),
    })?;
    match outcome {
        ActionOutcome::Answer { value } => Ok((
            StatusCode::OK,
            Json(json!({ "outcome": "answer", "value": value })),
        )),
        ActionOutcome::Run { cells } => {
            let run_id = state
                .start_run_selected(&id, false, expected, Some(cells.clone()))
                .await?;
            Ok((
                StatusCode::ACCEPTED,
                Json(json!({ "outcome": "run", "run_id": run_id, "cells": cells })),
            ))
        }
        ActionOutcome::Edit { span, replacement } => {
            let source = run.doc.source.clone();
            if span.0 > span.1
                || span.1 > source.len()
                || !source.is_char_boundary(span.0)
                || !source.is_char_boundary(span.1)
            {
                return Err(ApiError::unprocessable(format!(
                    "the element asked to replace bytes {}..{} of a {}-byte document",
                    span.0,
                    span.1,
                    source.len()
                )));
            }
            let mut next = String::with_capacity(source.len() + replacement.len());
            next.push_str(&source[..span.0]);
            next.push_str(&replacement);
            next.push_str(&source[span.1..]);
            state.write_source(&id, &next)?;
            state.rooms.apply_external_source(&id, &next).await;
            Ok((
                StatusCode::OK,
                Json(
                    json!({ "outcome": "edit", "span": [span.0, span.1], "doc": doc_json(&state, &id)? }),
                ),
            ))
        }
    }
}

/// `GET /api/docs/:id/render` — the block model for the current source.
///
/// Weave, never execute: the client asks for this on every load and after
/// every save, and a render that ran the document would turn scrolling into
/// arbitrary code execution.
pub async fn render_doc(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let run = state.weave(&id).await?;
    let mut value = crate::block_model_json(&run)
        .map_err(|e| ApiError::unprocessable(format!("render failed: {e}")))?;
    value["document"] = json!(id);
    value["source_hash"] = json!(hick_context_protocol::digest(run.doc.source.as_bytes()));
    Ok(Json(value))
}
