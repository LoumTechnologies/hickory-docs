<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Keep divergent output until an explicit decision names the write: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/api/types.ts](../../../shared/files/apps/web/src/api/types.ts.md)
- [apps/web/src/components/OutputEditorPane.tsx](../../../shared/files/apps/web/src/components/OutputEditorPane.tsx.md)
- [apps/web/src/shell/FolderTreePane.tsx](../../../shared/files/apps/web/src/shell/FolderTreePane.tsx.md)
- [crates/hickory-cli/src/serve/api.rs](../../../shared/files/crates/hickory-cli/src/serve/api.rs.md)
- [crates/hickory-cli/src/serve/mod.rs](../../../shared/files/crates/hickory-cli/src/serve/mod.rs.md)
- [crates/hickory-cli/src/serve/output_resolution.rs](../../../support/files/crates/hickory-cli/src/serve/output_resolution.rs.md)
- [crates/hickory-cli/src/serve/watch.rs](../../../shared/files/crates/hickory-cli/src/serve/watch.rs.md)
- [crates/hickory-cli/src/up/mod.rs](../../../shared/files/crates/hickory-cli/src/up/mod.rs.md)
- [crates/hickory-cli/src/up/reverse.rs](../../../shared/files/crates/hickory-cli/src/up/reverse.rs.md)
- [crates/hickory-cli/src/up/state.rs](../../../shared/files/crates/hickory-cli/src/up/state.rs.md)

## Tests and fixtures

- [apps/web/src/components/DivergedBanner.test.tsx](../../../shared/files/apps/web/src/components/DivergedBanner.test.tsx.md)
- [crates/hickory-cli/src/up/state/resolution_tests.rs](../../agent/ui-action-dispatch/files/crates/hickory-cli/src/up/state/resolution_tests.rs.md)
- [crates/hickory-cli/tests/round_trip.rs](../../../shared/files/crates/hickory-cli/tests/round_trip.rs.md)
- [crates/hickory-cli/tests/serve_output_resolution.rs](../../agent/ui-action-dispatch/files/crates/hickory-cli/tests/serve_output_resolution.rs.md)
- [crates/hickory-cli/tests/up_loop.rs](../../../shared/files/crates/hickory-cli/tests/up_loop.rs.md)
- [crates/hickory-cli/tests/up_stress.rs](../../../shared/files/crates/hickory-cli/tests/up_stress.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
