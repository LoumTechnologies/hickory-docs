<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Debugging A Document Never Writes To The Project

For: Authors debugging generated code while navigating its document source.

Given a document whose cells write files, when it is debugged — interactively
from the desktop app or the MCP tools, or non-interactively by a
`<hick:capture>` on a run — then nothing the debugger does reaches the
repository: no generated file, no output the debuggee wrote, no change to the
document, and no change to a recorded transcript.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/debugging-never-writes-to-the-project.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
