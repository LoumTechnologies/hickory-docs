# A Recipe Commit Can Be Replayed, Into A New Commit, Never Over The Old One

Given a commit whose trailers carry a recipe and whose tree matches its
recorded output, when Replay is pressed on its card in the history lens
(`POST /api/git/replay {sha}`), then the recipe's command is run again in a
detached worktree — never in the person's working tree — and what it wrote
is committed as a **new** recipe commit S2 carrying `Hick-Replay-Of` (the
commit replayed) and `Hick-Replay-Same` (whether the output tree is the one
that commit recorded); then S2 is joined to the history:

- **Above the publication floor**, S1 is a draft: S2 is made as S1's
  sibling (same parent) and the commits after S1 are rebased onto it, so
  the branch reads *parent → S2 → the edits*. S1 is gone from the branch.
- **Below the floor**, S1 is a record someone else may hold: S2 is made as
  S1's child and merged into HEAD, so S1 stays and the branch reads
  *… → S1 → edits → merge(S2)*.

Either way the person's later commits survive with their edits intact. A
dirty working tree is refused in words before anything runs; a merge
commit is refused as not a recipe; the first commit of a repository is
refused because nothing can be rebased onto it. A rebase or merge that
stops on a conflict answers `409` with git's own words, and the repository
is left where git left it — a rebase or merge in progress — never hidden
and never undone silently.

The lens draws S2's evidence apart from every claim: *replayed · same as
S1* or *replayed · differs from S1*, in place of *unrecorded · no evidence
of drift*. That chip is the only thing on a recipe card that comes from a
run rather than from the commit's own words.

**Why a new commit.** `expression-and-log.md`: a session may produce a
commit, and nothing may re-produce one that exists. Replay never rewrites
S1; it makes S2 and lets git's own rebase or merge carry the edits across,
with S1 (or S1's parent) as the base git already knows.

**Why a worktree.** The command sees a clean checkout, so the person's
uncommitted work is neither in its input nor beside its output, and the
repository's own `.gitignore` applies because a worktree is a real checkout
of it. The output folder is cleared in the worktree first, since a
scaffolder refuses a folder that is not empty.

## Boundary

The command runs on this machine through the shell (`sh -c`, or `cmd /C` on
Windows); `Hick-Image` is recorded and not pulled. Only the output folder
named by `Hick-Output` is taken from the run. `lenses.md` step 4.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hickory-cli/src/recipe.rs` — `replay` (the floor
  decides sibling-and-rebase or child-and-merge), `run_in_worktree`,
  `compose_tree`, `commit_object`, `JoinStopped`;
  `crates/hickory-cli/src/serve/story.rs::replay` (`409` for a stopped
  join); `crates/hickory-cli/src/serve/git.rs::recipe_of` reads
  `Hick-Replay-Of`/`Hick-Replay-Same`; `apps/web/src/views/HistoryLens.tsx`
  (`RecipeCell`: the Replay button only when `output_matches`, the replay
  chip).
- Test coverage: `crates/hickory-cli/src/recipe.rs` unit tests
  (`a_replay_above_the_floor_replaces_the_scaffold_and_keeps_the_edits`,
  `a_replay_below_the_floor_is_a_merge_and_a_dirty_tree_is_refused`, a
  scaffolder whose output changes between runs so *same* and *differs* are
  both exercised); `crates/hickory-cli/tests/story.rs::the_tail_runs_a_command_and_the_card_can_be_replayed`
  (over HTTP: the dirty-tree refusal, the rebase, the evidence on the log);
  `apps/web/src/views/HistoryLens.test.tsx` ("the story's verbs").
- Caveats: the conflict-stopped path is tested for a reorder (`story.rs`),
  not for a replay's rebase; both go through the same `409` and leave git's
  state. Exercised in jsdom, not in a real browser.
