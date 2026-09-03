# The Past Is Edited By Rebase, Above The Floor Only

Given the history lens open on a repository, when a card above the
publication floor (a draft) is reworded, moved earlier or later, or
dropped (`POST /api/git/reword`, `/api/git/move`, `/api/git/drop`), then
git's own interactive rebase runs over the drafts with a todo list this
product wrote — `reword`/`pick`, a swapped order, or a line left out — and
the editor git asks is answered from a file rather than typed. The result
is the same history git would have made from a terminal.

A card below the floor is a record someone else may hold: it carries no
verbs, and the routes refuse it by name. A dirty working tree is refused
before anything moves. A merge among the drafts is refused, because
rebasing through it would linearise a story that did not happen. A rebase
that stops on a conflict answers `409` with git's words, and the repository
is left as git left it — a rebase in progress, for the person to finish or
abort from a terminal — never hidden and never undone silently.

This is `lenses.md` step 6: editing a card's prose is a reword, moving a
card is a reorder, deleting one is a drop, and the floor is the line the
Git pane's amend and `hick emit` already draw.

## Boundary

One card per verb; a multi-card act (reorder several at once) is not
offered, so each act is one rebase and undoable as one. The sequence
editor is `cp`, which git for Windows ships in its own shell; a git with no
`cp` on the path cannot use these verbs and says so in git's words.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hickory-cli/src/story.rs` — `editable` (the three
  refusals), `rebase_with` (`GIT_SEQUENCE_EDITOR`/`GIT_EDITOR` as `cp
  <file>`, `--root` when nothing is published), `reword`, `drop`,
  `move_commit`; `crates/hickory-cli/src/serve/story.rs` (the three
  routes, `409` on "rebase stopped"); `apps/web/src/views/HistoryLens.tsx`
  (`DraftVerbs` on draft cards only, the inline reword textarea).
- Test coverage: `crates/hickory-cli/src/story.rs` unit tests
  (`a_draft_can_be_reworded_moved_and_dropped_and_a_record_cannot`,
  `a_dirty_tree_is_refused_before_anything_moves`);
  `crates/hickory-cli/tests/story.rs`
  (`drafts_are_reworded_moved_and_dropped_and_records_are_refused`,
  `a_reorder_that_conflicts_is_answered_with_gits_words_and_left_where_git_left_it`
  — `.git/rebase-merge` exists afterwards); `apps/web/src/views/HistoryLens.test.tsx`
  ("gives drafts reword, move and drop, and records nothing", "shows
  git's words when a verb is refused").
- Caveats: exercised in jsdom, not in a real browser.
