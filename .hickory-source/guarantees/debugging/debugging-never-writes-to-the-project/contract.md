<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Debugging A Document Never Writes To The Project

For: Authors debugging generated code while navigating its document source.

Given a document whose cells write files, when it is debugged — interactively
from the desktop app or the MCP tools, or non-interactively by a
`<hick:capture>` on a run — then nothing the debugger does reaches the
repository: no generated file, no output the debuggee wrote, no change to the
document, and no change to a recorded transcript.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/debugging-never-writes-to-the-project.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-dap/src/capture.rs](../../../mechanisms/crates/hick-dap/src/capture.rs.md#output-9b1310a1716d)
- [crates/hickory-cli/src/debug_sessions.rs](../../../mechanisms/crates/hickory-cli/src/debug_sessions.rs.md#output-24a9413a429b)
- [crates/hickory-cli/src/serve/debug_bridge.rs](../../../mechanisms/crates/hickory-cli/src/serve/debug_bridge.rs.md#output-b6fc115ecefd)

### Tests and fixtures

- [crates/hick-dap/tests/live_capture.rs](../../../mechanisms/crates/hick-dap/tests/live_capture.rs.md#output-74b67dfe72b0)
- [crates/hickory-cli/tests/capture_run.rs](../../../mechanisms/crates/hickory-cli/tests/capture_run.rs.md#output-71575bb4ea42)
- [crates/hickory-cli/tests/debug_desktop.rs](../../../mechanisms/crates/hickory-cli/tests/debug_desktop.rs.md#output-2ee357f70dc3)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
