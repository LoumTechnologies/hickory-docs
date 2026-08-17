//! One place that turns a provider selector into an [`LlmClient`].
//!
//! Every call site that wants a model — the CLI, the server's agent route,
//! `hick refresh` — needs the same three decisions: which vendor, which
//! model, and is a key present. Duplicating that logic is how a provider
//! ends up working in one entry point and silently missing from another, so
//! it lives here and each call site passes through what the user asked for.
//!
//! The selector is `anthropic` (the default), `openai`, `deepseek`,
//! `grok`/`xai`, or `openrouter`. Anthropic keeps its own client because it
//! is the only one with explicit prompt-cache breakpoints, `count_tokens`,
//! and the Batch API; the others share [`OpenAiCompatClient`].
//!
//! Keys come from two places, in a fixed order: the [`KeyStore`] the desktop
//! app's Settings page writes (see `key_store.rs`), then the provider's
//! environment variable. The `_with_store` variants consult the store FIRST
//! and fall through to the environment — one precedence rule, implemented
//! once, shared by selection and client construction alike.

use std::sync::Arc;

use crate::key_store::KeyStore;
use crate::llm::LlmClient;
use crate::llm_anthropic::AnthropicClient;
use crate::llm_openai::{OpenAiCompatClient, Provider};

/// A parsed provider selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderSelection {
    Anthropic,
    /// An OpenAI-compatible vendor.
    Compatible(Provider),
}

impl ProviderSelection {
    /// Parse a selector string. `None` for anything unrecognised — callers
    /// report the valid set rather than silently falling back, because a
    /// typo that silently ran on the default provider would bill the wrong
    /// account and produce a session that lies about its model.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "anthropic" | "claude" => Some(Self::Anthropic),
            other => Provider::parse(other).map(Self::Compatible),
        }
    }

    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::Compatible(p) => p.name(),
        }
    }

    /// The environment variable holding this provider's API key.
    pub fn key_env(self) -> &'static str {
        match self {
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::Compatible(p) => p.key_env(),
        }
    }

    /// Human-readable vendor name, for the desktop app's Settings page.
    pub fn label(self) -> &'static str {
        match self {
            Self::Anthropic => "Anthropic",
            Self::Compatible(Provider::OpenAi) => "OpenAI",
            Self::Compatible(Provider::DeepSeek) => "DeepSeek",
            Self::Compatible(Provider::XAi) => "xAI (Grok)",
            Self::Compatible(Provider::OpenRouter) => "OpenRouter",
        }
    }

    /// Every selector a user may pass, for error messages.
    pub const ALL: &'static [&'static str] =
        &["anthropic", "openai", "deepseek", "grok", "openrouter"];

    /// Every provider, in the order `ALL` names them. This is the list the
    /// key store and the Settings page enumerate — one canonical order so a
    /// provider added here shows up everywhere at once.
    pub fn all() -> [ProviderSelection; 5] {
        [
            Self::Anthropic,
            Self::Compatible(Provider::OpenAi),
            Self::Compatible(Provider::DeepSeek),
            Self::Compatible(Provider::XAi),
            Self::Compatible(Provider::OpenRouter),
        ]
    }
}

/// Resolve which provider to use when the user did not name one.
///
/// Precedence:
/// 1. An explicit selector (`--provider`), verbatim.
/// 2. `HICKORY_LLM_PROVIDER`, the documented environment override.
/// 3. The keys actually present in the environment:
///    - exactly one provider has a key → that provider, silently. A machine
///      with only `OPENAI_API_KEY` set has already said which vendor it uses;
///      failing with "anthropic needs a key" would ignore the obvious answer.
///    - several have keys, one of them Anthropic → `anthropic`, the
///      long-standing default.
///    - several have keys, none of them Anthropic → an error asking for
///      `--provider`: guessing here would bill an account the user did not
///      choose and write a session that names a model they did not pick.
///    - none → an error naming every variable that would have worked.
pub fn resolve_selector(explicit: Option<&str>) -> anyhow::Result<String> {
    resolve_selector_with(explicit, |var| std::env::var(var).ok())
}

/// [`resolve_selector`], with a [`KeyStore`] consulted before the
/// environment.
///
/// The precedence chain is otherwise identical — same auto-selection, same
/// errors — because the store is just another place a key can be "present":
/// a key entered in the desktop app's Settings page counts exactly as its
/// environment variable would, and wins over it when both exist.
pub fn resolve_selector_with_store(
    explicit: Option<&str>,
    store: &KeyStore,
) -> anyhow::Result<String> {
    resolve_selector_with(explicit, |var| {
        store.key_for_env(var).or_else(|| std::env::var(var).ok())
    })
}

