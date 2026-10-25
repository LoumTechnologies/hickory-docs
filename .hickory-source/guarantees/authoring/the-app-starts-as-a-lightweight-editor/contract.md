<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The App Starts As A Lightweight Editor

For: Authors editing executable documents and the files those documents produce.

Given the app is launched without an explicit file or folder, when its window
opens, then no folder is selected, even if an older launch remembered a folder.
It opens the editable introduction as an unsaved Untitled document, supports
Agent and Save, and shows no repository setup banner. Recent files and folders
remain available for explicit opening. File → New Window uses the same editor
surface. Closing a folderless window checks unsaved work through the same close
gate as a folder window.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-app-starts-as-a-lightweight-editor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
