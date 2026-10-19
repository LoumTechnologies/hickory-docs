# Review threads preserve the questioned target

Given selected text in a saved-path editor or a drawn provenance connection,
when the user opens a review and saves a comment, then its target retains the
quoted text, source SHA-256, path and line range, plus the relationship descriptor
when applicable. Replies cannot replace that target. Comments, resolutions and
reopenings append local workspace events; earlier events remain. Resolution is
a user declaration, separate from executed verification. Changed or unavailable
saved source is shown rather than silently relocating the original quotation.

The user can prepare and copy an investigation prompt containing the target and
review history. Preparing a prompt does not send it or record an agent response.
A separate send action can dispatch to one selected conversation and link observed
work, as specified in review-investigations-link-observed-agent-work.md.
Reviews are personal workspace state, not automatically committed or published.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: partially verified
- Originating requirement: comment on provenance ribbons/braces and agent claims.
- Evidence: `serve/reviews.rs`, `lib/reviews.ts`, `components/ReviewPanel.tsx`,
  `shell/Ribbons.tsx`, `editor/useBlame.ts`.
- Test coverage: `serve::reviews::tests` exercises original retention, refusing
  replacement targets, user-declared resolution, and changed source hashes.
- Assumptions and exclusions: local editable records are not authenticated or
  tamper-proof; retain/export workspace data for backup. Source state compares
  saved bytes, not subsequent unsaved editor changes. Replies are append-only
  through the API; manual filesystem edits remain possible.
- Witnesses and challenges: a target with identical quoted words elsewhere is
  not automatically relocated; a failed history read reports an error.
- Executed checks: `cargo test -p hickory-cli --lib serve::reviews::tests`
  passed (2); `cargo test -p hickory-cli --test serve_blame` passed (7),
  including persisted targets and workspace-scope refusal. Review selection,
  saving and resolution passed in Chromium, Firefox and WebKit. Frontend
  target-mapping tests cover ribbon offsets and filtered conversations.
- Human judgment: not recorded.

- Dependency fingerprint: `crates/hickory-cli/src/serve/reviews.rs` SHA-256 `a5bfe329e2ec272624c1a0faf36084600075fc6faad86226b7a6320180fedb91`; changes to this implementation or its cited integration paths require review.

Issue 72 adapter extension (2026-10-19):

- Review disclosure, bounded registry descriptors, selected thread/target,
  comment draft, investigation preparation/conversation and submissions use
  ReviewPanel's existing handlers. Agent submissions carry agent=true and are
  retained as agent-declared. Human submissions preserve user-declared.
- Agent-originated resolve/reopen is refused by serve/reviews.rs::append;
  the registered action returns a human-input request. The coach only counts
  user-declared criterion and resolution events. This local declaration is not
  authentication; local HTTP clients and editable storage retain their existing
  trust model.
- Executed checks: the focused Rust attribution/refused-resolution test passed;
  all three ReviewPanel tests passed, including actual adapter request bodies.
  Strict TypeScript checking of ReviewPanel and NativeWorkflowTour passed.
- Human judgment: no review or launch-readiness attestation was supplied here.
