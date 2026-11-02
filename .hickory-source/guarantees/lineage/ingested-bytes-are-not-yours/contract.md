<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Scaffold provenance remains navigable while its base stays immutable

For: engineers tracing generated code and editing the authored changes above a scaffold.

Captured scaffold bytes report `kind: "ingested"` with their document span and run fingerprint. The location remains navigable, but it is not an edit destination. Document views, generated-output views, source saves and reverse editing refuse to change the captured base or its hash. Put authored changes in an explicitly ordered patch instead; inserted patch bytes remain editable and retain their own source origins.

This supersedes the earlier editable-base requirement. The preserved detail below is historical wherever it permits editing ingested scaffold bytes; it remains useful evidence of the earlier design, not a current exception. The shared [integrity mechanism](../../../mechanisms/crates/hick-lang/src/scaffold.rs.md) validates the output hash before reconstruction, and the [editor guard](../../../mechanisms/apps/web/src/editor/immutableScaffolds.ts.md) protects the base without hiding its provenance.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/ingested-bytes-are-not-yours.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Supersede editable scaffold bases with immutable, hash-checked captures and authored patches. Retain earlier paragraphs as historical evidence.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [apps/web/src/api/types.ts](../../../mechanisms/apps/web/src/api/types.ts.md#output-8a4c3188ff6c)
- [apps/web/src/components/OutputEditorPane.tsx](../../../mechanisms/apps/web/src/components/OutputEditorPane.tsx.md#output-2571b0682164)
- [apps/web/src/styles.css](../../../mechanisms/apps/web/src/styles.css.md#output-8695905e2358)
- [crates/hick-flow/src/node.rs](../../../mechanisms/crates/hick-exec/src/node.rs.md#output-90c967a2b659)
- [crates/hick-flow/src/transform.rs](../../../mechanisms/crates/hick-exec/src/transform.rs.md#output-88934a253aef)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)
- [crates/hickory-agent/src/tools/mod.rs](../../../mechanisms/crates/hickory-agent/src/tools/mod.rs.md#output-54c7fa39e3da)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-lineage/src/lib.rs](../../../mechanisms/crates/hickory-lineage/src/lib.rs.md#output-bc43b0352ff3)

### Tests and fixtures

- [crates/hickory-cli/tests/ingest_scaffold.rs](../../../mechanisms/crates/hickory-cli/tests/ingest_scaffold.rs.md#output-6682c6282b3c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
