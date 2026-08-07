# An Empty Response Never Kills a Session

Given a model that returns an empty or whitespace-only response, when the
agent loop re-prompts it, then the correction turn succeeds: the blank turn is
recorded as `(no output)` rather than replayed verbatim, and no request built
by either client ever carries an empty content block.

This was found live, not by reasoning. An agent twelve edits into propagating
a decision through a document chain returned one empty turn. The loop pushed
that response into the history and asked for a correction; Anthropic answered
`400 — messages: text content blocks must be non-empty`; the session died and
returned an error. Every edit it had made was left half-applied. The recovery
mechanism destroyed the run it existed to rescue.

The fix is in two places on purpose. The loop no longer replays a blank
response, because "the model said nothing" is more useful to the model than a
blank turn and is the honest record. And both clients drop empty content when
building a request, because this must not depend on every future caller
remembering: an empty block carries no information and is rejected by every
provider, so dropping it is strictly better than failing the request.

The half-applied state itself was handled correctly by the rest of the system
— `hickory check` reported three stale documents immediately, and the chain
could be finished by a second session. That is the intended failure mode. The
crash was not.

---

Last LLM verification:
- Date: 2026-08-07
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-agent/src/react_loop.rs` — the `Turn::Invalid`
  branch substitutes `(no output)` for a whitespace-only response before
  pushing it to the history. `llm_anthropic.rs` `build_request` skips messages
  whose content trims to empty; `llm_openai.rs` `request_body` filters the
  same way.
- Test coverage: `crates/hickory-agent/tests/turn_budget.rs` —
  `an_empty_response_is_corrected_not_replayed` drives two consecutive blank
  turns through a scripted model and asserts the run recovers and completes;
  `neither_client_ever_sends_an_empty_content_block` inspects the serialized
  request bytes of both clients and asserts empty messages are absent and the
  surviving count is exact.
