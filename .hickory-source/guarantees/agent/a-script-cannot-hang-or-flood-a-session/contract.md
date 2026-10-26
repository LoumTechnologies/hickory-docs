<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Model snippets cannot hang or flood an agent session indefinitely

For: Authors using an agent to change documents while retaining review control.

Given a model snippet interpreted in Monty, memory, execution duration, callback
count, recursion, sleep and captured output have explicit bounds. Exceeding a
bound produces a failed observation that Python can handle within its iteration
limit. Snippets have no shell or CPython execution fallback. Hosted process
lifetime, frame size and tool requests are independently bounded by Hickory.
Output beyond 64 KiB is refused rather than silently treated as complete.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/a-script-cannot-hang-or-flood-a-session.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Implementation review

### Implementation

- [apps/agent/src/hickory_agent_worker/core.py](../../verification/private-proposals-wait-for-review/contract.md#output-afffc8ba20eb)
- [apps/agent/src/hickory_agent_worker/hosted.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/hosted.py.md#output-0e03aa3e36b2)
- [apps/agent/src/hickory_agent_worker/runtime.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/runtime.py.md#output-56e1a215e79c)
- [crates/hick-handlers/src/handlers/script.rs](../../../mechanisms/crates/hick-handlers/src/handlers/script.rs.md#output-59eb10541a39)
- [crates/hickory-cli/src/serve/python_agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/python_agent.rs.md#output-89d47ddc390d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve; no-existing-code-or-test-reference-resolved.

</storycode:doc>
