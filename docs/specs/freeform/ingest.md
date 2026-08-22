# Ingest: a meeting becomes a note

*Status: design of record for how outside material enters a notes folder.
Adopted 2026-08-18. Implements step 3 of `notes-ide.md`'s sequence. Depends on
`bare-documents.md` for the shape of what it writes, and pairs with
`provenance-and-standing.md`, which governs how the result is marked.*

A transcript is a file somebody else's tool produced. Ingest is the act of
turning it into a note in your repository without lying about where it came
from and without needing a network, a key, or a model to do it.

## Three ways in, no command to run

Material arrives one of three ways, and none of them is "remember to run
something":

1. **Download a file into the inbox.** Point the browser's download directory
   at it, or save an export there.
2. **Copy a file into the inbox.** Drag it, `cp` it, let a sync client put it
   there.
3. **Type into the scratchpad** in the app.

`hick up` watches an **inbox** directory inside the notes folder, so (1) and
(2) need nothing further; the desktop app runs the same loop in-process, so
there no command exists at all. `hick ingest` remains for scripting and for
people who would rather ask than watch.

A file appearing in the inbox is parsed into a note; the note is written beside
the others; the inbox file is **moved**, never deleted, into a sibling
`ingested/` directory.

Three rules make that safe to run unattended:

1. **Ingest never deletes the user's file.** The bytes came from outside and we
   do not own them. A move is reversible by anyone looking at the directory; a
   delete is a support ticket from someone who dropped in their only copy.
2. **Ingest never calls a model.** Parsing a `.vtt` into a note is offline,
   first-party, and deterministic. The summary passages are written *empty and
   stale*, and stay that way until the user runs `hick refresh` with their own
   key. A meeting recorded on a plane becomes a note on the plane; this is
   `config-and-environments`' degradation rule, not a new one.
3. **A file that is still arriving is left alone.** A download or a large copy
   lands in pieces, and reading one mid-flight yields a truncated transcript
   that would then be fingerprinted, ingested, and moved out of the inbox — a
   silently incomplete note, which is the worst outcome available. Two things
   prevent it: names browsers use while transferring (`.crdownload`, `.part`,
   and friends) are passed over **in silence**, since reporting a file that is
   working correctly is just noise; and any other file must stop growing before
   it is read.
4. **Ingesting the same transcript twice produces one note.** Identity is the
   content hash of the source bytes, recorded in the note's frontmatter. A
   re-dropped file is recognised and skipped, because the reflex when something
   looks like it failed is to try again.

The inbox directory name is configuration with a working default
(`HICKORY_INBOX`, defaulting to `inbox`), per `config-and-environments` — never
a hardcoded path. It is validated at construction and refuses an absolute path
or one containing `..`, because ingest *moves* files and an escaping path would
move somebody's file somewhere they did not ask for.

`hick ingest` drains the inbox once; `hick up` drains it before the first weave
and again whenever something appears there, so a note is woven in the same
cycle it was created.

## What it writes

```
---
date: 2026-08-18
attendees: [Sam, Nate]
source: sync-with-sam.vtt
source-format: vtt
source-sha256: 9f2c…
---

# Sync with Sam

<hick:transcript id="t" format="vtt">
WEBVTT

00:14:03.000 --> 00:14:11.000
<v Sam>We can't ship until the index lands.
…
</hick:transcript>

<hick:transform select="#t" instruct="Summarize…" from="">
</hick:transform>

<hick:transform select="#t" instruct="Extract every action item…" from="">
</hick:transform>
```

`attendees` is **derived from who actually spoke**, not asserted by whoever ran
the command — the same discipline as everything else in
`provenance-and-standing.md`.

Naming follows the meeting, not the download: a `YYYY-MM-DD` at the front of
the file name is the note's `date:` (the file's modification time is the
fallback), the title is the rest of the name, and the transcript's `id` is the
note's own name — `2026-08-20-checkout-latency-sync` — so its turns are
`#2026-08-20-checkout-latency-sync-u7` and two meetings upstream of one note
never both answer to `#transcript`.

The empty `from=""` is the point: `hick test` reports the passages as stale
immediately, so a note that has never been summarized is visibly unsummarized
rather than silently empty.

## `hick:transcript`: one element, two views

You asked for the raw block **and** speaker turns. Emitting both as text would
create a consistency obligation between two representations of the same
meeting — a drift surface, and a new thing to check.

So they are not two representations. `hick:transcript` holds the source bytes
verbatim, subject to the no-escaping invariant like any other raw content, and
the **speaker turns are derived from it when the document is prepared** by a
first-party parser — before selectors are resolved, which is what makes the
handles below work rather than being addressable in principle and selectable by
nothing. The same trick `hick:file` and `hick:paste` already use: one source of
truth, a projection over it, and no possibility of the two disagreeing because
one *is* the other.

The projection makes each utterance addressable with the selector grammar that
already exists — no new syntax:

| Handle | What it selects |
|---|---|
| `#t` | the whole transcript (an ingested note names its own `transcript`) |
| `#t-u12` | the twelfth utterance |
| `.said` | every utterance in the document |
| `.said-sam` | every utterance by Sam |

Which is what makes the rest possible: a summary can name the turns it came
from, `hick lineage` can point at *"Sam, 00:14:03"* rather than at a wall of
text, and a claim can be attached to one person's sentence.

