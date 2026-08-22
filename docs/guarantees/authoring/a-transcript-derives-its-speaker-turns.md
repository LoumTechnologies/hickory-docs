# A Transcript Derives Its Speaker Turns, And Cannot Disagree With Itself

Given a `<hick:transcript>` holding the bytes some other tool produced, when
the document is prepared, then its speaker turns are **derived** from those
bytes as `<hick:said>` elements — each carrying who spoke, when, a class per
speaker, and a span into the original — and each is addressable by the selector
grammar that already exists. When the format is not one this knows, then the
raw text survives untouched and no turns are derived.

The reason there is one representation and not two: a document holding both a
raw block *and* a hand-maintained list of turns would be two accounts of one
meeting that could disagree, and a new thing for `hick test` to check. A
projection cannot drift from its source, because the projection *is* the source
read a second way. The file on disk keeps only the raw block.

Four properties hold it up:

1. **The raw block is raw.** `transcript` is a raw-content element alongside
   `input` and `tool-result`, so a meeting where somebody said "the hick:copy
   tag" is not a parse error, and no byte the exporter wrote is normalised,
   reflowed, or escaped — there being no escaping in this language by design.
2. **Turns are selectable, or they are not addressable at all.** `transcript`
   and `said` are fragment tags, so `select="#t"` takes the whole meeting,
   `select="#t-u12"` one turn, and `select=".said-sam"` one person's. Selecting
   a whole transcript does **not** also select its turns, or a summary's input
   would contain every sentence twice. The same selectors work for
   `hick:paste` (a turn is registered as a pasteable fragment when the document
   is prepared, with its span in the meeting file) and **across
   `hick:upstream`**: the transcript travels the edge like any fragment, its
   turns are derived after the splice and keep the meeting file's stamp, and
   the edge itself stays in the tree holding what it brought — so the weave of
   a note downstream of a meeting does not reprint the meeting.
3. **A turn knows where it came from.** Each derived `said` carries a span into
   the transcript bytes, so lineage can answer *"Sam, at 00:14:03"* rather than
   pointing at a wall of text.
4. **Timestamps are quoted, never reformatted.** They are somebody else's
   rendering of when a thing was said; rewriting one would be the first small
   lie in a document about provenance.

## Boundary

**The speaker heuristic is conservative, and errs toward finding no speaker.**
A line-oriented export is only read as attributed when the name looks like a
name: a multi-word name must be Title Case, because `We discussed the
following: indexes` is prose with a colon in it and must not become a speaker.
A transcript full of invented speakers is worse than one with none.

**Whatever the exporter said is what the note says.** "Sam", "Sam H.", and
"sam@…" are three people as far as this is concerned, and a diarization error
is repeated faithfully. The raw block is the defence: the mistake is auditable
rather than laundered.

**A multi-line cue's span is longer than its joined text**, because line
endings differ, so lineage degrades to synthetic for those turns rather than
mapping to the wrong bytes — which is the documented behaviour of a mismatched
span, not a new rule.

---

Last LLM verification (2026-08-22, Claude Fable 5): re-verified after making
turns pasteable and upstream-selectable — `TranscriptHandler` in
`crates/hick-handlers/src/handlers/copy.rs`; `collect_fragments_any` and the
`upstream` arm of `resolve_includes_in_nodes` in `crates/hick-lang/src/lib.rs`;
`declare_nodes` in `crates/hick-literate/src/lib.rs`; the `"upstream"` weave arm
in `crates/hick-literate/src/weave.rs`. Tests:
`crates/hick-transcript/tests/upstream.rs`,
`examples/receipts/*/message.hick` (a reply file quoting `#transcript-u7` two
hops upstream; `hick lineage` names the meeting file for those bytes).

Previous LLM verification:
- Date: 2026-08-18
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hick-transcript/src/lib.rs` — `detect`, `parse` (WebVTT and
  SRT through the shared cue path, line exports through `parse_lines`),
  `split_speaker_line` / `is_speaker_name` (the conservative heuristic),
  `expand` / `expand_transcript` (the derivation), `sub_span` (spans into the
  raw bytes). `crates/hick-lang/src/lib.rs` — `is_raw_content_tag` now includes
  `transcript`, and `is_fragment_tag` makes `transcript` and `said` selectable
  with no recursion into a match. `crates/hick-literate/src/weave.rs` — the
  `"transcript"` and `"said"` arms. Derivation is applied in
  `hick_literate::prepare_pipeline` (after include resolution, so a spliced
  transcript is derived too), in `hickory_cli::run_doc`, and in
  `hickory_cli::transform_document`, which `hick test` and `hick refresh` share
  so their fingerprints cannot disagree.
- Test coverage: `crates/hick-transcript/src/lib.rs::tests` (22 tests) — WebVTT
  voice spans, colon fallback, NOTE/cue-identifier lines, SRT, multi-line cues,
  the three line spellings, continuation lines, prose-with-a-colon rejection,
  headings, handles vs multi-word names, unattributed prose having no format,
  utterance spans locating their own text, slugs, derivation with and without
  an id, a declared format beating detection, markup-lookalikes inside a
  transcript, and both selector properties (whole/turn/speaker, and no double
  counting). End to end on this machine: a `.vtt` note wove to
  `**Sam** (00:14:03.000): …` lines.
- Caveats — what LLM review could NOT establish:
  - **No app surface renders a transcript.** Whether a wall of turns is usable
    in the editor, and how a reader tells a derived speaker from an asserted
    `by=` on a `hick:claim`, is unverified — the spec makes that distinction
    non-negotiable and nothing enforces it yet.
  - **Nothing has been measured at scale.** Utterance projection is per-byte
    work applied on every document preparation, and no folder of hundreds of
    hour-long meetings has been woven.
  - **Only four export shapes were tried**, all hand-written in tests. No real
    Granola, Otter, Fathom, or Zoom export has been run through this, and those
    are the files it exists to read.
