# Only Namespace-Prefixed Tags Are Structured; All Other Text Is Raw

Given a `.hick` document whose root binds `xmlns:PREFIX` to the Hickory
namespace, when the document is parsed, then only tags carrying that prefix
are interpreted as structure, and every other byte — including `<`, `>`, `&`,
code in any language, and XML-looking text — is preserved verbatim with no
CDATA and no entity escaping.

---

Last LLM verification:
- Date: 2026-08-05
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `crates/hick-lang/src/lib.rs` — `detect_prefix` builds
  `open_marker`/`close_marker` from the declared prefix; `parse_children`
  scans only for those markers and comments; all other text becomes
  `HickNode::Text` untouched.
- Test coverage: hick-lang parser tests exercising raw `<`/`&` in bodies
  (vendored with the crate).
