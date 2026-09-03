# The Tail Of The Story Is The Next Commit

Given the history lens open on a repository, when the working tree holds
uncommitted changes, then the last card says so and offers a message box;
the message typed there, with Commit, stages everything and makes one
commit with it — `git add -A` and `git commit`, run as themselves. And
whether or not anything is uncommitted, the tail offers a command and an
output folder: Run and commit runs that command in a detached worktree at
HEAD, never in the working tree, and commits what it wrote under the folder
as HEAD's child, with `Hick-Recipe` and `Hick-Output: <tree> <folder>` in
the trailers; the folder's files then appear in the working tree, clean.

A folder that already holds anything, the repository's root, and a path
outside the folder are refused before the command runs. A command that
fails, or writes nothing git would keep, commits nothing. An empty message
is refused by git and its words are shown.

This is `hick emit` with a place to type it (`lenses.md` step 5): the only
place a lens creates something new, and it creates only forward — a child
of HEAD, never a change to anything that exists.

## Boundary

The prose commit is the Git pane's own commit verb and inherits its rules
(no amend from here; amend stays in the pane, refused below the floor). The
recipe run is on this machine through the shell; nothing is containerised.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hickory-cli/src/recipe.rs::emit` (checked output,
  occupied-folder refusal, `record` at HEAD as HEAD's child, `update-ref`,
  then `checkout HEAD -- <folder>`); `crates/hickory-cli/src/serve/story.rs::emit`
  on `POST /api/git/recipe`; `apps/web/src/views/HistoryLens.tsx::TailCard`
  (the message box and Commit through `gitStage({all})` + `gitCommit`; the
  command and folder through `gitRecipe`).
- Test coverage: `crates/hickory-cli/src/recipe.rs::the_tail_commits_a_commands_output_as_a_recipe_child_of_head`;
  `crates/hickory-cli/tests/story.rs::the_tail_runs_a_command_and_the_card_can_be_replayed`
  (over HTTP: the commit is HEAD, the files are in the working tree and
  clean, ignored output is not materialised);
  `apps/web/src/views/HistoryLens.test.tsx` ("commits the working tree
  from the tail, and runs a command as a recipe").
- Caveats: exercised in jsdom, not in a real browser.
