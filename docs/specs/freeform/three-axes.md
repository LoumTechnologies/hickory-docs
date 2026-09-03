# Three axes, not thirty modes

*Status: design of record, adopted 2026-09-03; all five steps of the
sequence built the same day. Written
because the dogfooding of 2026-09-02 (`an-output-that-cannot-be-carried-back-is-held.md`
and its neighbours) added two more states to a document that already had
too many, and because the question "should recordings be committed or
promoted?" turned out to be a symptom of the same thing. Pairs with
`expression-and-log.md` (a document is an expression, the repository is the
log), `provenance-and-standing.md` (derived vs declared), and
`sessions-you-run-again.md` (run, edited, staged).*

A `.hick` document, a cell in it, and a file it produces can each be in a
great many named states today, and a person using the product meets those
names in commands, attributes, badges, and banners. Counted from the code:

- **Kinds of document:** `hick:doc`; `hick:session` in three sub-kinds —
  *run* (harness-written), *edited* (a declared layer over a frozen base),
  *staged* (authored, never executed); a *carry* document; an ingested
  *note*; and the merged view, which is a lens and not a document at all.
- **Evidence a cell has:** never run (`NoBaseline::NotExecuted`), recorded,
  recorded but stale, frozen without a recording, frozen without a cache
  directory, agent without a runner, and the run-wide cache modes `Off`,
  `Reuse`, `Require`.
- **Verbs that bring bytes into a document:** `ingest` (a scaffolder's
  volume, or an inbox transcript), `adopt` (a plain file), `promote` (a
  session), `carry` (a session's distillate), `import` (another harness's
  session), and the recording promotion proposed on 2026-09-02, which would
  have been a sixth.
- **Verbs and modes that compare two weaves:** `test`, `equiv`, the refactor
  baseline ("Pin outputs as a baseline", session state under
  `/docs/{id}/refactor/*`), and the merge driver's three-way check.
- **States of a produced file:** generated and read-only, editable (mixed),
  *preserved* (the writer left it alone), *kept* (the weave target left
  alone), *held* (the loop refused an edit), and the plain-file `409`
  conflict, each with its own wording and its own way out.
- **Switches:** continuity on or off, format on save, `freeze=` per cell.

Every one of those was added for a reason that was right at the time. The
claim of this document is that they are answers to only **three
questions**, and that a person should meet three questions, not thirty
names.

## Axis 1 — Evidence: is there a recording, and does it still match?

For any cell there is exactly one question: **is there a recording of this
cell, and was it made from the inputs the cell has now?** The cache key
(`a-recording-is-keyed-by-the-cells-inputs.md`) already decides the second
half. So a cell is in one of three states:

| State | Meaning | Weave shows | `hick test` says |
|---|---|---|---|
| **recorded** | a recording exists and its key matches the cell's inputs | the recorded output | matches, or drift |
| **stale** | a recording exists and its key does not match | the last recorded output, marked stale | stale — run to refresh |
| **unrecorded** | no recording | nothing, or `[never run]` where nothing was ever there | unverifiable |

Everything else on this axis is a policy about what to do in the *stale*
and *unrecorded* states, and policies are not modes:

- `freeze="true"` / `CacheMode::Require` means "stale is an error, do not
  re-execute". It is a per-cell policy on the stale state. `Reuse` means
  "stale means run". `Off` means "always run". One attribute with three
  values, all describing the response to one state.
- `FrozenWithoutRecording` and `FrozenWithoutCacheDirectory` are the
  *unrecorded* state under the freeze policy, with the second saying where
  the recording would have gone. One state, one policy, one location.
- `AgentWithoutRunner` is *unrecorded* with a reason nothing on this machine
  can change. A reason, not a state.
- A session's three sub-kinds are this axis asked of a whole document. A
  *run* session is recorded throughout; a *staged* session is unrecorded
  throughout; an *edited* session is recorded with a declared layer on top —
  which is the ownership axis below, not a third kind of evidence. The
  never-run marking `sessions-you-run-again.md` carries "in the bytes" is
  the unrecorded state written down, and stays.

