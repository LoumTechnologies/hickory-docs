# The Dock Reports Spend and Runs the Chosen Model

Given a document's chat dock, when the user picks a provider and/or types a
model id and sends a prompt, then `POST /api/docs/:id/agent` runs that turn
on exactly that provider and model (an unknown provider selector is refused
with `400` naming the valid set — it never silently runs on another vendor),
the choice persists per document for the session and applies to subsequent
turns, and each turn records the provider and model it actually ran on so a
mid-conversation change stays visible per turn.

And when the dock lists the conversation, then
`GET /api/docs/:id/agent/turns` reports the current choice with defaults
resolved (`provider`/`model`, e.g. `anthropic` / `claude-sonnet-5`) plus
session totals — `totals: {usd, input, output, cache_read, cache_write}` —
summed across the document's finished turns with each turn priced on its own
model. `usd` is `null` whenever any usage-bearing turn ran on a model with no
known price: an unknown price is reported as unknown, never understated.

The dock renders the totals as a one-line header summary
(`$0.0342 · in 12.4k · out 3.1k · cache 78%`, tooltip carrying the full
four-way breakdown) where the cache hit rate is
`cache_read / (input + cache_read)` and shows a dash before any turn has
reported usage.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/serve/agent.rs` — `AgentRequest`
  carries optional `provider`/`model`; `start_turn` validates the selector
  against `ProviderSelection::parse`, folds the fields into the hub's
  per-doc `DocSelection` (`AgentHub::update_selection`; absent keeps,
  empty clears), and passes them through
  `resolve_selector_with_store`/`client_for_with_store`. `TurnRecord`
  stores `provider`, `model`, and the final `Usage`
  (`AgentHub::finish` from `AgentOutcome::total_usage`); `totals_of` sums
  usage and prices each turn with `hickory_agent::cost_usd` on that
  turn's model; `list_turns` returns `provider`, `model` (defaults via
  `default_model_for`), and `totals`.
- Tests: `crates/hickory-cli/tests/serve_agent.rs::
  model_choice_is_accepted_persisted_and_priced_in_totals` and
  `an_untouched_document_lists_resolved_defaults_and_zero_totals`
  (scripted client, no tokens spent); unit tests in
  `serve/agent.rs::tests` (`totals_sum_usage_and_price_each_turn_on_its_own_model`,
  `an_unpriced_model_with_real_usage_makes_the_total_unknown_not_understated`,
  `no_turns_total_to_zero_usage_and_zero_cost`). Web:
  `apps/web/src/components/ChatDock.test.tsx` covers the formatters
  (`formatTokens`, `formatUsd`, `cacheHitRate`, `statsLine`), the stats
  line + tooltip rendering, its absence before usage, and that the model
  control posts the chosen provider/model.
- Caveats: the per-provider default-model list in the dock
  (`ChatDock.tsx::PROVIDERS`) mirrors the Rust constants
  (`DEFAULT_ANTHROPIC_MODEL`, `Provider::default_model`) by hand; keep
  them in sync when a default changes.
