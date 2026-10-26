<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Breakpoint That Is Not Bound Yet Is Not Refused

For: Authors debugging generated code while navigating its document source.

Given a breakpoint set on a document line, when the debugger reports on it,
then it has **three** states rather than two, because "the adapter has not
confirmed this" and "there is nothing here to stop on" are different facts and
a person acts differently on each:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-breakpoint-that-is-not-bound-yet-is-not-refused.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/debug/cmDebug.ts](../../../mechanisms/apps/web/src/debug/cmDebug.ts.md#output-9df55a31763b)
- [apps/web/src/styles.css](../../../mechanisms/apps/web/src/styles.css.md#output-8695905e2358)
- [crates/hick-dap/src/session.rs](../../../mechanisms/crates/hick-dap/src/session.rs.md#output-3c41c22e0551)
- [crates/hickory-cli/src/serve/debug_bridge.rs](../../../mechanisms/crates/hickory-cli/src/serve/debug_bridge.rs.md#output-b6fc115ecefd)

### Tests and fixtures

- [apps/web/src/debug/cmDebug.test.ts](../../../mechanisms/apps/web/src/debug/cmDebug.ts.md#output-8d12c7278102)
- [crates/hick-dap/tests/live_session.rs](../../../mechanisms/crates/hick-dap/tests/live_session.rs.md#output-7cb62902a4d7)
- [crates/hick-dap/tests/live_session_csharp.rs](../../../mechanisms/crates/hick-dap/tests/live_session_csharp.rs.md#output-095fece59d06)
- [crates/hickory-cli/tests/debug_desktop.rs](../../../mechanisms/crates/hickory-cli/tests/debug_desktop.rs.md#output-2ee357f70dc3)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
