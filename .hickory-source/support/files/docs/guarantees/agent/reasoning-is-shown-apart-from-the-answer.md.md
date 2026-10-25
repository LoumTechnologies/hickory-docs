<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/agent/reasoning-is-shown-apart-from-the-answer.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `76fa2558fd06dc1186d6598de0c1efb298060edccdb75685ff77f5c5fa37c694`.

<book:entry path="docs/guarantees/agent/reasoning-is-shown-apart-from-the-answer.md">
# The Model's Reasoning Is Shown Apart From Its Answer, Folded

> 2026-10-18 amendment: the built-in loop is Python, not Rust. Branch data and
> evidence persist through the existing session/UI; Python constructs model
> context. Hosted completion requires a current accepted changeset and reports
> runtime checks unrun. Old Rust-runtime evidence below is historical. See
> [the migration audit](../audits/2026-10-18-python-agent-ownership.md).


Given a provider that streams the model's reasoning (Anthropic `thinking_delta`;
OpenAI-compatible `reasoning` / `reasoning_content`, as OpenRouter, DeepSeek,
and Gab AI send it), when the agent runs, then the reasoning is streamed as
its own event (`AgentEvent::Reasoning`), recorded in the session as
`<hick:reasoning>` — raw content, the first child of the assistant element —
and drawn folded (a `reasoning` disclosure) in the dock while streaming and
in the session view afterwards; and it is never part of the answer, so the
`<hick:next>` protocol and the recorded prose are untouched by it.

The reason: a model's reasoning is useful to read and dangerous to fold into
what it said — a protocol tag inside a thought would be parsed as a decision.
Kept apart it is a thing you can expand when you ask "why did it do that?",
and a thing the session preserves.

---

Last LLM verification:
- Date: 2026-08-22
- Reviewer: Claude Fable 5
- Result: verified (implemented and reviewed in the same change)
- Evidence: `ChatChunk::reasoning` in `crates/hickory-agent/src/llm.rs`; the
  `thinking` arm in `llm_anthropic.rs`; `reasoning`/`reasoning_content` in
  `llm_openai.rs`; `stream_completion` in `react_loop.rs`; `write_reasoning`
  in `session.rs`; `"reasoning"` in `hick_lang::is_raw_content_tag`;
  `Reasoning` in `apps/web/src/components/SessionTurns.tsx`; `appendReasoning`
  in `ChatDock.tsx`.
- Tests: `llm_openai::tests::reasoning_content_never_leaks_into_the_answer`;
  `session_view::tests`.
</book:entry>

</book:doc>
