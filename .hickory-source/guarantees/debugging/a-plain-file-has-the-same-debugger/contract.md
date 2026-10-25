<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Plain File Has The Same Debugger A Document Has

For: Authors debugging generated code while navigating its document source.

Given a folder open in the app and a file in it that is not a `.md`
document — `src/main.rs`, `tools/app.py`, `index.ts`, `Program.cs` — in a
language hick can debug, when it is opened in a pane, then the pane has the
breakpoint gutter, one Debug icon beside the first line in the right gutter, and the same debugger
controls beside the paused line in the right gutter as a document; a breakpoint on line *n* is a breakpoint on line *n* of the file;
the program runs **in its own project**, built by that project's own tools
where the language needs a build; and while paused the pane shows the paused
line, the inline values, the stack, the watches, the hover value and the
inline eval exactly as a document's editor does. A frame in another file of
the folder is opened in that file's tab at the adapter's line.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
