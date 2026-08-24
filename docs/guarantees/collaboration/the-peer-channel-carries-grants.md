# The Peer Channel Admits Only Keys You Hold, And Serves Them Only What Their Grants Allow

Given a machine paired into this fleet, when it connects to this session, then
the connection is authenticated as its ed25519 public key by the transport
itself, checked against the mutual key list, and every request it makes is
gated on that machine's grants before anything reaches this session's server.

**The transport is QUIC dialled by public key** (`iroh` 1.0). The design
sketched Noise over a WebSocket; iroh is that property with an implementation
nobody here had to write, and its endpoint identity **is** an ed25519 public
key — which is what a machine already is in this product. So the fleet's key
list is the allowlist directly: a connection arrives already authenticated as a
key, and either that key is enrolled or the connection is closed.

Corollaries that are part of the guarantee:

- **An unpaired key is refused**, with a message saying a fleet is mutual —
  because pairing in one direction only is the usual cause.
- **Grants gate every request.** `view` reads, `edit` writes, `execute` runs
  code. A refusal names the grant, the machine, and the exact command that
  would give it — and says why `execute` is off, not merely that it is.
- **The live room needs `edit`, even though its upgrade is a GET.** Treating
  it as a read because of its method would hand `view` the ability to edit
  every keystroke.
- **A cell run needs `execute`, not `edit`.** It is a POST to a document and
  would read as an ordinary write if the execute routes were not matched
  first. Listing terminals is `execute` too: the route family is the boundary,
  and splitting it by method would re-create the precision the single grant
  exists to refuse.
- **A query string cannot change the verdict.**
- **Deny by default.** A route the table does not know is refused, never
  passed through — otherwise every route added later would be silently
  reachable by every paired machine without anybody deciding so.
- **Settings are unreachable over the channel at all**, whatever the grants.
  They hold provider keys and the continuity switch; the fleet channel carries
  no key material by design rather than by rule, so there is nothing to enforce
  and nothing to get wrong.
- **A refusal is a sentence, never a hang** — the same reasoning the broker
  uses for a denied host.
- **Durable state never travels over it.** Two machines editing one document
  commit and push, and git reconciles; what crosses is the live room. Close
  both sessions without committing and the machines diverge — that is correct,
  and it is stated, because it will feel like a bug the first time.

## Who is in the path, and why it is said out loud

**The default posture is number0's** — chosen deliberately on 2026-08-24 over
a direct-only default, because it is what makes a café work with no setup. Two
third-party surfaces come with it, and both are named wherever reachability is
shown rather than inherited from a preset:

1. **Traffic may transit number0's relays** when no direct path is found.
   Encrypted end to end; they can see that two keys are talking, and how much.
2. **This machine's addresses are published to number0's DNS**, which is what
   lets a peer find it by key from anywhere. That is a *publication*, not
   merely a fallback.

Neither is a server *we* run, so `local-only.md`'s refusal — and the sentence
"nothing talks to a server we run" — are intact. That sentence is doing more
work than it used to, which is exactly why the product states the rest:
`hick fleet attach` says on every connection whether it was direct or relayed,
and `HICKORY_FLEET_RELAY` takes `direct` (no relays at all) or the URL of a
relay you run yourself (`iroh-relay` is separately published).

---

Last LLM verification:
- Date: 2026-08-24
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-peer/src/grants.rs` — `required` (execute matched before
    edit and view), `permitted`, `Denial`, and the deny-by-default fallthrough.
  - `crates/hickory-peer/src/tunnel.rs` — `PeerServer::bind` (the machine's
    own secret key), `admit` (`remote_id` → the fleet list, close on an
    unpaired key), `serve_request` (gate, then proxy to loopback),
    `Attached::how` (direct beats relayed when an IP path is active),
    `Reach` and `builder_for`.
  - `crates/hickory-fleet/src/lib.rs` — `Identity::secret_bytes`, documented
    as required to be the same key the fleet list holds.
  - `crates/hickory-cli/src/main.rs` — `hick fleet serve` / `attach`, and the
    per-connection direct-or-relayed line.
  - `apps/web/src/views/FleetPane.tsx` — the reachability note beside the
    machines.
  - Tests: `crates/hickory-peer` unit tests (12) and
    `tests/peer_channel.rs` (9, over real QUIC). **Offline by construction**:
    both endpoints use `Reach::Direct` and dial by direct address, so the
    suite passes on a machine that has never had internet.
- Caveat requiring LLM review: the gate is a path-and-method table, so a route
  added to the server without a line here is refused rather than exposed —
  safe, but it will look like a bug to whoever adds the route. That is the
  deliberate trade; the alternative fails open.
- Third caveat: **`iroh` is trimmed to `default-features = false, features =
  ["tls-ring"]`, and that is load-bearing rather than tidiness.** Its default
  `portmapper` feature pulls `attohttpc`, which is MPL-2.0 — copyleft, which
  the workspace forbids outright — and it also asks the router to open a port
  by UPnP/NAT-PMP, which is reconfiguring somebody's network without being
  asked. Re-enabling that feature reintroduces both. (`option-ext`, also
  MPL-2.0, is in the tree already via `dirs` and predates this.)
- Second caveat: `hick fleet attach` is a one-request verb for proving the
  channel from a terminal. Attaching the *app* to a remote session — the fleet
  pane's machine strip, following, and queueing for a machine that is asleep —
  is steps 3 to 6 of that design and is not built.
