<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
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


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/browser-output-has-runtime-source-evidence.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#output-31019d3567c7)
- [apps/web/src/debug/browser/runtime.test.ts](../../../mechanisms/apps/web/src/debug/browser/runtime.ts.md#output-de529cc60d5d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
