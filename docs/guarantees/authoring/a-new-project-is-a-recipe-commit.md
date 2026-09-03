# A New Project Is A Recipe Commit, Made As One Act

Given a folder open in the app that is a git repository, and a `dotnet new`
template chosen in File → New Project with a folder for its files, when the
project is created, then the scaffolder runs into a scratch directory and
what it wrote is committed — in one act, with nothing a person can do in
between — as a commit whose last paragraph carries the recipe:

```
Hick-Recipe: dotnet new console -o greeter -n Greeter --language 'C#' --no-restore
Hick-Image: mcr.microsoft.com/dotnet/sdk:10.0
Hick-Output: <git tree hash of greeter/ as written> greeter
```

The commit holds exactly the scaffolder's files (filtered by the
repository's own `.gitignore`) and nothing of the person's: work that was
staged stays staged, work that was unstaged stays unstaged, and the
scaffold's own paths are clean in `git status` afterwards. No `.hick`
document is written. The dialog shows the exact commit message before it is
made, rendered by the same function that writes it.

A scaffold that cannot run — no SDK, a template no SDK has — commits nothing
and leaves no folder behind. A target folder that already holds anything,
the repository's root, and a path outside the folder are refused by name
before any subprocess runs. A folder that is not a repository is refused
with `missing: "repository"` in the body, and the dialog draws its own
screen from that field — a sentence and `git init`, never a button — the
same shape as `missing: "dotnet"`.

**Why one act, and why a temporary index.** A recipe commit is honest only
if its tree is exactly what the recipe produced. An edit between the run
and the commit fuses into the commit where no replay can separate it, and
the commit can no longer be upgraded; so the product never produces that
moment. The commit is built through git's plumbing on an index of its own
(`read-tree HEAD`, `add` the scaffold's paths, `write-tree`, `commit-tree`,
`update-ref`), which is what lets it be made with other work in flight
without taking or touching it.

**Why a tree hash.** `Hick-Output` is the git tree hash of the scaffolded
folder, so the history lens can tell a commit that is exactly the scaffold
from one edited before it was committed with one `git rev-parse`, no
replay. It is still a claim about what the scaffolder wrote — a matching
hash can be written by hand — so the lens says "matches", never "verified".

This replaces `a-new-project-writes-the-command-it-ran.md` and
`a-new-project-that-cannot-run-still-leaves-a-document.md` (both retired
2026-09-03): a scaffold is an act, not an expression, and git holds the
past. `docs/specs/freeform/lenses.md`, step 3.

## Boundary

The scaffolder runs on this machine's own `dotnet`; `Hick-Image` is
recorded for a containerised replay and nothing here pulls it. Replay is
not built. `hick ingest --from '#cell'` is unchanged and still brings an
exec's output volume into a document a person is writing.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hickory-cli/src/scaffold_commit.rs` — `commit_scaffold`
  (the temporary index, the trailers, the real-index `add` afterwards),
  `checked_output`, `refuse_occupied`, `NotARepository`;
  `crates/hickory-cli/src/scaffold.rs` — `run_scaffold`, `commit_message`,
  `dotnet_new_command` (`-o <folder>`, from the root); `scaffold_document`
  and `SCAFFOLD_CELL` are deleted. `crates/hickory-cli/src/serve/scaffold.rs`
  — `create` (refusals before the subprocess, scratch directory, one act),
  `preview` (the message with the tree hash named as pending),
  `scaffold_error` (`missing: "dotnet"` and `missing: "repository"`).
  `apps/web/src/components/NewProjectDialog.tsx` — no document field, no
  run checkbox, the message as the preview, `NoRepositoryScreen` keyed off
  the field; `apps/web/src/App.tsx` announces the commit.
- Test coverage: `crates/hickory-cli/src/scaffold_commit.rs` unit tests
  (staged and unstaged work left alone, `.gitignore` honoured, the trailer
  equal to `HEAD:greeter`, a root commit in an empty repository, the two
  refusals by name); `crates/hickory-cli/tests/serve_scaffold.rs`
  (`the_preview_is_the_commit_that_gets_made`,
  `a_new_project_is_a_commit_holding_exactly_what_dotnet_wrote` — live
  against `dotnet`, with `output_matches: true` on the log and a second
  scaffold refused, `a_scaffold_that_cannot_run_commits_nothing_and_leaves_no_folder`,
  `the_folders_a_scaffold_may_not_have_are_refused`,
  `a_folder_without_a_repository_is_told_so_by_a_field`);
  `crates/hickory-cli/tests/scaffold_templates.rs::the_message_carries_the_recipe_as_trailers`;
  `apps/web/src/components/NewProjectDialog.test.tsx` (the preview is the
  commit, the create call, the not-a-repository screen).
- Caveats: the live test skips without `dotnet`. The dialog is exercised in
  jsdom.
