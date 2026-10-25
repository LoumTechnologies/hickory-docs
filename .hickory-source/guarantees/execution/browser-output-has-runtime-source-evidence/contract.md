<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Browser output carries actual runtime source and occurrence evidence

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a browser JS/TS run with a captured document digest, console output is
recorded in hick-observe v1 run/event shapes. Emission anchors come from the live
interpreter CallExpression, independently of paused-line state. TypeScript
source maps and literal-file segments map to exact UTF-8 document anchors;
unmapped code keeps block-only attribution. Exact anchors identify the emitting
statement location, not a fabricated whole-statement span across source fragments.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/browser-output-has-runtime-source-evidence.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
