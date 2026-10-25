<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Every Claim On The Home Page Is True Of The Downloaded Binary

For: Prospective users checking what the homepage demonstrates and promises.

Given a visitor reading hickorydocs.com, when they act on anything the page
tells them — a command, a flag, a file the tool writes, a hick fragment they
paste — then it behaves as the page said, because the page describes only what
the shipped binary does.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/landing/home-page-claims-are-true-of-the-binary.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
