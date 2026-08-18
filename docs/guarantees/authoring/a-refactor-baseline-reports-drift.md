# A Refactor Baseline Reports The Moment An Output Would Move

Given a document whose woven outputs were pinned as a refactor baseline
(the toolbar's "Refactor", `POST /api/docs/:id/refactor/begin`), when the
document is restructured — blocks split, prose interleaved, sections
reordered — then the live verdict (`GET …/refactor/status`, the toolbar
badge) says either that every output still matches the baseline or exactly
which outputs differ and how (changed, added, removed); and checking never
executes a cell.

This is the equivalence checker `hick equiv` uses, turned into a standing
invariant: restructure freely, and the badge tells the truth the whole
time. Weave-only checking is what makes that affordable — the same reason
`/render` never executes — so the verdict covers everything whose bytes are
derivable without running code; outputs that depend on execution are
compared as last woven.

Properties:

1. **One checker, two surfaces.** The badge and `hick equiv` both go
   through `hick_literate::equiv::compare_outputs`; they cannot disagree
   about what "identical" means.
2. **The baseline is session state, deliberately.** It lives in the app
   process's memory, keyed by document id — it survives the tab closing
   (the badge resumes by asking for status on mount) and dies with the app,
   because git holds the durable before-state.
3. **Ending is explicit.** "Done" (`…/refactor/end`) drops the baseline;
   nothing expires it behind the person's back.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Fable 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hickory-cli/src/serve/refactor.rs` — `begin` snapshots
  `state.weave(...)` outputs, `status` rebuilds the stored strings as
  `FileContent` and calls the shared `compare_outputs`, mapping `DiffKind`
  to `changed`/`added`/`removed`; `end` drops the entry; baselines live in
  `LocalState::refactors`. CLI: `cmd_equiv` in
  `crates/hickory-cli/src/main.rs` (exit 0 equivalent / 1 with
  `format_diff` output). UI: `apps/web/src/components/RefactorBadge.tsx`
  (resume-on-mount, 2.5 s polling while active, ok/fail coloring) rendered
  from the document toolbar in `apps/web/src/views/workspaceTabs.tsx`;
  words in `apps/web/src/lib/refactorSummary.ts`.
- Test coverage: `crates/hickory-cli/src/serve/refactor.rs::tests`
  (stored-string round-trip through the shared checker, all three kinds);
  `apps/web/src/lib/refactorSummary.test.ts` (badge wording); CLI verified
  by a live smoke run (adopt → equiv clean → edited copy → exit 1 with a
  content diff).
- Caveat requiring review: the badge's polling loop and resume-on-mount
  are untested by automation; and weave-only checking means a refactor that
  changes only exec-derived bytes shows up after the next run, not live —
  a documented boundary, not a bug, but worth revisiting if it surprises.
