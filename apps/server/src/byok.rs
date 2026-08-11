//! Bring your own key: the accounts' own provider credentials, and the one
//! place that decides *whose* key an agent run spends.
//!
//! `plans.json` distinguishes two agent entitlements, and this module is what
//! makes that distinction real rather than decorative:
//!
//! - `byo_key` (Open, Pro) — the account runs the agent on a credential it
//!   supplies. There is no fallback to the deployment's key, deliberately: a
//!   plan sold as "bring your own key" that quietly bills the operator when
//!   the user has not brought one is a pricing bug that only shows up on the
//!   operator's invoice.
//! - `metered_allowance` (Team, Business) — the deployment's key, drawn
//!   against the plan's monthly LLM budget. An account on such a plan that has
//!   nonetheless stored its own key gets its own key used: storing one is an
//!   explicit act, and silently ignoring it would spend an allowance the user
//!   was trying not to spend.
//!
//! Keys are sealed by [`crate::keyvault`] and never leave this module in the
//! clear except into the client that is about to make the request.

use anyhow::Context as _;
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use hickory_agent::ProviderSelection;
use uuid::Uuid;

use crate::AppState;
use crate::auth::User;
use crate::error::{ApiError, ApiResult};
use crate::keyvault::{Sealed, last4};
use crate::plans;

/// One stored credential, as the UI sees it: enough to identify the key,
/// never enough to use it.
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct StoredKey {
    /// `anthropic` | `openai` | `deepseek` | `grok`.
    pub provider: String,
    /// Last four characters of the key.
    pub last4: String,
    /// Model override, or `None` for the provider's default.
    pub model: Option<String>,
    /// True when this is the key agent runs will use.
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct KeyRow {
    provider: String,
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
    key_version: i32,
    last4: String,
    model: Option<String>,
    created_at: DateTime<Utc>,
    last_used_at: Option<DateTime<Utc>>,
}

impl KeyRow {
    fn sealed(&self) -> Sealed {
        Sealed {
            ciphertext: self.ciphertext.clone(),
            nonce: self.nonce.clone(),
            key_version: self.key_version,
        }
    }
}

/// The 503 every BYOK path answers when the deployment cannot seal keys.
///
/// It names the variable and how to mint one, because the person reading it
/// is an operator who can fix it in one command — and because "unavailable"
/// with no reason is the least actionable message a status page can carry.
fn vault_unconfigured() -> ApiError {
    ApiError::service_unavailable(
        "this deployment cannot store API keys: KEY_ENCRYPTION_KEY is unset, so there is \
         nothing to encrypt them with. An operator can mint one with `just gen-key` and set \
         it on the server (see docs/operators/ENVIRONMENTS.md).",
    )
}

/// Which stored key an agent run should use, when several are installed.
///
/// With exactly one key there is nothing to choose and nothing is asked —
/// a confirmation prompt whose answer is forced is not a choice. A stored
/// preference wins when it names a key that still exists; a preference left
/// pointing at a deleted key is treated as absent rather than as an error,
/// because deleting a key is not an implicit request to break the agent.
fn select_provider<'a>(rows: &'a [KeyRow], preferred: Option<&str>) -> Option<&'a KeyRow> {
    if let Some(p) = preferred
        && let Some(row) = rows.iter().find(|r| r.provider == p)
    {
        return Some(row);
    }
    match rows {
        [only] => Some(only),
        _ => None,
    }
}

