<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Construct files before touching the filesystem

For engineers implementing this part of Hickory.

A document must mean the same thing in a native command and in the browser.
Start with a pure project tree: entries carry bytes, kind and executable status,
while origins retain the source coordinates needed for edits. Comparison checks
that full portable value rather than visible text alone.

The first requirement is [complete portable identity](guarantees/literate-projects/complete-inventory-is-byte-exact/contract.md): deleting an asset or changing an executable bit must fail verification even when all Rust files still compile.

## Represent the complete output

The reserved editor-buffer namespace prevents transient UI state from becoming a real output path. Path validation and collision checks belong at tree insertion, so every construction uses the same boundary.

<book:include file="mechanisms/crates/hick-lang/src/project/tree.rs.md" />

## Evaluate explicit construction dependencies

Collect fragments and operations across the declared Markdown closure, then evaluate ready operations against their declared predecessors. Assembly order is explicit even when the explanation is arranged for the reader. Unsupported constructs must fail rather than return an incomplete project.

<book:include file="mechanisms/crates/hick-lang/src/project/evaluate.rs.md" />

## Assert the pure construction behavior

These assertions exercise the project engine without executing external tools. Read the exact-output and refusal cases together: successfully parsing a document is weaker than constructing the intended inventory.

<book:include file="mechanisms/crates/hick-lang/tests/project.rs.md" />

## Review the behavioral constraints

Product reviewers can read the current outcomes, exclusions and unresolved evidence here; engineers should use them when changing the code above.

- [Rebuild the program from its Markdown source](guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md).
- [Check every source file, including assets and permissions](guarantees/literate-projects/complete-inventory-is-byte-exact/contract.md).
- [Reject a patch that does not fit its exact input](guarantees/literate-projects/replay-is-exact-and-atomic/contract.md).

Continue with [the construction order](../project.md). Remaining modules are in [the mechanism reference](mechanisms/index.md); their explicit explanation gaps remain open.
</book:doc>
