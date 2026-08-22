# The Model's Reasoning Is Shown Apart From Its Answer, Folded

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
