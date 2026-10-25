<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `40226e32841be658c400ef2ef1782c9fd878bc77b79099c065ee4e93cf7fd6ed`.

<book:entry path="crates/hick-lang/README.md">
# hick-lang

Parser and AST for `.md` source files.

A hand-written recursive-descent parser that detects the `xmlns:PREFIX` binding
to `http://www.hickorydocs.com/1.0` and recognises only `<PREFIX:*>` tags,
treating everything else as raw text. Bare `<`, `>`, and `&` characters in
non-hick content are preserved as-is.

## Key types

- `HickDocument` / `HickNode` / `HickTag` — AST for pipeline documents
- `SessionDocument` / `SessionNode` / `ActionBlock` — AST for AI-agent session files
- `SourceSpan` — byte-level provenance for every node

## Key functions

- `resolve_includes` — recursively expands `<hick:include>` directives
- `dedent` — strips leading indentation from embedded content
- `parse_session` / `is_session_source` — session-replay path entry points
</book:entry>

</book:doc>
