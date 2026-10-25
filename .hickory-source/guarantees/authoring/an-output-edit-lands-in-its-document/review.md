<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Save a generated edit once and retain it when saving fails: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lib/outputSave.ts](../../../shared/files/apps/web/src/lib/outputSave.ts.md)
- [crates/hickory-cli/src/up/mod.rs](../../../shared/files/crates/hickory-cli/src/up/mod.rs.md)
- [crates/hickory-cli/src/up/reverse.rs](../../../shared/files/crates/hickory-cli/src/up/reverse.rs.md)
- [crates/hickory-cli/src/up/state.rs](../../../shared/files/crates/hickory-cli/src/up/state.rs.md)
- [crates/hickory-lineage/src/lib.rs](../../../shared/files/crates/hickory-lineage/src/lib.rs.md)

## Tests and fixtures

- [apps/web/src/lib/outputSave.test.ts](../../../shared/files/apps/web/src/lib/outputSave.test.ts.md)
- [apps/web/src/shell/GeneratedBufferRecovery.test.tsx](../../../shared/files/apps/web/src/shell/GeneratedBufferRecovery.test.tsx.md)
- [crates/hickory-cli/tests/serve_local.rs](../../../shared/files/crates/hickory-cli/tests/serve_local.rs.md)
- [crates/hickory-cli/tests/up_loop.rs](../../../shared/files/crates/hickory-cli/tests/up_loop.rs.md)
- [crates/hickory-cli/tests/up_stress.rs](../../../shared/files/crates/hickory-cli/tests/up_stress.rs.md)

## Focused witnesses

- `serializes overlapping saves so each diff sees the previous result` in `apps/web/src/lib/outputSave.test.ts`.
- `does not discard generated edits typed while an earlier save is settling` in `apps/web/src/lib/outputSave.test.ts`.

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
