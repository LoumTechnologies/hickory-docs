<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Distinguish an accepted change from a verified runtime result

For: Authors using an agent to change documents while retaining review control.

An agent finishes a changeset and says the tests passed. Accepting source equality must not manufacture an executed-test receipt.

## What must happen

Hosted Python completion requires a current independently accepted changeset checked against actual captured source bytes. A model summary, snippet print or declared check is insufficient. Iteration exhaustion and rejected completion remain incomplete. Equality establishes the captured source transition; runtime execution, semantic correctness and authorship require separate evidence. Direct generic shell/write/verify/debug primitives are withheld from the snippet surface, while registered attached-window operations may exercise existing host execution authority under their own policy and receipts.

## Limits of the evidence

Source equality cannot certify semantic correctness. External adapter reports remain declarations unless independently observed. Registered terminal/program operations can modify workspace files beyond reviewed document edits.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/completion-does-not-certify-verification.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Reconcile the old blanket withholding wording with the later explicitly granted UI capabilities. Withholding a direct tool is not a claim that the agent cannot cause registered host execution.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Implementation review

### Implementation

- [apps/agent/src/hickory_agent_worker/core.py](../../verification/private-proposals-wait-for-review/contract.md#output-afffc8ba20eb)
- [apps/agent/src/hickory_agent_worker/hosted.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/hosted.py.md#output-0e03aa3e36b2)
- [apps/agent/src/hickory_agent_worker/runtime.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/runtime.py.md#output-56e1a215e79c)
- [crates/hickory-cli/src/doc_tools.rs](../byo-agent-tool-surface/contract.md#output-011b12ab71c2)
- [crates/hickory-cli/src/serve/python_agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/python_agent.rs.md#output-89d47ddc390d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
