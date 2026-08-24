# Hickory Docs

A downloadable **note-taking IDE** whose notes are `.hick` documents:
reproducible, verifiable, executable documents in the hick language, meeting
transcripts and their AI summaries ingested as ordinary notes, and an AI agent
whose output is literate-programming files in git. Notes can run, and an AI
summary in one can be proven to still describe what it summarized.

Read three documents before changing anything:
`docs/specs/freeform/notes-ide.md` for what the product is **for** (notes are
documents; meetings are inputs; the phone reads and captures but never
executes), then
`docs/specs/freeform/local-only.md` for what the product **is** (a program you
download; no server, no account, no relay, nothing to buy; `hick up` and a
desktop app over one engine — still true, and widened on purpose only by
`notes-ide.md`), then
`docs/specs/freeform/architecture.md` for how it is **built** — accurate on the
language, crates, execution boundary, and verification, and superseded on
everything hosted.

The surface syntax is settled separately in
`docs/specs/freeform/bare-documents.md`: the `<hick:doc>` wrapper is optional, a
document may begin with markdown or YAML frontmatter, and every document weaves
a `.md` of its own name.

An open question, investigated and deliberately not built, is
`docs/specs/freeform/two-branches-in-one-document.md`: whether feature flags
could replace branching in a `.hick` document. The short answer is that the
branch→feature-set half is worth doing and the branches-as-flags half would
cost the provenance that this product exists for.

How a document owns files a scaffolder (`dotnet new`) wrote is
`docs/specs/freeform/owning-what-a-scaffolder-wrote.md`: **`hick ingest` reads
the exec's output volume** and writes the scaffold into the document as ordinary
`hick:file` bytes carrying the run's fingerprint, so your four lines are
ordinary edits and there is no anchor grammar at all. Nesting is
`exec > ingested > file` — never `exec > file`, because `file > exec` already
means "run this, paste the output here". The bytes need their own origin: they
must not read as `Literal` (they are not yours) or as `Exec` (that is synthetic
and kills the reverse edit). A re-run is a **three-way merge** with the recorded
hash as the base, coarse for a principled reason — two runs of a scaffolder
share no history, so no byte-precise thread exists to record. It supersedes
`docs/specs/freeform/scaffolded-files-and-derived-edits.md` on the mechanism,
which is still where the two refusals are argued: **CRDT edits** cannot express
a derivation from a foreign artifact, and **line-offset patches** are the
fifty-year-old version of the same mistake. What killed that document's `from=`
is worth remembering generally — **it built a durable claim on a gitignored
artifact** (`.hick-cache/`), so the base did not survive a clone. **Built
(2026-08-23):** `hick ingest --from '#cell' <doc>.hick`, the `<hick:ingested>`
element, the gitignore filter, the non-UTF-8 refusal, and the ingested origin —
plus one rule the spec did not name: **a volume a document has ingested is no
longer flushed as a pipeline output** (it arrives in
`PipelineResult::ingested_volume_files` instead), or the next `hick run`
overwrites your four lines. **Re-ingest is built too**, and it settled where
the base comes from: the document holds *ours* and records the run's hash, but
a hash verifies rather than reconstructs — so **the base is this document at
the commit that introduced that fingerprint**, recovered from git. Volatile
regions are deliberately not built until the merge has produced enough false
conflicts to show what they look like; note `volatile` is already a reserved
frontmatter key.

