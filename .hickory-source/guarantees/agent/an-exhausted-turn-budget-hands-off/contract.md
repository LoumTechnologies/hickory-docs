<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An exhausted Python iteration budget retains incomplete work

For: Authors using an agent to change documents while retaining review control.

Given a Python run that spends its configured iteration budget without current
accepted completion, the Python graph stops and reports an error. Hickory keeps
actual applied writes, proposals, model calls and tool evidence in the session.
It does not mark the unverified work successful or make a hidden Rust handoff
model call. A later turn can use the retained session to continue deliberately.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/an-exhausted-turn-budget-hands-off.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Implementation review

### Implementation

- [apps/agent/src/hickory_agent_worker/core.py](../../verification/private-proposals-wait-for-review/contract.md#output-afffc8ba20eb)
- [apps/agent/src/hickory_agent_worker/hosted.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/hosted.py.md#output-0e03aa3e36b2)
- [apps/agent/src/hickory_agent_worker/runtime.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/runtime.py.md#output-56e1a215e79c)
- [crates/hickory-cli/src/serve/python_agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/python_agent.rs.md#output-89d47ddc390d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve; no-existing-code-or-test-reference-resolved.

</storycode:doc>
