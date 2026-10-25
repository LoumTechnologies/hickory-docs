<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Install a supported debugger archive into its declared prefix

For: Authors debugging generated code while navigating its document source.

Install codelldb from its VSIX or another catalogue archive. The executable must be found in the declared extraction layout.

## What must happen

Catalogue archive installation selects a supported platform asset, extracts into its declared prefix and locates the declared executable. Archive layouts can include an explicit destination directory, including codelldb's extension tree. Executable metadata and bundled runtime layout are retained where supported. Existing user-installed adapters remain discoverable.

## Limits of the evidence

A catalogue entry is not a passed native-platform installation witness. Unsupported platforms fail explicitly.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/an-adapter-that-ships-as-an-archive-installs-like-any-other.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the codelldb archive shape into the original archive-install rule instead of keeping a second historical exception.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
