//! Billing chassis: plans, Stripe Checkout, signature-verified idempotent
//! webhooks. Conventions per docs/specs/freeform/pricing-strategy.md:
//! - the DB enforces idempotency (UNIQUE + `INSERT ... ON CONFLICT DO
//!   NOTHING RETURNING`), never check-then-insert;
//! - the event log (`stripe_events`) is a second layer only, released on
//!   handler failure so Stripe redelivery retries instead of being dropped;
//! - dunning restricts, it never deletes.

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use hmac::{Hmac, Mac as _};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::Sha256;
use uuid::Uuid;

use crate::AppState;
use crate::auth::{AuthUser, MaybeUser};
use crate::error::{ApiError, ApiResult};
use crate::plans;

// ---------------------------------------------------------------------------
// GET /api/billing/plans
// ---------------------------------------------------------------------------

/// PostHog feature-flag key selecting the plan set (experiment hook).
const PLAN_SET_FLAG: &str = "hickory-plan-set";

pub async fn get_plans(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
) -> ApiResult<Json<plans::PlansOut>> {
    // Explicit env override → PostHog flag → "default".
    let mut set = state.config.plan_set.clone();
    if set.is_none() {
        let distinct_id = user
            .as_ref()
            .map(|u| u.id.to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        set = state.analytics.feature_flag(&distinct_id, PLAN_SET_FLAG).await;
    }
    let set = set.unwrap_or_else(|| "default".to_string());
    Ok(Json(plans::plans_response(&state.catalog, &set)))
}

// ---------------------------------------------------------------------------
// POST /api/billing/checkout
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CheckoutRequest {
    pub price_key: String,
}

pub async fn checkout(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<CheckoutRequest>,
) -> ApiResult<Json<Value>> {
    let Some(stripe) = &state.config.stripe else {
        return Err(ApiError::service_unavailable("billing not configured"));
    };
    let Some((plan_key, plan, price)) = state.catalog.price(&body.price_key) else {
        return Err(ApiError::bad_request(format!("unknown price key '{}'", body.price_key)));
    };
    if price.active == Some(false) {
        return Err(ApiError::bad_request("this price is no longer offered"));
    }
    let Some(interval) = price.interval.as_deref() else {
        return Err(ApiError::bad_request("price has no billing interval"));
    };

    let base = state.config.app_base_url.trim_end_matches('/');
    // The Stripe catalog is Terraform-owned once launched; until price ids
    // are materialized in plans.json we fall back to inline price_data
    // (sandbox posture — see pricing-strategy.md readiness gate).
    let stripe_price_id = price.stripe_price_id.as_ref().and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        Value::Object(m) => {
            let env = match state.config.app_env {
                crate::config::AppEnv::Production => "production",
                _ => "staging",
            };
            m.get(env).and_then(Value::as_str).map(|s| s.to_string())
        }
        _ => None,
    });

    let mut form: Vec<(String, String)> = vec![
        ("mode".into(), "subscription".into()),
        ("success_url".into(), format!("{base}/billing/success?session_id={{CHECKOUT_SESSION_ID}}")),
        ("cancel_url".into(), format!("{base}/pricing")),
        ("client_reference_id".into(), user.id.to_string()),
        ("metadata[user_id]".into(), user.id.to_string()),
        ("metadata[price_key]".into(), price.key.clone()),
        ("metadata[plan_key]".into(), plan_key.to_string()),
        ("subscription_data[metadata][user_id]".into(), user.id.to_string()),
        ("subscription_data[metadata][price_key]".into(), price.key.clone()),
        ("subscription_data[metadata][plan_key]".into(), plan_key.to_string()),
        ("line_items[0][quantity]".into(), "1".into()),
    ];
    if let Some(trial) = plan.trial_days {
        form.push(("subscription_data[trial_period_days]".into(), trial.to_string()));
    }
    match stripe_price_id {
        Some(id) => form.push(("line_items[0][price]".into(), id)),
        None => {
            form.extend([
                ("line_items[0][price_data][currency]".into(), price.currency.clone()),
                ("line_items[0][price_data][unit_amount]".into(), price.amount_cents.to_string()),
                ("line_items[0][price_data][recurring][interval]".into(), interval.to_string()),
                (
                    "line_items[0][price_data][product_data][name]".into(),
                    format!("Hickory Docs {}", plan.name),
                ),
            ]);
        }
    }

    let resp = state
        .http
        .post("https://api.stripe.com/v1/checkout/sessions")
        .bearer_auth(&stripe.secret_key)
        .form(&form)
        .send()
        .await
        .map_err(|e| ApiError::internal(format!("stripe request failed: {e}")))?;
    let status = resp.status();
    let body: Value = resp
        .json()
        .await
        .map_err(|e| ApiError::internal(format!("stripe response unreadable: {e}")))?;
    if !status.is_success() {
        let msg = body
            .pointer("/error/message")
            .and_then(Value::as_str)
            .unwrap_or("checkout session creation failed");
        return Err(ApiError::internal(format!("stripe: {msg}")));
    }
    let url = body
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| ApiError::internal("stripe response missing url"))?;

    state.analytics.capture(
        &user.id.to_string(),
        "checkout_started",
        json!({
            "price_key": price.key,
            "plan_key": plan_key,
            "amount_cents": price.amount_cents,
            "currency": price.currency,
        }),
    );
    Ok(Json(json!({ "checkout_url": url })))
}

