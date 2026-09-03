# A Typed Path Opens That File

Given the command bar in files mode, when a query is typed, then the answers
are **ranked**, best first: a path typed out in full, then an exact file name,
then a path suffix, then a name that starts with the query, then a name that
contains it, then a path that contains it. Ties go to the shorter path.

The result limit is applied **after** ranking, over every file the folder tree
lists — never during the walk that collects them.

## Why

Neither half of that was true. Files mode matched with a bare
`path.includes(term)` and left the results in whatever order the tree happened
to be walked in, and the cap of 40 was applied *inside* that walk. Two
separate ways to be handed the wrong file:

1. **No ranking.** With several matches, first-in-tree-order won, and Enter
   takes the first. Nothing preferred the file whose *name* was what you
   typed over one that merely contained the string somewhere in its path.
2. **The cap truncated before anything compared.** A file matching perfectly
   but sitting late in the tree was dropped without ever being scored. The
   answer therefore depended on how the folder tree was built, which is not
   something a person can see or reason about.

Found by dogfooding on 2026-09-02, with the app open on this repository:
typing `crates/hick-lsp/src/lang_detect.rs` opened `default.json`. It is the
kind of defect that makes an editor feel untrustworthy out of all proportion
to its size — Go To File is the single most-used navigation in any JetBrains
IDE, and one that ignores what you typed is worse than not having one.

The bands are deliberately coarse and far apart, with ties broken by path
length. That is what makes `src/app.ts` beat `vendor/deep/nested/app.ts` for
`app.ts` without either needing a rule of its own.

The path-suffix band is anchored on a separator, so `detect.rs` is not read as
a suffix of `lang_detect.rs` — that is a name match, and the band below covers
it. Without the anchor, a query that happens to end a longer filename would
outrank the file actually called that.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/web/src/lib/fileRanking.ts` — `rankFile` (the bands) and
  `rankFiles` (sort, then cap); `apps/web/src/views/WorkspaceView.tsx` — the
  files branch of `commandCandidates` now collects every file and ranks it,
  where it used to filter and cap inside `walk`.
- Test coverage: `apps/web/src/lib/fileRanking.test.ts` (9) — including
  "puts a path typed out in full first" (the reported defect verbatim),
  "caps AFTER ranking, never during the walk" (the second half), "prefers a
  name match to a path match", "breaks a tie by the shorter path", and
  "does not read a name match as a path suffix".
- Caveat requiring LLM review: the ranking is substring-based, not fuzzy —
  `lngdtct` finds nothing, where several editors would find `lang_detect.rs`.
  That is a deliberate stopping point rather than an oversight: subsequence
  matching needs a scoring model to stay useful, and the defect being fixed
  here was caused by ranking that was too loose, not too strict. Nothing in
  this file forecloses adding it later as a band below the existing ones.
