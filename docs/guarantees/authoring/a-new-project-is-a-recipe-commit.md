# A New Project Is A Recipe Commit, Made As One Act

Given a `dotnet new` template chosen in File → New Project, a **location**
— any folder on this machine — and a folder name for its files, when the
project is created, then the scaffolder runs in a terminal, in a scratch
directory, and what it wrote is committed the moment it exits zero — in one
act, with nothing a person can do in between — as a commit whose last
paragraph carries the recipe:

```
Hick-Recipe: dotnet new console -o greeter -n Greeter --language 'C#' --no-restore
Hick-Image: mcr.microsoft.com/dotnet/sdk:10.0
Hick-Output: <git tree hash of greeter/ as written> greeter
```

The commit is made in **the repository that contains the location**, which
is not assumed to be the folder the app has open, and `-o` in the recipe
names the folder relative to *that* repository's root — because that is
where a replay runs it from. The dialog names the repository while the
person is still typing, and says so when it is not the open one.

The commit holds exactly the scaffolder's files (filtered by the
repository's own `.gitignore`) and nothing of the person's: work that was
staged stays staged, work that was unstaged stays unstaged, and the
scaffold's own paths are clean in `git status` afterwards. No `.hick`
document is written. The dialog shows the exact commit message before it is
made, rendered by the same function that writes it.

A scaffold that cannot run commits nothing and leaves no folder behind. No
SDK is refused before a terminal opens, with `missing: "dotnet"` in the
body; a template no SDK has is `dotnet`'s own refusal, made in the terminal
where a person can read it, and the app says only that nothing was
committed and where to look. A target folder that already holds anything,
the repository's root, and a folder name that is a path are refused by name
before any subprocess runs.

A location inside no repository is refused with `missing: "repository"` and
**the folder** in the body, and the dialog draws its own screen from those
fields: a button that runs `git init` there, creating the folder if it is
not there yet, and then returns to the form. A repository nested inside
another is refused. This is the one place the app runs `git init`, and it
is a button rather than a sentence because the person has already chosen
the folder by asking for a project in it — unlike the .NET SDK, which stays
a sentence and a link
(`docs/guarantees/debugging/a-missing-debugger-is-a-button.md`).

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

**Why a terminal.** `dotnet new` used to run as a subprocess whose stdout
was discarded and whose stderr survived as six joined lines inside an error
string. A command a person asked for is watched, never summarised
(`docs/guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal.md`),
and this is the case that proves why: every one of the interesting failures
here — an unknown template, a template that needs a workload, an SDK that
wants a first-run licence accepted — says exactly what is wrong in words the
`dotnet` team wrote, and a dialog that reduces all of them to one line
throws the answer away.

## Boundary

The scaffolder runs on this machine's own `dotnet`; `Hick-Image` is
recorded for a containerised replay and nothing here pulls it.
`hick ingest --from '#cell'` is unchanged and still brings an exec's output
volume into a document a person is writing. There is no folder *picker*: the
location is a path typed or pasted into a field, because the app has no
native file dialog and a browsed tree of the whole filesystem is a bigger
thing than this needs. A location that does not exist yet is fine — the
scaffolder makes it — but only the `git init` button creates a folder that
has no repository above it.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hickory-cli/src/scaffold_commit.rs` — `commit_scaffold`
  (the temporary index, the trailers, the real-index `add` afterwards),
  `resolve_target` / `repository_of` / `absolute_folder` / `normalize` (the
  location anywhere on the machine, and the repository that holds it),
  `checked_output`, `refuse_occupied`, `NotARepository { path }`;
  `crates/hickory-cli/src/scaffold.rs` — `dotnet_new_argv`, `require_sdk`,
  `commit_message`, `dotnet_new_command` (`-o <folder>`, from the root);
  `run_scaffold` is deleted, since nothing runs the scaffolder outside a
  terminal any more. `crates/hickory-cli/src/serve/scaffold.rs` — `create`
  (refusals before the terminal, scratch directory, the session),
  `Watch::run` (the commit on exit zero, the verdict injected into the
  session), `result`, `preview` (which reports a missing repository rather
  than refusing), `scaffold_error` (`missing: "dotnet"`, and
  `missing: "repository"` with `path`).
  `crates/hickory-cli/src/serve/git_ops.rs` — `init`, refusing a nested
  repository. `apps/web/src/components/NewProjectDialog.tsx` — the Location
  field, the resolved path under the folder name, the repository named in
  the preview, `NoRepositoryScreen` with its button;
  `apps/web/src/App.tsx` — `watchScaffold` hands the terminal to the
  workspace and polls for the verdict.
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
  commit, the terminal handed over rather than a commit awaited, a project
  made anywhere on the machine, the repository named when it is not the open
  one, the `git init` button and the return to the form).
- Caveats: the live test skips without `dotnet`. The dialog is exercised in
  jsdom.
