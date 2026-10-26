<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A literate view writes through to ordinary source

For: Authors editing executable documents and the files those documents produce.

Given ordinary UTF-8 files in a repository, opening a source-backed literate view
or organizing its exact code slices changes no repository files. Every selected
file reconstructs byte-for-byte. A view's source does not become a document in
the repository index.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-literate-view-writes-through-to-ordinary-source.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/views/RepresentationPane.tsx](../../../mechanisms/apps/web/src/views/RepresentationPane.tsx.md#output-cd66c1c2250e)
- [crates/hickory-cli/src/serve/acp/mcp.rs](../../agent/changeset-completion-is-negotiated-and-checked/implementation/crates/hickory-cli/src/serve/acp/mcp.rs.md#output-89fc1623f7e6)
- [crates/hickory-cli/src/serve/representation.rs](../../../mechanisms/crates/hickory-cli/src/serve/representation.rs.md#output-b1c72fefef89)
- [crates/hickory-cli/src/serve/representation_store.rs](../../../mechanisms/crates/hickory-cli/src/serve/representation_store.rs.md#output-c77b1ea387e3)
- [crates/hickory-cli/src/serve/representation_tools.rs](../../../mechanisms/crates/hickory-cli/src/serve/representation_tools.rs.md#output-b70a8636a99b)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [crates/hickory-cli/tests/engine_lifecycle.rs](../../../mechanisms/crates/hickory-cli/tests/engine_lifecycle.rs.md#output-6ea35164aaa4)
- [crates/hickory-cli/tests/literate_views.rs](../../../mechanisms/crates/hickory-cli/tests/literate_views.rs.md#output-3df513c37b3c)
- [crates/hickory-cli/tests/serve_acp.rs](../../../mechanisms/crates/hickory-cli/tests/serve_acp.rs.md#output-2fce65a349e6)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
