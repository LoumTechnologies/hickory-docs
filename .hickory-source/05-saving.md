<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Preserve the construction across saves and collaboration

For engineers implementing this part of Hickory.

A browser refusal does not protect disk writes from other clients. Native source
saves need the same immutable-capture rule before publication. Authoritative source
state and output generation have separate transitions: opening or observing a
file must not silently regenerate the project.

The [immutable-base rule](guarantees/lineage/ingested-bytes-are-not-yours/contract.md) applies beyond the UI: a source save that changes a captured byte or its hash must leave the previous file intact.

## Guard the shared document store

Compare old and proposed source before the atomic write. Both local saves and collaborative file persistence use this boundary; a refused rehash must leave disk unchanged, while incomplete authored text around an intact capture can still be saved.

<book:include file="mechanisms/crates/hickory-cli/src/serve/store.rs.md" />

## Expose the native serving boundary

Serving ties saved source, editor rooms and project operations together. Follow the store calls when changing persistence; do not infer that an opening watcher event authorizes filesystem construction.

<book:include file="mechanisms/crates/hickory-cli/src/serve/mod.rs.md" />

## Review the behavioral constraints

Product reviewers can read the current outcomes, exclusions and unresolved evidence here; engineers should use them when changing the code above.

- [Rebuild the program from its Markdown source](guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md).
- [Collaboration](guarantees/collaboration/index.md).

Continue with [the construction order](../project.md). Remaining modules are in [the mechanism reference](mechanisms/index.md); their explicit explanation gaps remain open.
</book:doc>
