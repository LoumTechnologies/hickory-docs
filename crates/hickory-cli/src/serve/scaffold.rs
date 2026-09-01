//! File → New Project: `dotnet new`, as a form and then as a document.
//!
//! Four routes, and the shape of them is the argument. Reading the catalogue
//! and reading one template's options are separate because the second is a
//! second `dotnet` process and the dialog only needs it once a template is
//! chosen — a New Project dialog that spawned forty-six help invocations to
//! open would be unusable on a laptop.
//!
//! ## Why there is a preview route
//!
//! The dialog shows the exact bytes it is about to write, the way the Insert
//! panel does, and for the same reason: this is a text format a person owns
//! and edits by hand, and a dialog that writes markup you never see teaches
//! you nothing. The preview could have been assembled in TypeScript — and
//! then there would be two implementations of the document, one shown and one
//! written, free to disagree. So the preview is the same function, called
//! over HTTP. The server is in this process on loopback; the round trip costs
//! nothing worth having a second renderer for.
//!
//! ## Creating is two acts, and only the first is guaranteed
//!
//! `POST /api/scaffold` writes the document and then runs it and ingests what
//! the generator wrote. The second half can fail for reasons that are not
//! this product's business — no SDK, a template that needs a NuGet feed, a
//! cell the executor will not run — and when it does, the **document still
//! exists**. It is a complete, correct, unrun document: the command is in it,
//! and pressing Run does the rest. Deleting it to keep the failure tidy would
//! throw away the part that worked, and leave the person with nothing to fix.
//! The response says which of the two happened.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult, new_doc_target};
use crate::ExecutorChoice;
use crate::scaffold::{self, NoDotnetSdk, ScaffoldSpec};

/// Turn a scaffold failure into an API error, keeping the one distinction the
/// dialog draws its own screen from.
///
/// A missing SDK is not a failed request, it is a machine that cannot do this
/// at all — so it carries `missing: "dotnet"` in the detail, and the client
/// keys off **that**, never off the sentence. The lesson is
/// `hick_dap::MissingAdapter`'s: a reworded message must not be able to take
/// a screen away.
fn scaffold_error(e: anyhow::Error) -> ApiError {
    if e.downcast_ref::<NoDotnetSdk>().is_some() {
        return ApiError::unprocessable(format!("{}", NoDotnetSdk))
            .with_detail(json!({ "missing": "dotnet" }));
    }
    ApiError::unprocessable(format!("{e:#}"))
}

/// `GET /api/scaffold/templates` — what this machine can scaffold.
pub async fn templates(State(_state): State<LocalState>) -> ApiResult<Json<Value>> {
    // `dotnet new list` is a subprocess that reads a template cache off disk;
    // on a cold cache it rebuilds it, which is seconds rather than
    // milliseconds. Holding a runtime worker for that is what
    // `spawn_blocking` is for.
    let catalog = tokio::task::spawn_blocking(scaffold::catalog)
        .await
        .map_err(|e| ApiError::internal(format!("the template listing did not finish: {e}")))?
        .map_err(scaffold_error)?;
    let image = scaffold::sdk_image(&catalog.sdk_version);
    Ok(Json(json!({
        "kind": "dotnet",
        "sdk_version": catalog.sdk_version,
        "image": image,
        "templates": catalog.templates,
    })))
}

#[derive(Deserialize)]
pub struct DetailQuery {
    /// The template's short name (`webapi`).
    pub template: String,
    /// Which language's options to ask for. Absent means the template's own
    /// default, which is what `dotnet new <name> --help` answers with.
    #[serde(default)]
    pub language: Option<String>,
}

/// `GET /api/scaffold/options?template=webapi&language=C%23` — one template's
/// options, as fields.
pub async fn options(
    State(_state): State<LocalState>,
    Query(query): Query<DetailQuery>,
) -> ApiResult<Json<Value>> {
    if query.template.trim().is_empty() {
        return Err(ApiError::bad_request(
            "name a template — `?template=webapi`. `GET /api/scaffold/templates` lists them.",
        ));
    }
    let template = query.template.clone();
    let language = query.language.clone().filter(|l| !l.trim().is_empty());
    let detail = tokio::task::spawn_blocking(move || {
        scaffold::template_detail(&template, language.as_deref())
    })
    .await
    .map_err(|e| {
        ApiError::internal(format!(
            "reading the template's options did not finish: {e}"
        ))
    })?
    .map_err(scaffold_error)?;
    Ok(Json(
        serde_json::to_value(detail).unwrap_or_else(|_| json!({})),
    ))
}

