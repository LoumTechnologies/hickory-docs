<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# New Document Creates Another Unnamed Buffer

For: Authors editing executable documents and the files those documents produce.

Given the app has any number of unnamed documents open, when a person chooses
File → New Document, its keyboard shortcut, or the tree's New document action,
then another blank unnamed tab opens and takes focus. Existing tabs and their
contents remain open. Tabs have distinct identities and captions (Untitled,
Untitled 2, and so on), like a conventional text editor.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/new-document-is-an-act.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
