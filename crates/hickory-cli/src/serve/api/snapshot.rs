use super::{ApiError, ApiResult};
use crate::serve::LocalState;
use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::{Value, json};

/// Export the same weave and transcripts the local viewer receives. No execution.
pub async fn snapshot_doc(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    use sha2::{Digest, Sha256};
    let run = state.weave(&id).await?;
    let rendered =
        crate::block_model_json(&run).map_err(|e| ApiError::unprocessable(e.to_string()))?;
    let source = &run.doc.source;
    let mut cells = Vec::new();
    if let Some(blocks) = rendered["blocks"].as_array() {
        for block in blocks.iter().filter(|block| block["kind"] == "exec") {
            let mut cell = block.clone();
            // Saved notebook output survives a restart without a transcript cache.
            // Keep it distinct from execution evidence: these are supplied source bytes.
            let source_id = run.doc.all_tags().into_iter().find_map(|tag| {
                (tag.name == "exec"
                    && tag
                        .source_span
                        .is_some_and(|span| Some(span.start as u64) == block["span"][0].as_u64()))
                .then(|| tag.get_attribute("id"))
                .flatten()
            });
            if let Some(id) = source_id {
                let selector = format!("#{id}");
                if let Some(output) = blocks
                    .iter()
                    .find(|output| output["kind"] == "output" && output["outputFor"] == selector)
                {
                    cell["saved_output"] = output["body"].clone();
                }
            }
            cells.push(cell);
        }
    }
    let path = state
        .index
        .path_of(&id)
        .map(|p| p.to_string())
        .unwrap_or_else(|| id.clone());
    Ok(Json(json!({
        "format":"hickory-viewer-snapshot", "version":1,
        "path":path, "source":source,
        "source_hash":format!("{:x}",Sha256::digest(source.as_bytes())),
        "captured_at_ms":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| ApiError::internal(e.to_string()))?.as_millis(),
        "execution":"recorded-output-snapshot; export does not execute checks",
        "cells":cells
    })))
}
