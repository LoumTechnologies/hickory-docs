<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Output Is Named From The Open Folder

For: Authors editing executable documents and the files those documents produce.

Given a document anywhere under the open folder — `tools/app.md` writing
`<hick:file path="app.py">` — when the app lists its outputs, opens one in a
generated-file tab, carries an edit back through it, or reads it at a
commit, then the file is named the way the tree names it, relative to the
open folder (`tools/app.py`), on every route and in every answer; and a
request that spells it the document's way (`app.py`) is answered too,
under the folder-relative name.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/an-output-is-named-from-the-open-folder.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
