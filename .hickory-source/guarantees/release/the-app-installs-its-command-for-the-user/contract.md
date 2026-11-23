<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The App Installs Its Command For The User

## Simple frontend cutover

Install `hick` from the CLI archive independently of the Simple desktop package.
The shell-command installer described below was part of the retired desktop
bundle. See [Install Hickory Simple](../../../support/files/docs/users/install-desktop.md.md).

## Earlier release design and evidence

For: Maintainers publishing installers and people checking what a download contains.

Given a packaged desktop app, then its bundle includes the real `hick` CLI.
When the user installs the command through Settings → Command line, then it
becomes available on the user's PATH in a new terminal, without requiring a
system PATH change. The command palette provides a route to those settings.
An existing conflicting command is identified and is never overwritten.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/the-app-installs-its-command-for-the-user.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

</storycode:doc>
