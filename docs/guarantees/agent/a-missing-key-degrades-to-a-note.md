# A Missing Key Degrades to a Note, Never a Crash

Given a local session (the desktop app's server) on a machine with no LLM
provider key in the environment, when the user sends a prompt to the in-app
agent, then `POST /api/docs/:id/agent` answers `503` with an error message
that **starts with** `agent not available` and goes on to name every
environment variable that would fix it — and nothing else about the session
degrades: documents still open, edit, weave, and run.

The leading phrase is a wire contract, not styling. ChatDock matches
`/agent not (configured|available)/i` and renders a quiet configuration note
instead of a red error, because "you have not set a key" is a fact about the
machine, not a failure of the request. Change the phrase and the client
together or the note becomes an alarm.

This is the `config-and-environments` rule made concrete: a credential is
never required for the tool to run. The agent is the one feature that cannot
exist without one, so it is the one feature that degrades — visibly, with the
exact variables to set — while everything offline keeps working.

---

Last LLM verification:
- Date: 2026-08-16
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/serve/agent.rs::start_turn` resolves the
  provider via `hickory_agent::resolve_selector_with_store(None, …)` (the
  Settings key store first, then the environment) before recording anything;
  both that failure and a `client_for_with_store` failure map to
  `ApiError::unavailable` with the `agent not available — ` prefix. The
  underlying resolution error names `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`,
  `DEEPSEEK_API_KEY`, `XAI_API_KEY`, and `OPENROUTER_API_KEY`
  (`crates/hickory-agent/src/provider.rs`). The client side of the contract
  is `apps/web/src/components/ChatDock.tsx` (the `unavailable` state).
- Test coverage: `crates/hickory-cli/tests/serve_agent.rs::
  no_key_degrades_to_the_note_never_a_crash` scrubs the provider variables,
  asserts the 503, the leading phrase, the named variable, and that
  `/api/health` still answers. The scripted-client tests in the same file
  prove the same route runs a real turn when a client is available.
