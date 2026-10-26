<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Adopt a plain file without changing its bytes

For: Authors editing executable documents and the files those documents produce.

Make a plain source file literate. The new document must regenerate the exact original file before adoption is accepted.

## What must happen

Supported plain-text adoption verifies the constructed output against the original before keeping the change. Failure restores the affected source/output state. Appending refuses an already-owned path and preserves the document's other generated products; its own woven reading may change as the authored document changes. The current adoption path refuses unsupported binary/document inputs and source syntax it cannot represent exactly.

## Limits of the evidence

Declared-project raw/hex entries can construct inputs that the plain-file adoption command does not support; those are separate interfaces.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/adoption-is-byte-exact-or-refused.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the bare-document amendment into the preservation rule. Do not promise that the woven reading stays unchanged when the maintained document itself changes.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Implementation review

### Implementation

- [apps/web/src/components/PlainFilePane.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-2f891a49445f)
- [apps/web/src/views/workspaceState.ts](../../../mechanisms/apps/web/src/views/workspaceState.ts.md#output-efd800f9ed5a)
- [crates/hickory-cli/src/adopt.rs](../../../mechanisms/crates/hickory-cli/src/adopt.rs.md#output-b820c141d77c)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-cli/src/serve/refactor.rs](../../../mechanisms/crates/hickory-cli/src/serve/refactor.rs.md#output-3a011c557915)

### Tests and fixtures

- [apps/web/src/views/workspaceState.test.ts](../../../mechanisms/apps/web/src/views/workspaceState.ts.md#output-14d3dbc939ff)
- [crates/hickory-cli/tests/serve_files.rs](../../../mechanisms/crates/hickory-cli/tests/serve_files.rs.md#output-44963d50453a)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
