<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Plain File Has The Same Language Server A Document Has

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a folder open in the app and a text file in it that is not a `.md`
document — `src/main.rs`, `app.py`, `index.ts` — when it is opened in a
pane, then every language question the document editor can ask is asked
about this file too: diagnostics, hover, completion, go to definition, type
definition and implementation, find references, document highlight, inlay
hints, semantic colouring, signature help, folding, rename and code actions;
and the answers are in the file's own coordinates, at the file's own path,
with the folder the app opened as the project root. A definition that lands
in another file opens that file's tab at that line, and the file's problems
count in the status bar and appear in the Problems list.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/a-plain-file-has-the-same-language-server.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
