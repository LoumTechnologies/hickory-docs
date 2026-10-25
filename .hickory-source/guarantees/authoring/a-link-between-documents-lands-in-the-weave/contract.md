<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Link Between Documents Lands In The Weave

For: Authors editing executable documents and the files those documents produce.

Given a `.md` document containing an ordinary markdown link to another
document in the folder — `[the plan](plan.md)` — when the document is woven,
then the woven markdown carries `[the plan](plan.md)` byte for byte. A
document is authored as `.md` and weaves the same `.md`, so a link to another
document already points at the file the reader has; there is nothing to
rewrite. When the caret is on that link in the editor, Mod-click opens the
linked document as a tab.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-link-between-documents-lands-in-the-weave.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
