<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A running agent can be stopped

For: Authors using an agent to change documents while retaining review control.

Given a hosted Python turn, Stop cancels in-flight inference or review and
terminates the confined process group. A pending proposal is not approved by
cancellation. The session records the failure/cancellation, the turn has status
stopped and no successful answer, and the existing WebSocket terminal event
restores the Send control. Standalone Python also accepts an injectable cancel
callback and its development CLI handles Ctrl-C.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/a-running-agent-can-be-stopped.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/agent/src/hickory_agent_worker/core.py](../../verification/private-proposals-wait-for-review/contract.md#output-afffc8ba20eb)
- [apps/agent/src/hickory_agent_worker/hosted.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/hosted.py.md#output-0e03aa3e36b2)
- [apps/agent/src/hickory_agent_worker/runtime.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/runtime.py.md#output-56e1a215e79c)
- [apps/web/src/api/client.ts](../../../mechanisms/apps/web/src/api/client.ts.md#output-1e909407d4cd)
- [apps/web/src/api/types.ts](../../../mechanisms/apps/web/src/api/types.ts.md#output-8a4c3188ff6c)
- [apps/web/src/components/ChatDock.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-57557cac78c8)
- [apps/web/src/components/icons.tsx](../../../mechanisms/apps/web/src/components/icons.tsx.md#output-fcdfd36508a0)
- [crates/hickory-cli/src/serve/agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/agent.rs.md#output-2ce16640df92)
- [crates/hickory-cli/src/serve/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/mod.rs.md#output-09ea558208cd)
- [crates/hickory-cli/src/serve/python_agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/python_agent.rs.md#output-89d47ddc390d)

### Tests and fixtures

- [apps/web/src/components/ChatDock.test.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-e7bd6d898a44)
- [crates/hickory-cli/tests/serve_agent.rs](../../../mechanisms/crates/hickory-cli/tests/serve_agent.rs.md#output-d24aa77e0987)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</storycode:doc>
