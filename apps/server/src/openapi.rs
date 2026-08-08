//! The generated OpenAPI spec (`just codegen` → `apps/server/openapi.json` →
//! `apps/web/src/api/generated/schema.d.ts`). Purely macro-derived from the
//! `#[utoipa::path]`/`ToSchema` annotations on the route handlers and
//! request structs below — no `AppState`, no DB, no env vars needed to build
//! it (see `src/bin/print_openapi.rs`).
//!
//! Response bodies are intentionally untyped (`serde_json::Value`) for now —
//! most handlers still build ad-hoc `json!()` responses rather than typed
//! structs. Typing them is tracked as a separate follow-up, done
//! route-by-route; only `GET /api/billing/plans` already returns a typed
//! struct (`plans::PlansOut`) and is documented as such.

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
        crate::routes::health::health,
        crate::routes::health::executor,
    ),
    components(schemas(
        crate::routes::auth::Credentials,
        crate::routes::auth::TokenBody,
        crate::routes::auth::EmailBody,
        crate::routes::auth::ResetBody,
        crate::routes::projects::CreateProject,
        crate::routes::projects::CreateDoc,
        crate::routes::docs::SaveDoc,
        crate::routes::outputs::EditRequest,
        crate::routes::outputs::NavRequest,
        crate::routes::runs::RunRequest,
        crate::routes::agent::AgentRequest,
        crate::routes::billing::CheckoutRequest,
        crate::plans::PlansOut,
        crate::plans::PlanOut,
        crate::plans::PlanPriceOut,
    ))
)]
pub struct ApiDoc;
