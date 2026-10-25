<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The App Runs The Up-Loop: External Edits Are Seen, Never Clobbered

For: Engineers using local sessions or pairing their own machines.

Given the desktop app open on a folder, when any other program — vim, a
formatter, a coding agent, `git checkout` — changes a `.md` document on
disk, then the live editor buffer follows the file (the change reconciles
into the document's room); when such a program saves an edit in a generated
output file, then the edit is carried back into the document it came from,
exactly as headless `hick up` would carry it; and when a new `.md` file
appears, then it is indexed, woven, and openable without restarting the app.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/the-app-sees-external-edits.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
