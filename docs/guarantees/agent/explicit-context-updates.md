# Explicit context updates preserve confirmed history

Given the confined Python agent and its retained branch,
when a user inspects changed context, then the non-modal agent pane compares
source-backed ancestry of confirmed requests against specific current disk or
unsaved-buffer candidates. Captures alone and model-output genealogy do not
advance exposure. Hidden projection segments are not supplied editor references.

Selecting an update freezes its candidate bytes. Queueing does not advance the
confirmed baseline, save an editor, rewrite history, or mutate an in-flight
request. Python imports selected updates at its next safe model boundary through
the generic context protocol. The original role/scope accompany the notice;
source text cannot promote itself to a higher authority. A later edit stays
visible against the frozen candidate. Delivery distinguishes queued, preparing,
prepared, transport-started, provider-confirmed, failed and uncertain cancellation.
A completed stream is host-observed delivery, not proof of understanding.

Branches inherit their parent's exposure and pending selections but subsequent
delivery does not change the parent's record. Local sidecars retain these states;
an interrupted preparation/transport is uncertain on reload. Source candidates
with ambiguous correspondence or unavailable bytes cannot be silently queued.

---

Last LLM verification:

- Date: 2026-10-19
- Reviewer: Codex
- Result: partially verified; acceptance remains subject to focused witnesses.
- Originating requirement: GitHub #51 and #47; live-agent-context design.
- Evidence: `context_updates.rs`, `python_agent.rs::Host::handle`,
  `serve/python_agent.rs::context_updates`, Python `HostedContext.boundary`,
  `ContextUpdates.tsx` and the existing `DiffView` renderer.
- Assumptions: confirmed means a completed provider stream; generic opaque
  parentage describes source ancestry, not semantic use. No remote telemetry.
- Exclusions/gaps: provider internals remain opaque. Deleted and inaccessible
  sources have distinct visible states; automatic filesystem rename
  discovery is absent. Instruction applicability remains the recorded request
  scope; this surface does not infer new authority from a filename. Browser
  witnesses and full instruction-scope acceptance need verification before closing #51.
- Witnesses: Rust inbox failure/branch test, HTTP confined-worker source/edit test,
  Python safe-boundary tests and UI explicit selection test.
- Challenges/falsifier: a click advancing the baseline, a changed queued candidate
  entering an earlier request, hidden editor content labeled supplied, or a child
  receipt updating its parent would violate the stated behavior.
- Dependencies: source/manifest protocol and immutable store at HEAD preceding
  this change; changes to bootstrap projection, request assembly or receipts
  invalidate this review.
- Executed checks: three Rust inbox/mapping/persistence tests passed; six Python
  context runtime tests passed; one real confined-worker HTTP witness passed;
  one UI selection/patch test passed; TypeScript compilation and diff check passed.
  Browser end-to-end coverage and rename discovery remain unverified.
- Human judgment: audio retention preference approved separately for #60; no
  declaration about context freshness or semantic support.
