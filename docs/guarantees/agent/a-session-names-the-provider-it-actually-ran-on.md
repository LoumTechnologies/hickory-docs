# A Session Names the Provider It Actually Ran On

Given an agent run configured with a provider selector, when that selector is
not one this build supports, or the selected provider has no API key, then the
run refuses before the first request — naming the accepted selectors, or the
exact environment variable that is unset. It never falls back to a different
provider.

Four providers now share one entry point (`anthropic`, `openai`, `deepseek`,
`grok`/`xai`), which introduces a failure this product cannot tolerate: a
typo in a selector silently running on the default. That would bill the wrong
account, and — worse for a system whose whole claim is that a document records
what actually happened — write a `hick:session` whose `SessionStarted` names a
model that was never called. The session log is evidence; evidence that quietly
lies is worse than no log.

The key check is equally deliberate. An empty key is not caught by the client;
it is caught by the vendor, mid-stream, as a provider-specific 401 several
layers below the person who forgot to export it. Checking at construction turns
that into one line naming the variable to set.

Cost for the OpenAI-compatible providers is reported as *unknown*, not
estimated. `usage.rs` holds only verified prices and Anthropic's cache
multipliers; a guessed dollar figure in a spend report is worse than a blank
one. Token counts are still captured in full, including the four-way split.

---

Last LLM verification:
- Date: 2026-08-06
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-agent/src/provider.rs` — `ProviderSelection::parse`
  returns `None` for anything unrecognised and `client_for` turns that into an
  error listing `ProviderSelection::ALL`; the key is resolved from the explicit
  argument then the provider's `key_env()`, and an empty one bails by name.
  Both the CLI (`--provider` on `agent` and `refresh`) and the server
  (`HICKORY_LLM_PROVIDER` + `AgentLlmConfig`) route through this one function,
  so no entry point can grow its own rules.
- Test coverage: `provider.rs` unit tests —
  `an_unknown_selector_never_falls_back_to_the_default`,
  `a_missing_key_is_reported_by_variable_name`, and
  `an_explicit_key_is_used_instead_of_the_environment`. Driven end-to-end
  against a local recording chat-completions endpoint: `hickory agent
  --provider deepseek` and `--provider openai` each reported their own model,
  and the recorded requests confirm the per-provider output-cap field
  (`max_tokens` vs `max_completion_tokens`) that a shared client would
  otherwise get wrong.
