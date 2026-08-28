# A Sample Shows Generated Lines Without Storing Them

Given a `<hick:sample path="…" from="…" to="…">` inside the `hick:exec` that
generates that file, when the document is woven, then those lines of the file
appear in the markdown as a fenced block tagged with the file's language,
captioned, and footed with the path and the exact line range shown — and the
`.hick` document contains none of those bytes, only the one self-closing
element that names them.

A sample never contributes to the cell it sits in. It is not part of the
command, and it is not stdin: a cell fed its own output is the opposite of
what a sample is for, so the DAG excludes it explicitly rather than by
accident.

When the file is not there, the weave says the file has not been generated
yet and to run the document. When the range has slid off the end of the file,
the weave says how many lines the file now has and to re-pick — never an empty
block, because a sample that quietly shows nothing is the stale illustration
this element exists to make impossible.

Picking one is a gesture in the app: selecting lines in a generated file
offers to show them in the document, and the server finds the owning document
and the owning cell from what the file tree already knows. A file no document
generates is refused with that reason — a sample can only show generated
bytes, since hand-written ones belong in the document itself.

## Why

The point of generating code is that nobody reads it. A document that pasted
the output back in would undo the thing it exists for, and one that showed
nothing asks the reader to believe a generator on trust. A window is the third
option: a few lines, once, to make the rule concrete.

Keeping the bytes out of the `.hick` is what makes it free. The sample costs
the document one line, it cannot be edited into a lie because there is nothing
there to edit, and it cannot go stale without the drift check saying so — the
woven markdown is compared on every `hick test` exactly like a transcript.

---

Last LLM verification:
- Date: 2026-08-28
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - This run's bytes, not the last one's: an output volume is not on disk
    until after the weave, so `crates/hick-literate/src/lib.rs` registers each
    produced volume file into `MultiDocumentState::register_produced_file`
    before `process_pipeline_outputs`, and `weave_sample` consults that first
    and falls back to disk. Without it a document's FIRST run showed
    "not generated yet" for a file it had just written.
  - Weave: `crates/hick-literate/src/weave.rs::weave_sample`, reached from the
    top-level `"sample"` arm and from the `hick:exec` child loop beside
    `weave_ingested_block` — the registry does not descend into a cell's
    children, so a sample under an exec weaves in that one place. Reads from
    disk rather than from the pipeline's produced files, because the case that
    matters is a **volume** output: a program wrote it and it is beside the
    document, not in a `hick:file` the weave holds.
  - Excluded from the run: `crates/hick-exec/src/dag.rs` — `sample` is in the
    list of tags a command's text skips, and in the list a copy body may hold
    without becoming stdin.
  - Picking: `crates/hickory-cli/src/serve/sample.rs` — `POST /api/samples`
    resolves the owning document with the same `generated_outputs` /
    `generated_by` the file tree uses, finds the declaring `path=`/`output=`
    attribute (longest claim wins, matching `generated_by`), walks out to the
    enclosing `<hick:exec>`, and inserts before its closing tag. The result is
    parsed before it is written, then pushed to the room the same way the
    reverse edit does.
  - App: `apps/web/src/shell/SamplePicker.tsx` and its wiring in
    `apps/web/src/shell/views.tsx` — the strip appears only while lines are
    selected.
  - Run end to end on this machine against `warehouse/30-api.hick`: a sample
    under the generator cell wove eight lines of
    `src/Warehouse.Web/Api/Generated/Endpoints.g.cs` into `30-api.md` with the
    caption and the `lines 11–18` footer, and the `.hick` grew two lines.
- Test coverage: `crates/hickory-cli/tests/samples.rs` (4 tests) drives the
  real binary against a document whose cell writes a volume: the lines reach
  the weave and the `.hick` is byte-identical afterwards; the element is
  neither the command nor its stdin; a range past the end of the file says how
  long the file is; a file that was never generated says to run the document.
  `crates/hickory-cli/src/serve/sample.rs::tests` (a volume
  output traced back to its cell; a `hick:file` outside any cell refused; the
  path written the way the document names it);
  `apps/web/src/shell/SamplePicker.test.tsx` (the line arithmetic, including
  that a caret is not a selection and that a selection ending at a line start
  does not count that line).
- Caveat requiring LLM review: the picker's own round trip (select, click,
  document changes) has no browser test; the server half and the line
  arithmetic are covered separately and the wiring between them was verified
  by hand.
