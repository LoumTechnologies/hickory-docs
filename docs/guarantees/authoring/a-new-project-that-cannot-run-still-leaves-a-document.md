# A New Project Whose Scaffolder Fails Still Leaves The Document, And Says What Happened

Given File → New Project with "Run it now" chosen, when the generator does not
run — no SDK, an unknown template, a cell the executor refuses, a template
needing a feed this machine cannot reach — then the `.hick` document is still
written, holding the command, and the response carries what the run said.

Deleting the document to keep the failure tidy would throw away the part that
worked and leave the person with nothing to fix. What is written is a
complete, correct, **unrun** document: the command is in it, and pressing Run
is all that is left to try.

Parts of the guarantee:

- **`POST /api/scaffold` answers 201 either way.** `ingested` is the run's
  report or `null`; `note` is what the run said, carried whole rather than
  flattened to "scaffolding failed". The dialog reports which of the two
  happened.
- **A machine with no SDK gets its own screen, keyed off a field.** The route
  answers 422 with `missing: "dotnet"` in the body, and the dialog keys off
  that rather than off the sentence — the same lesson as
  `docs/guarantees/debugging/a-missing-debugger-is-a-button.md`, where the
  decision is made by downcasting a **type** so a reworded message cannot
  silently take a screen away.
- **It is a sentence and a link, not a button.** Unlike a debug adapter, the
  .NET SDK is not something this product has a catalogue for or any business
  fetching: it is a platform install with its own installer and its own
  licence. The screen says where to get it, and says that a running process
  keeps the PATH it started with — so an SDK installed just now is not visible
  until the window reopens, which is the mistake that otherwise gets made
  twice.
- **"New" is never a way to lose something.** A path that already exists is
  refused, and the existing file is untouched. A path that is not `.hick`, is
  empty, or escapes the folder is refused before anything is written — the
  same `new_doc_target` rules `POST /api/projects/:id/docs` uses, shared
  rather than copied.
- **A failed create leaves the form standing.** The dialog shows what went
  wrong and stays open; the answers already filled in are not lost to a
  dismissed dialog.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-cli/src/serve/scaffold.rs` — `create`'s `match outcome`
    (the document stands, the error becomes `note`), `scaffold_error`'s
    `downcast_ref::<NoDotnetSdk>` and `with_detail({"missing":"dotnet"})`.
  - `crates/hickory-cli/src/scaffold.rs` — `NoDotnetSdk` as a type, and its
    Display naming the install page and the stale-PATH cause.
  - `crates/hickory-cli/src/serve/api.rs` — `new_doc_target`, shared with
    `create_doc`.
  - `apps/web/src/components/NewProjectDialog.tsx` — `NoSdkScreen`, the
    `missing === "dotnet"` branch, and the failure line that keeps the form.
  - Tests: `crates/hickory-cli/tests/serve_scaffold.rs` —
    `a_scaffold_that_cannot_run_still_leaves_a_document`,
    `the_paths_a_document_may_not_have_are_refused`,
    `the_catalogue_answers_or_says_there_is_no_sdk`.
    `apps/web/src/components/NewProjectDialog.test.tsx` — the no-SDK screen
    keyed off the field, the other-failure screen, and the failure that keeps
    the form open.
- Caveat requiring LLM review: `note` is the run's error text, which for a
  failing cell is the executor's message. It is accurate but not always short;
  the dialog's notice shows it in full rather than truncating, on the grounds
  that a truncated cause is worse than a long one.
