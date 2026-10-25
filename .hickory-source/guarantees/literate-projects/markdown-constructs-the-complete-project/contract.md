<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Rebuild the program from its Markdown source

For: Maintainers moving this project to a fresh checkout or reviewing a source reorganization.

Copy project.md and its declared Markdown inputs into an empty folder. Generating them must recover the same application files without borrowing the old checkout.

## What must happen

Given an entry document and its explicitly declared Markdown closure, pure generation starts from an empty tree and constructs the current project using only those inputs. Missing sources, missing or cyclic predecessors, conflicting unordered writes, and unresolved runtime inputs fail explicitly. Declared fragments and valid retained recordings may supply bytes; ambient caches and earlier Git revisions may not. Completed New combines the independent contributions; a patch sees only its declared predecessors.


## Limits of the evidence

This promises reconstruction, not that the reconstructed application is correct. The full CLI lineage-conversion path has a known scale problem; the successful whole-repository witness uses the existing evaluator and native writer directly.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the later explicit-fragment and recording amendment into the current statement; retain the earlier exclusions as history.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
