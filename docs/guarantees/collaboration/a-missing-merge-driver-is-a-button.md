# A Missing Merge Driver Is A Button, Not A Command To Go And Type

Given a clone whose `.gitattributes` routes `*.hick` at the hick merge
driver but whose git config does not define it, when the app opens that
folder and shows the banner saying so, then the banner carries a **Run
hick init** button that runs `hick init` on the folder inside the engine
— the same `run_init` the CLI runs: hook, `.gitignore`, `.gitattributes`,
the driver definition, the editor and agent files — and then says what it
wrote and that `.hick` documents now merge through hick in this clone. If
it cannot (the folder is not a git work tree, git is missing), the warning
stays and the reason is beside it, in the engine's words.

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

The driver definition `hick init` writes still names a `hick` command for
git to run at merge time; a machine without the CLI on its `PATH` will
have the definition and still not be able to merge through it. That is a
separate gap, not closed here.

---

Last LLM verification:
- Date: 2026-09-05
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hickory-cli/src/serve/history.rs` `run_init`, routed
  as `POST /git/merge-driver` in `serve/mod.rs`;
  `apps/web/src/components/MergeDriverNotice.tsx` (`runInit`, the three
  states); `apps/web/src/api/client.ts` `initRepository`;
  `apps/web/src/api/types.ts` `InitOutcome`.
- Test coverage: `apps/web/src/components/MergeDriverNotice.test.tsx`
  ("the banner's button"); `crates/hickory-cli/tests/merge_driver_button.rs`
  drives the route over real HTTP on a fresh repository.
