<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Keep language tools and debug launches tied to their captured project

For: Maintainers reorganizing source while preserving the complete generated program.

Read one side of a comparison while another file is renamed. Its tools must use the captured side and filename; choosing a debugger must not silently change ordinary language-service scope.

## What must happen

Language and debug tools receive explicit captured project and source identities. Ordinary document language services use the completed endpoint. File-block and generated-output debug requests identify the displayed code; the host rejects changed source and chooses the latest supported matching whole tree. Comparisons retain the immutable capture used by their tools, breakpoints and launch. Opening a comparison can register memory-only language context but does not execute the program. Removed or ambiguous mappings remain unavailable.

## Limits of the evidence

A tool context is not an execution receipt. Additional native platforms and adapters require their own witnesses.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Remove the historical global-selector and auto-open-different-endpoint expectations. Retain the later exact-source rule and revision-local filenames.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
