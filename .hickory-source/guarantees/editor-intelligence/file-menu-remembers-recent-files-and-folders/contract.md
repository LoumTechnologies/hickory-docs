<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# File remembers recent files and folders

For: Authors using completion, diagnostics and refactoring inside literate source.

Given files and folders successfully opened in the desktop app, when the person
opens File → Recent Files or File → Recent Folders, then the corresponding
submenu lists up to ten existing paths, newest first, without duplicates.
Full paths distinguish identically named files or folders. An empty list has a
disabled “No Recent Files” or “No Recent Folders” item.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/file-menu-remembers-recent-files-and-folders.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
