<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Workspace Tree Is A Lens

For: Authors editing executable documents and the files those documents produce.

Given files and other objects associated with folders, when the workspace tree
is shown, then each object is a node at its place; expanding and editing a node
asks its source through capabilities that node declared; the tree has no save
path or serialized editable form of its own; and a batch spanning independent
sources reports an outcome per node rather than claiming atomicity.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-workspace-tree-is-a-lens.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