fn resolve_selector_with(
    explicit: Option<&str>,
    lookup: impl Fn(&str) -> Option<String>,
) -> anyhow::Result<String> {
    if let Some(s) = explicit {
        return Ok(s.to_string());
    }
    if let Some(v) = lookup("HICKORY_LLM_PROVIDER").filter(|v| !v.trim().is_empty()) {
        return Ok(v);
    }
    let keyed: Vec<&str> = ProviderSelection::ALL
        .iter()
        .copied()
        .filter(|name| {
            ProviderSelection::parse(name)
                .and_then(|sel| lookup(sel.key_env()))
                .is_some_and(|k| !k.trim().is_empty())
        })
        .collect();
    match keyed.as_slice() {
        [only] => Ok((*only).to_string()),
        [] => anyhow::bail!(
            "no LLM provider key found in the environment.\n  \
             Set one of: ANTHROPIC_API_KEY, OPENAI_API_KEY, DEEPSEEK_API_KEY, XAI_API_KEY, \
             OPENROUTER_API_KEY —\n  \
             or pass --provider / set HICKORY_LLM_PROVIDER to name the vendor explicitly."
        ),
        many if many.contains(&"anthropic") => Ok("anthropic".to_string()),
        many => anyhow::bail!(
            "keys for several providers are set ({}) and none of them is the default \
             (anthropic).\n  \
             Pass --provider {} (or set HICKORY_LLM_PROVIDER) to choose — guessing would \
             bill an account you did not pick.",
            many.join(", "),
            many[0]
        ),
    }
}

/// Build the client for `selector`, optionally overriding the model and the
/// API key.
///
/// `api_key` is for callers that hold the key themselves — the server's
/// agent route, and BYO-key plans that construct one client per request.
/// Pass `None` to take it from the provider's environment variable.
///
/// Returns an error naming the accepted selectors for an unknown one, and
/// an error naming the missing environment variable when no key is
/// available — a run that reaches the API with an empty key fails deep
/// inside a stream with a vendor-specific 401, which is a much worse place
/// to learn it.
pub fn client_for(
    selector: &str,
    model: Option<&str>,
    api_key: Option<&str>,
) -> anyhow::Result<Arc<dyn LlmClient>> {
    let selection = ProviderSelection::parse(selector).ok_or_else(|| {
        anyhow::anyhow!(
            "unknown provider {selector:?}; expected one of: {}",
            ProviderSelection::ALL.join(", ")
        )
    })?;

    let key = match api_key.map(str::to_string) {
        Some(k) => Some(k),
        None => std::env::var(selection.key_env()).ok(),
    }
    .filter(|k| !k.trim().is_empty());
    let Some(key) = key else {
        anyhow::bail!(
            "provider {} needs an API key in {}",
            selection.name(),
            selection.key_env()
        );
    };

    Ok(match selection {
        ProviderSelection::Anthropic => {
            let mut c = AnthropicClient::new().with_api_key(key);
            if let Some(m) = model {
                c = c.with_model(m);
            }
            Arc::new(c)
        }
        ProviderSelection::Compatible(p) => {
            let mut c = OpenAiCompatClient::new(p).with_api_key(key);
            if let Some(m) = model {
                c = c.with_model(m);
            }
            Arc::new(c)
        }
    })
}

