> Retired 2026-10-05. The homepage now contains one short document with a
> real browser debugger, paused on first load, and a desktop download section.
> Interest disclosures, the identity question, and the recorded-run demo are
> no longer on the homepage. The historical rationale below remains a record,
> not a guarantee of the current page. See `landing/homepage-opens-paused.md`.

# The Home Page's Demo Runs The Real Mechanism, And Names What Is Simulated

Given a visitor on the home page with no account, no sign-up and no network
beyond loading the page, when they drive the demo, then the lineage they see is
computed by the same code the product runs — the document is parsed by the real
`.hick` parser, woven by the real weaver, and the Sankey ribbons are drawn from
real provenance ranges over live editor geometry. Nothing on the page replays a
recording of a weave that happened somewhere else.

There is exactly **one** demo, and it shows the product's one sentence:
literate programming where you can edit the generated files. Specifically:

- The document weaves its files on arrival, without the visitor clicking
  anything — one shared fragment tangled into two files, so editing it from
  either end moves both.
- An edit made on a generated file is resolved backwards through provenance
  into the fragment that produced it — not into a second copy of it — so the
  other file pasting the same fragment moves with it.
- An edit that lands on text the weaver wrote is refused and named, and the
  buffer is put back, rather than silently dropped.
- The woven banner is commented in the generated file's own language.

Given the executable cells, when the visitor runs them, then the transcript
shown agrees with the expectation the document pins beside them, and every
command in it is one the document actually contains. A browser cannot start a
container, so this transcript **is** a recording — and the page says so in
plain words, next to itself, along with what `hick run` and `hick test` do with
the same cells on the visitor's own machine.

Given a step that changes the document, when it runs, then the document pane
scrolls to the text that changed and flashes it — and it scrolls the
**editor**, never the page, so a visitor reading the paragraph above is never
yanked somewhere else.

Given any part of the page, then no demo contacts a third-party service, and no
demo demonstrates a capability the product does not have. The retired
knowledge-work walkthrough (a simulated Jira integration) and collaboration
demo (two people in one document) both failed the second clause: see
`docs/specs/freeform/local-only.md`, which records that collaboration is not a
feature of this product and that there is no hosted integration to sell.

---

Last LLM verification:
- Date: 2026-08-13
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/web/src/landing/demos/scripts.ts` holds the one demo document
  as data. `apps/web/src/landing/demos/DemoSplit.tsx` weaves it with
  `lib/weave.weaveOutputs`, derives ribbons with `lib/ribbons.deriveRibbons`,
  and draws them with `lib/ribbonGeometry` — the same three modules
  `views/SplitView.tsx` uses in the desktop app's split view. Output edits go
  through `lib/diff.computeEdits` → `lib/weave.mapEditsToSource` →
  `applySourceEdits`, and an edit landing on weaver-generated text is refused
  with `SyntheticRangeViolation` and reverted rather than silently dropped
  (`DemoSplit.onOutputChange`). The recording is labelled in
  `ProgramDemo.tsx` ("A recorded transcript, replayed here — a browser tab
  cannot start a container"). The reveal is `DemoSplit`'s `markSource`, which
  sets `scrollDOM.scrollTop` directly rather than using CodeMirror's
  `scrollIntoView` — the latter walks up and scrolls every ancestor scroller,
  including the window.
- Test coverage: `apps/web/src/landing/demos/scripts.test.ts` (weave, one
  fragment into two files, the reverse edit reaching both, the language-correct
  banner, the transcript agreeing with the pinned expectation, and every exec
  naming a declared container) and
  `apps/web/src/landing/demos/demos.test.tsx` (files woven before any click, no
  transcript until the cells are run, the recording labelled on screen,
  engagement reported once).
- Caveat requiring human review: the ribbon *geometry* is asserted only in
  `lib/ribbonGeometry.test.ts` (pure math) — jsdom performs no layout, so no
  test can prove the demo's ribbons land on the right lines on screen. A change
  to the demo layout or to `DemoSplit`'s measurement still needs a human to
  look at the page. The claim "nothing here contacts a third party" is likewise
  structural: it holds because no demo module imports `api/client`, which a
  reviewer must re-check when a demo gains a feature.
