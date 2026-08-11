# A Shared Session Cannot Run A Guest's Code As The Host

Given `hickory serve --share`, when the link's scope includes `run` and the
executor is the unsandboxed `LocalExecutor`, then the session **refuses to
start**, naming both ways out. It does not warn and continue.

Three rules together, each of which is load-bearing:

1. **The host is not a guest.** A session carries two credentials: the host's,
   printed on their console and always full capability, and the guest's, which
   carries the link's scope. Conflating them would mean `hickory serve doc.hick`
   could not run its own document — which was the first version of this, and it
   was wrong.
2. **Running is granted separately from editing.** `--scope edit` is the
   default for a shared session, because editing together is the point.
   Executing is code on someone else's machine, so it needs `--scope run` said
   out loud; a guest holding an editing link is refused with a message naming
   what to ask for.
3. **Refusal, not a warning.** A warning printed above a URL that works anyway
   is a grant. `LocalExecutor` runs commands as the host's user, with the
   host's files and network, so a shared runnable session on it is a shell on
   that machine handed to whoever the link reaches next.

The check is on *sharing*, not on the executor: the unsandboxed executor is
entirely correct for a session someone runs alone, which is the common case.

Two further boundaries the implementation holds:

- **The API is closed without a token.** Every `/api` route sits behind one
  middleware that resolves a presented token into a capability; no token, no
  handler. Static assets are public, because the browser must fetch the bundle
  before it has a credential and that bundle is the same artifact
  hickorydocs.com serves.
- **Writes cannot escape the served directory.** A path arriving through
  provenance is resolved and checked against the root, so a guest cannot steer
  a lineage edit into the host's home directory.

## What this does not claim

**Tokens do not expire and cannot be revoked** without restarting the session:
a link lives as long as the process. A session someone runs for an afternoon
and ends with Ctrl-C is the case this was built for; a long-lived one needs a
real token lifecycle before it deserves to be trusted.

A guest with `edit` scope is not harmless. They can write a document that the
host later runs — the same trust as accepting a pull request. Stated here so
nobody reads the sandbox rule as more than it is.

---

Last LLM verification:
- Date: 2026-08-11
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/serve/share.rs` — `ShareGuard::enforce`
  refuses `shared && scope.can_run() && ExecutorChoice::Local` with a message
  naming `HICKORY_EXECUTOR=docker` and `--scope edit`; it is called from
  `serve::prepare`, so a caller that bypasses the CLI cannot bypass the guard.
  `Caller::host()` / `Caller::guest(scope)` and `LocalState::caller_for` give
  the host full capability and the link its scope; `share::authorize` is the
  single middleware in front of `/api`; `serve/api.rs::require_edit` /
  `require_run` are the only scope checks, and `serve/socket.rs` drops inbound
  Yjs updates from a read-only caller. `LocalState::resolve_doc_path` refuses a
  path outside the served root.
- Test coverage: `crates/hickory-cli/tests/serve_local.rs` —
  `a_guest_cannot_run_unless_the_link_says_so_and_the_host_always_can`,
  `a_read_only_link_can_watch_but_not_write` (both the socket and the REST
  path), `the_api_is_closed_to_anyone_without_the_link` (including that an
  unauthenticated WebSocket is never upgraded), and
  `preparing_a_shared_runnable_session_on_the_local_executor_fails`. Unit tests
  in `share.rs` cover the scope lattice and the guard's three permitted
  combinations.
