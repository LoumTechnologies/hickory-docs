# The Git Pane Does The Daily Loop

Given a folder that is a git repository, when the Git pane is open, then
above the commit graph it shows the working tree — the branch, its upstream
and how far apart they are, every changed file on the staged and unstaged
sides — and offers, each as one git command run as itself: stage and unstage
a file or everything, discard a file's changes after a second click, show a
file's diff, commit what is staged with a message, amend the last commit,
push (setting an upstream the first time), pull fast-forward only, switch to
or create a branch, and stash and pop. When git refuses, git's own words are
shown. Amending is refused when `HEAD` is below the publication floor, since
that commit is a record somebody else may hold. Nothing prompts for a
credential; a push that needs one fails saying so rather than hanging.

The pane was read-only for a while, on the argument that a half-built git UI
teaches a workflow it cannot finish. The argument was right about "half";
the answer was to finish the loop a person runs every day, and to leave
merge and rebase — decisions with conflicts in them — to a terminal that is
one click away on the tree.

## Boundary

Pull is `--ff-only`. There is no force push, no rebase, no interactive
anything, and no merge: a button is the wrong place to make a decision that
can have conflicts in it. A folder that is not a repository shows the graph
pane's existing explanation and no verbs.

---

Last LLM verification:
- Date: 2026-09-02
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hickory-cli/src/serve/git_ops.rs` (`changes`, `diff`,
  `stage`, `unstage`, `discard`, `commit` with the floor check through
  `crate::floor`, `push`, `pull`, `branches`, `checkout`, `stash`; `git`
  sets `GIT_TERMINAL_PROMPT=0` and a batch-mode ssh); routes in
  `crates/hickory-cli/src/serve/mod.rs`; `apps/web/src/views/GitPane.tsx`
  (`WorkingTree`, `SideList`, `sides`); `apps/web/src/components/DiffView.tsx`;
  `apps/web/src/api/client.ts` (`gitChanges` … `gitStash`).
- Test coverage: `crates/hickory-cli/tests/git_ops.rs` (four tests over HTTP
  against real repositories, including a push to a bare remote and the
  refused amend below the floor); `apps/web/src/views/GitPane.test.tsx`
  ("the working tree"); `apps/web/src/components/DiffView.test.tsx`.
- Caveats: push and pull against a network remote are exercised only with a
  local bare repository; credential failure is by construction of the
  environment variables, not by a test.
