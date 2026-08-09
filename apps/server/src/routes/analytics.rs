//! `POST /api/analytics/capture` — the landing page's first-party event beacon.
//!
//! The browser never holds a PostHog key. It posts a typed event here and the
//! server forwards it with the credential it already has (`POSTHOG_API_KEY`,
//! see `crate::analytics`). Three reasons this beats loading a vendor SDK in
//! the bundle:
//!
//! - **One canonical variable name set.** No `VITE_POSTHOG_KEY` twin of
//!   `POSTHOG_API_KEY` to keep in sync across GitHub Environments.
//! - **The deploy image stays promotable.** `import.meta.env.VITE_*` is
//!   inlined at `docker build` time (see the `web` stage in `Dockerfile`), so
//!   a build-time key would bake one environment's project id into the image
//!   that the promote path then ships to the other.
//! - **It keeps measuring.** A third-party analytics script is the single
//!   most-blocked request on the web; a same-origin `/api` POST is not.
//!
//! The endpoint is unauthenticated by necessity — the visitors it exists to
//! measure have no account yet — so it is deliberately narrow: a fixed
//! allowlist of event names, a cap on how many properties and how long each
//! may be, and scalar property values only. There is nothing here an
//! untrusted caller can write that the server does not already understand.

use std::collections::BTreeMap;

use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::AppState;
use crate::error::{ApiError, ApiResult};

/// Every event the beacon will forward. Adding a landing-page event means
/// adding it here *and* to `apps/web/src/analytics/events.ts` — the two lists
/// are the same contract seen from each side.
const ALLOWED_EVENTS: &[&str] = &[
    "landing_viewed",
    "interest_expanded",
    "interest_clicked",
    "segment_declared",
    "segment_corrected",
    "cta_clicked",
    "demo_engaged",
];

/// Anonymous ids are client-generated UUIDs; anything longer is not one.
const MAX_DISTINCT_ID: usize = 64;
/// Comfortably above the widest event (`landing_viewed`, ~10 properties).
const MAX_PROPERTIES: usize = 24;
/// Long enough for a referrer or a UTM value, short enough to bound the body.
const MAX_STRING: usize = 300;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CaptureRequest {
    /// Anonymous id generated in the browser and stable for that browser.
    /// Never an account id — this endpoint predates the visitor having one.
    pub distinct_id: String,
    /// One of `ALLOWED_EVENTS`.
    pub event: String,
    /// Flat property bag. Values are scalars only (string, number, bool);
    /// a nested object or array is rejected rather than silently flattened,
    /// because PostHog would store a shape no query here expects.
    #[serde(default)]
    pub properties: BTreeMap<String, Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CaptureOut {
    /// `true` once the event is queued for forwarding. Also `true` when
    /// PostHog is unconfigured: capture is a no-op then, and a caller that
    /// treated that as failure would retry forever against a working server.
    pub accepted: bool,
}

#[utoipa::path(
    post,
    path = "/api/analytics/capture",
    request_body = CaptureRequest,
    responses(
        (status = 200, description = "event queued (or no-opped when PostHog is unconfigured)", body = CaptureOut),
        (status = 400, description = "unknown event name, or a property the beacon will not forward"),
    ),
    tag = "analytics"
)]
pub async fn capture(
    State(state): State<AppState>,
    Json(req): Json<CaptureRequest>,
) -> ApiResult<Json<CaptureOut>> {
    validate(&req)?;
    let mut properties = serde_json::Map::new();
    for (k, v) in req.properties {
        properties.insert(k, v);
    }
    state
        .analytics
        .capture(&req.distinct_id, &req.event, Value::Object(properties));
    Ok(Json(CaptureOut { accepted: true }))
}

