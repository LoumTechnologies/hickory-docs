<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Save Can Format First, With The File's Own Formatter

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a file or document with a language server behind it, when the person
presses Shift+Alt+F, then the buffer is formatted by whatever formatter that
language server offers — rustfmt through rust-analyzer, the Python server's
own, prettier through the TypeScript server — and the edits are applied as
one user edit; and given the per-user setting **Format on save** is on, when
the person chooses Save or Save All, then the focused buffer is formatted
first and the formatted text is what is saved. A `hick:file` block in a
document is formatted in the block's own indentation. The setting is off by
default.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/save-can-format-first.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
