<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Exception Breakpoint Uses The Adapter's Own Filters

For: Authors debugging generated code while navigating its document source.

Given a debug session, when the adapter reports
`exceptionBreakpointFilters`, then the strip draws one switch per filter, in
the adapter's own words, and toggling one sends `setExceptionBreakpoints`
with the ids that are on. An adapter reporting none draws nothing.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/debug/DebugStrip.tsx](../../../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-09306f4ec1a5)
- [apps/web/src/debug/client.ts](../../../mechanisms/apps/web/src/debug/client.ts.md#output-9c2c3f79dace)
- [apps/web/src/debug/useDebugger.ts](../../../mechanisms/apps/web/src/debug/useDebugger.ts.md#output-597e991b795b)
- [crates/hickory-cli/src/serve/debug_bridge.rs](../../../mechanisms/crates/hickory-cli/src/serve/debug_bridge.rs.md#output-b6fc115ecefd)

### Tests and fixtures

- [apps/web/src/debug/DebugStrip.test.tsx](../../../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-1063c352c210)
- [apps/web/src/debug/useDebugger.test.ts](../../../mechanisms/apps/web/src/debug/useDebugger.ts.md#output-101aa3d58173)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