How one engineer's several machines see each other's IDE sessions — remote
view and remote control, one identity, no relay and no account — is
`docs/specs/freeform/one-engineer-many-machines.md`. Editing one file across several of them —
and across branches and worktrees — is `docs/specs/freeform/the-merged-view.md`:
one tab synthesizes a file from N sources with `<hick:when>`-shaped variants
that **exist on disk nowhere**, so **the view is the merge, held live** (a
shared region is agreed by construction and cannot conflict later). It is a
**lens, not a document** — no save path, no `.hick` extension — and it closes
`two-branches-in-one-document.md` without adopting branches-as-flags. A
committed variant names a **property, never a machine**: *the distinction lives
in the document, which machine matches lives on the machine.* The invariant that
keeps it honest — **CI weaves with no facts at all, so the shared reading must
stand alone and a variant may only add** — comes from the superseded
`machine-scoped-edits.md`, which is still the place the reasoning is written
down. A machine bought to run an agent on is
`docs/specs/freeform/the-broker-and-the-sealed-machine.md`: a sealed machine
holds no real key and has one road out through a broker the engineer runs,
which allows, denies, asks, or substitutes the real credential. Say "one road
out, with a toll booth" — **never** "airgapped", which a machine that talks to
a model is not. The open question under all three is
`docs/specs/freeform/changes-not-commits.md`: the reverse edit is already
jujutsu's move-a-change-where-it-belongs on the tangle axis, a session should
record which commit its writes *landed* in, and **emission is one-way** — a
session may produce a commit, but nothing may re-produce one that exists, which
is the only form of generated history that survives blame.

Whether a document that emits commits replaces the repository is
`docs/specs/freeform/expression-and-log.md`, and it does not: **a document is an
expression, a repository is the log of its values**, so a document that emits
history needs git *more*, each emitted commit carries the document version that
emitted it, and **the document describes the present while git holds the past**
— it never accumulates corrections, because the record of what generated an old
commit is inside that commit. Machines and repos are **places** (spanning them
is coordination); commits are a **time** (spanning them is rewriting), allowed
only above the publication floor. Across repositories: **read across, write
local.** **Built (2026-08-23):** the floor itself — computed on every read
(`merge-base(HEAD, @{upstream})`, then `origin/HEAD`, then `origin/master`),
carried on `GET /api/git/log`, and marked per row in the git pane. Nothing
emits anything.

Provenance *across* versions of a document is
`docs/specs/freeform/provenance-across-versions.md`. The data is not missing,
the identity is: `(commit, path, line)` addresses published bytes exactly (and
only below the publication floor — re-emission churns hashes above it), replay
recomputes exact lineage at any commit, and neither can **correlate** two
versions. The mechanism is a **recorded correspondence** — refactor mode proves the
outputs unchanged, which makes them a join key, and a merge is the richest
recording site of all because base, ours and theirs are in hand. **There are no
element ids**: an unwatched edit is guessed at by an agent, confirmed by a
person, and recorded as an assertion, or shrugged at. Register `hick-merge` as a
git merge driver rather than writing a team rule — and check it is configured,
since an undefined driver silently falls back to git's line merge. The
correspondence journal is a **record**, not a cache, so it may be committed, and
that choice is the only thing deciding whether CI can check anything here.
Continuity is a **fourth provenance family, off by default**, and the whole of
it — ribbon, journal, and the pre-commit repair — rides that one switch.
**Built (2026-08-23):** replay (`hick lineage --at`/`--history`, the time
slider, the grammar boundary stated in those words) and the merge driver with
its configured-driver check at project open and in `hick test` — never in the
pre-commit hook, since `hick init` installs the hook. Continuity itself stays
**also built:** the correspondence journal (`.hick-journal/`, gitignored by
`hick init` — delete that line to let CI check it), refactor mode as a
byte-precise recording site, the re-ingest as a diff-precise one, and the
pre-commit repair. All of it rides one switch, per-user in
`hickory-workspace`, **off by default**: no continuity, no journal, no check.
Journal entries above the floor are **provisional** — the concession that part
of a record is derived, which is the cost of recording at the moment both
sides are in hand. Nothing draws continuity yet.

A machine bought to run an agent on is sealed and reaches the network through
`hick broker` — a CONNECT proxy with a per-host policy and a log
(`hickory-broker`, built 2026-08-23; `allow`/`deny` only, no TLS termination,
no keys, no CA). What a re-emission would produce is `hick emit`
(`crates/hickory-cli/src/emission.rs`, read-only): one stage, one commit,
refusing to rewrite anything below the floor.