async fn preferred_provider(state: &AppState, user_id: Uuid) -> Result<Option<String>, ApiError> {
    Ok(
        sqlx::query_scalar("SELECT preferred_llm_provider FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&state.db)
            .await?
            .flatten(),
    )
}

async fn key_rows(state: &AppState, user_id: Uuid) -> Result<Vec<KeyRow>, ApiError> {
    Ok(sqlx::query_as::<_, KeyRow>(
        "SELECT provider, ciphertext, nonce, key_version, last4, model, created_at, last_used_at
         FROM user_llm_keys WHERE user_id = $1 ORDER BY provider",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?)
}

/// Every stored key for an account, with the active one marked.
pub async fn list_keys(state: &AppState, user_id: Uuid) -> Result<Vec<StoredKey>, ApiError> {
    let rows = key_rows(state, user_id).await?;
    let preferred = preferred_provider(state, user_id).await?;
    let active = select_provider(&rows, preferred.as_deref()).map(|r| r.provider.clone());
    Ok(rows
        .into_iter()
        .map(|r| StoredKey {
            active: Some(&r.provider) == active.as_ref(),
            provider: r.provider,
            last4: r.last4,
            model: r.model,
            created_at: r.created_at,
            last_used_at: r.last_used_at,
        })
        .collect())
}

/// Store (or replace) an account's key for one provider.
///
/// The key is validated against the vendor before it is stored: a typo caught
/// here is one error message in a settings form, and a typo caught later is a
/// failed agent run whose cause is three layers down in a streamed 401.
pub async fn save_key(
    state: &AppState,
    user_id: Uuid,
    provider: &str,
    api_key: &str,
    model: Option<&str>,
) -> ApiResult<StoredKey> {
    let vault = state
        .config
        .key_vault
        .as_ref()
        .ok_or_else(vault_unconfigured)?;

    let selection = ProviderSelection::parse(provider).ok_or_else(|| {
        ApiError::bad_request(format!(
            "unknown provider {provider:?} — expected one of: {}",
            ProviderSelection::ALL.join(", ")
        ))
    })?;
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(ApiError::bad_request(format!(
            "no key given for {}; paste the value from your {} account",
            selection.name(),
            selection.name()
        )));
    }

    let client = hickory_agent::client_for(selection.name(), model, Some(api_key))
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    if let Err(e) = client.validate_credentials().await {
        return Err(ApiError::bad_request(format!(
            "{} rejected this key: {e}. Check that it was copied whole, that it is a key and \
             not an organization id, and that the account it belongs to has API access enabled.",
            selection.name()
        )));
    }

    let sealed = vault
        .seal(user_id, selection.name(), api_key)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let last4 = last4(api_key);

    sqlx::query(
        "INSERT INTO user_llm_keys
             (user_id, provider, ciphertext, nonce, key_version, last4, model)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (user_id, provider) DO UPDATE SET
             ciphertext = EXCLUDED.ciphertext,
             nonce = EXCLUDED.nonce,
             key_version = EXCLUDED.key_version,
             last4 = EXCLUDED.last4,
             model = EXCLUDED.model,
             updated_at = now(),
             -- A replaced key is a new credential; its usage history belongs
             -- to the key that is gone.
             last_used_at = NULL",
    )
    .bind(user_id)
    .bind(selection.name())
    .bind(&sealed.ciphertext)
    .bind(&sealed.nonce)
    .bind(sealed.key_version)
    .bind(&last4)
    .bind(model)
    .execute(&state.db)
    .await?;

    let keys = list_keys(state, user_id).await?;
    keys.into_iter()
        .find(|k| k.provider == selection.name())
        .ok_or_else(|| ApiError::internal("stored key vanished immediately after being written"))
}

/// Forget an account's key for one provider.
pub async fn delete_key(state: &AppState, user_id: Uuid, provider: &str) -> ApiResult<()> {
    let selection = ProviderSelection::parse(provider)
        .ok_or_else(|| ApiError::bad_request(format!("unknown provider {provider:?}")))?;
    let deleted = sqlx::query("DELETE FROM user_llm_keys WHERE user_id = $1 AND provider = $2")
        .bind(user_id)
        .bind(selection.name())
        .execute(&state.db)
        .await?
        .rows_affected();
    if deleted == 0 {
        return Err(ApiError::not_found(format!(
            "no {} key is stored for this account",
            selection.name()
        )));
    }
    // Leave a preference pointing at the deleted provider in place: it costs
    // nothing (selection ignores a preference it cannot resolve) and it means
    // re-adding the same provider restores the previous choice.
    Ok(())
}

/// Choose which stored key agent runs use.
pub async fn set_preferred(state: &AppState, user_id: Uuid, provider: &str) -> ApiResult<()> {
    let selection = ProviderSelection::parse(provider)
        .ok_or_else(|| ApiError::bad_request(format!("unknown provider {provider:?}")))?;
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM user_llm_keys WHERE user_id = $1 AND provider = $2)",
    )
    .bind(user_id)
    .bind(selection.name())
    .fetch_one(&state.db)
    .await?;
    if !exists {
        return Err(ApiError::bad_request(format!(
            "no {} key is stored for this account — save one before selecting it",
            selection.name()
        )));
    }
    sqlx::query("UPDATE users SET preferred_llm_provider = $1 WHERE id = $2")
        .bind(selection.name())
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(())
}

/// The credential one agent run will spend, and whose it is.
#[derive(Debug, Clone)]
pub struct AgentCredential {
    pub provider: String,
    pub api_key: String,
    pub model: Option<String>,
    /// True when this is the account's own key rather than the deployment's.
    /// Runs on an account's own key are not drawn against a plan allowance.
    pub byo: bool,
}

