<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Call Hierarchy Answers In Documents

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a symbol in a `.md` document or a plain file, when a client asks
`textDocument/prepareCallHierarchy` and then `callHierarchy/incomingCalls` or
`outgoingCalls`, then `hick-lsp` forwards both to the child language server
and answers in **document coordinates** — `uri`, `range`, and
`selectionRange`, which is the one an editor puts the cursor on.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/call-hierarchy-answers-in-documents.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-lsp/src/backend.rs](../../../mechanisms/crates/hick-lsp/src/backend.rs.md#output-c936496a03c1)

### Tests and fixtures

- [crates/hickory-cli/tests/lsp_languages.rs](../../../mechanisms/crates/hickory-cli/tests/lsp_languages.rs.md#output-7c4650d33fc8)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
