<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Typed Path Opens That File

For: Authors using completion, diagnostics and refactoring inside literate source.

Given the command bar in files mode, when a query is typed, then the answers
are **ranked**, best first: a path typed out in full, then an exact file name,
then a path suffix, then a name that starts with the query, then a name that
contains it, then a path that contains it. Ties go to the shorter path.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/a-typed-path-opens-that-file.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

</storycode:doc>
