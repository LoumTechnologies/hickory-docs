# A Cell With An Id Is Quotable, And The Quote Keeps The Cell's Provenance

Given `<hick:exec id="p95" …>` that shows output, when another place in the
document says `<hick:paste select="#p95" />` — before or after the cell, in
prose or in a generated file — then what the cell SHOWS (its transcript,
rendered per `show=`, without the trailing line break) is pasted there, and
the pasted bytes carry the cell's **exec** origin, so `hick lineage` and the
app's ribbons end at the computation rather than one hop short of it.

The reason is the gap the receipts example kept running into: the number a
cell printed had to be retyped into a `copy` beside it for the rest of the
chain to quote, so the ribbon from "489 ms" in a message ended at a typed
finding with a `hick:expect` keeping it honest, never at the cell that
computed it. A quotable cell closes that hop.

Three properties hold it up:

1. **Registered at declaration, from the transcript.** `declare_nodes` builds
   the cell's quotable node from the handler transcripts before content is
   processed, so a paste that precedes the cell in the document resolves
   too.
2. **Origin preserved through the paste.** The paste handler returns an
   exec-origin node as itself instead of wrapping it as a literal paste — an
   exec origin is not a literal span, and a wrapper that kept only literal
   spans would have made the bytes synthetic.
3. **A copy inside a claim registers.** The declaration phase descends into
   `hick:claim` (as it does into `hick:upstream`), so a claim can wrap the
   fragment it asserts and the fragment stays pasteable.

## Boundary

A cell is quotable by `hick:paste`, not by `hick:transform select=` — a
transform's input is fragment TEXT, and a cell's text is its command. Quote
the cell into a `copy` if a transform must read its output. `show="none"`
makes a cell unquotable on purpose: it shows nothing, so it quotes as nothing.

---

Last LLM verification:
- Date: 2026-08-22
- Reviewer: Claude Fable 5
- Result: verified (implemented and reviewed in the same change)
- Evidence: `declare_nodes` in `crates/hick-literate/src/lib.rs`;
  `quotable_exec_node` in `crates/hick-handlers/src/handlers/exec.rs`; the
  `SourceOrigin::Exec` arm in `crates/hick-handlers/src/handlers/paste.rs`.
- Tests: `crates/hickory-cli/tests/exec_paste.rs`.
