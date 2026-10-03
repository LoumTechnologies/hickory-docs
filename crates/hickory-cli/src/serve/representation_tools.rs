//! Structured AI arrangements use exact source slices; code is never retyped.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    representation::{self, Backing, Edit, Representation},
};
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct Section {
    pub path: String,
    pub from: usize,
    pub to: usize,
    pub heading: String,
    #[serde(default)]
    pub explanation: String,
}
#[derive(Deserialize)]
pub struct Arrangement {
    pub revision: String,
    pub sections: Vec<Section>,
}

pub async fn arrange_value(
    state: &LocalState,
    id: &str,
    body: Arrangement,
) -> ApiResult<Representation> {
    let view = state
        .representations
        .lock()
        .await
        .get(id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("Read the literate view again."))?;
    if view.revision != body.revision {
        return Err(ApiError::conflict(
            "Read the current view before rearranging it.",
        ));
    }
    let Backing::Files { paths } = &view.backing else {
        return Err(ApiError::bad_request(
            "Rearrange a persistent document with its document tools.",
        ));
    };
    let invalid = || {
        ApiError::bad_request(
            "Sections must cover every source byte exactly once, using nonoverlapping UTF-8 byte ranges. Read the view's files and try again.",
        )
    };
    if body.sections.len() > 256 {
        return Err(invalid());
    }
    let mut source = "# Literate view\n\nAI-generated explanations describe the source revision shown here; review their accuracy.\n\n".to_string();
    for (i, section) in body.sections.iter().enumerate() {
        let file = view
            .files
            .iter()
            .find(|f| f.path == section.path)
            .ok_or_else(invalid)?;
        let slice = file
            .content
            .get(section.from..section.to)
            .ok_or_else(invalid)?;
        source.push_str(&format!(
            "## {}\n\n{}\n\n<hick:copy id=\"fragment-{i}\">{slice}</hick:copy>\n\n",
            section.heading, section.explanation
        ));
    }
    for path in paths {
        let file = view
            .files
            .iter()
            .find(|f| &f.path == path)
            .ok_or_else(invalid)?;
        let mut chunks: Vec<_> = body
            .sections
            .iter()
            .enumerate()
            .filter(|(_, s)| &s.path == path)
            .collect();
        chunks.sort_by_key(|(_, s)| s.from);
        let mut end = 0;
        if chunks.is_empty() {
            return Err(invalid());
        }
        source.push_str(&format!("<hick:file path=\"{path}\">"));
        for (i, s) in chunks {
            if s.from != end || s.to < s.from {
                return Err(invalid());
            }
            end = s.to;
            source.push_str(&format!("<hick:paste select=\"#fragment-{i}\" />"));
        }
        if end != file.content.len() {
            return Err(invalid());
        }
        source.push_str("</hick:file>\n");
    }
    let files = representation::compile(state, id, &source, &view.backing).await?;
    if view.files.iter().any(|f| {
        !files
            .iter()
            .any(|new| new.path == f.path && new.content == f.content)
    }) {
        return Err(ApiError::unprocessable(
            "This arrangement changes code bytes. Try different fragment boundaries; nothing was saved.",
        ));
    }
    let mut arranged = representation::edit_value(
        state,
        id,
        Edit {
            revision: body.revision,
            source,
        },
    )
    .await?;
    // This prose now describes the supplied current byte slices, still an AI claim.
    let mut views = state.representations.lock().await;
    if let Some(current) = views.get_mut(id)
        && current.revision == arranged.revision
    {
        current.explanation_stale = false;
        if let Err(e) = super::representation_store::update_if_kept(state, current) {
            current.local_warning = Some(format!(
                "The reading is current, but could not be kept locally: {}",
                e.message()
            ));
        }
        arranged = current.clone();
    }
    Ok(arranged)
}

pub async fn arrange(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<Arrangement>,
) -> ApiResult<Json<Representation>> {
    Ok(Json(arrange_value(&state, &id, body).await?))
}

pub async fn save(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let views = state.representations.lock().await;
    let view = views
        .get(&id)
        .ok_or_else(|| ApiError::not_found("Open the view again before keeping it."))?;
    let path = super::representation_store::keep(&state, view)?;
    Ok(Json(json!({"path":path})))
}

pub fn catalogue() -> Vec<Value> {
    vec![
        json!({"name":"create_literate_view","description":"Create a disposable literate view over ordinary source files. It verifies byte-exact reconstruction and changes no repository files. Then use organize_literate_view to explain and arrange exact source fragments.","inputSchema":{"type":"object","properties":{"paths":{"type":"array","items":{"type":"string"}}},"required":["paths"]}}),
        json!({"name":"read_literate_view","description":"Read the view, its revision, and each original file's exact content and provenance. Read before editing.","inputSchema":{"type":"object","properties":{"id":{"type":"string"}},"required":["id"]}}),
        json!({"name":"organize_literate_view","description":"Organize and explain exact source slices in reading order. Sections contain path, from/to UTF-8 BYTE offsets, heading and explanation. Every file must be covered exactly once; the engine supplies its bytes and proves all outputs unchanged. Do not invent code or claim explanations are verified.","inputSchema":{"type":"object","properties":{"id":{"type":"string"},"revision":{"type":"string"},"sections":{"type":"array","items":{"type":"object","properties":{"path":{"type":"string"},"from":{"type":"integer"},"to":{"type":"integer"},"heading":{"type":"string"},"explanation":{"type":"string"}},"required":["path","from","to","heading"]}}},"required":["id","revision","sections"]}}),
        json!({"name":"edit_literate_view","description":"Save edits to the literate representation using its current revision. Code edits write through to the backing files or persistent document; prose and rearrangement remain local for source-backed views. Refuses stale source. Run the repository's own build/tests after changing code.","inputSchema":{"type":"object","properties":{"id":{"type":"string"},"revision":{"type":"string"},"source":{"type":"string"}},"required":["id","revision","source"]}}),
    ]
}

pub async fn call(state: &LocalState, name: &str, args: &Value) -> ApiResult<Value> {
    let id = args["id"].as_str().unwrap_or("");
    let view = match name {
        "create_literate_view" => {
            representation::create_value(
                state,
                Backing::Files {
                    paths: serde_json::from_value(args["paths"].clone())
                        .map_err(|e| ApiError::bad_request(e.to_string()))?,
                },
            )
            .await?
        }
        "read_literate_view" => {
            representation::get(State(state.clone()), Path(id.into()))
                .await?
                .0
        }
        "organize_literate_view" => {
            arrange_value(
                state,
                id,
                serde_json::from_value(args.clone())
                    .map_err(|e| ApiError::bad_request(e.to_string()))?,
            )
            .await?
        }
        "edit_literate_view" => {
            representation::edit_value(
                state,
                id,
                serde_json::from_value(args.clone())
                    .map_err(|e| ApiError::bad_request(e.to_string()))?,
            )
            .await?
        }
        _ => return Err(ApiError::bad_request("Unknown literate view tool.")),
    };
    Ok(
        json!({"content":[{"type":"text","text":serde_json::to_string(&view).map_err(anyhow::Error::from)?}],"isError":false}),
    )
}
