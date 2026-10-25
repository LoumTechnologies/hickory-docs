<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Markdown Paragraphs Unwrap By Default

For: Authors editing executable documents and the files those documents produce.

Given an editable document in the Document view, when it opens, receives source
from the live room, or receives pasted or dropped text, then ordinary Markdown
paragraphs have their soft line breaks replaced with spaces. The editor wraps
the resulting paragraph visually at the ruler's measure.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/paragraphs-unwrap-by-default.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
