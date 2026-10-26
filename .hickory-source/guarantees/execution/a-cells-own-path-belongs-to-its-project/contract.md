<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Cell's Own Path Belongs To Its Project, Not To Your Shell

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a document run through a local executor (`LocalExecutor`, and the
`SandboxedExecutor` wrapping it), when its cells execute, then the scratch
directory holding their workdirs is named after **the project the document
lives in** — so two runs of one document agree about that path no matter which
directory the person was standing in, and two different projects never share
one.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)
- [Requirements 3](requirements/03.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-cells-own-path-belongs-to-its-project.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />
- [Requirement detail](requirements/03.md)
<storycode:include file="requirements/03.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [crates/hickory-executor-sandbox/src/lib.rs](../../../mechanisms/crates/hickory-executor-sandbox/src/lib.rs.md#output-3ed8f409da24)
- [crates/hickory-executor/src/lib.rs](../../../mechanisms/crates/hickory-executor/src/lib.rs.md#output-87ce480a6eab)

### Tests and fixtures

- [crates/hickory-cli/tests/cache_inputs.rs](../../../mechanisms/crates/hickory-cli/tests/cache_inputs.rs.md#output-518e60ffbf9d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
