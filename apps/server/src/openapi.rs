//! The generated OpenAPI spec (`just codegen` → `apps/server/openapi.json` →
//! `apps/web/src/api/generated/schema.d.ts`). Purely macro-derived from the
//! `#[utoipa::path]`/`ToSchema` annotations on the route handlers and
//! request/response structs below — no `AppState`, no DB, no env vars needed
//! to build it (see `src/bin/print_openapi.rs`).
//!
//! Every handler now returns a typed response struct. A few fields stay
//! `serde_json::Value` on purpose, documented at each site: values sourced
//! from an external crate with no `ToSchema` impl (`hickory_lineage::{
//! OutputEdit, SourceEdit, Provenance}`), or genuinely dynamic shapes the
//! handler builds by mutating/normalizing JSON in place rather than through
//! a fixed Rust type (`docs::render_doc`'s run-status overlay,
//! `outputs::outputs_nav`'s LSP location union).

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "Hickory Docs API", version = "0.1.0"),
    paths(
        crate::routes::auth::signup,
        crate::routes::auth::login,
        crate::routes::auth::me,
        crate::routes::auth::send_verification,
        crate::routes::auth::confirm_verification,
        crate::routes::auth::request_reset,
        crate::routes::auth::confirm_reset,
        crate::routes::llm_keys::list_llm_keys,
        crate::routes::llm_keys::save_llm_key,
        crate::routes::llm_keys::delete_llm_key,
        crate::routes::llm_keys::select_llm_key,
        crate::routes::projects::list_projects,
        crate::routes::projects::create_project,
        crate::routes::projects::list_docs,
        crate::routes::projects::create_doc,
        crate::routes::docs::get_doc,
        crate::routes::docs::put_doc,
        crate::routes::docs::render_doc,
        crate::routes::outputs::list_outputs,
        crate::routes::outputs::get_output_file,
        crate::routes::outputs::edit_outputs,
        crate::routes::outputs::outputs_nav,
        crate::routes::runs::run_doc,
        crate::routes::runs::check_doc,
        crate::routes::runs::get_run,
        crate::routes::agent::start_agent,
        crate::routes::agent::list_turns,
        crate::routes::billing::get_plans,
        crate::routes::billing::checkout,
        crate::routes::billing::webhook,
        crate::routes::analytics::capture,
        crate::routes::health::health,
        crate::routes::health::executor,
    ),
    components(schemas(
        // Requests
        crate::routes::auth::Credentials,
        crate::routes::auth::TokenBody,
        crate::routes::auth::EmailBody,
        crate::routes::auth::ResetBody,
        crate::routes::llm_keys::SaveLlmKey,
        crate::routes::llm_keys::SelectProvider,
        crate::routes::projects::CreateProject,
        crate::routes::projects::CreateDoc,
        crate::routes::docs::SaveDoc,
        crate::routes::outputs::EditRequest,
        crate::routes::outputs::NavRequest,
        crate::routes::runs::RunRequest,
        crate::routes::agent::AgentRequest,
        crate::routes::billing::CheckoutRequest,
        crate::routes::analytics::CaptureRequest,
        // Responses
        crate::routes::auth::UserOut,
        crate::routes::auth::AuthOut,
        crate::routes::auth::StatusOut,
        crate::byok::StoredKey,
        crate::routes::llm_keys::LlmKeysOut,
        crate::routes::projects::ProjectOut,
        crate::routes::projects::DocSummaryOut,
        crate::routes::docs::DocOut,
        crate::routes::docs::RenderOut,
        crate::routes::outputs::OutputFileMeta,
        crate::routes::outputs::OutputsListOut,
        crate::routes::outputs::OutputFileOut,
        crate::routes::outputs::EditOutputsOut,
        crate::routes::outputs::NavOut,
        crate::routes::runs::RunStartOut,
        crate::routes::runs::RunOut,
        crate::routes::agent::TurnRow,
        crate::routes::agent::TurnsOut,
        crate::routes::agent::AgentStartOut,
        crate::routes::billing::CheckoutOut,
        crate::routes::billing::WebhookOut,
        crate::routes::analytics::CaptureOut,
        crate::routes::health::HealthOut,
        crate::routes::health::ExecutorOut,
        crate::plans::PlansOut,
        crate::plans::PlanOut,
        crate::plans::PlanPriceOut,
    ))
)]
pub struct ApiDoc;
