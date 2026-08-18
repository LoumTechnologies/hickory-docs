# Local-only: a downloadable tool, no cloud, no money

*Status: design of record for the product's shape. Adopted 2026-08-12.
**Partly superseded** by `notes-ide.md` (2026-08-18) on exactly two points: the
statement of purpose below ("literate programming where you can edit the
generated files") is widened to a note-taking IDE whose notes are `.hick`
documents, and the iOS/Android row in the retirement table is reversed. This
document remains the design of record for the product's **shape** — no server,
no account, no relay, no money, no telemetry, one engine behind two front
doors — none of which `notes-ide.md` reopens.
**Supersedes** `local-first.md`, which kept a relay, a pricing model, and a
React client shared with a hosted server. All three are gone. `local-first.md`
remains an accurate record of the step between the hosted product and this one;
read it for the reasoning that killed the hosted workspace, not for what the
product is now.*

Hickory Docs is a program you download. It runs on your machine, edits files in
your repository, and executes on your hardware. There is no server, no account,
no relay, no subscription, and nothing to buy.

What it is for, stated as narrowly as it deserves — and **since widened by
`notes-ide.md`, which makes this the mechanism rather than the pitch**:
**literate programming where you can edit the generated files.** You write a `.hick` document that assembles
`analysis.py` and runs it. You open `analysis.py` in whatever editor you
already use, change it, and the change lands in the document byte-exactly. The
document stays the source of truth; the generated file is a working surface
onto it.

Everything else in this repository exists to make that sentence true, or is
being deleted.

## Two front doors, one engine

| | What it is | Who it is for |
|---|---|---|
| **`hick up`** | Headless. Weaves a folder and keeps it woven; edits saved in generated files land in their documents. Terminal output only | Anyone who already has an editor and wants their files to behave |
| **The desktop app** | The same engine with a window: lineage ribbons, transcripts, run buttons | Anyone who wants to see where a byte came from |

They are the same engine. The desktop app links it as a library and runs the
local server in-process on loopback. One code path for weaving, one for
lineage, one for carrying an output edit back into a document — not a CLI
implementation and a GUI implementation that drift.

**The CLI never serves HTML.** `serve::prepare` hands back a `Router` carrying
only `/api` routes; the desktop crate layers the built UI onto it and points
its window at `http://127.0.0.1:<port>`. Serving both from one origin is what
lets the frontend keep using relative `fetch` paths and a `location.host`
WebSocket URL — no CORS policy, and nothing that has to be told which port the
server landed on.

This also disposes of what `local-first.md` called a *blocking prerequisite*:
embedding the web client into the CLI binary so a downloaded `hick` has
something to serve. The CLI has no UI, so there is nothing to embed. The
desktop binary embeds its own with `rust-embed`, which is a property of that
binary rather than a problem for the whole product.

The desktop app takes the same directory lock `hick up` takes. Opening a folder
in the app while `hick up` watches it is refused, for the reason it has always
been refused: two processes writing the same files, only one of which knows
which writes are its own.

## Why no cloud at all

`local-first.md` shrank the cloud to three things: a relay, billing, and a
static site. Two of them are now gone, and the reasoning is the reasoning that
document already contained, followed one step further.

1. **The relay existed to make a laptop reachable. Nothing needs to reach a
   laptop any more.** The relay was there for shared sessions; shared sessions
   were the thing a solo author did not need, and `hick up` covers the solo
   author completely. Collaboration is a feature this product does not have.
2. **The relay was the only meterable thing left.** `local-first.md` worked out
   that every entitlement in `plans.json` measures something the cloud does —
   private projects, editors, exec minutes, CI verification — and that all of
   them become meaningless once execution is on the author's machine. It
   proposed rebuilding pricing around relay usage. Delete the relay and there
   is nothing left to meter, which is the honest end of that argument rather
   than a new one.
3. **A relay with no identity is a free tunnel service.** `local-first.md`
   listed "the relay needs identity from day one" as a prerequisite. Identity
   means accounts, accounts mean a database, and a database means the hosted
   product coming back through a side door.
4. **No money means no compliance surface.** No card data, no subscription
   lifecycle, no dunning, no tax, no refunds, no sandbox/live key mismatch that
   charges someone real money.

The static site survives, because a downloadable tool needs somewhere to be
downloaded from. It is static files and a link to GitHub releases — no pricing
table, no signup, no server.

## Why the React client stops being shared

Today `apps/web` is one codebase serving two masters: the hosted server's
client and the local session's client. That is why `api.rs` answers
`/billing/plans`, `/analytics/capture`, and `/me` with static stubs — the
client asks for them on load, and a 404 would look like a broken deploy.

Those three routes are the whole tell. They exist because the client was
written for a product this no longer is.

So the React codebase becomes the desktop app's frontend and nothing else. The
three stub routes go, along with every account view — login, signup, verify,
reset, forgot, settings — which existed to talk to a server that no longer
exists.

The consequence worth naming: **the HTTP API stops being a contract and becomes
an implementation detail.** `docs/specs/freeform/api.md` currently pins a wire
shape because two independent hosts had to agree on it. With one host, compiled
and shipped together with its only client, the API can change in the same commit
as the code that calls it. That is a real simplification and it should be taken,
not preserved out of habit.

### One source tree, two builds

`apps/web` stays a single package with two entry points rather than becoming
two packages:

* `index.html` → `dist/` — the editor. Tauri bundles it.
* `site.html` → `dist-site/` — the marketing page. Deployed to
  hickorydocs.com.

The split is by *entry point*, not by package, because the demos on the
marketing page are live demos of the editor: they import the same CodeMirror
configuration, the same ribbon geometry, and the same diff code the app uses.
Forcing a package boundary between them would invent a shared library to hold
code that is genuinely one thing, and the boundary would have to be crossed on
every change to either side.

What is *not* shared is what each build reaches for at runtime. The site has no
API client, no router beyond its own page, and no knowledge that a server
exists. That is the property "there shouldn't be a shared React client"
actually asks for — not that the two never compile from the same directory.

## The CRDT survives, and it is not obvious why

With no collaboration, the reflex is to delete `hickory-collab`, `hick-grove`,
and the Yjs machinery. That would be wrong.

Concurrency does not require two people. It requires two writers, and this
product has two by design: the desktop app's editor buffer, and the file on
disk that `hick up` and your other editor are both touching. Someone editing a
document in the app while a formatter rewrites the file underneath is the same
merge problem as two collaborators, with the same correct answer.

`hickory-collab` already reconciles an external source into a live room, and
its `DocStore` already treats the file as the durable state. That is exactly
the mechanism the single-user case needs. What goes is the *sharing* around it:
capability links, scopes, seats, awareness of other humans.

Which resolves the open problem from the `hick up` work — reverse edits go
straight to the `.hick` file, so a live editor buffer would silently diverge
from disk — but only where the problem exists:

* **`hick up` alone runs no rooms.** There is no editor buffer to reconcile
  with, so a reverse edit writes the file directly. That is what is built today
  and it is correct for a headless loop.
* **The desktop app runs the up-loop and the rooms in one process.** There a
  reverse edit goes into the room, and the room owns the file — because there
  *is* a second writer, and it is the window the user is looking at.

So the CRDT is not a layer everything pays for. It appears exactly when a
second writer does.

## What retires

`local-first.md` wrote a retirement list and deliberately deferred it. This
document executes it, and adds to it.

| Retires | Why |
|---|---|
| `apps/server` in full — routes, Postgres, 7 migrations, git store, auth, BYOK, LSP bridge, render cache | There is no hosted workspace. `hick up` answers everything a document view needs, from files |
| `apps/relay` and `hickory-relay` | Nothing needs to reach a laptop |
| `hickory-identity` | Existed for relay auth. No accounts, no identity |
| `serve/relay.rs`, `serve/tunnel.rs` | The laptop's half of a tunnel that no longer exists |
| `serve/share.rs` — capability links, scopes | Nothing is shared |
| `hick login`, `hick logout`, `hick whoami` | The only thing an account bought was a relay tunnel |
| The `hick serve` **command** | The local server stops being a CLI entry point and becomes a library the desktop app links. Same code, no command |
| `/billing/plans`, `/analytics/capture`, `/me` | Written for a client this no longer has |
| `plans.json`, `routes/billing.rs`, `plans.rs`, the Stripe integration | Nothing is sold |
| The pricing page, pricing experiments, the billing chassis | Nothing is sold |
| Postgres, `docker-compose`, the dev database, `env-parity` | No server, no environments to keep at parity |
| `Dockerfile`, `fly.toml`, Deploy Production, `deploy-fly.md` | The site moves to static hosting; a paid machine and a container image to serve files that need neither |
| `serve/mod.rs` static-file serving, `--web-dist` | The CLI has no UI |
| ~~iOS/Android targets~~ | ~~A tool whose job is editing files in a git repo and running code on your machine has no phone story yet~~ — **reversed by `notes-ide.md`**: a notes IDE has an obvious phone story (capture and read), and `hick weave` already renders without executing, which is the only thing a phone could not do |

`terraform/dns` and `terraform/posthog` survive: the domain still resolves to
the static site, and the site still measures its visitors.

`hick-grove`, `hick-store`, `hick-token`, `hick-classify`, `hick-sink`,
`hickory-collab`, and every vendored crate are untouched — they belong to the
language and the editor, not to a deployment.

## What survives

The hick language and its parser's no-escaping invariant. The `Executor` trait,
`LocalExecutor`, and the Docker executor. `hickory-collab` and `hick-grove` for
the reason above. `hick-lsp`. The agent and the bring-your-own-agent tool
surface, running on the user's own key from their own environment. The
pre-commit drift gate. Guarantees and their verification discipline.

`hickory-executor-canopy` survives as an **optional** executor, because a
Canopy node is one the *user* runs — the same category as their Docker daemon,
not a service we operate. Nothing may require it, and it must stay off the
default path.

The whole CLI, minus three commands: `run`, `test`, `up`, `weave`, `lineage`,
`promote`, `agent`, `refresh`, `init`, `doc`, `mcp`.

## What gets better

- **The thing that made this product interesting becomes the whole product.**
  Editable generated files with byte-exact lineage is the differentiator; a
  workspace with seats was table stakes for a market this was never going to
  win.
- **The release channels stop being infrastructure nobody uses.** The download
  is the product, so `unstable-release.yml` and `stable-release.yml` are on the
  critical path instead of beside it.
- **One environment, no secrets.** No staging/production split, no GitHub
  Environments, no key rotation, no `SIGNUP_ALLOWLIST`, no Postgres backup to
  test, no analytics project per environment.
- **Nothing to be liable for.** No customer data on our disks, because there
  are no disks.

## Open edges

- **The repository must go public** for `scripts/install.sh` to work for anyone
  who is not us; it currently needs a token against a private repo.
- **The static site keeps PostHog; the product has none.** This is a real
  boundary and it is worth stating precisely, because "we use analytics" and
  "the tool phones home" are the sort of pair that collapses into each other by
  accident. The marketing site is a web page, and measuring whether anyone
  arrives at a web page is ordinary. The *downloaded binary* sends nothing,
  ever — no telemetry, no update check, no crash report, no first-run ping.
  The two never share a key, a build, or a code path, and nothing in the
  product links the analytics client. `/analytics/capture` (the old server-side
  proxy) still goes: the site talks to PostHog directly, since the proxy only
  existed to keep one environment's key out of a promotable image and there is
  no longer anything to promote.
- **No revenue model.** This is a deliberate choice, not an oversight, and it
  has a cost: nothing funds the work. If that changes, the honest options are a
  paid licence for a private build or support contract — not a feature gate
  retrofitted onto a tool that shipped without one.

## What would reverse this

Someone paying for collaboration. Not asking for it — paying for it. The relay,
identity, and pricing are all recoverable from git history, and
`local-collaboration.md` still describes the mechanism. Rebuilding is a known
quantity; carrying the unbuilt version indefinitely is what this document
declines to do.

## Sequence

1. **This document**, so the next session does not rebuild toward a server.
2. **`AGENTS.md` and the instruction modules** — the stack section still names
   Fly, a PaaS, and continuous deployment to production.
3. **Merge `hick serve` into `hick up`**, deleting sharing, relay, and tunnel,
   and routing reverse edits through the room.
4. **Delete `apps/server` and `apps/relay`** with their crates, database, and
   deploy path.
5. **Move the static site off Fly**, deleting the Dockerfile, `fly.toml`, and
   the Deploy Production workflow.
6. **Split the landing page out of the app**, and turn `apps/mobile`'s Tauri v2
   shell into `apps/desktop`.
