<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Tree Row Hands Its File To The Rest Of The Machine

For: Authors editing executable documents and the files those documents produce.

Given a folder open in the app, when a row in the file tree is right-clicked —
a file, a directory, a binary the app cannot display, or the folder itself —
then a menu offers the path as text (absolute, relative, and the bare name),
the path shown in this desktop's own file manager under that manager's real
name, and, for a file, the path opened in whatever program this machine
already opens that kind of file with; and when the named path is not inside
the open folder, then nothing is started and the refusal says why.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-tree-row-opens-in-the-platform.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
