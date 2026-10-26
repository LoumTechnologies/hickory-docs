<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Editable Set Is the Pipeline Closure

For: Authors using an agent to change documents while retaining review control.

Given an agent session opened on a document that declares `hick:upstream`
edges, when the agent reads that document, then the result names every
document reachable through those edges — and `read_doc` and `edit_doc` accept
a `doc` argument selecting any of them, by path or by bare file name.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/the-editable-set-is-the-pipeline-closure.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hickory-agent/src/tools/mod.rs](../../../mechanisms/crates/hickory-agent/src/tools/mod.rs.md#output-54c7fa39e3da)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</storycode:doc>
