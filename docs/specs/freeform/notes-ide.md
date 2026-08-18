# The notes IDE: notes are documents, meetings are inputs

*Status: design of record for what the product is **for**. Adopted 2026-08-18.
**Supersedes** `local-only.md` on two points only: its one-sentence statement
of purpose ("literate programming where you can edit the generated files"), and
the row in its retirement table that deletes iOS and Android targets. Everything
else in `local-only.md` stands unchanged and is not reopened here — no server,
no account, no relay, no subscription, no telemetry, one engine behind two front
doors. This document widens what the tool is for; it does not soften what the
tool is.*

Hickory Docs is a **note-taking IDE**. You keep your notes in a folder of files
in your own git repository, you edit them in an app that treats a note the way
an IDE treats source — structure, search, navigation, lineage, live errors —
and meeting transcripts and their AI summaries land in that folder as ordinary
notes rather than living in somebody's SaaS.

The literate-programming machinery is not retired and is not a second product.
It becomes the reason this notes IDE is unlike the others: **a note here can
run, and an AI summary in it can be proven to still describe the thing it
summarized.**

## Why this is a widening and not a rewrite

`local-only.md` narrowed the product to one sentence and said everything else
"exists to make that sentence true, or is being deleted." That sentence is now
too small — but almost nothing built under it was wasted, because the pieces a
notes IDE needs are the pieces that already exist:

| A notes IDE needs | What already does it |
|---|---|
| A folder of notes that stay in sync with their rendered form | `hick up` — weaves a folder and keeps it woven, and carries edits made in a generated `.md` back into its document |
| Turning an existing plain note into a first-class one | `hick adopt` — wraps a file's content byte-exactly and refuses if the weave does not reproduce it |
| An AI summary that cannot silently drift from its source | `hick:transform` — prose written by a model, pinned by fingerprint to exactly the bytes it read, checked by `hick test` offline and without a key |
| Search across every note | `hick search` — lexical offline out of the box, semantic if the user installs a model |
| Editing the same note in two places at once | `hickory-collab` + `hick-grove` — already justified by two writers (editor buffer and file on disk); a phone is simply the third |
| Knowing where a sentence came from | `hick lineage` — byte-precise provenance from output back to source span |
| Knowing that a summary was written by a model, and from which bytes | `hick:transform` with `from=` — a fingerprint checked offline, plus `git blame` composed through lineage in `crates/hickory-cli/src/agent_lineage.rs` |
| Material arriving without anyone running a command | The inbox `hick up` already watches, plus the app's scratchpad — see `ingest.md` |
| A note that looks like a note when you open it | Bare documents — see `bare-documents.md`. A `.hick` file may begin with markdown or YAML frontmatter, and always weaves a `.md` of the same name |
| Reading a note on a device that cannot execute anything | `hick weave` — weaves from cached transcripts, marking never-run blocks as never-run |

The last row is the one that makes mobile possible at all, and it was built for
an unrelated reason.

## What a meeting becomes

A transcript is dropped into a watched inbox directory — exported from Granola,
Otter, Fathom, Zoom, or saved by hand. `hick up` notices it and ingests it into
a note. The shape of that note is fixed:

```
<hick:copy id="transcript" class="source">…the transcript, raw, byte-for-byte…</hick:copy>

<hick:transform select=".source" instruct="Summarize…" from="…">
…the summary…
</hick:transform>

<hick:transform select=".source" instruct="Extract every action item…" from="…">
…the action items…
</hick:transform>
```

Three properties fall out of that shape, and each is the reason to prefer it
over pasting a summary into a markdown file:

1. **The original bytes survive.** The transcript is inside a `hick:copy`, so
   the no-escaping invariant applies and what the recorder produced is what the
   file contains. Nothing is normalized, reflowed, or lost to a parser.
2. **The summary is a claim with provenance.** `from=` fingerprints exactly the
   bytes summarized. Correct a name in the transcript and `hick test` fails with
   *stale transform* until someone re-runs `hick refresh`. A pasted summary
   silently keeps describing a document that has moved.
3. **Verification costs nothing.** `hick test` never spends a token — it checks
   the fingerprint, not the prose. Notes stay verifiable on a machine with no
   key and no network.

**Ingest never calls a model.** Parsing a `.vtt`, `.srt`, or an exporter's
markdown into that shape is offline, first-party, and deterministic; the
`hick:transform` bodies are left empty and stale until the user runs `hick
refresh` with their own key. This is `config-and-environments`' rule, not a new
one: a missing credential degrades to a note with an unfilled summary, never a
failure to ingest. It also means a meeting recorded on a plane still becomes a
note on the plane.

Transcript parsers are first-party per `AGENTS.md` — no third-party CLI is
integrated to read a subtitle file.

## Mobile: what a phone is and is not

`apps/mobile` returns as a Tauri v2 shell, and it is deliberately **not the
IDE**. It is a third Vite entry point beside `index.html` (the editor) and
`site.html` (the marketing page), for the same reason the site is one: the
demos and the editor genuinely share code, and a package boundary would have to
be crossed on every change.

| On the phone | Why |
|---|---|
| Read notes, rendered | `hick weave` from cached transcripts — no execution required |
| Write and edit notes | The editor buffer and the CRDT already exist |
| Capture — a transcript shared in from another app, a note typed in a hurry | This is the thing a phone is actually good at |
| Search | `hick search` lexical mode is offline and has no native dependency |
| Sync | Below |