/// [`client_for`], with a [`KeyStore`] consulted before the environment.
///
/// A stored key for this provider becomes the explicit `api_key`, so it wins
/// over the environment variable through the exact precedence [`client_for`]
/// already implements; with nothing stored the behavior is byte-for-byte
/// `client_for(selector, model, None)`.
pub fn client_for_with_store(
    selector: &str,
    model: Option<&str>,
    store: &KeyStore,
) -> anyhow::Result<Arc<dyn LlmClient>> {
    client_for(selector, model, store.key_for(selector).as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Arc<dyn LlmClient>` is not `Debug`, so `unwrap_err` is unavailable.
    fn err_of(r: anyhow::Result<Arc<dyn LlmClient>>) -> String {
        match r {
            Ok(_) => panic!("expected an error"),
            Err(e) => e.to_string(),
        }
    }

    #[test]
    fn selectors_parse_including_the_familiar_aliases() {
        assert_eq!(
            ProviderSelection::parse("anthropic"),
            Some(ProviderSelection::Anthropic)
        );
        assert_eq!(
            ProviderSelection::parse("claude"),
            Some(ProviderSelection::Anthropic)
        );
        assert_eq!(
            ProviderSelection::parse("grok"),
            Some(ProviderSelection::Compatible(Provider::XAi))
        );
        assert_eq!(
            ProviderSelection::parse("deepseek"),
            Some(ProviderSelection::Compatible(Provider::DeepSeek))
        );
        assert_eq!(
            ProviderSelection::parse("openrouter"),
            Some(ProviderSelection::Compatible(Provider::OpenRouter))
        );
    }

    #[test]
    fn an_unknown_selector_never_falls_back_to_the_default() {
        // Silently running on Anthropic because "opennai" was a typo would
        // bill the wrong account and write a session that names the wrong
        // model.
        assert_eq!(ProviderSelection::parse("opennai"), None);
        let err = err_of(client_for("opennai", None, None));
        assert!(err.contains("unknown provider"), "{err}");
        assert!(
            err.contains("deepseek"),
            "the error must list valid ones: {err}"
        );
    }

    #[test]
    fn an_explicit_key_is_used_instead_of_the_environment() {
        // BYO-key callers hold the key; the environment may have none.
        unsafe { std::env::remove_var("XAI_API_KEY") };
        assert!(client_for("grok", None, Some("sk-explicit")).is_ok());
    }

    fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let pairs: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |var| pairs.iter().find(|(k, _)| k == var).map(|(_, v)| v.clone())
    }

    #[test]
    fn a_single_key_selects_its_provider_without_asking() {
        // The one key present has already named the vendor; asking (or
        // failing with "anthropic needs a key") would ignore that.
        let sel = resolve_selector_with(None, env_of(&[("OPENAI_API_KEY", "sk-x")])).unwrap();
        assert_eq!(sel, "openai");
    }

    #[test]
    fn an_explicit_selector_wins_over_everything() {
        let sel = resolve_selector_with(
            Some("deepseek"),
            env_of(&[
                ("ANTHROPIC_API_KEY", "sk-a"),
                ("HICKORY_LLM_PROVIDER", "grok"),
            ]),
        )
        .unwrap();
        assert_eq!(sel, "deepseek");
    }

    #[test]
    fn the_documented_env_override_is_honoured() {
        let sel = resolve_selector_with(
            None,
            env_of(&[
                ("HICKORY_LLM_PROVIDER", "grok"),
                ("ANTHROPIC_API_KEY", "sk-a"),
            ]),
        )
        .unwrap();
        assert_eq!(sel, "grok");
    }

    #[test]
    fn several_keys_including_anthropic_keep_the_default() {
        let sel = resolve_selector_with(
            None,
            env_of(&[("ANTHROPIC_API_KEY", "sk-a"), ("OPENAI_API_KEY", "sk-o")]),
        )
        .unwrap();
        assert_eq!(sel, "anthropic");
    }

    #[test]
    fn several_non_default_keys_ask_rather_than_guess() {
        let err = resolve_selector_with(
            None,
            env_of(&[("OPENAI_API_KEY", "sk-o"), ("DEEPSEEK_API_KEY", "sk-d")]),
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("--provider"), "{err}");
        assert!(err.contains("openai") && err.contains("deepseek"), "{err}");
    }

    #[test]
    fn no_keys_at_all_names_every_variable_that_would_work() {
        let err = resolve_selector_with(None, env_of(&[]))
            .unwrap_err()
            .to_string();
        for var in [
            "ANTHROPIC_API_KEY",
            "OPENAI_API_KEY",
            "DEEPSEEK_API_KEY",
            "XAI_API_KEY",
            "OPENROUTER_API_KEY",
        ] {
            assert!(err.contains(var), "{err}");
        }
    }

    /// The lookup `resolve_selector_with_store` composes: store first, env
    /// second. Tested against a fake environment so it cannot race the
    /// process-global one.
    fn store_then(
        store: KeyStore,
        env: &[(&str, &str)],
    ) -> impl Fn(&str) -> Option<String> + use<> {
        let pairs: Vec<(String, String)> = env
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |var| {
            store
                .key_for_env(var)
                .or_else(|| pairs.iter().find(|(k, _)| k == var).map(|(_, v)| v.clone()))
        }
    }

    #[test]
    fn a_single_stored_key_auto_selects_its_provider() {
        // A key entered in Settings counts as "present" exactly as its
        // environment variable would — with nothing in the environment at
        // all, the stored key alone picks the vendor.
        let mut store = KeyStore::default();
        store
            .set("openrouter", Some("sk-or-stored".into()))
            .unwrap();
        let sel = resolve_selector_with(None, store_then(store, &[])).unwrap();
        assert_eq!(sel, "openrouter");
    }

    #[test]
    fn a_stored_key_wins_over_the_environment_for_the_same_provider() {
        let mut store = KeyStore::default();
        store.set("openai", Some("sk-stored".into())).unwrap();
        let lookup = store_then(store, &[("OPENAI_API_KEY", "sk-env")]);
        assert_eq!(lookup("OPENAI_API_KEY").as_deref(), Some("sk-stored"));
        // And selection still lands on that provider.
        assert_eq!(resolve_selector_with(None, lookup).unwrap(), "openai");
    }

    #[test]
    fn a_stored_key_builds_a_client_with_no_environment_at_all() {
        // The desktop app's promise: keys live in the Settings store, not in
        // environment variables.
        unsafe { std::env::remove_var("OPENROUTER_API_KEY") };
        let mut store = KeyStore::default();
        store
            .set("openrouter", Some("sk-or-stored".into()))
            .unwrap();
        assert!(client_for_with_store("openrouter", None, &store).is_ok());

        // And an empty store degrades to exactly the env-only behavior.
        let err = err_of(client_for_with_store(
            "openrouter",
            None,
            &KeyStore::default(),
        ));
        assert!(err.contains("OPENROUTER_API_KEY"), "{err}");
    }

    #[test]
    fn a_missing_key_is_reported_by_variable_name() {
        // Guarantee: the failure names what to set, before any request.
        unsafe { std::env::remove_var("DEEPSEEK_API_KEY") };
        let err = err_of(client_for("deepseek", None, None));
        assert!(err.contains("DEEPSEEK_API_KEY"), "{err}");
    }
}
