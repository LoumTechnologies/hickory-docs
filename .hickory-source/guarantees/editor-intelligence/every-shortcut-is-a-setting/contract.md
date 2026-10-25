<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Every Shortcut Is A Setting

For: Authors using completion, diagnostics and refactoring inside literate source.

Given the app's keyboard shortcuts — the menu bar's (new, open, save,
terminal, files, settings, zoom, insert), the editor's (find, format,
rename, code action, add next occurrence, select all occurrences), the
workspace's (find in folder, problems, debug continue) and the tree's
dired and column-selection keys — when a person opens Settings → Keyboard, then every one of
them is listed with the key it is on, a **profile** binds them all the way
one IDE does (Hickory's own keys, which are VS Code's; VS Code; JetBrains;
Visual Studio), and any single action can be changed by pressing its key
and typing a new one (a chord of two strokes included), unbound, or reset
to the profile's. The choice and the overrides persist beside the window
title, so the desktop app and `hick up` agree; the editor's and the
tree's bindings take effect at once for editors opened after the change,
and the menu bar's at the next launch, which the page says.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/every-shortcut-is-a-setting.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
