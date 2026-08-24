# Replay Recomputes Exact Lineage At Any Commit, And Names Its One Limit

Given a `.hick` document in a git repository, when a reader asks for its
lineage at an earlier commit — `hick lineage --at <commit>`, or the time
slider on the lineage view — then that commit's document is read out of git,
woven **without executing anything**, and its provenance reported exactly;
and when the old document does not parse with today's binary, then the answer
says *replay works back to the last grammar change* rather than surfacing a
parse error.

This needs no new data model at all. Each commit contains its document, a
weave is deterministic, and weave-only checks are already affordable — so
lineage at a commit is recomputed rather than recalled, and there is nothing
stored that could be believed over the repository.

Corollaries that are part of the guarantee:

- **Nothing is checked out.** The tree at the commit is materialized with
  `git archive` into a scratch directory that is deleted afterwards. The
  working tree is untouched, `git status` is unchanged, and the up-loop never
  sees a file appear.
- **The whole tree, not the document alone.** A document's pastes, includes
  and upstream edges read their siblings; replaying without them would report
  a lineage that commit never had.
- **Weave-only, never execute.** A replay of last March must not run last
  March's commands against today's machine, network and containers: whatever
  came out would be neither what happened then nor what happens now.
- **Renames are followed.** The slider walks `git log --follow`, and a replay
  reads the name the document had *at* that commit — a slider that stopped at
  a rename would say the document was born there.
- **The grammar boundary is a fact about the tool, not about the commit.**
  The message says so in those words, names `git show <commit>:<path>` as the
  way to read the document anyway, and is reported as a boundary (HTTP 200
  with `grammar_boundary: true`) rather than an error. A document that parses
  but will not weave is reported as a *different* fact, because it is one.
- **Replay gives the state AT a commit, never the thread BETWEEN two of
  them.** Correlating one version's spans with another's is a recorded
  correspondence and a different mechanism; the slider says so where a reader
  would otherwise assume it.
- **A folder that is not a repository has no history and that is not an
  error** — the slider says there is nowhere to slide to.

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-cli/src/replay.rs` — `history` / `parse_history`
    (`--follow --name-only`, the path the document had then), `materialize`
    (`git archive`, no checkout), `replay_at` (parse first so the boundary is
    reported as itself; `RunMode::Weave`), `grammar_boundary_message`.
  - `crates/hickory-cli/src/main.rs` — `LineageArgs::at` / `history`,
    `doc_in_repo`, `cmd_lineage_history`, `replay_run`.
  - `crates/hickory-cli/src/serve/history.rs` — `GET /api/docs/:id/history`
    and `GET /api/docs/:id/replay`, the boundary as a 200.
  - `apps/web/src/lineage/TimeSlider.tsx` and `apps/web/src/views/LineageView.tsx`
    — the slider, its stops (oldest → newest, ending at the working tree,
    which is not a commit), the boundary notice, and the caveat about what
    replay does not relate.
  - Tests: `crates/hickory-cli/tests/replay_and_floor.rs`
    (`replay_reports_the_lineage_the_document_had_at_a_commit` — including
    that the working tree is undisturbed —,
    `history_names_the_commits_the_slider_can_stop_at`,
    `a_document_past_the_grammar_boundary_says_so_rather_than_failing_obscurely`);
    `apps/web/src/lineage/TimeSlider.test.tsx` (seven cases).
- Caveat requiring LLM review: `git archive` of a whole tree runs per replay,
  with no cache. On a large repository a slider drag is several archives. The
  cost has not been measured against a repository with real history, and a
  per-commit cache is the obvious answer if it bites.
