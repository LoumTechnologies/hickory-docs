<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The App Installs Its Command For The User

For: Maintainers publishing installers and people checking what a download contains.

Given a packaged desktop app, then its bundle includes the real `hick` CLI.
When the user installs the command through Settings → Command line, then it
becomes available on the user's PATH in a new terminal, without requiring a
system PATH change. The command palette provides a route to those settings.
An existing conflicting command is identified and is never overwritten.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/the-app-installs-its-command-for-the-user.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