**What changes:** the words. A cell, a file, a document and a session all
report *recorded / stale / unrecorded*, in the CLI, the gutter, the status
bar and the tree. `[never run]` survives only as the text a weave writes
where nothing has ever been; it is never written over anything
(`a-weave-without-a-recording-keeps-the-artifact.md`).

**What this settles:** where a recording lives. A recording is **evidence a
document makes about itself**, and the `from=` post-mortem
(`scaffolded-files-and-derived-edits.md`) already ruled that a durable claim
must not point into `.hick-cache/`. So a recording the document wants to
keep is written **into the document**, beside the cell it records, with the
key it was recorded under, and `.hick-cache/transcripts/` goes back to being
a cache — evictable, gitignored, and never cited. Committing the cache
directory (2026-09-02) was the wrong half of the right instinct and is
reversed by this document. The element that holds a recording is the one
grammar addition here, and it is decided in `## The grammar` below rather
than in a command.

## Axis 2 — Ownership: who owns these bytes?

For any run of bytes in a document there is exactly one question: **does the
document own them, or does it point at something else that does?** Owned
bytes are literal text and `hick:file` content. Pointed-at bytes are a
`hick:paste` of a `hick:copy`, a `hick:include`, a `hick:sample` window, a
`cites=`, and the contents of a volume or a session the document did not
write.

Bringing bytes across that line — from pointed-at to owned — is **one verb**,
and the product has five spellings of it:

| Today | Source | What it does |
|---|---|---|
| `hick ingest --from '#cell'` | a cell's output volume | copies the bytes in as `hick:file`, fingerprinted with the run |
| `hick ingest` (inbox) | a transcript file in the notes inbox | copies the bytes in as a note, verbatim |
| `hick adopt` | a plain file | wraps the file's bytes in a `hick:file`, proving the weave reproduces them first |
| `hick promote` | a `hick:session` | rewrites the session as a `hick:doc` |
| `hick carry` | a `hick:session` | writes what carries to a next attempt as a new document |
| `hick import` | another harness's session log | writes it as a `hick:session` |
| *(proposed 2026-09-02)* | a cell's recording | writes the recording beside the cell |

Every row takes bytes the document did not write, gives them a home in a
document, records where they came from, and — where the bytes are
generated — **proves the weave is unchanged before writing anything**.
That proof is the same check in every case, and it is the check `hick equiv`,
`hick test`, the refactor baseline and the merge driver each implement
separately today (axis 3 has it). So:

- **One verb, `hick ingest`, with the source as its argument.** A volume, a
  file, a session, a recording, an inbox transcript, a Claude Code log. The
  distinct words survive as the *reason* in the provenance record, not as
  commands: an ingested block says whether it came from a run, a file, a
  session or a recording, which is what `provenance-and-standing.md` wants
  said anyway. `adopt`, `promote`, `carry` and `import` become spellings of
  `ingest --from`, kept as aliases for exactly as long as `pre-launch.md`
  allows, which is not at all.
- **One element, `hick:ingested`,** already built, holding the bytes and the
  fingerprint of what produced them. A recording is ingested the same way: a
  `hick:ingested` child of the cell, holding the recorded output and the key
  it was recorded under. No `hick:recorded` element; the one that exists
  already says the right thing.
- **The equivalence gate is the same function every time**, and it lives in
  one place (axis 3).

The *edited* session sub-kind is this axis: a frozen base the document points
at, with owned bytes layered over it. The merged view is the other end of the
same axis, a lens over bytes the view owns nowhere — which is why it is
correctly not a document, and stays that way.

## Axis 3 — Agreement: does the disk hold what the document produces?

For any file a document produces there is exactly one question: **are the
bytes on disk the bytes the document produces right now?** Two answers:

- **agreed** — the file is what the weave writes. Editable regions carry
  back; generated regions are read-only; nothing to say.
- **diverged** — the file holds bytes the document did not produce. Somebody
  wrote them: a person, a `git checkout`, a merge, another program.

Today the diverged answer has five names — *preserved* (the CLI writer left
an artifact alone), *kept* (the weave target left alone), *held* (the loop
refused an edit), the plain-file `409`, and a merge conflict — and each has
its own wording and its own set of buttons. They are one state, and it
deserves **one surface with the same three ways out everywhere**:

