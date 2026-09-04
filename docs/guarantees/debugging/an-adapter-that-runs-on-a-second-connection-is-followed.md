# An Adapter That Runs On A Second Connection Is Followed

Given an adapter that launches a supervisor rather than a program — js-debug
today — when a session starts, then hick answers its `startDebugging` request,
opens a **sibling connection to the same server**, and runs the real session
there: breakpoints, stack, values and stepping all belong to the child. The
supervising connection is held open for the life of the session, because it
owns the server process, and is shut down after the child.

Hick declares `supportsStartDebuggingRequest` because it is now true.

The child's launch arguments are the `configuration` the adapter handed back,
**unchanged** — it carries `__pendingTargetId`, the adapter's own name for the
target, and inventing any part of it attaches the child to nothing.

Only adapters that say they need this pay for it. `multi_session` is declared
per adapter beside `transport` and the launch keys, so no other language waits
for a request that will never come.

## Why

Measured on 2026-09-04: js-debug answers `launch`, returns the breakpoint as
`verified: false, "breakpoint.provisionalBreakpoint"`, and then sends a
`startDebugging` **reverse request**. The connection that launched runs
nothing. A client that ignores it — which hick did, turning every reverse
request into an event nobody answered — attaches to a program and never stops
in it, which is worse than refusing outright.

That is why JavaScript and TypeScript were reported as **not** debuggable for
a day between
`an-adapter-that-listens-is-connected-to.md` and this: the TCP transport made
js-debug reachable and not driveable, and a debugger that never stops is not a
debugger.

Two things this needed that the transport work did not. DAP is
**bidirectional**, and an adapter that asks something and is never answered
simply stops — so `Adapter::respond` had to exist. And the sibling shares the
server's process rather than owning one, which is what `Adapter::sibling`
means and why the process handle stays with the connection that spawned it:
killing a server twice is not better than killing it once.

## What TypeScript actually rests on

A raw `.ts` file is run by **node itself**. Type stripping is on by default
from v23.6; before that node cannot execute TypeScript without a loader, and
there is nothing for a debugger to attach to. So the claim for TypeScript is
narrower than the one for JavaScript, and the live test skips loudly with the
node version rather than failing — an old node is not hick being wrong. A
document that generates JavaScript has no such limit.

---

Last LLM verification:
- Date: 2026-09-04
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hick-dap/src/adapter.rs` — `Adapter::sibling` (a second
  connection to the same port, with no process of its own) and
  `Adapter::respond`; `crates/hick-dap/src/session.rs` —
  `wait_for_start_debugging` (which answers the request and returns the
  configuration untouched), `Session::start_on` and its one level of
  recursion, `Session::parent`, and a `shutdown` that closes the child first;
  `crates/hick-dap/src/discovery.rs` — `Candidate::multi_session`, true only
  for js-debug; `crates/hickory-cli/src/dap_install.rs` — the js-debug
  archive, pinned, since the npm package it used to name does not exist.
- Test coverage: `crates/hick-dap/tests/live_session_js.rs` and
  `live_session_ts.rs`, which stop on a document line, assert the frame is
  `lineTotal`, and read `quantity == 3` out of the real frame — both assert
  the adapter really is multi-session first, so the test cannot quietly pass
  on a single-session path. All seven live languages pass together on this
  machine: Python, C, C#, Go, JavaScript, Rust, TypeScript.
- Caveat requiring LLM review: exactly **one** child is followed, which is
  what a Node launch produces. A target that spawns further targets — a
  worker, a browser tab, a child process — sends more `startDebugging`
  requests, and those are answered and then ignored, so their stops are not
  seen. That is a real limit of this design rather than an oversight: a
  session tree with several live children needs the UI to have a notion of
  which one is focused, and nothing in the app has that yet. Browser
  debugging (`pwa-chrome`) is not offered at all.
