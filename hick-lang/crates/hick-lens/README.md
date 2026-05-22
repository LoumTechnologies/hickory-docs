# hick-lens

Tree-sitter bridge that maps generated output files back to their `.hick` source
positions for AI-assisted editing.

Given a converged output file and a `ProvenanceMap`, this crate parses the
output with tree-sitter, identifies structural elements (`AstElement` /
`AstElementKind`: functions, structs, classes, imports), and correlates them
with source spans.

## Key types

- `Selector` / `SelectorMatch` — structural queries (e.g., `fn name`, `struct Name`, `#id`)
- `SourceEdit` / `EditConfidence` — which `.hick` byte ranges to modify for a given selector replacement
- `PasteInsertion` / `compute_insertions` — where to insert new content into `<hick:paste>` slots

Used by `hick-agent-hick-edit` and `hick-query-edit` to translate high-level
"edit this function" requests into precise `.hick` source mutations.
