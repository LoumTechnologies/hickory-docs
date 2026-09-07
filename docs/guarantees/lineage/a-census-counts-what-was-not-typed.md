# A Census Counts What Was Not Typed

Given a folder of `.hick` documents, when `hick census [PATH]` is run, then
every document under PATH is **woven without executing anything** — recorded
transcripts where present, no cell ever runs — and every line of every file
those documents produce is counted in exactly one of three columns:

- **authored** — every byte of the line, its newline included, came from a
  `Literal` or `Paste` span of some document: prose, a `hick:copy`/`hick:cut`
  fragment pasted in verbatim, the literal text inside a `hick:file`.
- **generated** — the line carries at least one byte that came from anywhere
  else: an exec transcript, a `hick:sample`, a `hick:transform` passage, a
  `hick:val`, ingested bytes, an agent cell's writing, or a byte the weaver
  synthesised (a heading, a fence, an attribution line, an SVG it drew).
- **cell-output** — the line belongs to a file a cell wrote into an output
  volume. It has no byte lineage and is attributed instead to the cell:
  the volume's name, the `container=` ids that mount it, and their
  `hick:exec` source lines.

**A line with mixed bytes is generated.** This is the rule that makes the
count honest rather than flattering: the fence the weaver draws around a
`hick:file` block is the weaver's line even though every line of code inside
it is the author's, so the fence counts generated and the code inside counts
authored. Counting the fence as authored would credit a person for
punctuation they never typed; counting the block as generated because of its
fence would erase the code they did.

The report also gives, per document, that document's **own** lines split
into prose, fragment (code) and tag, so "how much of what I wrote is code"
is answerable beside "how much of what landed on disk did I write".

Three things are never counted, and the report says so rather than dropping
them silently: files under `.hick-cache/`, files git ignores, and files no
document produces — the last as a footer, `N tracked files are not produced
by any document`.

`--json` emits the same report as one object whose top-level key order is
fixed (`root`, `documents`, `files`, `folders`, `totals`,
`document_totals`, `unproduced`, `empty_volumes`), so a `hick:exec` cell
running `hick census --json .` can pin its own project's census with a
`hick:expect`.

## Boundary

**A weave cannot see an output volume's contents.**
`hick_literate::PipelineResult::volume_outputs` is filled by the live
pipeline only; the dry-run and weave-only constructors set it empty, and
running a cell to fill it is precisely what this command refuses to do. So
cell-output lines are counted **from disk**, where the last real run left
the files: every file under a declared volume's `output=` prefix that no
document tangles. That is a coarser claim than a byte range and is kept in
its own column for that reason. If nothing is on disk under a volume's
prefix, the volume is **named** in the report (`empty_volumes`) rather than
scored zero, because "no cell-output lines" and "no cell wrote" are
different facts.

A volume declared over a folder that a *sibling* document also tangles into
is resolved in the tangler's favour: byte lineage wins over a volume claim,
and volume claims are made in a second pass over the whole census for that
reason.

Binary outputs (a `hick:file` that writes a picture) are marked produced —
so they do not fall into the unproduced footer — but contribute no lines.

---

Last LLM verification:
- Date: 2026-09-06
- Reviewer: Claude Fable 5.1
- Result: verified by test and by running the command over a 822-document
  literate repository (`/home/loumtech/Documents/src/pilockdb-literate`),
  which reported 638,401 produced lines — 91.6% authored, 8.2% generated,
  0.2% cell-output — in 35 seconds, writing nothing.
- Evidence: `crates/hickory-cli/src/census.rs` (`census`, `classify_output`,
  `line_spans`, `document_lines`, `output_volumes`, `claim_volume`,
  `unproduced_count`), `crates/hickory-cli/src/bin_census.rs` (the table,
  the footer, `--by document`, `--only-generated`),
  `crates/hickory-cli/src/main.rs` (`Command::Census`). The authored set is
  `hickory_lineage::Origin::{Literal, Paste}`; every other origin, and every
  byte no provenance span covers, is generated.
- Test coverage: `crates/hickory-cli/tests/census.rs` — five tests over one
  fixture holding a literal `hick:file`, a pasted fragment, a `hick:file`
  whose bytes are an exec's output, and a cell writing into an output
  volume. They pin the three columns per file, the cell attribution
  (`volume`, `cells`, `document`), the mixed-line rule on the woven
  markdown, the JSON key order, the folder/total arithmetic, the
  per-document prose/fragment/tag split, and `--only-generated`.
- Caveat needing LLM review: the *completeness* of the "generated" set is
  argued from `hickory_lineage::from_provenance_map` rather than tested
  origin by origin — there is no fixture here for an agent cell or for
  ingested bytes, both of which are generated by the same `matches!` and
  neither of which has its own assertion.
