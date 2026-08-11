# Local-first: the product is the binary, the cloud is a relay

*Status: design of record for the product's shape. Adopted 2026-08-11.
**Supersedes** the hosted-workspace parts of `architecture.md` — see
[What this supersedes](#what-this-supersedes). Builds on
`local-collaboration.md`, which is the mechanism this decision is made of.*

Hickory Docs is a program you install. It runs on your machine, edits files in
your repository, executes on your hardware, and shares a live session by
handing someone a link. There is no hosted workspace: no server-side copy of
your documents, no account to make, no execution minutes to buy.

The cloud shrinks to three things that a laptop genuinely cannot do:

| | What it is | Why it cannot be local |
|---|---|---|
| **The relay** | Forwards bytes so a session on a laptop is reachable from outside its network | A machine behind NAT has no public address |
| **Billing** | Stripe checkout + webhook, if money changes hands | A webhook needs a durable public URL |
| **The site** | Marketing, demos, docs, downloads | Static files; not a server at all |

Everything else that runs today — Postgres, the git store, cloud execution, the
hosted agent, accounts, email verification — is not part of the product.

## Why

Four facts, none of them speculative, all of them already in this repo:

1. **`hickory serve` covers the hosted workspace.** The same client, the same
   Yjs protocol, the same rooms, with the file standing in for the database
   (`local-collaboration.md`). The one thing it does *better* is lineage: a
   local weave is computed from the file in front of you, while the hosted one
   serves the last successful run.
2. **Hosting other people's execution is the expensive, dangerous half.**
   Production runs `HICKORY_EXECUTOR=local` on a shared Fly machine, which is
   why signup sits behind `SIGNUP_ALLOWLIST`: a stranger's document is code
   running as our user. Moving execution to the author's machine deletes that
   whole category — the cost *and* the risk *and* the reason the front door is
   locked.
3. **The artifact was always a file.** `.hick` documents live in the user's git
   repository; the hosted side mirrored them into Postgres and committed them
   to a server-side git store. That mirror was scaffolding around the real
   thing.
4. **A relay costs bytes.** CRDT updates are tiny. The per-user cost of the
   cloud stops being an executor minute and becomes a websocket.

## What this supersedes

`architecture.md` remains correct about the language, the crates, the execution
boundary, and native verification. These parts of it are superseded:

- **Product architecture** — `apps/server` is no longer the product's backend.
  It is a hosted workspace that exists for as long as it is useful and is not
  built on further.
- **State model** — "Postgres holds accounts, workspaces, run metadata,
  billing" describes the hosted deployment only. In the product, the git repo
  of `.hick` files is the *entire* state model; Yrs CRDT docs are the live
  layer within one session.
- **Delivery** — "continuous deployment to production" describes the hosted
  service. The product's delivery path is the release channels
  (`unstable-release.yml`, `stable-release.yml`) and the one-line installer.
- **Sharing state across web/iOS/Android** — still true of the client, but the
  server it talks to is now usually `hickory serve` on someone's machine.

`architecture.md` gets a pointer at the top rather than a rewrite: it is an
accurate record of how the system was built, and this document is what the
system is for.

## What survives untouched

The hick language and every vendored crate. The `Executor` trait and its
implementations. `hickory-collab` (that is the point — one room
implementation, two hosts). `hickory-cli` in full: `run`, `test`, `weave`,
`lineage`, `promote`, `agent`, `refresh`, `init`, `doc`, `mcp`, `serve`. The
agent, including the bring-your-own-agent tool surface. The pre-commit drift
gate. `hick-lsp`.

Which is to say: everything a user touches.

## What retires, and when

**Nothing is deleted today.** hickorydocs.com stays up while the relay is
built, because it costs a Fly machine and it is the escape hatch if local-first
turns out to need one. Retirement is a later, separate act — but the list is
written now so nobody keeps building on it:

| Retires | Why |
|---|---|
| `apps/server` REST: projects, docs, render, outputs, runs, checks | `hickory serve` answers all of it, from files |
| The WS handler, `output_rooms`, `EditorTracker` | Rooms moved to `hickory-collab`; seats stop existing |
| Postgres and every migration | No server-side documents, accounts, or runs |
| The git store (`gitstore.rs`, the Fly volume) | The user's own repo is the store |
| Cloud execution + `hickory-executor-canopy` in the server | Execution is the author's machine |
| The hosted agent route | `hickory agent` and `hickory mcp`, with the user's own key |
| Auth: signup, login, verification, password reset | No accounts |
| Server-side BYOK (`byok.rs`, `keyvault.rs`, `routes/llm_keys.rs`, migration 0007, `SettingsView`) | Built 2026-08-11 to let the *hosted* agent spend a user's key; the CLI reads the key from the environment and never needed it |

That last row is worth stating plainly rather than discovering later: a few
hours of work this morning becomes dead code under this decision. The provider
selection in `hickory-agent` survives, because the CLI uses it; the storage
does not.

`hick-grove`, `hick-store`, `hick-token`, `hick-classify`, `hick-sink` are
untouched here — they belong to the language and security stacks, not to the
hosted deployment.

## The consequence nobody will like: pricing has nothing left to meter

Every entitlement in `plans.json` today measures something the cloud does:

| Entitlement | Under local-first |
|---|---|
| `private_projects` | Meaningless — they are directories in your repo |
| `editors` | Meaningless — a capability link has no seats |
| `exec_minutes_month` | Meaningless — your CPU |
| `ci_verification` | Free — it is `hickory test` in *your* CI |
| `agent` (`byo_key` / `metered_allowance`) | Your key, your machine |
| `sso`, `review_workflow`, `priority_execution`, `byon` | Attached to a workspace that no longer exists |

So the pricing model has to be rebuilt around what remains meterable: **relay
usage** (sessions, participants, or connected minutes), and whatever a
**licence** is worth to a team that wants support, private builds, or an
indemnity. That is a real piece of work and it is not this document's job —
but pretending the current grid survives would be worse than saying it does
not.

Until it is rebuilt, the honest interim is: the tool is free, the relay is
free while it is small enough not to matter, and nothing on the pricing page
claims otherwise.

## What gets better

- **The front door unlocks.** `SIGNUP_ALLOWLIST` exists because production
  executes strangers' code. With no hosted execution there is nothing to gate:
  the call to action becomes `curl … | sh`, and the interactive demos on the
  landing page — which already run entirely in the browser, with no API calls —
  do the selling.
- **The download becomes the product**, so the release channels stop being
  infrastructure nobody uses.
- **The lineage ribbons work on your own files**, which was impossible while
  the split view needed a database row.
- **One less environment to be wrong about.** No staging/production analytics
  split to get right, no `SIGNUP_ALLOWLIST` to remember, no Postgres backup to
  test.

## Prerequisites and open edges

- **The repository must go public** for the installer to work for anyone who
  is not us: `scripts/install.sh` currently needs `HICKORY_GITHUB_TOKEN`
  against a private repo. `architecture.md` already plans MIT.
- **A shipped binary carries no web client yet.** `hickory serve` finds
  `apps/web/dist` in a checkout; a release build has nothing to serve.
  Embedding the client is a prerequisite for local-first being usable by
  anyone who did not clone the repo.
- **The relay needs identity from day one.** A relay that forwards arbitrary
  traffic to a laptop, unauthenticated and unmetered, is a free tunnel service
  that will be used for things that are not hickory documents. Retrofitting
  abuse controls onto a live open relay is misery; a licence key or an OAuth
  identity at the door is not.
- **Sessions are ephemeral by decision** (`local-collaboration.md`): a document
  dies with the process, and continuity is the reserve — the thing to build if
  someone asks where the shared copy lives.

## Sequence

1. **This document.** Stops the next session rebuilding toward a hosted
   workspace.
2. **The static site.** Analytics straight to PostHog (the server proxy existed
   to avoid baking one environment's key into a promotable image; with one
   environment there is nothing to promote), pricing from an embedded
   `plans.json`, and a download call to action. No server.
3. **The relay**, with identity attached from the start.

Then, and only then, the retirement list above becomes a deletion.

## What would reverse this

Any of these, and the hosted workspace stops being scaffolding and becomes the
product again:

- People want to collaborate on a document whose author is offline, and say so
  more than once.
- The install step turns out to be where the funnel dies — a browser workspace
  has no install step, and that is its whole advantage.
- The relay's identity and abuse surface grows until it is indistinguishable
  from running a hosted service anyway, at which point hosting the documents
  too is nearly free.
- Someone will pay materially more for a hosted workspace than for a local
  tool, which is a pricing discovery, not an architecture one.

None of these is unlikely. This decision is reversible *because* nothing is
being deleted yet — which is the reason the retirement list has a sequence
attached rather than a date.
