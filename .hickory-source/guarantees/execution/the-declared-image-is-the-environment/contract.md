<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Declared Image Is the Environment

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a document that declares `<hick:container image="python:3.12">`, when it
runs under the docker executor, then its cells execute inside that image — not
against whatever toolchain the host happens to have.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/the-declared-image-is-the-environment.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
