<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Patch elements use the shared diff renderer

For: Maintainers reorganizing source while preserving the complete generated program.

Given a saved executable patch element, when the document view renders its
block, then the element registry selects the patch view and draws its raw unified
diff through the same DiffView used for existing changes. The context labels
identify New as the completed project and Old as the scope input. Rendering
does not apply a patch, execute a cell, build code or launch a native tool.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/patch-view-uses-the-shared-diff-renderer.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
