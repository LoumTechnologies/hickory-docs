# Receipts for a message: a Slack sentence, traced to the meeting

*Status: a walked-through test, 2026-08-22, with what it fixed and what it
found. Pairs with `provenance-and-standing.md` and `ingest.md`. The worked
example lives in `examples/receipts/` — one variant authored by Claude Code,
one by the built-in agent — and is verified by `hick test examples/`.*

The question this answers: **when someone questions a sentence I posted in
Slack, can I show the exact reasoning and tool output behind it, with my own
eyes, by following a ribbon?** The chain is

```
inbox/*.vtt  ─ingest→  meetings/<date>-sync.hick        the meeting, turns derived
                         ↓ hick:upstream
                       analysis.hick                     a cell computes the numbers, pinned
                       fix.hick                          the code the meeting asked for, tested
                         ↓ hick:upstream (both)
                       message.hick                      each sentence a fragment; a check under each
                         ↓ slack:file
                       messages/2026-08-24-eng.txt       what gets pasted into Slack
                       messages/2026-08-24-eng-reply.txt the answer to a question, quoting Sam
```

and `hick lineage message.hick --output messages/2026-08-24-eng-reply.txt`
points the quoted line at the **meeting file's bytes** — as do the app's
ribbons (`docs/guarantees/lineage/a-ribbon-crosses-documents.md`).

## What a check is

Under each sentence of the message sits

```
<slack:transform select="#m1,.finding,.said" instruct="…BACKED or UNSUPPORTED…" from="…">
**BACKED.** "… 300 ms SLO" is backed by [#transcript-u3] "The SLO is p95 under 300
milliseconds for checkout" and [#p95-before-after] "…212 ms from the fix…"
</slack:transform>
```

The transform's input is every selected fragment as a labelled paragraph —
`[#transcript-u3] Sam (00:00:20.000): The SLO is…` — so the verdict can name
its sources by id. `from=` fingerprints that input and the instruction;
`hick test` reports the verdict STALE the moment the meeting note, a finding,
or the sentence changes, without spending a token. `provider=`/`model=` say
which model wrote the verdict. The sentence that is a judgment is marked
`<slack:claim by="nate" standing="judgment">` and is the one thing nothing
checks.

**The flow caught a fabrication on its first real run.** The built-in agent,
asked to draft the message from the findings upstream, wrote plausible numbers
of its own (287 ms, a pool of 20, a 500 ms timeout, a flag nobody named). The
checks under those sentences came back **UNSUPPORTED**, naming the parts with
no source. That document is committed as it happened
(`examples/receipts/hick-agent/message.hick`) because it is the point.

## What the test fixed on the way

Each of these blocked the chain and is now in the tree, with a guarantee:

- A `hick:transform` whose `select=` reached an upstream fragment
  fingerprinted an **empty** input — `hick test`/`hick refresh` parsed without
  resolving `hick:upstream`. Both now see the same document the pipeline
  sees. (`docs/guarantees/verification/a-transform-is-checked-against-the-bytes-it-read.md`)
- The transform input was the selected fragments run together with no
  separator and no labels; the checker could not tell the message sentence
  from Sam's first turn. It is now one labelled paragraph per fragment.
- `hick refresh` recorded no model, and could not restamp a document that
  bound another prefix (`<slack:transform>`); a never-written passage made
  the model echo the prompt scaffolding. All three fixed.
- `hick:upstream` carried only `copy`/`cut`: a note could not quote a meeting
  turn from the meeting upstream of it. It carries every fragment now, the
  edge stays in the tree holding what it brought, and the weave renders none
  of it. (`docs/guarantees/authoring/a-transcript-derives-its-speaker-turns.md`)
- `hick:paste select="#transcript-u7"` did not resolve even inside the
  meeting note itself: turns were selectable by `transform` and by nothing
  else. A `TranscriptHandler` registers them.
- The built-in agent's prompt carried no grammar; it invented attributes and,
  unable to see the project from its scratch workspace, **fabricated a CSV
  and pinned invented numbers**. The prompt now carries a reference and the
  rule; an expectation that starts with a line break is named by the
  mismatch message. (`docs/guarantees/agent/the-agent-is-told-the-language.md`)
- The overlay drew ribbons only to the focused document; every interesting
  ribbon in this chain is to another one. It draws to every document the
  outputs reach. (`docs/guarantees/lineage/a-ribbon-crosses-documents.md`)

## What is still clunky or painful

In rough order of how much it hurt:

1. ~~**The agent's scripts cannot see the project.**~~ Addressed the same
   day by `read_file` — a read-only, recorded tool — and by context provenance
   built on the session record (`three-provenances.md`). The scratch workspace
   still cannot see the project, on purpose; the agent now has a real way to
   look, and what it looked at is on the record.
2. ~~**An exec's output is not a fragment.**~~ A cell with an `id` is now
   quotable — `<hick:paste select="#cell"/>` pastes what it shows, with the
   cell's exec origin, so the ribbon ends at the computation
   (`docs/guarantees/authoring/a-cell-with-an-id-is-quotable.md`).
3. **The message still has to be pre-cut into fragments** (a `slack:file` is
   not selectable), but the check is one element now:
   `<slack:check claim="#m1" against=".finding,.said" from="">` — a transform
   with the question built in, fingerprinted and refreshed like one, and its
   verdict's citations draw as declared provenance (`three-provenances.md`).
4. ~~**Ingest names.**~~ A date in the file name is the meeting's date, the
   title is the rest, and the transcript's id is the note's own name
   (`#2026-08-20-checkout-latency-sync-u7`), so two meetings upstream of one
   note never collide. The examples here were ingested before that and keep
   `id="transcript"`.
5. **The overlay follows the focused document.** From the meeting note you
   still cannot see who quotes it. Since this was written: a cross-document
   click lands on the span once the document's editor mounts; the project
   graph has a **Graph** button in the status bar; and the three provenances
   draw apart (`three-provenances.md`). Still open: provenance KINDS within
   lineage (typed / transcribed / summarized / executed) look alike, and
   `hick:claim` has no rendering in the app.
6. ~~**`hick:claim` around a `copy` hides the copy.**~~ Declarations now
   descend into a claim.
7. **Small ones**, mostly gone: the mount warning fires only when a command
   actually uses the absolute path; `hick test` takes several paths; the
   CLI's `refresh`/`agent` read the desktop app's key store before the
   environment (`HICKORY_KEY_STORE` overrides the location). Still true: the
   agent's `edit_doc after=` put a title at the bottom of a document once, and
   five sessions (two failed) were needed to get three documents — two of
   those failures are what `read_file` and the grammar crib now prevent.

## The two variants

`examples/receipts/claude-code/` was authored by Claude Code writing the
documents directly and running `hick ingest`, `hick refresh`, `hick run`,
`hick test`. `examples/receipts/hick-agent/` was authored by `hick agent`
against stubs holding only a title and the `hick:upstream` edge, one prompt
per document, with `sessions/` committed — the reasoning and tool results the
question asks for. Read the README there for the commands.
