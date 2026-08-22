# A Transform Is Checked Against The Bytes It Read, Wherever They Live

Given a `<hick:transform select="…" instruct="…" from="…">`, when `hick test`
checks it or `hick refresh` rewrites it, then the fingerprint is taken over
the **same bytes `hick:paste` would find at weave** — including fragments the
document reaches through `hick:upstream` or `hick:include` — so a passage
summarizing a meeting turn in another document goes stale the moment that
turn changes. When `hick refresh` writes a passage, then it stamps `from=`
**and** the `provider=` and `model=` that wrote it, by the tag's own span, so
a document that binds another prefix (`<slack:transform>`) is restamped too.

Three properties hold it up:

1. **Test and refresh see one view.** Both load the document through
   `transform_document`, which resolves includes and upstreams and derives
   transcript turns before any selector is evaluated. A transform over
   `#transcript-u7` in an upstream meeting fingerprints that utterance's bytes.
   Before this, both paths parsed without resolution: the selection was
   empty, the fingerprint was over nothing, and the passage could never go
   stale when the thing it summarized changed.
2. **A document checks only its own transforms.** An included file's
   transforms are stamped with that file's span id and are skipped by the
   includer — the included file checks and refreshes them itself. Otherwise
   the includer would report them twice and `hick refresh` would write a
   passage at the included file's offsets into the including file.
3. **The model is named, by the tag's span.** The fingerprint says which bytes
   under which instruction; `provider=`/`model=` say by whom — the one fact
   nobody can reconstruct from the file later. The restamp locates the tag by
   its parsed span, not by searching for `<hick:transform`, because any
   prefix may be bound to the namespace.

`<hick:check claim="#m1" against=".finding,.said">` is a transform spelled
for one question — is this sentence backed by these sources — with the
instruction built in (`hickory_cli::CHECK_INSTRUCT`, overridable by
`instruct=`): `transform_spec` gives it the same selection-and-instruction
shape, and everything above holds for it unchanged.

## Boundary

The fingerprint attests to inputs and instruction, never to the prose being
true. `provider=`/`model=` are written by refresh and are as editable as any
other attribute — they are a record, not a signature.

---

Last LLM verification:
- Date: 2026-08-22
- Reviewer: Claude (Fable 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `transform_document`, `own_transforms`, `stale_transforms` in
  `crates/hickory-cli/src/lib.rs`; `cmd_refresh`, `RefreshJob`, `body_span`,
  `restamp` in `crates/hickory-cli/src/main.rs`.
- Tests: `crates/hickory-cli/tests/transform_tests.rs`
  (`a_transform_over_an_upstream_fragment_fingerprints_the_upstream_bytes`,
  `an_included_files_transforms_are_not_the_includers`);
  `crates/hickory-cli/src/main.rs::restamp_tests`.
- Caveat: no test exercises a real `hick refresh` against a provider (it
  spends tokens); the restamp is unit-tested and the refresh loop is reviewed.
