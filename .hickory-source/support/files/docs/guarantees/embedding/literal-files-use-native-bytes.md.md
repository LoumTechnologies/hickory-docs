<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/embedding/literal-files-use-native-bytes.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `746b36a5f060892a7fd514af49542840ed4b5c2cc994c810c387d03413085c6a`.

<book:entry path="docs/guarantees/embedding/literal-files-use-native-bytes.md">
# Literal files use native bytes

Given an unconditional top-level literal `file` in a document,
when the browser materializes it for local debugging,
then the Rust parser, native opening-break operation and native dedent operation
produce its bytes. Each emitted segment points to exactly equal UTF-8 source
bytes, including Unicode, BOMs and normalized indented CRLF. Nonliteral file
children, includes, conditions, duplicate file declarations, and declared
project/patch/entry construction are refused rather than executed as an incomplete
program. Complete project generation uses the separate pure evaluator with its
full Markdown closure; this does not add browser container or native execution.

---

Last LLM verification:

- Date: 2026-10-04
- Reviewer: Codex
- Result: verified
- Evidence: `crates/hick-lang/src/literal_files.rs`, exported by the crate's
  facade; `crates/hick-lang-wasm/src/lib.rs::literal_files` returns versioned JSON.
  The native file/weave path now imports `hick_lang::strip_opening_break`.
  Existing `hick_lang::dedent` is used unchanged. `compile.ts` composes TS emit
  mappings through these byte segments into the frozen document.
- Test coverage: Rust literal-file tests and native pipeline parity in
  `crates/hick-literate/tests/browser_literal_files.rs`; `DocumentEmbed.test.tsx` checks the
  WASM byte segments; browser runtime and E2E tests exercise Unicode mapping.
- Scope: literal files only. This is not full weave/lineage/reverse-edit parity.


Verification update (2026-10-12, Codex): the new declared-project/patch guard is
in `literal_files.rs::literal_files` and protected by
`hick-lang/tests/project.rs::the_legacy_literal_subset_cannot_return_a_half_patched_program`
and `apps/web/src/elements/patch/view.test.tsx` using the actual WASM parser.
Executed checks and source hashes are in
`docs/guarantees/literate-projects/implementation-evidence.md`. Human judgment
has not been recorded.
</book:entry>

</book:doc>
