<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Teach the agent the document syntax its tools will accept

For: Authors using an agent to change documents while retaining review control.

Ask the agent to produce an executable document. Its syntax guidance should match the supported elements instead of inventing attributes.

## What must happen

The current built-in Python runtime supplies document-language guidance and shared tool descriptions. That guidance identifies supported elements and their semantics, including the declared boundaries for diagrams and generated source. Host parsing and validation remain authoritative; a prompt is not an alternative grammar or permission boundary.

## Limits of the evidence

A documented syntax list does not guarantee that a model follows it or that every explanation is accurate.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/the-agent-is-told-the-language.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Move ownership of the prompt to the current Python runtime while preserving the requirement that the advertised document vocabulary be accurate.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Implementation review

### Implementation

- [crates/hick-literate/src/expect.rs](../../../mechanisms/crates/hick-literate/src/expect.rs.md#output-162ca50044aa)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</storycode:doc>
