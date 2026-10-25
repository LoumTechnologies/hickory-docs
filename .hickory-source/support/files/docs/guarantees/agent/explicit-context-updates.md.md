<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/agent/explicit-context-updates.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `ee8426e707c2abe5e60cc0408f730e2c01f13a8b7f8d5960da746469636e362c`.

<book:entry path="docs/guarantees/agent/explicit-context-updates.md">
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

Issue 72 UI adapter extension (2026-10-19):

- ContextUpdates registers actual panel/patch disclosure, bounded status/patch
  reads, eligible selection and queue/refresh through the same inspect handler.
  Selection carries the observed revision. Results report queued states without
  claiming provider confirmation. Hidden patch content is excluded from both
  descriptor values and opaque revision tokens.
- Changing branches clears selected state; an in-flight inspect response cannot
  replace the newly selected branch's UI. An already dispatched originating-branch
  operation may have effects and must be inspected there before retrying.
- Executed checks: both focused component tests passed, including exact queued
  reference/revision requests and no hidden patch text in discovery. The existing
  real confined request-boundary integration remains separate evidence.

## October 21 provider-workflow continuation

A real attached-provider correction run exposed an immediate-request flood:
`ContextUpdates` restarted its polling effect whenever the parent supplied a
new `getContext` closure. The trace contained nearly 2,000 context POSTs and
repeated completion-time editor-capture timeouts. The flood is established;
its causal contribution to those timeouts remains under investigation.

The poll now reads the latest callback through a ref, keeps its five-second
interval across callback replacement, and allows only one pending automatic
capture/request per mounted branch. A retired capture cannot post its context
after its branch changes. Explicit Refresh and Queue retain their separate
user-requested semantics. The two new focused tests challenge rapid callback
replacement, fresh callback use, slow captures and retired-branch replies.

The existing disclosure test also found the nested DiffView's generic reading
adapter registered while its native details were closed. Its reading now follows
both the outer context disclosure and the individual patch disclosure using the
shared visibility provider. This is progressive-disclosure parity; it is not a
#77 target-answer quarantine or permission to deliver held-out values.

LLM implementation review: partially verified for these bounded lifecycle and
visibility rules. Four ContextUpdates cases and targeted strict TypeScript passed.
Human judgment: not supplied. The handoff records live-provider results and exact
source hashes separately; a passing mocked context test is not a provider witness.


October 22 lifecycle extension: keyed document/branch owners retire the old view
before replacement. Manual requests use synchronous admission and a request
version; retired captures refuse before POST. Earlier automatic responses cannot
replace a newer manual queue result. Already dispatched branch-local requests
can still have effects. Six focused component tests and targeted TypeScript
passed; LLM review remains partial and human judgment absent. Exact dependencies
are recorded in the October 22 handoff context-ownership checkpoint. A queued
selection appearing on a replacement branch or an older response replacing its
queue receipt would falsify this bounded lifecycle claim.
</book:entry>

</book:doc>
