<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Language Server Is Found Without Being Configured

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a `hick:file` block whose `path` names a language — `app.py`,
`src/main.rs`, `index.ts` — when `hick-lsp` needs a server for that block,
then it finds one already installed on the machine and uses it, with no
configuration written by anyone. Nothing in the product asks a user which
language server to use, and a repository with no `.hick-lsp.json` gets the
same intelligence as one with a hand-tuned file.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/a-language-server-is-found-without-being-configured.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-lsp/src/child_lsp.rs](../../../mechanisms/crates/hick-lsp/src/child_lsp.rs.md#output-ebc4fbd32a25)
- [crates/hick-lsp/src/discovery.rs](../../../mechanisms/crates/hick-lsp/src/discovery.rs.md#output-9369ac0436fb)
- [crates/hick-plugins/src/lsp_discovery.rs](../../../mechanisms/crates/hick-plugins/src/lsp_discovery.rs.md#output-25da6e354bac)
- [crates/hickory-cli/src/init.rs](../../../mechanisms/crates/hickory-cli/src/init.rs.md#output-a329ec22d0b8)

### Tests and fixtures

- [crates/hickory-cli/tests/lsp_languages.rs](../../../mechanisms/crates/hickory-cli/tests/lsp_languages.rs.md#output-7c4650d33fc8)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