1. **Keep mine** — the disk stays; the document is the one that is behind.
   Edit the document, or ingest the file (axis 2), and the state resolves
   itself.
2. **Take the document's** — regenerate the file from the document. The one
   write over diverged bytes, taken only when asked by name.
3. **Merge** — the three-way merge the plain-file pane already has, over the
   bytes the document produces, the bytes on disk, and the last agreed
   version as the base.

The rule under all three is the invariant the round-trip work was built to:
**the tool never writes a byte nobody asked for, and never destroys a byte a
person or git wrote.** Diverged is loud and stays diverged until a person
picks a way out. Nothing restores, nothing quietly re-weaves.

The comparisons collapse onto this axis too. `hick test`, `hick equiv`, the
refactor baseline and the merge driver's check all ask "would these two
weaves produce the same bytes?" — a document against its recordings, a
document against its restructured self, a document against its pinned
outputs, a merged document against its two parents. **One function,
`compare_outputs`, one report shape, one badge.** The refactor baseline stops
being a mode you enter: pinning is "record the agreed state now", and the
badge is axis 3 measured against that pin instead of against the last weave.

## What survives as a switch

Three things are preferences, not states, and stay as switches: continuity
(`hickory-workspace`, off by default), format on save (`ui.json`, off by
default), and the per-cell freeze policy on axis 1. A switch changes what the
tool does; it never changes what a document *is*.

## What is deleted

- The three session sub-kinds as distinct concepts. A session is a document
  whose evidence (axis 1) and ownership (axis 2) are reported like any
  other's. The bytes that mark them stay; the vocabulary goes.
- `adopt`, `promote`, `carry`, `import` as commands. They are `ingest --from`.
- Refactor mode as a mode. It is a pin on axis 3.
- The kept / held / preserved distinction, and the separate `409` banner.
  They are *diverged*, with one surface.
- `NoBaseline`'s four variants as things a person is told apart. They are
  *unrecorded*, with a reason attached.
- Committed `.hick-cache/transcripts/`. Recordings the document keeps are
  ingested; the cache is a cache.

## The grammar

One addition and one reuse, because `pre-launch.md` says a document format
deserves more care than code:

- **Reuse:** `hick:ingested` gains a `key=` attribute for the recording case,
  carrying the cache key the bytes were recorded under, beside the
  `fingerprint=` it already carries for a run. A weave that finds a
  `hick:ingested` recording whose key matches the cell's inputs uses it; one
  whose key does not match is *stale* (axis 1), exactly as a cache entry is.
- **Addition:** none. `[never run]` and the session markings already exist.

## Sequence

1. Vocabulary first, no grammar: every surface says *recorded / stale /
   unrecorded* and *agreed / diverged*. This is renames and one shared
   report shape; it can land in a day and it is what a person meets.
2. One diverged surface: fold kept, held, preserved and the `409` into one
   banner with the three ways out, in the tree, the generated pane and the
   plain-file pane. `hold_output` and `restore_output` already implement two
   of the three.
3. One ingest: `hick ingest --from` takes a file, a session, a recording; the
   old commands are removed. The equivalence gate is `compare_outputs`.
4. Recordings into documents: ingest every committed transcript into its
   document, verify each with the gate, delete the transcripts, restore
   `.hick-cache/` to `.gitignore` in `hick init`.
5. One comparison: `equiv`, `test`, the pin and the merge check share the
   function and the report.

Each step is reversible on its own and each is worth having without the
others. None of them changes a byte of a committed document except step 4,
which is the one gated by the equivalence proof.

## Open edges

- **Whether a recording belongs in the document at all when its output is
  large.** A cell that prints a megabyte should not be a megabyte of
  document. The honest answer is probably that it is — the document is the
  expression and the recording is its evidence — with the editor folding it,
  the way it folds a session's agent turns. Decide when the first such
  document is met, not before.
- **The weave target's own place on axis 3.** It is a produced file like any
  other, but it is also what a person reads, so "diverged" on it is the
  common case during editing rather than an exception. It may want the
  agreed/diverged badge and not the banner.
