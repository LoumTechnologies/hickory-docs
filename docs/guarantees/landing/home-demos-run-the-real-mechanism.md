# The Home Page's Demos Run The Real Mechanism, And Name What Is Simulated

Given a signed-out visitor on the home page with no account, no sign-up and no
network beyond loading the page, when they drive any of the three demos, then
the lineage they see is computed by the same code the product runs — the
document is parsed by the real `.hick` parser, woven by the real weaver, and
the Sankey ribbons are drawn from real provenance ranges over live editor
geometry. Nothing on the page replays a recording of a weave that happened
somewhere else.

Specifically:

- The knowledge-work walkthrough generates **nothing** until the agent step
  runs, and then generates exactly one artifact per note. Every ticket
  description traces back, byte for byte, to the sentence in the notes that
  produced it, and an edit made on the generated side is resolved backwards
  through provenance into that same sentence — not into a second copy of it.
- No step of the walkthrough rewrites a byte the human typed. The agent
  appends its session, its tool calls, and the files it weaves; the notes stay
  exactly as written, and moving between steps never discards an edit the
  visitor made.
- The literate-programming demo tangles one shared fragment into two files, so
  editing it from either end moves both.
- The collaboration demo is two real Yjs clients with their own awareness
  states. The document is seeded by exactly one of them, so the text cannot
  double.

Given a step that changes the document, when it runs, then the document pane
scrolls to the text that step actually added and flashes it — and it scrolls
the **editor**, never the page, so a visitor reading the paragraph above is
never yanked somewhere else. A step that adds nothing scrolls to the text it
is asking the visitor to look at instead.

Given the same page, when a part of a demo is **not** real — the Jira
integration, the git remote — then the page says so in plain words next to the
thing that is faked, rather than leaving the visitor to assume an integration
exists. No demo contacts a third-party service.

---

Last LLM verification:
- Date: 2026-08-09
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/web/src/landing/demos/scripts.ts` holds every demo document
  as data. `apps/web/src/landing/demos/DemoSplit.tsx` weaves it with
  `lib/weave.weaveOutputs` (the same weaver `src/mock/mockApi.ts` serves the
  contract routes with), derives ribbons with `lib/ribbons.deriveRibbons`, and
  draws them with `lib/ribbonGeometry` — the same three modules
  `views/SplitView.tsx` uses in the signed-in workspace. Output edits go
  through `lib/diff.computeEdits` → `lib/weave.mapEditsToSource` →
  `applySourceEdits`, and an edit landing on weaver-generated text is refused
  with `SyntheticRangeViolation` and reverted rather than silently dropped.
  `collabRoom.ts` builds two `Y.Doc`s with a direct relay and seeds from one
  peer only. The simulated halves are labelled in `KnowledgeWorkDemo.tsx`
  ("no issue tracker is contacted", "(simulated)") and
  `CollaborationDemo.tsx` ("The git strip is simulated").
  The reveal is `DemoSplit`'s `markSource`, which sets `scrollDOM.scrollTop`
  directly rather than using CodeMirror's `scrollIntoView` — the latter walks
  up and scrolls every ancestor scroller, including the window. Its target
  comes from `scripts.changedRange` (line-snapped) or, for a step that adds
  nothing, `scripts.findRange` over that step's `focus` text.
- Test coverage: `apps/web/src/landing/demos/scripts.test.ts` (weave, lineage,
  round trip, one-fragment-two-files, the hand-written-prefix invariant, and
  that `changedRange` points at the tool calls a step inserted *inside* an
  existing element rather than at the end of the document),
  `apps/web/src/landing/demos/demos.test.tsx` (nothing generated before the
  agent step; the simulated labels are on screen; a pulled commit lands in
  both panes), `apps/web/src/landing/demos/collabRoom.test.ts` (single seed,
  convergence both ways, presence relayed), and
  `apps/web/src/landing/demos/git.test.ts` (the git strip cannot commit or
  push nothing, and a pull cannot swallow uncommitted work).
- Caveat requiring human review: the ribbon *geometry* is asserted only in
  `lib/ribbonGeometry.test.ts` (pure math) — jsdom performs no layout, so no
  test can prove the demo's ribbons land on the right lines on screen. A
  change to the demo layout or to `DemoSplit`'s measurement still needs a
  human to look at the page. The claim "nothing here contacts a third party"
  is likewise structural: it holds because no demo module imports
  `api/client`, which a reviewer must re-check when a demo gains a feature.
