<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Syntax Error Names The Invariant It Hit

For: Authors editing executable documents and the files those documents produce.

Given a document whose prose quotes hick's own tag syntax — a sentence or
code span containing literal text shaped like `<prefix:name attr="value">` —
when that text is malformed enough to fail parsing, then the error explains
that hick has no escaping and points at the `h:`-prefix convention for
writing about hick's own syntax, rather than naming only the unexpected
token.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-syntax-error-names-the-invariant-it-hit.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
