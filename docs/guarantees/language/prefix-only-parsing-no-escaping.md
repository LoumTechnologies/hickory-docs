# Only Namespace-Prefixed Tags Are Structured; All Other Text Is Raw

Given a `.hick` document whose root binds `xmlns:PREFIX` to the Hickory
namespace, when the document is parsed, then only tags carrying that prefix
are interpreted as structure, and every other byte — including `<`, `>`, `&`,
code in any language, and XML-looking text — is preserved verbatim with no
CDATA and no entity escaping.

## Comments are a PROSE construct (2026-09-06)

`<!--` is the one thing besides a prefixed tag that the parser ever treated
as structure, and it was treated that way everywhere — including inside the
body of a `<hick:file>`. That contradicted this guarantee outright: a
document could not hold `index.html`, a Markdown fixture, or any Rust source
with `<!--` in a string, because those four characters and everything after
them up to the next `-->` vanished between the document and the file it
tangled. With no escaping to reach for — by design — the parser is the only
thing that can stop looking.

So it does. `parse_children` (`crates/hick-lang/src/lib.rs`) carries a
`comments` flag: true at the top level and inside the wrappers whose
children are prose, false inside the body of an element that holds bytes
(`holds_bytes` in `crates/hick-lang/src/bodies.rs`: `copy`, `cut`, `file`,
`exec`, `expect`, `diagram`, `table`, `math`). Once false it stays false for
everything nested inside, so a `<hick:when>` gate inside a file does not
turn commenting back on halfway down it. `is_raw_content_tag`'s elements
(`input`, `tool-result`, `transcript`, `reasoning`, `context`) — the
stronger form of the same idea, and its neighbour in that module — were
already immune, because their bodies are not parsed at all.

Two consequences worth naming: an UNCLOSED `<!--` inside a body is no longer
an `UnclosedComment` error, because there is no comment there to be
unclosed; and commenting an entire `<hick:file>` block out from OUTSIDE it
still works, because that comment starts in prose.

---

Last LLM verification:
- Date: 2026-09-06
- Reviewer: Claude (Fable 5.1)
- Result: verified, and the one exception closed
- Evidence: `crates/hick-lang/src/lib.rs` — `detect_prefix` builds
  `open_marker`/`close_marker` from the declared prefix; `parse_children`
  scans only for those markers, and for `<!--` only where its `comments`
  flag says it is in prose (`holds_bytes`, in `crates/hick-lang/src/bodies.rs`,
  decides); all other text becomes
  `HickNode::Text` untouched. The editor runs this same parser through
  WebAssembly (`hick-lang-wasm`), so the fix reaches it with no second
  change; `apps/web/src/editor/hickDoc.ts` was read directly and has no
  comment scanner of its own (the only `<!--` in `apps/web` is
  `src/lib/weave.ts`, which WRITES a comment when commenting a block out in
  markdown or HTML, and reads none).
- Test coverage: hick-lang parser tests exercising raw `<`/`&` in bodies
  (vendored with the crate), plus five added for the comment rule in
  `crates/hick-lang/tests/comments_in_bodies.rs`: a marker inside a
  `hick:file` body, inside a `hick:copy` body, an unclosed marker inside a
  body, a gate nested inside a body that must not re-enable comments, and a
  comment inside a `hick:claim` that must still be one. `crates/hickory-cli/tests/comments_are_prose_only.rs` drives
  the real binary end to end: a copy holding `<!-- x -->` tangles
  byte-exact into a `hick:file` and appears twice in the weave, an HTML file
  keeps its comment, an unclosed marker is not an error, and a `hick:file`
  commented out from outside still writes nothing. The pre-existing
  `comment_out_hick_file_block`, `comment_is_skipped`, `multiple_comments`,
  `unclosed_comment_error` and `comment_preserves_surrounding_text` all
  still pass unchanged.
- Caveat requiring LLM review: `holds_bytes` is a closed list of tag names,
  like `SILENT_DECLARATION_TAGS` in the weave. A future element whose body is
  content would need adding by hand; the symptom of forgetting is this
  guarantee broken again for that element only.
