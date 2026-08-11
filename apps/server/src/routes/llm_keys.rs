//! `/api/me/llm-keys` — the account's own provider credentials (BYOK).
//!
//! Every handler is thin on purpose: the rules about which key is used, what
//! a plan entitles, and how a key is sealed live in [`crate::byok`] and
//! [`crate::keyvault`], so the agent route and this route can never disagree
//! about them.
//!
//! A stored key is write-only over the API. There is no endpoint that returns
//! one, not even to the account that saved it: a key the server can hand back
//! is a key an XSS or a stolen token can exfiltrate whole, and the user
//! already has the value — they got it from the vendor.

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::AppState;
use crate::auth::AuthUser;
use crate::byok::{self, StoredKey};
use crate::error::ApiResult;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct LlmKeysOut {
    pub keys: Vec<StoredKey>,
    /// True when the deployment can store keys at all (`KEY_ENCRYPTION_KEY`
    /// is set). The UI explains the gap rather than offering a form that
    /// cannot succeed.
    pub storage_available: bool,
    /// `byo_key` (the account supplies the key) or `metered_allowance` (the
    /// plan includes agent spend).
    pub plan_agent: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SaveLlmKey {
    /// The provider key, as issued by the vendor.
    pub api_key: String,
    /// Model override; omit for the provider's default.
    #[serde(default)]
    pub model: Option<String>,
    /// Make this the key agent runs use. Ignored when it is the only key —
    /// a single key is already the choice.
    #[serde(default)]
    pub preferred: bool,
}

/// GET /api/me/llm-keys — which keys this account has stored.
#[utoipa::path(
    get,
    path = "/api/me/llm-keys",
    responses((status = 200, description = "stored provider keys", body = LlmKeysOut)),
    tag = "llm-keys"
)]
pub async fn list_llm_keys(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<LlmKeysOut>> {
    let ents = crate::plans::resolve(
        &state.catalog,
        &user.plan_key,
        user.price_key.as_deref(),
        &user.billing_status,
    );
    Ok(Json(LlmKeysOut {
        keys: byok::list_keys(&state, user.id).await?,
        storage_available: state.config.key_vault.is_some(),
        plan_agent: ents.agent,
    }))
}

/// PUT /api/me/llm-keys/{provider} — store or replace a key.
#[utoipa::path(
    put,
    path = "/api/me/llm-keys/{provider}",
    params(("provider" = String, Path, description = "anthropic | openai | deepseek | grok")),
    request_body = SaveLlmKey,
    responses(
        (status = 200, description = "the stored key's metadata", body = StoredKey),
        (status = 400, description = "unknown provider, or the vendor rejected the key"),
        (status = 503, description = "this deployment cannot store keys"),
    ),
    tag = "llm-keys"
)]
pub async fn save_llm_key(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(provider): Path<String>,
    Json(body): Json<SaveLlmKey>,
) -> ApiResult<Json<StoredKey>> {
    let stored = byok::save_key(
        &state,
        user.id,
        &provider,
        &body.api_key,
        body.model.as_deref(),
    )
    .await?;
    if body.preferred {
        byok::set_preferred(&state, user.id, &stored.provider).await?;
    }
    state.analytics.capture(
        &user.id.to_string(),
        "llm_key_saved",
        // The key itself is obviously absent; so is `last4`, which is a
        // display aid and has no business in an analytics warehouse.
        serde_json::json!({ "provider": stored.provider }),
    );
    Ok(Json(stored))
}

/// DELETE /api/me/llm-keys/{provider} — forget a key.
#[utoipa::path(
    delete,
    path = "/api/me/llm-keys/{provider}",
    params(("provider" = String, Path, description = "anthropic | openai | deepseek | grok")),
    responses(
        (status = 200, description = "the remaining stored keys", body = LlmKeysOut),
        (status = 404, description = "no key stored for that provider"),
    ),
    tag = "llm-keys"
)]
pub async fn delete_llm_key(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(provider): Path<String>,
) -> ApiResult<Json<LlmKeysOut>> {
    byok::delete_key(&state, user.id, &provider).await?;
    list_llm_keys(State(state), AuthUser(user)).await
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SelectProvider {
    pub provider: String,
}

/// PUT /api/me/llm-keys — choose which stored key agent runs use.
#[utoipa::path(
    put,
    path = "/api/me/llm-keys",
    request_body = SelectProvider,
    responses(
        (status = 200, description = "the stored keys, with the new active one", body = LlmKeysOut),
        (status = 400, description = "no key stored for that provider"),
    ),
    tag = "llm-keys"
)]
pub async fn select_llm_key(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<SelectProvider>,
) -> ApiResult<Json<LlmKeysOut>> {
    byok::set_preferred(&state, user.id, &body.provider).await?;
    list_llm_keys(State(state), AuthUser(user)).await
}
