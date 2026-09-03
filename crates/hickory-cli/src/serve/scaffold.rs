//! File → New Project: `dotnet new`, as a form and then as a commit.
//!
//! Four routes, and the shape of them is the argument. Reading the catalogue
//! and reading one template's options are separate because the second is a
//! second `dotnet` process and the dialog only needs it once a template is
//! chosen — a New Project dialog that spawned forty-six help invocations to
//! open would be unusable on a laptop.
//!
//! ## Why there is a preview route
//!
//! The dialog shows the exact commit message it is about to write, the way
//! the Insert panel shows its markup, and for the same reason: a recipe is a
//! text a person will read in `git log` for years, and a dialog that writes
//! one you never saw teaches you nothing. The preview is the same function,
//! called over HTTP, so what is shown and what is committed cannot be two
//! renderers that agree today.
//!
//! ## Creating is one act
//!
//! `POST /api/scaffold` runs the scaffolder into a scratch directory and
//! commits what it wrote, through an index of its own, with the recipe in
//! the trailers (`crate::scaffold_commit`). There is no moment between the
//! two for an edit to slip into the recipe commit, which is what keeps the
//! commit upgradeable. A scaffold that fails commits nothing and leaves the
//! working tree as it was: there is no half-finished document to keep,
//! because there is no document. `docs/specs/freeform/lenses.md`, step 3.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};
use crate::scaffold::{self, NoDotnetSdk, ScaffoldSpec};
use crate::scaffold_commit::{self, NotARepository};

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
    // The same shape for the other thing a machine can lack: a repository
    // to record the scaffold in.
    if e.downcast_ref::<NotARepository>().is_some() {
        return ApiError::unprocessable(format!("{}", NotARepository))
            .with_detail(json!({ "missing": "repository" }));
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

/// What the dialog is asking to be scaffolded and committed.
#[derive(Deserialize)]
pub struct CreateRequest {
    #[serde(flatten)]
    pub spec: ScaffoldSpec,
}

/// The place in the message a preview cannot fill in yet.
const OUTPUT_PENDING: &str = "<the tree hash, once it is committed>";

/// `POST /api/scaffold/preview` — the exact commit, without making it.
pub async fn preview(
    State(_state): State<LocalState>,
    Json(body): Json<CreateRequest>,
) -> ApiResult<Json<Value>> {
    let output = scaffold_commit::checked_output(&body.spec.output)
        .map(|o| o.to_string())
        .unwrap_or_else(|_| body.spec.output.trim().to_string());
    Ok(Json(json!({
        "output": output,
        "command": scaffold::dotnet_new_command(&body.spec),
        "message": scaffold::commit_message(&body.spec, OUTPUT_PENDING, &output),
    })))
}

/// `POST /api/scaffold` — run the scaffolder and commit what it wrote, as
/// one act.
pub async fn create(
    State(state): State<LocalState>,
    Json(body): Json<CreateRequest>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    if body.spec.name.trim().is_empty() {
        return Err(ApiError::bad_request(
            "a project needs a name — it becomes the .NET root namespace and the assembly name.",
        ));
    }
    let output = scaffold_commit::checked_output(&body.spec.output)
        .map_err(|e| ApiError::bad_request(format!("{e:#}")))?;
    let root = state.index.root().to_path_buf();
    let spec = ScaffoldSpec {
        output: output.clone(),
        ..body.spec
    };

    let committed = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        // The cheap refusals first, before a subprocess runs.
        if !scaffold_commit::is_repository(&root) {
            anyhow::bail!(NotARepository);
        }
        scaffold_commit::refuse_occupied(&root, &output)?;
        // Into scratch, never the repository: a scaffolder that fails halfway
        // leaves nothing behind, and one that succeeds is committed whole.
        let scratch = tempfile::tempdir()?;
        let into = scratch.path().join("out");
        scaffold::run_scaffold(&spec, &into)?;
        scaffold_commit::commit_scaffold(&root, &spec, &into)
    })
    .await
    .map_err(|e| ApiError::internal(format!("the scaffold did not finish: {e}")))?
    .map_err(scaffold_error)?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "sha": committed.sha,
            "short": committed.short,
            "output": spec_output_of(&committed.message),
            "files": committed.files,
            "message": committed.message,
            "output_tree": committed.output_tree,
        })),
    ))
}

/// The output folder, read back from the message's own trailer so the
/// answer and the record cannot disagree.
fn spec_output_of(message: &str) -> String {
    message
        .lines()
        .find_map(|l| l.strip_prefix("Hick-Output: "))
        .and_then(|v| v.split_once(' '))
        .map(|(_, path)| path.to_string())
        .unwrap_or_default()
}