How a session is refined is `docs/specs/freeform/sessions-you-run-again.md`.
A **re-run is not a reenactment**: it is a second real session that stands
without the first, which is a draft the gitignore already discards. Three kinds
of session must never be mistaken for each other — **run** (harness-written,
evidence), **edited** (a declared layer over a frozen base, marked *in the
bytes*, not merely in the rendering), and **staged** (authored, never executed,
carried by the existing never-run marking). **Built (2026-08-23):** rewind and
re-run named apart in the dock, and `hick carry` — which settled the carry's
home by inventing nothing: it is an ordinary `.hick` document, written outside
the gitignored `sessions/`. Equivalence between two attempts is
an **instrument, never a gate** — the person is the judge, and their
understanding is allowed to move. The constraint that does the work is
legibility, not provenance: **a session must be followable by a reader who has
only the repository**, which is also the real argument for the interview form —
a prescient opening prompt concentrates everything you learned into one
unexplained monolith, while a dialogue lets each requirement arrive attached to
the question that provoked it.

How a sentence someone posts proves itself — meeting → analysis → message,
with ribbons across documents — is walked through in
`docs/specs/freeform/receipts-for-a-message.md`, including what is still
clunky. The three kinds of provenance that answer "why is this here?" —
lineage (the weave), context (what the model was shown, from the session),
declared (`cites=`) — and why they must never look alike, are
`docs/specs/freeform/three-provenances.md`. Sessions are the user's own
record and are not checked in; `hick init` ignores `sessions/`. A session
file is the conversation: one file per dock conversation, each turn naming
its parent, drawn in the app by the same cards as the chat
(`docs/guarantees/agent/a-session-is-the-conversation.md`); the model's
reasoning, when a provider streams it, is kept apart and folded
(`docs/guarantees/agent/reasoning-is-shown-apart-from-the-answer.md`). A Claude Code transcript becomes a session document with
`hick import claude-code` — what maps to what, what is dropped and counted,
and how the no-escaping invariant is honoured are
`docs/specs/freeform/claude-code-sessions.md`. How outside material enters a notes folder is `docs/specs/freeform/ingest.md`;
how the result is marked is `docs/specs/freeform/provenance-and-standing.md`.
The rule that governs both: **provenance is derived and checkable, standing is
declared and unverifiable, and the two must never render alike.** Say
"AI-touched" or "no evidence of AI" — **never** "human-written" or
"human-verified", which nothing can prove.

## Stack (settled — do not relitigate)

- CLI + language + local server: Rust (edition 2024), axum, tokio. No database.
- Frontend: React + TypeScript + Vite, shipped as the **desktop and mobile
  apps'** UI via Tauri v2 — one package, three entry points (`index.html` the
  editor, `site.html` the marketing page, mobile its own). No other frontend
  framework. It is not a client for any server we run — the only server it
  talks to is the one in the same process.
- Mobile (`notes-ide.md`): read and capture only. **No executor on iOS** — it
  cannot spawn a subprocess, so no cells, no terminal, no LSP, no DAP. Notes
  render via `hick weave` from cached transcripts. Devices meet through the
  **user's own git remote**, never through anything we run.
- Distribution (`shipping-mobile-and-desktop.md`): **App Store and Play Store**
  for mobile; **`.dmg`, `AppImage`, `.msi`** for desktop, from GitHub releases.
  Two thin Tauri shells over one core — never one shell threaded with
  `#[cfg(mobile)]`. The portable half is the language, documents, weave, and
  transcripts (`hick-lang` and `hick-transcript` verified compiling for
  `aarch64-linux-android`); the host-process half never goes near a phone.
- Live sync: Yrs (Yjs) CRDTs (`hick-grove`, `hickory-collab`). Durable state:
  the user's git repository. The CRDT survives the removal of collaboration
  because the app's editor buffer and the file on disk are still two writers.