| Not on the phone | Why |
|---|---|
| Executing cells | iOS cannot spawn a subprocess. There is no `LocalExecutor` on iOS and there will not be one |
| Terminal, LSP, DAP | All three are subprocess supervision |
| `hick refresh` | It calls a model, which is fine, but a stale transform is not urgent on a phone; keep the surface small |

A note whose cells cannot run here is not broken — it renders from its cached
transcripts with never-run blocks marked as never-run, which is a state the
weave already models and the UI already draws. **Stating the limit as a feature
rather than discovering it during the port is the point of writing it down
here.**

## Sync: the user's own git remote

There is no server, so the phone and the desktop meet somewhere the user
already owns: a git remote they control. The notes folder is a git repository —
the user's own `.git`, not `hick-store`'s internal `.hick/git/` — and the phone
commits and pushes to it.

- **Auth is the user's.** A token or SSH key in the platform keychain, put there
  by them. We never hold a credential, and there is no account to make.
- **Git on a phone is a library, not a binary.** There is no `git` executable on
  iOS, so the mobile shell links one. Note that this is a **new capability, not
  a port**: nothing in this codebase clones, fetches, or pushes today, on any
  platform. The library choice turns on whether it can push — see
  `shipping-mobile-and-desktop.md`.
- **Conflicts are resolved by `hick-merge`.** Three-way merge over `.hick`
  documents already exists. On desktop it can be registered as a git merge
  driver; **on mobile it cannot**, because a merge driver is a config line that
  invokes a binary and a phone has neither the binary nor a `git` to read the
  config. So the merge is called **in-process** by the sync code, and the driver
  is the desktop spelling of the same thing — one implementation, two ways of
  reaching it. See `shipping-mobile-and-desktop.md`.
- **The CRDT does not cross the remote.** `hickory-collab` reconciles concurrent
  writers in one process; git reconciles devices. Confusing the two would
  rebuild the relay under a new name.

### The wording this changes

`local-only.md` is about a product with no server *we* operate, and that is
still exactly true. But a git remote means notes leave the user's disk to a host
**they** chose, so any claim shaped like "your notes never leave this machine"
is now wrong and must be written as "nothing here talks to a server we run."
The distinction matters for the same reason the PostHog boundary in
`local-only.md` matters: two nearby sentences, one true and one not, collapse
into each other by accident.

## Open edges

- ~~**App stores are a distribution channel we do not control.**~~ **Decided
  2026-08-18** — App Store and Play Store for mobile, `.dmg`/`AppImage`/`.msi`
  for desktop. See `shipping-mobile-and-desktop.md` for what that costs and for
  the measurement of how much source actually shares. The paragraph below is
  kept as the record of the question, not of the answer.
- **The original framing of that edge:** This is a real
  collision with `continuous-delivery-downloadable.md`, whose whole delivery
  model is a GitHub release and a one-line installer. An iOS build additionally
  requires a paid Apple Developer relationship and human review — the first
  thing in this product that requires either. The honest options are TestFlight
  and Android sideload (keeps the downloadable model, reaches fewer people), or
  accepting store review (reaches people, adds a gate we cannot make green
  ourselves). **Not decided here.** Until it is, mobile is not an advertised
  platform, per that module's rule that an unverified target is not a supported
  one.
- **`hick test` in CI over a notes repo is a new user story.** A repository of
  meeting notes with stale transforms wants a pre-commit gate the same way a
  code repo does, and `hick init` already installs one. Whether a notes user
  wants their commit blocked because a summary is stale is a genuine question,
  not an obvious yes.
- **Search over years of notes is a different scale than search over a repo.**
  Lexical ranking is fine at repo scale; nobody has measured it at ten thousand
  meetings.
- **The one-sentence pitch is now two sentences.** "Notes that can run" and
  "AI summaries that cannot lie about their sources" are both true and neither
  alone is the product. That is a marketing problem this document does not
  solve, and `landing-discovery.md` is where it gets solved.

## What retires

Nothing. This document adds; the retirement table in `local-only.md` stands,
minus its iOS/Android row.

## Sequence

1. **This document**, and the pointer edits in `AGENTS.md` and `local-only.md`
   so the next session does not rebuild toward the narrow product sentence.
2. **Bare documents** (`bare-documents.md`) — **built.** The wrapper becomes optional and
   every document weaves a markdown file of its own name. This comes before
   ingest because it fixes the shape ingest writes, and before mobile because
   it is what makes a note openable by someone who has never seen this format.
3. **Ingest** (`ingest.md`) — transcript parsers and the watched inbox, wired
   into `hick up`. Entirely offline, entirely on the existing engine, and useful
   on the desktop before any phone exists. **Built.**
4. **Provenance and standing** (`provenance-and-standing.md`) — what a note
   says about where its information came from and how much weight it carries.
   Half of it already exists in `SourceOrigin` and `agent_lineage.rs`.
   `hick:claim` and the derived speaker attribution are **built**; the
   has-an-AI-touched-this report is **not**.
5. **The notes-shaped IDE surface** — an inbox, a note list, search as a first
   surface rather than a command; the editor is already there.
6. **`apps/mobile`** — the Tauri v2 shell, read and capture only, no executor.
7. **Sync** — git remote, keychain credentials, `hick-merge` as a merge driver.
8. ~~**The distribution decision** for iOS and Android~~ — **decided**, see
   `shipping-mobile-and-desktop.md`. What replaces it as the last step is the
   engine split it implies: separating the portable core (language, documents,
   weave, transcripts) from the host-process layer (executor, terminal, LSP,
   DAP, watcher) so a mobile shell can link the first without the second.
