# Local collaboration: the host is the server

*Status: design of record for `hick serve`. Adopted 2026-08-11. Extends
`architecture.md`, which already names `serve` as a `hick` subcommand and
already reaches a non-cloud machine through a PortZero tunnel.*

Someone runs `hick serve` on their own machine, gets a link, and sends it to
whoever they are working with. Those people open it and edit the document
together — live, in the same web app hickorydocs.com serves — while the
document stays a file in the host's git repository and every cell that runs,
runs on the host's hardware.

This is not a second product. It is the same React client, the same Yjs
protocol, the same `Executor` trait, and the same `.hick` files, with the
server role moved from a Fly machine to a laptop.

## Why this shape

Four problems collapse into one answer:

- **Local lineage.** The Sankey ribbon view (`SplitView`) is driven by
  `api.outputs(docId)` / `api.outputFile(docId, path)` — server endpoints keyed
  by a database row. There has been no way to see the ribbons for a `.hick`
  file on your own disk. A local server answers those endpoints from the file
  and the ribbons appear, with nothing in the client changed.
- **The downloadable CLI had no reason to exist** beyond `hick test` in CI.
  Now it is how you collaborate.
- **Execution cost and execution risk both move to the host.** Production runs
  `HICKORY_EXECUTOR=local` on a shared Fly machine, which is why open signup is
  gated behind `SIGNUP_ALLOWLIST` today: a stranger's document is code running
  as our user. When the work happens on the host's own machine, a generous free
  tier stops being an invitation.
- **A relay costs bytes, not compute.** CRDT updates are tiny. The hosted
  service's per-user cost stops being an executor minute.

## The three roles

**Host** — runs `hick serve [doc|dir]`. Owns the files, the git repo, the
executor, and the session's lifetime. Closing the laptop ends the session; that
is a real limit, not a bug (see *What hosted still sells*).

**Guest** — opens the link. No account, no install. Gets the document, the
outputs, the ribbons, live presence, and whatever the host's policy allows.

**Relay** — optional, ours. Makes the host reachable from outside its network
and nothing more: it forwards bytes and never sees a `.hick` file it is
entitled to keep, never executes anything, and holds no durable state.

## Trust model

The dangerous sentence in this design is "a guest can cause code to run on the
host's machine". It is addressed directly rather than hedged.

1. **Capability links, not accounts.** A share URL carries an unguessable
   token (128 bits) and a scope — `read`, `edit`, or `run`. There is no account
   system in local mode and there must not be one; the link *is* the
   credential, which is why the scope is narrow by default.

   **Not implemented: expiry and revocation.** A token lives exactly as long as
   the process, and the only way to revoke one is to restart the session. That
   is defensible while a session is a thing someone runs for an afternoon and
   ends with Ctrl-C, and it stops being defensible the moment sessions are
   long-lived — which is the point at which this needs a real token lifecycle.
2. **`edit` is the default; `run` is not.** A guest can change the document.
   Executing it is a separate grant, because the two have nothing in common in
   consequence.
3. **A session that grants `run` requires a sandboxing executor.**
   `LocalExecutor` runs commands as the host's user with the host's filesystem
   and network — fine for the host's own work, not fine for someone with a
   link. `hick serve --share` with `run` scope therefore **refuses to start**
   unless `HICKORY_EXECUTOR=docker` (or canopy), and says so, naming the flag.
   Refusing is the house style: a silent degrade here would be a silent grant.
4. **Writes stay inside the served root.** Every path that arrives from
   outside — a document id, or a `doc_path` carried by provenance through a
   lineage edit — is resolved and checked against the served root, so a guest
   cannot steer an edit into the host's home directory. (Note the boundary: a
   `<hick:file path="…">` block written by a *run* is placed by the pipeline,
   which has its own rules; this check covers the collaboration surface.)
5. **The host can see everything that happened.** Every run is in the
   transcript, and `HICKORY_SESSION` records tool-level work. A guest cannot act
   invisibly.

Deliberately *not* claimed: that a guest with `edit` scope is harmless. They can
write a document that the host later runs. That is the same trust as accepting
a pull request, and it is stated so nobody assumes otherwise.

## Storage: the file is the truth

The hosted server persists a room to Postgres (`docs.source` + `docs.crdt_state`,
migration 0003). Locally the same two values belong to the filesystem: the
`.hick` file, and a small sidecar for the encoded CRDT state under
`.hick-cache/`. Same debounce, same "resume, never re-seed" rule that migration
0003 exists to enforce.

That difference is the *only* thing the collaboration layer needs to abstract,
so it is the whole of the shared interface:

```rust
trait DocStore {
    async fn load_source(&self, key: &DocKey) -> Result<String>;
    async fn load_crdt(&self, key: &DocKey) -> Result<Option<Vec<u8>>>;
    async fn save(&self, key: &DocKey, source: &str, crdt: &[u8]) -> Result<()>;
}
```

The room machinery around it — the Yjs sync protocol, the stable client id, the
external-source reconciliation, the debounced persist, the broadcast — moves
into `crates/hickory-collab` and is used unchanged by both. **Two
implementations of a Yjs room is how the 49 MB document-doubling bug happens
twice.**

One subtlety the extraction must preserve: `stable_client_id` derives from the
first four bytes of the doc's UUID. Documents already have persisted CRDT state
minted under that derivation, so it stays exactly as-is for UUID-shaped keys;
non-UUID keys (local paths) hash instead.

