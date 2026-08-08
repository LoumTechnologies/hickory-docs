//! `just codegen`'s spec-generation step. Deliberately separate from
//! `main.rs`: it must never need `Config::from_env()`, a database, or any
//! env var — the OpenAPI document is fully derived from `#[utoipa::path]`/
//! `ToSchema` macros at compile time, so printing it needs nothing running.
use hickory_server::openapi::ApiDoc;
use utoipa::OpenApi;

fn main() -> anyhow::Result<()> {
    println!("{}", ApiDoc::openapi().to_pretty_json()?);
    Ok(())
}
