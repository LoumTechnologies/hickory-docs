# Literal files use native bytes

Given an unconditional top-level literal `file` in a document,
when the browser materializes it for local debugging,
then the Rust parser, native opening-break operation and native dedent operation
produce its bytes. Each emitted segment points to exactly equal UTF-8 source
bytes, including Unicode, BOMs and normalized indented CRLF. Nonliteral file
children, includes, conditions and duplicate file declarations are refused
rather than executed as an incomplete program.

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