- Sharing (`one-engineer-many-machines.md`): a machine is a **keypair**, a
  fleet is a mutual list of public keys, and reachability comes from the
  engineer's own transport — LAN, their overlay, their SSH host — behind the
  same provider seam `--public` already uses. **Never add a relay we operate**;
  `local-only.md`'s four reasons still hold when both endpoints belong to one
  person. Durable state crosses git; only liveness crosses the peer channel.
  **Built:** `hickory-fleet` — identity, the mutual key list, per-verb grants
  (`execute` off by default; a phone can never have it), `hick fleet` — and
  `hickory-peer`, the channel itself: **iroh 1.0, QUIC dialled by public key**,
  whose endpoint identity IS an ed25519 key, so the fleet list is the
  allowlist directly. Requests are gated by a **deny-by-default** path table;
  settings are unreachable over it at all. The spec's 60-second pairing code
  assumed a rendezvous we would have to run, so enrolment is a self-contained
  invitation instead. **On relays (decided 2026-08-24):** the default is
  number0's relays *and* number0's address publishing — neither is a server
  **we** run, so the refusal and the sentence hold, but that sentence now does
  more work, so the product says on every connection whether it was direct or
  relayed and `HICKORY_FLEET_RELAY` takes `direct` or your own relay. Never
  add a relay **we** operate; using somebody else's is a different decision
  and is recorded as one. **`iroh` is pinned to `default-features = false,
  features = ["tls-ring"]`** and must stay that way: the default `portmapper`
  feature pulls the MPL-2.0 `attohttpc` (copyleft, which this workspace
  forbids) and asks the router to open a port without being asked.
- Execution: `Executor` trait; `LocalExecutor` (default) and the Docker
  executor. NEVER reintroduce the wasm container runtime, and NEVER integrate
  third-party CLIs (cram, VHS, etc.) — verification and transcript capture are
  first-party.
- Product shape: **local-only** (`local-only.md`). There is no backend, no
  account, no relay, and no monetization. Do not add one. Anything that would
  need a server we operate is out of scope, not a later phase. Say "nothing
  talks to a server we run" — **never** "your notes never leave your machine",
  which sync to the user's own git remote makes untrue.
- Delivery: a **downloadable product**. `master` only. The release channels
  (`unstable-release.yml`, `stable-release.yml`) and the one-line installer are
  the delivery path; hickorydocs.com is static files and a download link.
  See `.instructions/continuous-delivery-downloadable.md`, as amended by
  `shipping-mobile-and-desktop.md`: mobile publication is App Review's verb, not
  a green CI run, and signing is part of delivery rather than a later polish
  step — an unsigned installer reads to a new user as malware.

## Rules

- `just` is the only task-runner entry point; recipes live at repo root.
- The hick parser's no-escaping invariant is sacred: only namespace-prefixed
  tags are structured; all other text is raw, byte-for-byte. No CDATA, no
  entity escaping. Docs about hick use a different prefix (`h:`) so `hick:`
  examples stay literal — and because rebinding the prefix requires the explicit
  `<h:doc xmlns:h="…">` root, those documents keep their wrapper while ordinary
  notes drop it (`bare-documents.md`).
- Public API of each crate is its explicit `pub use` facade in `lib.rs`.
- Guarantees live in `docs/guarantees/` (one file per guarantee, Given/When/Then
  + verification block); update them in the same change as the implementation.
- Vendored crates keep their names; upstream repos are frozen references —
  fixes happen here, not there.
- cloud-canopy is being modified concurrently by another agent — only
  `hickory-executor-canopy` may know its API; never edit the cloud-canopy repo.
  It stays an **optional** executor pointed at a node the *user* runs; it is
  not a service we operate, and nothing may require it.
- **MIT only.** A dependency whose licence is GPL or otherwise copyleft cannot
  be linked into this product. The live example: `grit` splits its licence —
  `grit-lib` is MIT and usable, `grit-cli` is GPL-2.0 and is not. Check the
  crate, not the project.
- No server, no account, no payment, no telemetry. A change that needs any of
  them is out of scope by decision — see `local-only.md`.

@.instructions/config-and-environments.md
@.instructions/continuous-delivery-downloadable.md
@.instructions/continuous-integration.md
@.instructions/dev-environment.md
@.instructions/documentation-layout.md
@.instructions/framework-agnostic-system-tests.md
@.instructions/github-issues.md
@.instructions/just.md
@.instructions/one-man-team.md
@.instructions/pre-commit-ci-parity.md
@.instructions/pre-launch.md
@.instructions/semble.md
@.instructions/specification-levels.md
@.instructions/third-party-integration-mocking.md
@.instructions/user-facing-errors.md