## What a guest's browser talks to

The local server answers the subset of the API the document view actually uses,
from files:

| Endpoint | Local meaning |
| --- | --- |
| `GET /api/me` | the capability's scope, as a pseudo-user |
| `GET /api/projects`, `/projects/:id/docs` | the served directory |
| `GET /api/docs/:id`, `POST /api/docs/:id/render` | the file, woven |
| `GET /api/docs/:id/outputs`, `/outputs/:path` | the weave + lineage — the ribbons |
| `POST /api/docs/:id/run`, `/check` | the host's executor, subject to scope |
| `WS /api/ws?doc=doc:<id>&token=…` | the room, `DocStore` = files |

Ids are stable hashes of the path relative to the served root, so a link
survives a restart. Billing and analytics endpoints answer a static "not
applicable" rather than 404, because the client asks for plans on load.

## What hosted still sells

Local sharing routes around the `editors` entitlement — today `ws.rs`
`authorize()` meters distinct concurrent editors against the owner's plan, and
Team charges $12/editor beyond ten. A link that needs no account cannot be
metered that way, and pretending otherwise teaches people the product is
optional. So the hosted service sells what a laptop cannot:

- **the relay** — reachability is the one thing the host genuinely cannot
  provide itself, and it is the honest metered unit;
- **continuity** — the document outlives the laptop, keeps its URL, and is
  still there tomorrow. *Deferred, not built* — see the decision below; this
  is the reserve, the thing to build when someone asks for it;
- **identity and access control** — named collaborators, revocation, an audit
  trail, instead of a link anyone can forward;
- **CI verification and the hosted agent**, which never depended on seats.

`plans.json` needs to follow this — that is a pricing change, tracked
separately, not something `hick serve` decides.

## What shipped

All three stages, in this order:

1. **`hick serve [doc|dir]`** — loopback by default, `--share` to bind the
   LAN. Serves the client, answers the API from files, runs the rooms.
2. **Capability links** — `--scope read|edit|run`, two tokens (the host's,
   always full; the guest's, carrying the scope), one middleware in front of
   `/api`, and `ShareGuard` refusing a shared runnable session on the
   unsandboxed executor.
3. **`--public`** — a relay, through a provider seam rather than a hard
   dependency.

Two decisions worth recording, because both were arrived at by writing the
code and neither was obvious from the design:

- **The host is not a guest.** The first version applied the link's scope to
  everyone, which meant `hick serve doc.hick` could not run its own
  document. Two tokens fixed it: the scope governs the people the link was
  sent to.
- **Write permission is per *message*, not per socket.** Dropping every
  inbound Yjs frame from a read-only caller also dropped `SyncStep1` — the
  message a client sends to *ask for the document*. A read-only link that
  cannot read is not read-only; it is broken. `handle_yjs_payload` now takes
  `may_write` and drops only `SyncStep2`/`Update`. Awareness stays allowed, so
  a viewer's cursor is still visible to the people editing.

### The relay, concretely

`--public` resolves a base URL from, in order: `HICKORY_PUBLIC_URL` (any
tunnel the host already runs — Cloudflare, ngrok, Tailscale, an SSH reverse
tunnel), then `PZ_TUNNEL` resolved through `portzero url`. Neither is a
dependency: with neither present the session refuses to start and names both
paths.

This answers the open question below without settling the business one. A
first-party relay can be added as a third provider whenever it is worth
building; nothing above it changes. Note the honesty requirement in the
implementation: a `*.portzero.local` domain reaches machines on the overlay,
not the internet, and the banner says so rather than calling that link public.

## Decided: the session is ephemeral, and that is the product (2026-08-11)

**A document lives on the host's machine and the session dies with the
process.** No continuity, no permanent URL, nothing that outlives Ctrl-C. This
is accepted deliberately rather than deferred, because it is what lets the
mode stay simple: no accounts, no server-side document store, no sync service,
no "where did my document go" support surface.

What it costs, stated rather than discovered: a collaborator who joins a
session and comes back tomorrow has nothing to come back to, and the person
whose laptop it was is the only one holding the file. The mitigation is that
the file is a *file* — in a git repo, in their editor, in their normal backup
path — not a row in a database only we can read.

**Revisit when** someone wants a document to be addressable when its author is
offline, or a team asks where the shared copy lives. That is the point at
which continuity becomes a feature worth building rather than a limitation
worth explaining — and it is the thing a hosted workspace sells.

## Open questions

- **Does the guest's browser need the web assets from the host, or from
  hickorydocs.com?** Serving them from the host makes a session work with no
  internet at all; serving them from the CDN keeps the binary small and the
  client always current. Today `hick serve` serves a directory
  (`--web-dist`, `HICKORY_WEB_DIST`, or `apps/web/dist` found upwards), which
  means a *shipped binary* has no client to serve yet. Embedding it behind a
  feature flag is the next step and the reason this is still open.
- **What happens to a guest's in-flight edits when the host disappears?**
  Narrowed by the decision above: the *session* ending is intended, but a guest
  watching their last few keystrokes evaporate is a bug, not a design. Their
  CRDT state is still in the browser, so the options are "offer a download" or
  "reconnect when the host returns". Not yet decided.
- **Does the hosted pricing follow?** `plans.json` still meters `editors`,
  which a capability link routes around. Changing that is a pricing decision,
  tracked separately from this mode.