/// What the dialog is asking to be written, and where.
#[derive(Deserialize)]
pub struct CreateRequest {
    /// The `.hick` path, relative to the served folder.
    pub path: String,
    #[serde(flatten)]
    pub spec: ScaffoldSpec,
    /// Run the cell and ingest what it writes. On by default: a New Project
    /// that leaves you an unrun command is a snippet, not a project.
    #[serde(default = "yes")]
    pub run: bool,
}

fn yes() -> bool {
    true
}

/// `POST /api/scaffold/preview` — the exact bytes, without writing them.
pub async fn preview(
    State(_state): State<LocalState>,
    Json(body): Json<CreateRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(json!({
        "path": body.path,
        "command": scaffold::dotnet_new_command(&body.spec),
        "source": scaffold::scaffold_document(&body.spec),
    })))
}

/// `POST /api/scaffold` — write the document, run it, and ingest the scaffold.
pub async fn create(
    State(state): State<LocalState>,
    Json(body): Json<CreateRequest>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    if body.spec.name.trim().is_empty() {
        return Err(ApiError::bad_request(
            "a project needs a name — it becomes the .NET root namespace and the assembly name.",
        ));
    }
    if body.spec.output.trim().is_empty() {
        return Err(ApiError::bad_request(
            "name the folder the generated tree lands in — it is the volume's `output=`, and \
             an empty one would scatter the scaffold across the notes folder.",
        ));
    }

    let (rel, absolute) = new_doc_target(&state, &body.path)?;
    let source = scaffold::scaffold_document(&body.spec);
    std::fs::write(&absolute, &source)
        .map_err(|e| ApiError::internal(format!("could not write {}: {e}", absolute.display())))?;
    let id = state.index.add(&rel);

    let mut ingested = Value::Null;
    let mut note = Value::Null;
    if body.run {
        let today = crate::ingest::today().unwrap_or_else(|| "unknown".to_string());
        let outcome = crate::ingest_exec::ingest_from_exec(
            &absolute,
            &format!("#{}", scaffold::SCAFFOLD_CELL),
            state.executor,
            &today,
        )
        .await;
        match outcome {
            Ok(outcome) => {
                ingested = json!({
                    "from": outcome.from,
                    "fingerprint": outcome.fingerprint,
                    "files": outcome.ingested,
                    "skipped": outcome.skipped,
                });
                // Then weave, so the tree is on disk when the dialog closes.
                //
                // Not belt-and-braces: an ingested volume is deliberately no
                // longer flushed as a pipeline output — the document owns
                // those bytes, and flushing a fresh run over them would
                // overwrite the edits it exists to protect — so after an
                // ingest the ONLY thing that puts the files on disk is a
                // weave. Without this, New Project finished with a document
                // full of a project and a folder with no project in it, and
                // whether the files appeared came down to whether a file
                // watcher happened to be running.
                if let Err(e) = weave_to_disk(&absolute).await {
                    note = json!(format!(
                        "the scaffold was ingested, but writing the files out \
                         failed: {e:#}\n  Next step: run `hick weave {rel}` — \
                         the bytes are in the document, so nothing is lost."
                    ));
                }
            }
            // The document is the part that worked and it stays. What the run
            // said is the part the person has to act on, so it is carried
            // through whole rather than flattened to "scaffolding failed".
            Err(e) => note = json!(format!("{e:#}")),
        }
    }

    let written = std::fs::read_to_string(&absolute).unwrap_or(source);
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": id,
            "path": rel,
            "source": written,
            "ingested": ingested,
            "executor": state.executor.as_str(),
            "note": note,
        })),
    ))
}

/// Weave one document to disk, no execution: cached transcripts where there
/// are any, and the ingested block is not one of them — its bytes are in the
/// document, so a weave alone reproduces the whole tree. That is the property
/// `owning-what-a-scaffolder-wrote.md` exists for, exercised here at the
/// moment it is first true.
async fn weave_to_disk(doc: &std::path::Path) -> anyhow::Result<()> {
    let run = crate::run_doc(doc, &[], crate::RunMode::Weave, ExecutorChoice::Local).await?;
    crate::write_outputs(&run, None)?;
    Ok(())
}
