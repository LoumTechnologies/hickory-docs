//! `just codegen`'s plans step: print the pricing catalogue as the web app
//! consumes it.
//!
//! The projection from `plans.json` to what a pricing page shows — which plans
//! are in the set, which prices are active, the human feature bullets derived
//! from entitlements — lives in `plans::plans_response` and must have exactly
//! one implementation. Generating the web app's copy from it means a static
//! site can render pricing with no API call and still be the same pricing the
//! server would have served.
//!
//! Pure, like `print-openapi`: no env, no database, no network. The `default`
//! plan set is used, because a static build has no visitor to run a plan-set
//! experiment against — the flag lookup belongs to a running server.
use hickory_server::plans::{Catalog, plans_response};

fn main() -> anyhow::Result<()> {
    let catalog = Catalog::load()?;
    let out = plans_response(&catalog, "default");
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
