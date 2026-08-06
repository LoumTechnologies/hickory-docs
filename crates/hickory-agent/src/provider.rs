//! One place that turns a provider selector into an [`LlmClient`].
//!
//! Every call site that wants a model — the CLI, the server's agent route,
//! `hickory refresh` — needs the same three decisions: which vendor, which
//! model, and is a key present. Duplicating that logic is how a provider
//! ends up working in one entry point and silently missing from another, so
//! it lives here and each call site passes through what the user asked for.
//!
//! The selector is `anthropic` (the default), `openai`, `deepseek`, or
//! `grok`/`xai`. Anthropic keeps its own client because it is the only one
//! with explicit prompt-cache breakpoints, `count_tokens`, and the Batch
//! API; the other three share [`OpenAiCompatClient`].

use std::sync::Arc;

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

    /// Every selector a user may pass, for error messages.
    pub const ALL: &'static [&'static str] = &["anthropic", "openai", "deepseek", "grok"];
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
    }

    #[test]
    fn an_unknown_selector_never_falls_back_to_the_default() {
        // Silently running on Anthropic because "opennai" was a typo would
        // bill the wrong account and write a session that names the wrong
        // model.
        assert_eq!(ProviderSelection::parse("opennai"), None);
        let err = err_of(client_for("opennai", None, None));
        assert!(err.contains("unknown provider"), "{err}");
        assert!(err.contains("deepseek"), "the error must list valid ones: {err}");
    }

    #[test]
    fn an_explicit_key_is_used_instead_of_the_environment() {
        // BYO-key callers hold the key; the environment may have none.
        unsafe { std::env::remove_var("XAI_API_KEY") };
        assert!(client_for("grok", None, Some("sk-explicit")).is_ok());
    }

    #[test]
    fn a_missing_key_is_reported_by_variable_name() {
        // Guarantee: the failure names what to set, before any request.
        unsafe { std::env::remove_var("DEEPSEEK_API_KEY") };
        let err = err_of(client_for("deepseek", None, None));
        assert!(err.contains("DEEPSEEK_API_KEY"), "{err}");
    }
}
