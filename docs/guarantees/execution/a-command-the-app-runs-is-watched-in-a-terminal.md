# A Command The App Runs On Your Behalf Is Watched In A Terminal

Given a button, menu item or gutter mark in the app whose job is to run an
**external command line program** on this machine — `dotnet new`, `cargo
build`, `pytest`, `dotnet test`, `go test` — when a person triggers it, then
that program runs in a terminal session they can read, scroll and keep, and
its stdout and stderr are the program's own, byte for byte. The app reports
*around* the output; it never reports *instead of* it.

Concretely:

* The route answers with the **session**, not the result — the way `POST
  /api/tests/run` and `POST /api/scaffold` both do — so the work is
  watchable rather than finished, and the client opens a tab for it
  (`showTerminalRequest`).
* The session is **named after the task** ("New project: Greeter", "test:
  adds"), never numbered, so the tab is something a person can point at.
* Whatever the app does *after* the program exits — commit the scaffold,
  refresh the tree — announces itself **in that session's own scrollback**,
  by injection, so the act and the command that earned it are one thing to
  read.
* Anything the app can decide **before** the program starts is refused
  before the terminal exists. A tab that opens only to say "that folder is
  not empty" is a tab the person has to close for a mistake a field could
  have caught.
* A notice elsewhere in the app says at most one sentence and points at the
  terminal. It never paraphrases a failure the terminal already explained.

**Why.** A failing external program says why in its own words, in its own
formatting, with its own next steps — and the app has no idea what any of
that means. Reducing it to a status line is not summarising, it is deleting.
This is the same argument that made a build a watched terminal rather than a
spinner (`docs/specs/freeform/launching-what-a-document-builds.md`) and a
test run a terminal rather than a pass/fail count
(`a-test-runs-from-the-line-it-is-written-on.md`); it is stated here as the
general rule because it kept being re-derived per feature, and because New
Project was built without it and cost a real debugging session as a result.

**The failure that named this rule (2026-09-03).** File → New Project
answered "Unprocessable Entity" — the HTTP status line, with no field, no
value and no route in it. Two independent things had to be wrong for that
sentence to reach a person, and both are now fixed:

1. `dotnet new` ran as a subprocess whose stdout was discarded and whose
   stderr survived only as six joined lines inside an error string. Nothing
   the SDK said could reach the screen.
2. The web client read only `{"error": …}` out of a failed response and fell
   back to `res.statusText` for anything else — so axum's own extractor
   rejection, `text/plain` and naming the exact missing field, was thrown
   away in favour of two words that name nothing.

## Boundary

This is about **external programs run on the person's behalf**. It does not
apply to the app's own work — a weave, a render, a save — which has no
stdout and whose failures the app does understand and should state plainly.

A terminal opened this way is a **watching** terminal: it has no input path
of its own, and the transcript is not a record of anything
(`docs/specs/freeform/a-terminal-that-writes-the-document.md` — nothing is
ever verified against what a terminal showed). Two things are still true of
every one of them: an anchored terminal is a different thing, and a
person's own shell session is a third.

Not every such command is converted yet. `hick lsp install` / `hick dap
install` behind `POST /api/install`, and the git pane's own commands, still
report rather than watch — git's case is the weaker one, since a git
refusal is one line the pane already shows verbatim.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Opus 5)
- Result: verified for the two surfaces named; the boundary lists what is
  not converted
- Evidence: `crates/hickory-cli/src/serve/scaffold.rs` — `create` opens a
  `hick_term` session and answers `202` with its summary, `Watch::run`
  waits for the exit, commits on zero and `say`s the verdict into the
  session by `Session::inject`, `result` is how the app (not the person)
  learns the outcome. `crates/hickory-cli/src/serve/test_run.rs` — `run`,
  the same shape, and the module comment that first made the argument.
  `crates/hick-term/src/session.rs` — `inject` ("say something IN the
  terminal without saying it TO the shell"). `apps/web/src/App.tsx` —
  `watchScaffold` calls `showTerminalRequest` and then says one sentence.
  `apps/web/src/api/client.ts` — `apiError`, which reads a JSON refusal,
  then the body's own text, and only then the status line.
- Test coverage: `crates/hickory-cli/tests/serve_scaffold.rs`
  (`a_new_project_is_a_commit_holding_exactly_what_dotnet_wrote` — `202`
  with a session, then the commit after `settle`;
  `a_scaffold_that_cannot_run_commits_nothing_and_leaves_no_folder` — an
  unknown template fails in the terminal and commits nothing;
  `the_folders_a_scaffold_may_not_have_are_refused` — the refusals that
  happen before a terminal exists). `apps/web/src/api/client.test.ts` (a
  plain-text error body is read rather than replaced by the status line).
  `apps/web/src/components/NewProjectDialog.test.tsx`
  (`hands over the terminal it started, and closes`). The injected verdict
  is asserted against the live session: `a_new_project_is_a_commit_…` reads
  `GET /api/terminals` after the commit and finds the verdict as that
  session's last line.
- Caveats: nothing asserts that the verdict *renders* in the pane — that is
  xterm's job and is not covered here. The claim that a person can
  scroll and keep the session is the terminal dock's behaviour, tested
  elsewhere. The live scaffold tests skip without `dotnet`.
