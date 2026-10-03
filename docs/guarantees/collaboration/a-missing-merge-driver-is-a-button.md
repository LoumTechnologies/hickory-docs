# A Missing Merge Driver Is A Button, Not A Command To Go And Type

Given a clone whose `.gitattributes` routes `*.md` at the hick merge
driver but whose git config does not define it, when the app opens that
folder and shows the banner saying so, then the banner carries a **Run
hick init** button that runs `hick init` on the folder inside the engine
— the same `run_init` the CLI runs: hook, `.gitignore`, `.gitattributes`,
the driver definition, the editor and agent files — and then says what it
wrote and that `.md` documents now merge through hick in this clone. If
it cannot (the folder is not a git work tree, git is missing), the warning
stays and the reason is beside it, in the engine's words.

When no folder is open, the app shows no merge-driver banner, even if its
internal session storage sits inside a git repository. The status route
reports no applicable repository, and the init route refuses with a request
to open a folder before writing anything. A single-file window also waits
for an explicitly open folder before offering repository setup.

Until this the banner ended with "Next step: run `hick init` in this
repository", which is exactly the kind of sentence
`debugging/a-missing-debugger-is-a-button.md` forbids: a fixable failure
told as a command to go and type. It was also the wrong command for a
desktop user, who has no `hick` on their `PATH` — the app and the CLI are
separate downloads — so the sentence could not even be followed.

Two properties hold it up:

1. **The engine does it, not a shell.** `POST /api/git/merge-driver`
   (`serve/history.rs` `run_init`) calls `crate::init::run_init` on a
   blocking thread and answers with what changed and the re-read status.
   No terminal, no `hick` binary, no `PATH`.
2. **The button says what happened.** Success replaces the warning with
   the list of what was written; failure keeps the warning and appends the
   error; the button is disabled while it runs so a double click is one
   init, which is idempotent anyway.

## Boundary

The driver definition names the executable `hick init` ran from — from
the app, the app's own binary, whose `main` answers `merge-driver` and
`merge-generated` before opening any window
(`hickory_cli::merge_driver::run_argv`). From an AppImage that path is the
image itself (`APPIMAGE`), never the `/tmp/.mount_*` directory that
exists only while that launch is running — which is what the first
version wrote, and which git would have reported as a command not found
the day after.

---

Last LLM verification:
- Date: 2026-10-03
- Reviewer: Codex
- Result: verified
- Evidence: `crates/hickory-cli/src/serve/history.rs` `run_init`, routed
  as `POST /git/merge-driver` in `serve/mod.rs`;
  `apps/web/src/components/MergeDriverNotice.tsx` (`runInit`, the three
  states); `apps/web/src/api/client.ts` `initRepository`;
  `apps/web/src/api/types.ts` `InitOutcome`;
  `crates/hickory-cli/src/merge_driver.rs` `driver_exe` and `run_argv`
  with `driver_definition_tests`; `apps/desktop/src-tauri/src/main.rs`
  dispatching the two git verbs.
- Test coverage: `apps/web/src/components/MergeDriverNotice.test.tsx`
  ("the banner's button"); `crates/hickory-cli/tests/merge_driver_button.rs`
  drives the route over real HTTP on a fresh repository.
- Folderless coverage: `merge_driver_button::a_folderless_session_neither_checks_nor_initializes_its_storage_repository`
  starts real HTTP over internal storage nested in a repository with a defined
  driver and missing attributes, checks the quiet status and refused init, and
  verifies no setup files were written. `WorkspaceView` mounts the notice only
  when `folderOpen` is true; `App.test.tsx` verifies a folderless startup editor
  can open Agent without querying merge-driver setup or showing its button.
