//! Token-to-dollar price table and fee calculation.
//!
//! Prices are per million tokens (MTok) as of May 2026.  Update this table
//! when provider pricing changes — it directly affects the 1% fee we charge.

/// Prices for a specific model, in USD per million tokens.
struct ModelPrice {
    input_per_mtok: f64,
    output_per_mtok: f64,
    /// Anthropic cache-write rate (writing tokens to the prompt cache).
    cache_write_per_mtok: f64,
    /// Anthropic cache-read rate (serving tokens from the prompt cache).
    cache_read_per_mtok: f64,
}

fn lookup(provider: &str, model: &str) -> ModelPrice {
    match (provider, model) {
        // Anthropic Claude 4.x
        ("anthropic", m) if m.contains("opus-4") => ModelPrice {
            input_per_mtok: 15.0,
            output_per_mtok: 75.0,
            cache_write_per_mtok: 18.75,
            cache_read_per_mtok: 1.50,
        },
        ("anthropic", m) if m.contains("sonnet-4") => ModelPrice {
            input_per_mtok: 3.0,
            output_per_mtok: 15.0,
            cache_write_per_mtok: 3.75,
            cache_read_per_mtok: 0.30,
        },
        ("anthropic", m) if m.contains("haiku-4") => ModelPrice {
            input_per_mtok: 0.80,
            output_per_mtok: 4.0,
            cache_write_per_mtok: 1.0,
            cache_read_per_mtok: 0.08,
        },
        // OpenAI GPT-4o family
        ("openai", m) if m.starts_with("gpt-4o-mini") => ModelPrice {
            input_per_mtok: 0.15,
            output_per_mtok: 0.60,
            cache_write_per_mtok: 0.0,
            cache_read_per_mtok: 0.075,
        },
        ("openai", m) if m.starts_with("gpt-4o") => ModelPrice {
            input_per_mtok: 2.50,
            output_per_mtok: 10.0,
            cache_write_per_mtok: 0.0,
            cache_read_per_mtok: 1.25,
        },
        ("openai", m) if m.starts_with("o3") => ModelPrice {
            input_per_mtok: 10.0,
            output_per_mtok: 40.0,
            cache_write_per_mtok: 0.0,
            cache_read_per_mtok: 2.50,
        },
        // xAI Grok
        ("xai", m) if m.contains("grok-3") => ModelPrice {
            input_per_mtok: 3.0,
            output_per_mtok: 15.0,
            cache_write_per_mtok: 0.0,
            cache_read_per_mtok: 0.0,
        },
        // DeepSeek
        ("deepseek", _) => ModelPrice {
            input_per_mtok: 0.27,
            output_per_mtok: 1.10,
            cache_write_per_mtok: 0.0,
            cache_read_per_mtok: 0.07,
        },
        // Unknown: fall back to a conservative OpenAI GPT-4o estimate so we
        // never under-collect the fee.
        _ => ModelPrice {
            input_per_mtok: 2.50,
            output_per_mtok: 10.0,
            cache_write_per_mtok: 0.0,
            cache_read_per_mtok: 0.0,
        },
    }
}

/// Estimate the provider cost in USD for a single LLM API call.
pub fn estimate_cost_usd(
    provider: &str,
    model: &str,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_tokens: u64,
    cache_read_tokens: u64,
) -> f64 {
    let p = lookup(provider, model);
    let mtok = 1_000_000.0_f64;
    (input_tokens as f64) / mtok * p.input_per_mtok
        + (output_tokens as f64) / mtok * p.output_per_mtok
        + (cache_creation_tokens as f64) / mtok * p.cache_write_per_mtok
        + (cache_read_tokens as f64) / mtok * p.cache_read_per_mtok
}

/// Our 1% fee on top of the estimated provider cost.
pub const FEE_RATE: f64 = 0.01;

/// Create a Stripe customer.  Returns the Stripe customer ID (`cus_...`).
pub async fn stripe_create_customer(
    api_key: &str,
    github_login: &str,
    email: Option<&str>,
) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let mut params = vec![("metadata[github_login]", github_login.to_string())];
    if let Some(e) = email {
        params.push(("email", e.to_string()));
    }
    let resp = client
        .post("https://api.stripe.com/v1/customers")
        .basic_auth(api_key, Some(""))
        .form(&params)
        .send()
        .await?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().await?;
    if !status.is_success() {
        anyhow::bail!("Stripe error {}: {}", status, body);
    }
    let id = body["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing id in Stripe customer response"))?
        .to_string();
    Ok(id)
}

/// Report accumulated fee usage to Stripe's metered billing API.
///
/// `subscription_item_id` is the `si_...` ID of the metered price item on the
/// user's subscription.  This is a separate set-up step (see docs/specs/
/// telemetry-billing.md) not handled here.
///
/// `quantity` is in cents (rounded up) to avoid fractional units.
pub async fn stripe_report_usage(
    api_key: &str,
    subscription_item_id: &str,
    quantity_cents: u64,
) -> anyhow::Result<()> {
    if quantity_cents == 0 {
        return Ok(());
    }
    let client = reqwest::Client::new();
    let params = [
        ("quantity", quantity_cents.to_string()),
        ("action", "increment".to_string()),
    ];
    let resp = client
        .post(format!(
            "https://api.stripe.com/v1/subscription_items/{subscription_item_id}/usage_records"
        ))
        .basic_auth(api_key, Some(""))
        .form(&params)
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await?;
        anyhow::bail!("Stripe usage record error {}: {}", status, body);
    }
    Ok(())
}
