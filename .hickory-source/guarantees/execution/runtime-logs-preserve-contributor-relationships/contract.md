<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Execution logs preserve contributor relationships and historical event identity

For: Authors running documents on their own machine and reviewers assessing their side effects.

The execution log and inline view select the same run and actual invocation
occurrences. Output is LF-framed per stream/category with lossless per-event
chunk-byte and displayed-line-byte contributions. UTF-8 is decoded after
assembly; CR/escape controls are visibly escaped. Partial lines and lines with
multiple emitting statements retain all contributor identities, without text
matching or invented single origins.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/runtime-logs-preserve-contributor-relationships.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
