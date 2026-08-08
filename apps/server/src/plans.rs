//! The plan catalog (`plans.json`, embedded at build time) and the ONE
//! entitlements module: every can/quota question is answered here from the
//! account's *purchased* price key mapped back through `plans.json`
//! (grandfathering falls out — retired plans stay in the file).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The repo-root plans.json, embedded so the binary is self-contained.
pub const PLANS_JSON: &str = include_str!("../../../plans.json");

#[derive(Debug, Clone, Deserialize)]
pub struct Catalog {
    pub plan_sets: HashMap<String, Vec<String>>,
    pub plans: HashMap<String, PlanDef>,
    #[serde(default)]
    pub enterprise: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlanDef {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub trial_days: Option<u32>,
    #[serde(default)]
    pub highlight: Option<bool>,
    #[serde(default)]
    pub retired: Option<bool>,
    #[serde(default)]
    pub prices: Vec<PriceDef>,
    pub entitlements: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceDef {
    pub key: String,
    pub kind: String,
    #[serde(default)]
    pub interval: Option<String>,
    pub amount_cents: i64,
    pub currency: String,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub per_seat: Option<bool>,
    #[serde(default)]
    pub stripe_price_id: Option<Value>,
}

impl Catalog {
    pub fn load() -> anyhow::Result<Catalog> {
        Ok(serde_json::from_str(PLANS_JSON)?)
    }

    /// Plan key owning a given price key, across all plans (all environments,
    /// retired included) — purchased-price → plan resolution.
    pub fn plan_for_price(&self, price_key: &str) -> Option<(&str, &PlanDef)> {
        self.plans.iter().find_map(|(k, p)| {
            p.prices
                .iter()
                .any(|pr| pr.key == price_key)
                .then_some((k.as_str(), p))
        })
    }

    pub fn price(&self, price_key: &str) -> Option<(&str, &PlanDef, &PriceDef)> {
        self.plans.iter().find_map(|(k, p)| {
            p.prices
                .iter()
                .find(|pr| pr.key == price_key)
                .map(|pr| (k.as_str(), p, pr))
        })
    }
}

// ---------------------------------------------------------------------------
// Entitlements
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limit {
    Count(u64),
    Unlimited,
}

impl Limit {
    pub fn allows(&self, current: u64) -> bool {
        match self {
            Limit::Count(n) => current < *n,
            Limit::Unlimited => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entitlements {
    pub plan_key: String,
    pub private_projects: Limit,
    pub editors: u64,
    pub exec_minutes_month: u64,
    pub ci_verification: bool,
    pub agent: String,
}

fn limit_of(v: Option<&Value>) -> Limit {
    match v {
        Some(Value::Number(n)) => Limit::Count(n.as_u64().unwrap_or(0)),
        Some(Value::String(s)) if s == "unlimited" => Limit::Unlimited,
        _ => Limit::Count(0),
    }
}

impl Entitlements {
    fn from_plan(plan_key: &str, def: &PlanDef) -> Entitlements {
        let e = &def.entitlements;
        Entitlements {
            plan_key: plan_key.to_string(),
            private_projects: limit_of(e.get("private_projects")),
            editors: e.get("editors").and_then(Value::as_u64).unwrap_or(1),
            exec_minutes_month: e
                .get("exec_minutes_month")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            ci_verification: e
                .get("ci_verification")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            agent: e
                .get("agent")
                .and_then(Value::as_str)
                .unwrap_or("byo_key")
                .to_string(),
        }
    }
}

/// Resolve an account's entitlements. `price_key` (the purchased price) wins
/// over `plan_key`; dunning (`billing_status = past_due`) restricts to the
/// free tier — never deletes anything.
pub fn resolve(
    catalog: &Catalog,
    plan_key: &str,
    price_key: Option<&str>,
    billing_status: &str,
) -> Entitlements {
    if billing_status == "past_due"
        && let Some(open) = catalog.plans.get("open")
    {
        return Entitlements::from_plan("open", open);
    }
    if let Some(pk) = price_key
        && let Some((key, def)) = catalog.plan_for_price(pk)
    {
        return Entitlements::from_plan(key, def);
    }
    if let Some(def) = catalog.plans.get(plan_key) {
        return Entitlements::from_plan(plan_key, def);
    }
    // Unknown plan: fall back to the free tier.
    catalog
        .plans
        .get("open")
        .map(|d| Entitlements::from_plan("open", d))
        .unwrap_or(Entitlements {
            plan_key: "open".to_string(),
            private_projects: Limit::Count(0),
            editors: 1,
            exec_minutes_month: 0,
            ci_verification: false,
            agent: "byo_key".to_string(),
        })
}

// ---------------------------------------------------------------------------
// GET /api/billing/plans response (shape pinned in api.md / web types.ts)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PlanPriceOut {
    pub key: String,
    pub interval: String,
    pub amount_cents: i64,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_seat: Option<bool>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PlanOut {
    pub key: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trial_days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight: Option<bool>,
    pub prices: Vec<PlanPriceOut>,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PlansOut {
    pub plans: Vec<PlanOut>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub enterprise: Option<Value>,
}

fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Human feature bullets derived from a plan's entitlements.
fn features_of(def: &PlanDef) -> Vec<String> {
    let e = &def.entitlements;
    let mut out = vec!["Unlimited public projects".to_string()];
    match e.get("private_projects") {
        Some(Value::String(s)) if s == "unlimited" => {
            out.push("Unlimited private projects".to_string())
        }
        Some(Value::Number(n)) => {
            let n = n.as_u64().unwrap_or(0);
            if n == 1 {
                out.push("1 private project".to_string());
            } else if n > 1 {
                out.push(format!("{n} private projects"));
            }
        }
        _ => {}
    }
    if let Some(n) = e.get("editors").and_then(Value::as_u64) {
        if n == 1 {
            out.push("1 editor".to_string());
        } else {
            out.push(format!("Up to {n} editors"));
        }
    }
    if let Some(n) = e.get("exec_minutes_month").and_then(Value::as_u64) {
        out.push(format!("{} execution minutes/mo", thousands(n)));
    }
    if e.get("ci_verification").and_then(Value::as_bool) == Some(true) {
        out.push("CI verification checks + badge".to_string());
    }
    match e.get("agent").and_then(Value::as_str) {
        Some("byo_key") => out.push("AI agent (bring your own API key)".to_string()),
        Some("metered_allowance") => {
            if let Some(cents) = e
                .get("agent_llm_budget_cents_month")
                .and_then(Value::as_u64)
            {
                out.push(format!(
                    "AI agent allowance included (${}/mo LLM budget)",
                    cents / 100
                ));
            } else {
                out.push("AI agent allowance included".to_string());
            }
        }
        _ => {}
    }
    if e.get("review_workflow").and_then(Value::as_bool) == Some(true) {
        out.push("Review workflow".to_string());
    }
    if e.get("priority_execution").and_then(Value::as_bool) == Some(true) {
        out.push("Priority execution".to_string());
    }
    if e.get("sso").and_then(Value::as_bool) == Some(true) {
        out.push("SSO/SAML".to_string());
    }
    if e.get("audit_export").and_then(Value::as_bool) == Some(true) {
        out.push("Audit provenance export".to_string());
    }
    if e.get("byon").and_then(Value::as_bool) == Some(true) {
        out.push("Bring your own execution node".to_string());
    }
    out
}

/// Build the plans response for a named plan set (falling back to `default`).
pub fn plans_response(catalog: &Catalog, plan_set: &str) -> PlansOut {
    let keys = catalog
        .plan_sets
        .get(plan_set)
        .or_else(|| catalog.plan_sets.get("default"))
        .cloned()
        .unwrap_or_default();
    let plans = keys
        .iter()
        .filter_map(|key| catalog.plans.get(key).map(|def| (key, def)))
        .filter(|(_, def)| def.retired != Some(true))
        .map(|(key, def)| PlanOut {
            key: key.clone(),
            name: def.name.clone(),
            description: def.description.clone(),
            trial_days: def.trial_days,
            highlight: def.highlight,
            prices: def
                .prices
                .iter()
                .filter(|p| p.active.unwrap_or(true) && p.per_seat != Some(true))
                .filter_map(|p| {
                    p.interval.as_ref().map(|interval| PlanPriceOut {
                        key: p.key.clone(),
                        interval: interval.clone(),
                        amount_cents: p.amount_cents,
                        currency: p.currency.clone(),
                        per_seat: None,
                    })
                })
                .collect(),
            features: features_of(def),
        })
        .collect();
    PlansOut {
        plans,
        enterprise: catalog.enterprise.clone(),
    }
}
