<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Filesystem Acts Keep Dired's Safety

For: Authors editing executable documents and the files those documents produce.

Given an entry in the editable Files buffer, when a person opens its context
menu, then rename, move, copy, delete, new-folder, and new-file acts are routed
to one filesystem-operation endpoint inside the open folder. A rename never
overwrites, a move or copy names a destination, `.git` and paths outside the
root are refused, and the Git pane reports the resulting working-tree change.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-tree-is-a-dired.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
