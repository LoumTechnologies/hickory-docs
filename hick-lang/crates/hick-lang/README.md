# hick-lang

Parser and AST for `.hick` source files.

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
