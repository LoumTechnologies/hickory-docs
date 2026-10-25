<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Plain File Opens, Edits, And Never Silently Loses A Byte

For: Authors editing executable documents and the files those documents produce.

Given a folder open in the desktop app, when the tree shows a text file that
is neither a `.md` document nor a woven output of an open document — a
README, a workflow, a justfile — then clicking it opens an editable pane, and
edits save back to the same path on disk; and when the file was rewritten on
disk between the load and the save, then the save is refused and the person
chooses — reload the disk copy or overwrite it — rather than either version
being lost silently.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-plain-file-opens-and-saves.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
