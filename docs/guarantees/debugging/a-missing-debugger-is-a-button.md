# A Missing Debugger Is A Button, Not A Command To Go And Type

Given a debug session that cannot start because this machine has no adapter
for the program's language, when hick can fetch one, then the app says so in
one short sentence and offers a control that installs it and starts the
session — never a paragraph naming a shell command.

The offer appears **only when a catalogue can actually serve it**. For a
language whose adapter comes from its own ecosystem — delve via `go install`,
`lldb-dap` from LLVM, `rdbg` from the debug gem — there is no button, and the
prose that names the ecosystem's own way is shown in full, because that is all
there is to say.

Installing goes through the same catalogue, the same confinement and the same
prefix `hick dap install` uses. A tool fetched from the app and a tool fetched
in a terminal are the same tool; this adds an affordance, never a second
mechanism.

The offer is cleared the moment a session starts. It belongs to the failure it
came from, and an install that worked must not leave "No Python debugger on
this machine" standing beside "finished — exit code 0".

## Why

This was the last place in the app that answered a fixable problem with
homework. The strip said *"no debug adapter for python on this machine.
Install one with `hick dap install python`, or…"* — and truncated it at the
width of the strip, so the actionable half of the sentence was off the end of
the control that could have performed it. Every other editor makes this a
button. The gap between "here is a command" and "yes, do that" is most of what
separates an IDE from a tool with a CLI attached.

Deciding *which* failure this is must not be done by reading English.
`hick_dap::MissingAdapter` is a type for exactly that reason: the server
downcasts to it, asks it whether the language is installable, and the client
gets a field. A message somebody rewords cannot silently take the button away.

---

Last LLM verification:
- Date: 2026-08-28
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - The type: `crates/hick-dap/src/program.rs::MissingAdapter`, returned by
    `adapter_for`, carrying `language`, the prose `how`, and `installable`
    (from `discovery::suggests_hick_install`, the same function that writes
    the sentence — so the two cannot disagree).
  - The wire: `crates/hickory-cli/src/serve/debug_bridge.rs` — `Response::Failed`
    gained `offer_install`, set by downcasting the error and filtering on
    `installable`.
  - The install: `crates/hickory-cli/src/serve/install.rs` — `POST /api/install`
    calls `crate::dap_install::install`, the same entry point `hick dap
    install` uses, on a blocking task so a download does not hold a runtime
    worker.
  - The control: `apps/web/src/debug/DebugStrip.tsx::MissingTool`, wired in
    `apps/web/src/views/workspaceTabs.tsx` to install and then re-start the
    same program.
  - Run end to end on this machine against `warehouse/50-review-surface.hick`:
    the strip showed "No Python debugger on this machine. [Install it]",
    the click produced `.hick-cache/adapters/python/` and the session then ran
    to `finished — exit code 0`.
- Test coverage: `apps/web/src/debug/DebugStrip.test.tsx` — the offer replaces
  the sentence and reports the pick; a language with no offer still shows the
  prose in full (the `delve` case). `apps/web/src/debug/useDebugger.test.ts` —
  the offer is carried out of the failure and **dropped when a session
  starts**, which is the bug the `clearFailure` pairing exists to prevent.
  `crates/hick-dap/src/program.rs::tests::a_missing_adapter_is_a_type_the_app_can_act_on`
  — the failure downcasts, names the language, and its Display is unchanged.
- Caveat requiring LLM review: the install is blocking with no progress. A
  pinned-hash download of a debug adapter takes seconds, and a progress
  protocol for that wait would cost more than it returns — but a very slow
  connection gets "Installing…" and nothing else. There is also no offer for
  a missing **language server**, which is the same gesture and is not wired.
