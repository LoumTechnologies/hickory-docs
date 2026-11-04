<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Resolve native subscription billing before API clients

For maintainers preserving billing and tool boundaries while adding native subscription transports.

## Keep billing explicit through selection and restart

This registry centralizes subscription identity so selection validation, durable restoration and cost accounting agree. Only an explicit API provider selection can leave a retained subscription. Unsupported predictor limits fail before any API client is constructed.

<storycode:copy id="subscription-v2-069ce56f7f-0">
//! Subscription selectors are resolved before API-key factories and remain sticky across restarts.
use super::{
    LocalState,
    agent::{DocSelection, TurnRecord},
    api::{ApiError, ApiResult},
};
use hickory_agent::{LlmClient, Usage, cost_usd};
use std::sync::Arc;
pub fn is_subscription(provider: &str) -> bool {
    matches!(provider, "codex-subscription" | "grok-subscription")
}
pub fn default_model(provider: &str) -> Option<&'static str> {
    match provider {
        super::codex_subscription::SELECTOR => Some(super::codex_subscription::DEFAULT_MODEL),
        super::grok_subscription::SELECTOR => Some(super::grok_subscription::DEFAULT_MODEL),
        _ => None,
    }
}
pub fn retained_choice(turns: &[TurnRecord]) -> Option<DocSelection> {
    let turn = turns
        .iter()
        .filter(|t| t.backend == "python")
        .max_by(|a, b| a.created_at.cmp(&b.created_at))?;
    is_subscription(&turn.provider).then(|| DocSelection {
        provider: Some(turn.provider.clone()),
        model: Some(turn.model.clone()).filter(|m| !m.is_empty()),
    })
}
pub fn select_client(
    state: &LocalState,
    choice: &DocSelection,
    limit: Option<u32>,
) -> ApiResult<(Arc<dyn LlmClient>, String, String)> {
    if limit.is_some() {
        return Err(ApiError::unavailable(
            "Subscription inference is unavailable for private predictions; no fallback was attempted.",
        ));
    }
    let client: Arc<dyn LlmClient> = match choice.provider.as_deref() {
        Some(super::codex_subscription::SELECTOR) => {
            return super::codex_subscription::select_client(state, choice, limit);
        }
        Some(super::grok_subscription::SELECTOR) => Arc::new(
            super::grok_subscription::Client::new(state, choice.model.as_deref())
                .map_err(|e| ApiError::unavailable(format!("{e:#}")))?,
        ),
        _ => {
            return Err(ApiError::unavailable(
                "Unknown subscription backend; choose a provider explicitly",
            ));
        }
    };
    Ok((
        client.clone(),
        client.provider_name().into(),
        client.model_name().into(),
    ))
}
pub fn totals_of(turns: &[TurnRecord]) -> (Usage, Option<f64>) {
    let mut total = Usage::default();
    let mut usd = 0.0;
    let mut known = !turns.iter().any(|t| {
        t.provider.starts_with("acp:") || super::subscription::is_subscription(&t.provider)
    });
    for turn in turns {
        let Some(usage) = &turn.usage else { continue };
        total.add(usage);
        match cost_usd(&turn.model, usage) {
            Some(cost) => usd += cost,
            None if usage.is_zero() => {}
            None => known = false,
        }
    }
    (total, known.then_some(usd))
}
</storycode:copy>

<storycode:file path="crates/hickory-cli/src/serve/subscription.rs"><storycode:paste select="#subscription-v2-069ce56f7f-0"/></storycode:file>
</storycode:doc>
