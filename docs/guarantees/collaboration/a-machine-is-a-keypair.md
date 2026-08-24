# A Machine Is A Keypair, A Fleet Is A Mutual List Of Keys, And Nothing Is Reachable Yet

Given one engineer with several machines, when they pair two of them, then
each machine holds the other's public key in a list under its own per-user
state directory — and there is no account, no directory, and no server
anywhere in it.

- **A machine is a keypair**, generated on first run. The private half never
  leaves the machine and is never in a repository; the public half is what
  travels.
- **A fleet is a mutual list of public keys.** Both machines accept the other,
  which is `ssh-copy-id` in each direction.
- **Revocation is deleting a public key**, and it is complete, because no
  server holds a session you cannot reach.
- **Nothing is reachable.** This is identity, not connectivity, and the tool
  says so where somebody would otherwise assume the machines can now see each
  other.

**Enrolment is SSH's ceremony without a rendezvous.** The design sketches a
short one-time code that expires in sixty seconds; a code that short cannot
carry a public key, so it assumes a rendezvous where the machines find each
other — and the only rendezvous available to two machines that cannot yet
reach each other is one we would run. `local-only.md` deletes that. So an
**invitation** is self-contained: name, public key, checksum, in one line that
works over a LAN, a screenshot, or read aloud.

Corollaries that are part of the guarantee:

- **Grants are per machine, per verb**: `view` and `edit` on, **`execute`
  off**. "My laptop was stolen" must not read as "every machine I own now
  executes whatever the thief types". Three earlier grants (`run`, `terminal`,
  `agent`) are one, because they all mean "code runs on that machine as me" and
  three switches implied a precision the security model does not have.
- **What `execute` costs is a property of the machine, not of the grant** — a
  cell under the sandbox or Docker is confined to its own workdir and a shell
  is not — and the tool says so when the grant is given.
- **A phone can never be granted `execute`.** It reads and captures; it cannot
  spawn a subprocess, so there is no executor to grant.
- **Pairing twice is idempotent and does not re-grant.** Re-pairing must not
  silently restore a grant somebody took away.
- **Two keys under one name is refused**, because a name is how you tell your
  machines apart and a key is how they are identified — two keys under one name
  makes the first indistinguishable from an impostor.
- **A damaged invitation fails loudly**, checksum-first, rather than enrolling a
  key that is not the one the other machine holds.
- **An unreadable fleet file is an empty fleet.** Nothing is reachable, which is
  the safe reading.
- **The private half is owner-only on disk** (`0600` on Unix).

Two things are deliberately **not** claimed. A peer with `edit` is not
harmless — it can write a document you later run, which is the same trust as
accepting a pull request. And a compromised machine with `execute` on an
unsandboxed peer is a full compromise of that peer: the grant model bounds the
blast radius, it does not defeat it.

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-fleet/src/lib.rs` — `Identity` (generate-once, `restrict`
    to `0600`), `Invitation` (self-contained, checksummed), `Fleet`
    (add/remove/`set_grant`, the idempotence and the name-collision refusal),
    `Grant` (`defaults()` without `Execute`), `Kind::Phone`.
  - `crates/hickory-cli/src/main.rs` — `hick fleet whoami|invite|accept|
    list|grant|remove` and what each says.
  - `crates/hickory-cli/src/serve/history.rs` — `GET /api/fleet/invite`,
    `POST /api/fleet/accept`, `PUT /api/fleet/grant`, `POST
    /api/fleet/remove`: the same ceremony from the app.
  - `apps/web/src/views/FleetPane.tsx` — pairing, granting and revoking in the
    pane; `apps/web/src/views/workspaceState.ts` — `openFleetTab`, and
    `WorkspaceView.tsx` renders it, so it is REACHABLE and not merely written.
  - `crates/hickory-peer/src/grants.rs` — `/api/fleet/…` writes are refused
    over the peer channel whatever the grants, so a peer cannot grant itself
    `execute` from inside the channel those grants bound.
  - Tests: `crates/hickory-fleet/src/lib.rs` unit tests (12);
    `crates/hickory-cli/tests/fleet.rs` (6, two state directories standing in
    for two machines).
- Caveat, corrected 2026-08-24: an earlier version of this guarantee cited
  `FleetPane` as evidence while **nothing rendered it**. Its tests passed
  because a test renders a component directly, which is the one thing a user
  cannot do. `apps/web/src/views/toolTabs.test.ts` now asserts every tool pane
  is reachable from the shell, which is the check whose absence let that
  stand.
- Superseded caveat: **the peer channel is not built.** Keys are
  enrolled and grants are recorded, and nothing consults them yet, because
  nothing connects. The design's step 2 — authenticated Noise over a WebSocket
  carrying the existing API and rooms — is the next step and is not here;
  `Identity::sign` / `verify` exist so that transport cannot be built without
  proving possession, but no transport calls them.
- Second caveat: the design says the private half lives in the platform
  keychain. It lives in a file only this user can read, which is weaker
  against a user-level compromise of the machine. Named rather than implied
  away.
