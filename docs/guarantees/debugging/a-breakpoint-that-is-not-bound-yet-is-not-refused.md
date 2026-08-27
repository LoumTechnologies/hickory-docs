# A Breakpoint That Is Not Bound Yet Is Not Refused

Given a breakpoint set on a document line, when the debugger reports on it,
then it has **three** states rather than two, because "the adapter has not
confirmed this" and "there is nothing here to stop on" are different facts and
a person acts differently on each:

- **bound** — the adapter confirmed it. The program will stop here. Drawn as a
  solid dot.
- **pending** — not confirmed yet. Normal for a compiled language, where
  nothing can bind until the module is loaded. Drawn half-filled, and promoted
  to *bound* the moment the adapter's `breakpoint` event says so.
- **refused** — this document line maps to no generated code at all. Drawn
  hollow with a cross and the reason on hover.

**The only refusal hick makes is its own.** A `verified: false` from an
adapter means "not confirmed", never "never will be" — DAP does not
distinguish them, and reading an adapter's human-readable `message` to guess
is refused, because that string belongs to another project. So the certain
refusal is the one made here, before any adapter is asked: a line that maps to
prose.

Everything that used to branch on a bare `verified` now branches on a
refusal, and each of those was the same bug: **run to here** refused to run
to a line that was merely unconfirmed, `<hick:capture>` reported "the debugger
would not place a breakpoint" about every capture in a working C# document,
and the MCP tool told an agent `NOT BOUND` about a breakpoint that was about
to work. A capture now asks *after* the run, where "still not bound" is a fact
rather than a guess.

---

Last LLM verification:
- Date: 2026-08-27
- Reviewer: Claude (Opus 5)
- Result: verified against two real adapters that behave differently
- Evidence:
  - `BindState` and its promotion in `crates/hick-dap/src/session.rs`: the
    event pump handles `"breakpoint"`, matches it to a document line by the
    adapter's own breakpoint **id** (recorded at set time, because an adapter
    may have slid the breakpoint to another line), and refuses to promote a
    status that hick itself refused.
  - `Session::breakpoint_statuses()` is what callers read after the program is
    running; the set-time return value is only the first answer.
  - `crates/hickory-cli/src/serve/debug_bridge.rs` pushes a fresh
    `breakpoints` response on every stop, so the gutter corrects itself
    without the client asking.
  - The gutter's third mark is `.cm-bp-pending` in `apps/web/src/styles.css`;
    `apps/web/src/debug/cmDebug.ts` chooses it.
- Test coverage:
  - `crates/hick-dap/tests/live_session_csharp.rs` — against **real
    netcoredbg**: the breakpoint is `Pending` at set time (asserted, so this
    fails loudly if netcoredbg ever confirms synchronously and the promotion
    becomes dead code), the program stops on it, and
    `breakpoint_statuses()` then reports `Bound`.
  - `crates/hick-dap/tests/live_session.rs` — against **real debugpy**: the
    same breakpoint is `Bound` at set time, and a prose line is `Refused`.
    The two adapters disagreeing is the whole reason for the enum, and both
    halves are observed rather than reasoned about.
  - `apps/web/src/debug/cmDebug.test.ts` — bound, pending and refused draw
    differently, and pending does not wear the refused mark.
  - `crates/hickory-cli/tests/debug_desktop.rs` — the wire carries `"bound"`
    for a Python breakpoint and `"refused"` for a prose line.
- Caveat requiring review:
  - **A breakpoint that never binds now says "pending" forever under an
    adapter that verifies lazily.** Nothing distinguishes "the module has not
    loaded yet" from "this module will never load", so a typo'd line in a C#
    document stays half-filled rather than becoming a cross. That is a
    weaker statement than before for an adapter that binds lazily, and a
    truer one — but a person looking for a breakpoint that never fires gets
    less help than they would from debugpy. `<hick:capture>` closes this
    after the run; the interactive gutter does not, because there is no
    moment at which the app currently decides a session is "past the point
    where this should have bound".
  - **The promotion is only observed for `reason: "changed"`-shaped events
    carrying an `id`.** An adapter that reports verification without an id,
    or only by source and line, is not handled; nothing in this test set does
    that, so it is untested rather than known to work.
  - **Only two adapters have been driven.** debugpy and netcoredbg. delve,
    codelldb, lldb-dap and rdbg are assumed to fall into one of the two
    behaviours, and that is an assumption.