/// Decide which credential an agent run for `user` should use.
///
/// Errors are the user's next action, not a diagnosis: which plan they are
/// on, what that plan expects, and where to fix it.
pub async fn resolve_agent_credential(state: &AppState, user: &User) -> ApiResult<AgentCredential> {
    let ents = plans::resolve(
        &state.catalog,
        &user.plan_key,
        user.price_key.as_deref(),
        &user.billing_status,
    );

    // A deployment with no vault cannot read stored keys even if rows exist,
    // so there is nothing to resolve against; fall through to the plan rules
    // with an empty set rather than reporting a decryption failure per key.
    let rows = match state.config.key_vault {
        Some(_) => key_rows(state, user.id).await?,
        None => Vec::new(),
    };
    let preferred = preferred_provider(state, user.id).await?;
    let chosen = select_provider(&rows, preferred.as_deref());

    if let Some(row) = chosen {
        let vault = state
            .config
            .key_vault
            .as_ref()
            .ok_or_else(vault_unconfigured)?;
        let api_key = vault
            .open(user.id, &row.provider, &row.sealed())
            .map_err(|e| ApiError::new(StatusCode::FAILED_DEPENDENCY, e.to_string()))?;
        let _ = sqlx::query(
            "UPDATE user_llm_keys SET last_used_at = now() WHERE user_id = $1 AND provider = $2",
        )
        .bind(user.id)
        .bind(&row.provider)
        .execute(&state.db)
        .await;
        return Ok(AgentCredential {
            provider: row.provider.clone(),
            api_key,
            model: row.model.clone(),
            byo: true,
        });
    }

    // Several keys, no preference: the account has to say which one. Naming
    // them is the difference between a question and a dead end.
    if rows.len() > 1 {
        let names: Vec<&str> = rows.iter().map(|r| r.provider.as_str()).collect();
        return Err(ApiError::bad_request(format!(
            "{} API keys are stored ({}) and none is selected — choose one in \
             Settings → API keys",
            rows.len(),
            names.join(", ")
        )));
    }

    match ents.agent.as_str() {
        "byo_key" => Err(ApiError::new(
            StatusCode::PAYMENT_REQUIRED,
            format!(
                "the {} plan runs the agent on your own provider key, and none is stored. \
                 Add one in Settings → API keys ({}), or upgrade to a plan with an included \
                 agent allowance.",
                ents.plan_key,
                ProviderSelection::ALL.join(" / ")
            ),
        )),
        _ => {
            let cfg = state.config.agent_llm.clone().ok_or_else(|| {
                let key_env = ProviderSelection::parse(&state.config.agent_provider)
                    .map(|s| s.key_env())
                    .unwrap_or("ANTHROPIC_API_KEY");
                ApiError::service_unavailable(format!(
                    "the included agent allowance is unavailable on this deployment \
                     ({key_env} unset for provider {}). Add your own key in \
                     Settings → API keys to run the agent meanwhile.",
                    state.config.agent_provider
                ))
            })?;
            Ok(AgentCredential {
                provider: cfg.provider,
                api_key: cfg.api_key,
                model: cfg.model,
                byo: false,
            })
        }
    }
}

/// Build the LLM client for a resolved credential.
pub fn client_for(
    cred: &AgentCredential,
) -> anyhow::Result<std::sync::Arc<dyn hickory_agent::LlmClient>> {
    hickory_agent::client_for(&cred.provider, cred.model.as_deref(), Some(&cred.api_key))
        .with_context(|| format!("building the {} client", cred.provider))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(provider: &str) -> KeyRow {
        KeyRow {
            provider: provider.to_string(),
            ciphertext: Vec::new(),
            nonce: Vec::new(),
            key_version: 1,
            last4: "0000".into(),
            model: None,
            created_at: Utc::now(),
            last_used_at: None,
        }
    }

    /// Guarantee: docs/guarantees/agent/byok-key-selection.md
    #[test]
    fn one_key_is_selected_without_being_asked_about() {
        let rows = vec![row("anthropic")];
        assert_eq!(select_provider(&rows, None).unwrap().provider, "anthropic");
    }

    #[test]
    fn several_keys_without_a_preference_select_nothing() {
        let rows = vec![row("anthropic"), row("grok")];
        assert!(select_provider(&rows, None).is_none());
    }

    #[test]
    fn a_preference_wins_and_a_stale_one_is_ignored() {
        let rows = vec![row("anthropic"), row("grok")];
        assert_eq!(
            select_provider(&rows, Some("grok")).unwrap().provider,
            "grok"
        );
        // Preference naming a provider whose key was deleted: not an error,
        // and not a silent wrong choice either.
        assert!(select_provider(&rows, Some("deepseek")).is_none());
        // …but with a single key left, that key still wins.
        let one = vec![row("anthropic")];
        assert_eq!(
            select_provider(&one, Some("deepseek")).unwrap().provider,
            "anthropic"
        );
    }

    #[test]
    fn no_keys_select_nothing() {
        assert!(select_provider(&[], None).is_none());
        assert!(select_provider(&[], Some("anthropic")).is_none());
    }
}