// ---------------------------------------------------------------------------
// POST /api/billing/webhook
// ---------------------------------------------------------------------------

/// Verify a `Stripe-Signature` header (t=...,v1=...) against the payload.
pub fn verify_stripe_signature(
    secret: &str,
    header: &str,
    payload: &[u8],
    now_unix: i64,
) -> bool {
    let mut timestamp: Option<i64> = None;
    let mut sigs: Vec<String> = Vec::new();
    for part in header.split(',') {
        let mut kv = part.trim().splitn(2, '=');
        match (kv.next(), kv.next()) {
            (Some("t"), Some(v)) => timestamp = v.parse().ok(),
            (Some("v1"), Some(v)) => sigs.push(v.to_string()),
            _ => {}
        }
    }
    let Some(t) = timestamp else { return false };
    if (now_unix - t).abs() > 300 {
        return false;
    }
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac accepts any key");
    mac.update(format!("{t}.").as_bytes());
    mac.update(payload);
    let expected = hex::encode(mac.finalize().into_bytes());
    sigs.iter().any(|s| {
        // Constant-time comparison.
        s.len() == expected.len()
            && s.bytes()
                .zip(expected.bytes())
                .fold(0u8, |acc, (a, b)| acc | (a ^ b))
                == 0
    })
}

pub async fn webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<Value>> {
    let Some(stripe) = &state.config.stripe else {
        return Err(ApiError::service_unavailable("billing not configured"));
    };
    let Some(secret) = &stripe.webhook_secret else {
        return Err(ApiError::service_unavailable("billing not configured"));
    };
    let sig = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::bad_request("missing Stripe-Signature header"))?;
    if !verify_stripe_signature(secret, sig, &body, chrono::Utc::now().timestamp()) {
        return Err(ApiError::bad_request("invalid webhook signature"));
    }

    let event: Value = serde_json::from_slice(&body)
        .map_err(|_| ApiError::bad_request("invalid webhook payload"))?;
    let event_id = event
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| ApiError::bad_request("event missing id"))?
        .to_string();
    let event_type = event
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let object = event.pointer("/data/object").cloned().unwrap_or(Value::Null);

    // Second-layer event log: skip already-seen event ids.
    let claimed: Option<(String,)> = sqlx::query_as(
        "INSERT INTO stripe_events (id) VALUES ($1) ON CONFLICT (id) DO NOTHING RETURNING id",
    )
    .bind(&event_id)
    .fetch_optional(&state.db)
    .await?;
    if claimed.is_none() {
        return Ok(Json(json!({ "received": true, "duplicate": true })));
    }

    let result = handle_event(&state, &event_type, &object).await;
    if let Err(e) = result {
        // Release the claim so Stripe's redelivery retries the event
        // instead of it being silently dropped as already-seen.
        let _ = sqlx::query("DELETE FROM stripe_events WHERE id = $1")
            .bind(&event_id)
            .execute(&state.db)
            .await;
        return Err(ApiError::internal(format!("webhook handler failed: {e}")));
    }
    Ok(Json(json!({ "received": true })))
}

async fn handle_event(state: &AppState, event_type: &str, object: &Value) -> anyhow::Result<()> {
    match event_type {
        "checkout.session.completed" => checkout_completed(state, object).await,
        "customer.subscription.updated" => subscription_updated(state, object).await,
        "customer.subscription.deleted" => subscription_deleted(state, object).await,
        "invoice.payment_failed" => payment_failed(state, object).await,
        other => {
            log::debug!("ignoring stripe event type {other}");
            Ok(())
        }
    }
}

fn meta_str<'a>(object: &'a Value, key: &str) -> Option<&'a str> {
    object.pointer(&format!("/metadata/{key}")).and_then(Value::as_str)
}

