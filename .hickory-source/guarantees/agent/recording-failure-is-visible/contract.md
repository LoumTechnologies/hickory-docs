<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Recording failure is visible

For: Authors using an agent to change documents while retaining review control.

Given a Hickory-handled agent action, when its intent cannot be recorded,
then that action is refused before execution. When recording an already executed
result fails, the caller receives an explicit incomplete-record error and must
inspect actual files; the system does not claim rollback.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/recording-failure-is-visible.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hickory-agent/src/session.rs](../../../mechanisms/crates/hickory-agent/src/session.rs.md#output-9c7ae0f28c14)
- [crates/hickory-cli/src/doc_tools.rs](../byo-agent-tool-surface/contract.md#output-011b12ab71c2)
- [crates/hickory-cli/src/serve/acp/record.rs](../../../mechanisms/crates/hickory-cli/src/serve/acp/record.rs.md#output-56d0a133283e)
- [crates/hickory-cli/src/serve/agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/agent.rs.md#output-2ce16640df92)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
