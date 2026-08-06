//! Token usage accounting and USD cost computation.
//!
//! Every LLM call reports [`Usage`] split four ways — `input_tokens`,
//! `cache_creation_input_tokens`, `cache_read_input_tokens`,
//! `output_tokens` — because the four are billed at different rates. A run
//! that only reports "total tokens" is not a result (see
//! `docs/specs/freeform/token-economics.md`).
//!
//! Prices come from a small static table; unknown models cost `None`
//! rather than a guessed number.

use serde::{Deserialize, Serialize};

/// Cache-write multiplier at the default 5-minute TTL. (The 1-hour TTL
/// costs 2.0x; we only ever request the default TTL.)
pub const CACHE_WRITE_MULTIPLIER_5M: f64 = 1.25;

/// Cache-read multiplier (approximately 0.1x the input rate).
pub const CACHE_READ_MULTIPLIER: f64 = 0.1;

/// Token usage for one LLM call (or an accumulated total), split the four
/// ways the API bills them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    /// Uncached input tokens (full input price).
    #[serde(default)]
    pub input_tokens: u64,
    /// Tokens written to the prompt cache (1.25x input price at 5m TTL).
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    /// Tokens read from the prompt cache (~0.1x input price).
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    /// Output tokens (output price).
    #[serde(default)]
    pub output_tokens: u64,
}

impl Usage {
    /// Accumulate another usage record into this one.
    pub fn add(&mut self, other: &Usage) {
        self.input_tokens += other.input_tokens;
        self.cache_creation_input_tokens += other.cache_creation_input_tokens;
        self.cache_read_input_tokens += other.cache_read_input_tokens;
        self.output_tokens += other.output_tokens;
    }

    /// Whether every counter is zero (e.g. a scripted client with no usage
    /// attached).
    pub fn is_zero(&self) -> bool {
        *self == Usage::default()
    }
}

/// Per-MTok price for one model.
#[derive(Debug, Clone, Copy)]
pub struct ModelPrice {
    /// USD per million input tokens.
    pub input_per_mtok: f64,
    /// USD per million output tokens.
    pub output_per_mtok: f64,
}

/// Look up the price for a model id. Matches on the alias stem so dated
/// snapshots (`claude-sonnet-5-20250929`) price like their alias.
///
/// List prices (USD/MTok). Note: claude-sonnet-5 has an intro rate of
/// $2/$10 through 2026-08-31; we price at the list rate $3/$15 so reports
/// never *understate* cost.
pub fn price_for_model(model: &str) -> Option<ModelPrice> {
    const TABLE: &[(&str, ModelPrice)] = &[
        (
            "claude-sonnet-5",
            ModelPrice {
                input_per_mtok: 3.0,
                output_per_mtok: 15.0,
            },
        ),
        (
            "claude-haiku-4-5",
            ModelPrice {
                input_per_mtok: 1.0,
                output_per_mtok: 5.0,
            },
        ),
        (
            "claude-opus-5",
            ModelPrice {
                input_per_mtok: 5.0,
                output_per_mtok: 25.0,
            },
        ),
    ];
    TABLE
        .iter()
        .find(|(stem, _)| model.starts_with(stem))
        .map(|(_, p)| *p)
}

/// Compute the USD cost of `usage` on `model`, with cache multipliers
/// applied correctly: creation 1.25x (5m TTL), read ~0.1x, both against the
/// *input* rate. Returns `None` for models not in the price table — an
/// unknown price is reported as unknown, never guessed.
pub fn cost_usd(model: &str, usage: &Usage) -> Option<f64> {
    let p = price_for_model(model)?;
    let mtok = 1_000_000.0;
    Some(
        usage.input_tokens as f64 * p.input_per_mtok / mtok
            + usage.cache_creation_input_tokens as f64
                * CACHE_WRITE_MULTIPLIER_5M
                * p.input_per_mtok
                / mtok
            + usage.cache_read_input_tokens as f64 * CACHE_READ_MULTIPLIER * p.input_per_mtok
                / mtok
            + usage.output_tokens as f64 * p.output_per_mtok / mtok,
    )
}

/// Minimum cacheable prefix (tokens) for a model: below this the API
/// silently does not cache. Sonnet-class models need 1024; Opus 5 and
/// Fable 5 need 512. Unknown models get the conservative 1024.
pub fn min_cacheable_prefix_tokens(model: &str) -> u64 {
    if model.starts_with("claude-opus-5") || model.starts_with("claude-fable-5") {
        512
    } else {
        1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_way_split_prices_correctly() {
        let usage = Usage {
            input_tokens: 1_000_000,
            cache_creation_input_tokens: 1_000_000,
            cache_read_input_tokens: 1_000_000,
            output_tokens: 1_000_000,
        };
        // sonnet-5: 3 + 3*1.25 + 3*0.1 + 15 = 22.05
        let cost = cost_usd("claude-sonnet-5", &usage).unwrap();
        assert!((cost - 22.05).abs() < 1e-9, "got {cost}");
    }

    #[test]
    fn dated_snapshot_prices_like_alias() {
        assert!(price_for_model("claude-haiku-4-5-20251001").is_some());
        assert!(price_for_model("claude-opus-5-1").is_some());
    }

    #[test]
    fn unknown_model_is_none_not_a_guess() {
        assert!(cost_usd("gpt-oops", &Usage::default()).is_none());
    }

    #[test]
    fn cache_minimums_per_model() {
        assert_eq!(min_cacheable_prefix_tokens("claude-sonnet-5"), 1024);
        assert_eq!(min_cacheable_prefix_tokens("claude-opus-5"), 512);
        assert_eq!(min_cacheable_prefix_tokens("claude-fable-5"), 512);
        assert_eq!(min_cacheable_prefix_tokens("mystery"), 1024);
    }

    #[test]
    fn usage_accumulates() {
        let mut total = Usage::default();
        total.add(&Usage {
            input_tokens: 10,
            cache_creation_input_tokens: 5,
            cache_read_input_tokens: 2,
            output_tokens: 1,
        });
        total.add(&Usage {
            input_tokens: 1,
            ..Default::default()
        });
        assert_eq!(total.input_tokens, 11);
        assert_eq!(total.output_tokens, 1);
        assert!(!total.is_zero());
    }
}