async fn checkout_completed(state: &AppState, object: &Value) -> anyhow::Result<()> {
    let subscription_id = object
        .get("subscription")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("checkout.session.completed without subscription"))?;
    let user_id: Uuid = meta_str(object, "user_id")
        .or_else(|| object.get("client_reference_id").and_then(Value::as_str))
        .ok_or_else(|| anyhow::anyhow!("checkout session missing user_id"))?
        .parse()?;
    let price_key = meta_str(object, "price_key")
        .ok_or_else(|| anyhow::anyhow!("checkout session missing price_key"))?;
    let (plan_key, _plan) = state
        .catalog
        .plan_for_price(price_key)
        .ok_or_else(|| anyhow::anyhow!("unknown price key '{price_key}' in webhook"))?;

    // THE idempotency mechanism: unique fulfillment claim on the
    // subscription id. Distinct Stripe event ids for the same subscription
    // collapse here.
    let created: Option<(String,)> = sqlx::query_as(
        "INSERT INTO subscriptions (stripe_subscription_id, user_id, price_key, plan_key, status)
         VALUES ($1, $2, $3, $4, 'active')
         ON CONFLICT (stripe_subscription_id) DO NOTHING
         RETURNING stripe_subscription_id",
    )
    .bind(subscription_id)
    .bind(user_id)
    .bind(price_key)
    .bind(plan_key)
    .fetch_optional(&state.db)
    .await?;

    // Side effects are gated on *this* caller having created the claim.
    if created.is_some() {
        sqlx::query(
            "UPDATE users SET plan_key = $1, price_key = $2, stripe_customer_id = $3,
                              billing_status = 'active'
             WHERE id = $4",
        )
        .bind(plan_key)
        .bind(price_key)
        .bind(object.get("customer").and_then(Value::as_str))
        .bind(user_id)
        .execute(&state.db)
        .await?;
        state.analytics.capture(
            &user_id.to_string(),
            "purchase_completed",
            json!({
                "price_key": price_key,
                "plan_key": plan_key,
                "amount_cents": object.get("amount_total"),
                "currency": object.get("currency"),
            }),
        );
    }
    Ok(())
}

async fn subscription_updated(state: &AppState, object: &Value) -> anyhow::Result<()> {
    let sub_id = object
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("subscription event without id"))?;
    let status = object.get("status").and_then(Value::as_str).unwrap_or("active");
    let row: Option<(Uuid,)> = sqlx::query_as(
        "UPDATE subscriptions SET status = $1, updated_at = now()
         WHERE stripe_subscription_id = $2 RETURNING user_id",
    )
    .bind(status)
    .bind(sub_id)
    .fetch_optional(&state.db)
    .await?;
    let Some((user_id,)) = row else {
        log::warn!("subscription.updated for unknown subscription {sub_id}");
        return Ok(());
    };
    // Dunning restricts; it never deletes.
    let billing_status = match status {
        "past_due" | "unpaid" | "incomplete" => "past_due",
        _ => "active",
    };
    sqlx::query("UPDATE users SET billing_status = $1 WHERE id = $2")
        .bind(billing_status)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    if status == "canceled" {
        return downgrade_to_open(state, user_id).await;
    }
    Ok(())
}

async fn subscription_deleted(state: &AppState, object: &Value) -> anyhow::Result<()> {
    let sub_id = object
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("subscription event without id"))?;
    let row: Option<(Uuid,)> = sqlx::query_as(
        "UPDATE subscriptions SET status = 'canceled', updated_at = now()
         WHERE stripe_subscription_id = $1 RETURNING user_id",
    )
    .bind(sub_id)
    .fetch_optional(&state.db)
    .await?;
    if let Some((user_id,)) = row {
        downgrade_to_open(state, user_id).await?;
        state
            .analytics
            .capture(&user_id.to_string(), "subscription_canceled", json!({}));
    }
    Ok(())
}

/// Downgrade to the free tier. Data is never deleted: over-quota private
/// projects stay readable; only *new* private projects are gated.
async fn downgrade_to_open(state: &AppState, user_id: Uuid) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE users SET plan_key = 'open', price_key = NULL, billing_status = 'active'
         WHERE id = $1",
    )
    .bind(user_id)
    .execute(&state.db)
    .await?;
    Ok(())
}

async fn payment_failed(state: &AppState, object: &Value) -> anyhow::Result<()> {
    let sub_id = object.get("subscription").and_then(Value::as_str);
    let user_id: Option<(Uuid,)> = match sub_id {
        Some(sid) => {
            sqlx::query_as("SELECT user_id FROM subscriptions WHERE stripe_subscription_id = $1")
                .bind(sid)
                .fetch_optional(&state.db)
                .await?
        }
        None => None,
    };
    if let Some((user_id,)) = user_id {
        sqlx::query("UPDATE users SET billing_status = 'past_due' WHERE id = $1")
            .bind(user_id)
            .execute(&state.db)
            .await?;
        state
            .analytics
            .capture(&user_id.to_string(), "payment_failed", json!({}));
    }
    Ok(())
}
