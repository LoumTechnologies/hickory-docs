---
id: TASK-23.2
title: 'Token cost model: Anthropic pricing tables and cost computation crate'
status: To Do
assignee: []
created_date: '2026-05-16 12:12'
labels:
  - evaluation
  - cost-model
milestone: m-2
dependencies: []
parent_task_id: TASK-23
priority: high
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement a Rust crate that knows current Anthropic per-token pricing and computes true dollar cost from a token usage breakdown. Cost is a first-class output of every eval run.

## Why this is non-trivial

Not all tokens cost the same. A single agent run may produce:
- **Cache-write input tokens** (most expensive input tier — you pay to populate the cache)
- **Cache-read input tokens** (cheapest — heavily discounted vs uncached)
- **Uncached input tokens** (standard input price)
- **Output tokens** (standard output price, typically higher than input)
- **Thinking tokens** (extended-thinking mode; priced as output tokens on most models)

Pricing differs by model family:
- `claude-opus-4-7`
- `claude-sonnet-4-6`
- `claude-haiku-4-5`

And may differ by API tier or batch mode in the future.

## Deliverable

A crate at `hick-agent/crates/hick-eval-cost/` (or similar) exposing:

```rust
pub struct TokenUsage {
    pub model: String,
    pub cache_write_input: u64,
    pub cache_read_input: u64,
    pub uncached_input: u64,
    pub output: u64,
    pub thinking: u64,
}

pub struct CostBreakdown {
    pub cache_write_usd: f64,
    pub cache_read_usd: f64,
    pub uncached_input_usd: f64,
    pub output_usd: f64,
    pub thinking_usd: f64,
    pub total_usd: f64,
}

pub fn compute_cost(usage: &TokenUsage) -> Result<CostBreakdown>;
```

Pricing tables must be easy to update (a single const table, not scattered literals). Include the date the prices were last verified in a comment.

## Source of token usage data

The agent SDK (`hick-agent-sdk`) receives API responses that include usage metadata. TASK-23.3 (eval runner) will extract these and pass them here. This crate does not call the API itself.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Crate compiles and is reachable from the eval runner workspace.
- [ ] #2 Unit tests cover all five token categories for each supported model, with expected USD values hard-coded from the current published price sheet.
- [ ] #3 Pricing table is in one place and has a last-verified date comment.
- [ ] #4 compute_cost returns Err for unknown model strings rather than silently returning zero.
- [ ] #5 CostBreakdown serialises to XML via serde (for inclusion in eval result files).
<!-- AC:END -->