Utterance numbering is positional and stable for a given transcript, because it
is derived from bytes that do not change. Editing the transcript renumbers what
follows — which is correct, and is why anything that needs to survive editing
should reference the transcript, not an utterance index.

## The scratchpad is not an ingest

Text typed in the app becomes a note too, but by a **different** path and into a
different shape: no `hick:transcript`, no summary transforms, no format
detection. This is not a detail of implementation, it is the provenance rule
from `provenance-and-standing.md` applied honestly.

A `hick:transcript` means *bytes another tool produced*. That is exactly right
for a downloaded export and exactly wrong for a sentence somebody just typed:
on the attributable axis, scratchpad text is the strongest material there is —
a named human wrote it, here, and git will say so. Wrapping it as
machine-conveyed material would throw that away, and would imply a machine step
between the person and the words that never happened.

So the scratchpad writes prose, records `source: scratchpad`, and summarizes
nothing — there is nothing to summarize that the person did not already say.

Two consequences worth stating:

- **Saving is explicit, never automatic.** A scratchpad that committed every
  keystroke would turn a place to think into a place that keeps a record, and
  people stop thinking in those.
- **Prose containing `<hick:` is refused.** Prose becomes document content, and
  there is no escaping in this language by design, so the text stays in the box
  with an explanation rather than being silently transformed.

## Formats

WebVTT, SRT, **SubViewer** (`.sbv` — what Google Meet's caption export
produces, which has neither a cue index nor `-->`, so neither the WebVTT nor
the SRT reader recognises it), the markdown exports Granola / Otter / Fathom
produce, and plain text. Parsers are **first-party** — `AGENTS.md` forbids integrating a
third-party CLI to read a subtitle file, and these formats are small.

### Google Workspace is a special case, because its exports are not files

A Google Meet transcript and a Gemini "notes for me" document are **Google
Docs**, not files on disk. Two things follow:

- **A `.gdoc` in a synced Drive folder is a shortcut, not a document.** It
  holds a URL; the words are still in the cloud. Ingesting one would produce a
  note containing a JSON stub — worse than useless, because it looks like it
  worked. So they are refused by name, with the export path spelled out:
  File > Download > Markdown, then put *that* in the inbox.
- **What Meet exports directly is `.sbv` captions**, which is why that format
  is supported.

A Gemini notes document is a *summary a machine wrote*, not a transcript. It
ingests as unrecognised material, which is the honest outcome: nothing about it
can be parsed into speaker turns, because there are none.

**An unrecognised format is adopted, not refused.** It becomes a note whose
transcript block is the file's bytes with no speaker turns derived, plus a
warning naming the format we could not read. Refusing would leave the file in
the inbox forever; adopting keeps the material and loses only the structure.
This mirrors the diagram rule in `a-diagram-names-what-proves-it.md`: degrade to
a warning, never block the author.

## What ingest does not do

- **It does not read the clock at all.** The note's `date:` is the source
  file's own modification time, so ingesting the same bytes twice produces
  byte-identical notes. "When did I get round to it" is not a fact about the
  meeting, and a note that differed depending on the day it was made could not
  be tested.
- **It does not summarize, classify, or judge.** Everything a model contributes
  arrives later, through `hick refresh`, in a `hick:transform` whose fingerprint
  says which bytes it read.
- **It does not invent provenance, but it does record what the operating system
  already knew.** If the file was downloaded, macOS and Windows record where
  from (`kMDItemWhereFroms`, the NTFS `Zone.Identifier` stream), and some Linux
  browsers set `user.xdg.origin.url`. That lands in the note as `source-url:` —
  derived evidence in the sense of `provenance-and-standing.md`, since nobody
  typed it. **The query string is dropped**, deliberately: an export link from
  Drive or any signing service carries its credential in the query, and this
  value is about to be written into a file that goes into a git repository and
  may be pushed somewhere other people can read.
- **It does not attribute standing.** Who is an expert on what is not knowable
  from a transcript. See `provenance-and-standing.md`.

## Open edges

- **Speaker identity is whatever the exporter wrote.** "Sam", "Sam H.", and
  "sam@…" are three people as far as this is concerned. Mapping them to one
  identity is a real feature and is not in this spec.
- **Diarization errors are invisible to us.** If the recorder attributed a
  sentence to the wrong person, the note repeats the error faithfully. The raw
  block is the defence: it is exactly what the exporter produced, so the
  mistake is auditable rather than laundered.
- **A transcript is large.** Nobody has measured weaving a folder of hundreds
  of hour-long meetings, and utterance projection is per-byte work applied on
  every document preparation.
- **Download origin is recorded unevenly, and `None` is ambiguous.** Windows and
  macOS are reliable; on Linux only some browsers and file managers set the
  attribute, and `curl` sets none. An absent `source-url:` means "nothing was
  recorded", never "this was not downloaded" — and no surface may imply
  otherwise.
- **Settling is a heuristic, not a guarantee.** A transfer that stalls for
  longer than the settle window, then resumes, could in principle be read
  mid-flight. Filesystem events keep arriving, so the next pass corrects it,
  but the first note could be short-lived and wrong.
- **A file containing `</hick:transcript>` cannot be ingested.** There is no
  escaping in this language by design, so it is refused with the reason rather
  than mangled. This is rare and the refusal is honest, but it is a real hole.
- **No real exporter output has been through this.** Every fixture is
  hand-written; Granola, Otter, Fathom, and Zoom exports are what it exists to
  read and none has been tried.