fn validate(req: &CaptureRequest) -> Result<(), ApiError> {
    if req.distinct_id.is_empty() || req.distinct_id.len() > MAX_DISTINCT_ID {
        return Err(ApiError::bad_request(format!(
            "distinct_id must be 1–{MAX_DISTINCT_ID} characters (got {}). \
             It should be the anonymous browser id from \
             apps/web/src/analytics/attribution.ts, not an account id.",
            req.distinct_id.len()
        )));
    }
    if !ALLOWED_EVENTS.contains(&req.event.as_str()) {
        return Err(ApiError::bad_request(format!(
            "unknown analytics event '{}'. Allowed: {}. Adding an event means \
             adding it to ALLOWED_EVENTS in apps/server/src/routes/analytics.rs \
             and to the LandingEvent union in apps/web/src/analytics/events.ts.",
            req.event,
            ALLOWED_EVENTS.join(", ")
        )));
    }
    if req.properties.len() > MAX_PROPERTIES {
        return Err(ApiError::bad_request(format!(
            "too many properties: {} (max {MAX_PROPERTIES}). Send only the \
             properties the event declares.",
            req.properties.len()
        )));
    }
    for (key, value) in &req.properties {
        check_property(key, value)?;
    }
    Ok(())
}

fn check_property(key: &str, value: &Value) -> Result<(), ApiError> {
    if key.len() > MAX_STRING {
        return Err(ApiError::bad_request(format!(
            "property name is {} characters (max {MAX_STRING})",
            key.len()
        )));
    }
    match value {
        Value::String(s) if s.len() > MAX_STRING => Err(ApiError::bad_request(format!(
            "property '{key}' is {} characters (max {MAX_STRING}). Truncate it \
             in the browser before sending — a URL or referrer this long is \
             almost always a tracking parameter that was not meant to ship.",
            s.len()
        ))),
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => Ok(()),
        Value::Array(_) | Value::Object(_) => Err(ApiError::bad_request(format!(
            "property '{key}' must be a string, number, or boolean — nested \
             objects and arrays are not forwarded. Flatten it into separate \
             properties (e.g. 'utm_source' and 'utm_campaign')."
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn req(event: &str, properties: Value) -> CaptureRequest {
        CaptureRequest {
            distinct_id: "1a2b3c4d-0000-4000-8000-000000000000".to_string(),
            event: event.to_string(),
            properties: serde_json::from_value(properties).unwrap(),
        }
    }

    // Protects docs/guarantees/analytics/landing-beacon-is-narrow.md
    #[test]
    fn accepts_a_declared_event_with_scalar_properties() {
        let r = req(
            "interest_expanded",
            json!({ "interest_id": "ci-drift", "dwell_ms": 4200, "first_open": true }),
        );
        assert!(validate(&r).is_ok());
    }

    // Protects docs/guarantees/analytics/landing-beacon-is-narrow.md
    #[test]
    fn rejects_an_event_name_that_is_not_on_the_allowlist() {
        let err = validate(&req("arbitrary_event", json!({}))).unwrap_err();
        assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST);
        // The message must name the fix, not just the rejection.
        assert!(err.message.contains("ALLOWED_EVENTS"), "{}", err.message);
    }

    // Protects docs/guarantees/analytics/landing-beacon-is-narrow.md
    #[test]
    fn rejects_nested_property_values() {
        let err = validate(&req("cta_clicked", json!({ "nested": { "a": 1 } }))).unwrap_err();
        assert!(err.message.contains("Flatten"), "{}", err.message);
        let err = validate(&req("cta_clicked", json!({ "list": [1, 2] }))).unwrap_err();
        assert!(err.message.contains("Flatten"), "{}", err.message);
    }

    // Protects docs/guarantees/analytics/landing-beacon-is-narrow.md
    #[test]
    fn rejects_an_oversized_property_or_id() {
        let long = "x".repeat(MAX_STRING + 1);
        let err = validate(&req("landing_viewed", json!({ "path": long }))).unwrap_err();
        assert!(err.message.contains("max"), "{}", err.message);

        let mut r = req("landing_viewed", json!({}));
        r.distinct_id = "y".repeat(MAX_DISTINCT_ID + 1);
        assert!(validate(&r).is_err());
        r.distinct_id = String::new();
        assert!(validate(&r).is_err());
    }

    // Protects docs/guarantees/analytics/landing-beacon-is-narrow.md
    #[test]
    fn rejects_a_property_bag_wider_than_the_cap() {
        let mut properties = serde_json::Map::new();
        for i in 0..=MAX_PROPERTIES {
            properties.insert(format!("k{i}"), json!("v"));
        }
        let err = validate(&req("landing_viewed", Value::Object(properties))).unwrap_err();
        assert!(
            err.message.contains("too many properties"),
            "{}",
            err.message
        );
    }
}
