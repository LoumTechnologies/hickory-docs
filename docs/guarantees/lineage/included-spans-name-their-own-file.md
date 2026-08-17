# An Included Span Names Its Own File, And An Edit Through It Lands There

Given a document that splices text from other files (`<hick:include>`,
`<hick:upstream>`), when provenance is built for any output those spans feed,
then each entry's `doc_path` names the file whose bytes the span offsets
actually index — the included file, canonically — and when an edit in a
generated output maps onto such a span, then the edit is applied to that
file, byte-exactly, while the including document is untouched.

Spliced spans are byte offsets into the file they were parsed from. Before
this guarantee they were attributed to the *including* document: a reverse
edit was then written into the wrong file at those offsets — silent
corruption of whatever text happened to sit there, or a process abort when
the offset split a UTF-8 character. For a tool whose one claim is "edits land
back byte-exactly", the multi-file document — the ordinary literate-
programming case — must not be the corruption path.

Three properties hold it up:

1. **Stamping at the splice.** Include/upstream resolution stamps every
   spliced span with an id into the document's span-file table
   (`HickDocument::span_files`), innermost splice first, so nested includes
   keep their own attribution.
2. **Resolution at every origin.** Each place a `SourceOrigin::Literal` is
   built (file content, woven prose, woven fenced blocks, copy blocks)
   resolves the span's file through the table before naming a `doc_path`.
3. **A stale span is an error, not a panic.** `apply_source_edits` refuses a
   span that is out of bounds or splits a UTF-8 character, naming the
   document — it never `replace_range`s into it.

## Boundary

The built-in agent's `edit_output` tool holds an edit session for one
document; an edit that maps to an included file is *refused with routing*
(the message names the owning file and the `edit_doc` call that reaches it)
rather than applied cross-document. `hick up` and the serve API apply
cross-document edits for real. The `hick up` staleness guard covers included
files because the loop records their at-weave-time text from
`PipelineResult::span_files`.

---

**Verification notes (2026-08-16).** Stamping:
`stamp_span_file`/`span_file_id` in `crates/hick-lang/src/lib.rs`
(`resolve_includes_in_nodes`, both splice arms). Resolution:
`ProcessingContext::file_of_span` (`crates/hick-handlers/src/lib.rs`), used
by `process_file_children` and `span_file_table` in
`crates/hick-literate/src/lib.rs`, `origin_file` in
`crates/hick-literate/src/weave.rs`, and the copy handler
(`crates/hick-handlers/src/handlers/copy.rs`). Boundary check:
`apply_source_edits` in `crates/hickory-lineage/src/lib.rs`. Consumers:
`reverse::apply_to_documents` was already multi-document;
`consume_output_save`'s `expected` map now includes spliced files
(`crates/hickory-cli/src/up/mod.rs`, via `PipelineResult::span_files`);
agent routing refusal in `crates/hickory-agent/src/tools/mod.rs::edit_output`.
Test coverage: `crates/hick-lang/tests/upstream_tests.rs`
(`span_file_attribution` module — stamping, byte-exact offsets, empty table)
and `crates/hickory-cli/tests/include_lineage.rs` (include round-trip,
upstream-paste round-trip, mid-character span error). Caveats: the desktop
app's serve path applies cross-document edits but refuses (403) an included
file outside the served root; a change to an included file does not yet
re-weave documents that include it (dependency-driven re-weave is not
implemented) — until the includer is next woven, its outputs can lag the
included file.
