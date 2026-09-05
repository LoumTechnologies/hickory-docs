# The Editor Reads A Document With The Parser The Server Uses

Given any `.hick` source — well formed or half typed — when the Document
view draws its structure (which spans are tags, which are blocks, where a
block's content starts and ends, and which prefix the document binds),
then that structure is what `hick-lang` reads: the same parser, the same
bytes, the same offsets. In particular a document that rebinds the prefix
(`<h:doc xmlns:h="…">`, `<slack:doc …>`) is drawn with *its* tags and its
literal `<hick:…>` text as prose; the content of a verbatim element
(`hick:input`, `hick:tool-result`, `hick:transcript`, `hick:reasoning`,
`hick:context`) is drawn as text however many tags it quotes; and a document
`hick run` would refuse is still drawn, with the first thing it would refuse
reported beside the structure.

The editor used to carry a parser of its own — one regular expression in
TypeScript — and a differential run over this repository's 29 documents on
2026-09-05 found it disagreed with `hick-lang` on 22 of them: three example
documents had no structure at all in the editor because their prefix was not
`hick`, the guide to hick showed 95 literal examples as live tags, and every
session file grew forty to sixty phantom blocks from tool results. The
editor is a view of the document; a view that reads the document by its
own rules is a second definition of the language.

Three properties hold it up:

1. **One parser.** `crates/hick-lang-wasm` compiles `hick-lang` to
   WebAssembly with one exported function, `structure`, and the web app
   loads it once at boot (`apps/web/src/editor/hickLang.ts`) before anything
   parses. The TypeScript parser is deleted; `hickDoc.ts` converts the
   parser's byte offsets to string indices and adds the markdown prose scan,
   which is about prose and not about hick.
2. **A parse that never fails.** `hick_lang::parse_lenient` recovers from
   every error the strict parser reports — an unclosed element runs to the
   end of the document, a stray closer is text, a tag that does not parse is
   text from its `<` onward, an unclosed comment is text, a broken root reads
   as a bare document — and returns the first error alongside. The recovered
   tree is for reading; `parse` decides what is a document.
3. **The built parser is a generated file.** `just codegen` builds it into
   `apps/web/src/editor/generated/hick-lang`, `scripts/check-codegen.sh`
   rebuilds and diffs it in the pre-commit hook and CI, and the build is
   reproducible byte for byte on the pinned toolchain, so a parser change
   that is not rebuilt cannot reach `master`.

## Boundary

Nothing here changes what the *server* renders: the block model, lineage
and the weave read the strict parse as before. Offsets crossing the
boundary are UTF-8 bytes on the Rust side and UTF-16 indices on the
JavaScript side, converted once per parse; the conversion is the only place
the two meet. The markdown scan (headings, quotes, tasks, inline marks) is
not part of the language and stays in TypeScript.

---

Last LLM verification:
- Date: 2026-09-05
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hick-lang/src/lib.rs` — `parse_lenient`, the `lenient`
  and `recovered` fields on `Parser`, `HickTag::close_span`,
  `HickDocument::root_tag`; `crates/hick-lang/src/structure.rs` —
  `structure` and its tests (rebound prefix, verbatim content, unclosed
  element, stray closer, malformed tag, unclosed comment, broken root, BOM
  offsets); `crates/hick-lang-wasm/src/lib.rs`;
  `apps/web/src/editor/hickLang.ts` and `hickDoc.ts` (`parseStructure`,
  `byteToUnit`); `apps/web/src/editor/hickDoc.test.ts` — the "tags" block,
  including the three former disagreements and a non-ASCII offset test;
  `apps/web/src/test-setup.ts` loads the bytes; `scripts/check-codegen.sh`
  and `.github/workflows/ci.yml` rebuild and diff.
- Caveats: reproducibility of the `.wasm` was observed (two builds, one
  hash) on the pinned toolchain with `wasm-opt` disabled; a toolchain bump
  will produce a new artifact, which is a codegen commit, not drift.
