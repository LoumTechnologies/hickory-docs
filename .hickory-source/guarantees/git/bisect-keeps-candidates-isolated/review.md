<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Bisect keeps candidates isolated: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/views/BisectPane.tsx](../../../support/files/apps/web/src/views/BisectPane.tsx.md)
- [crates/hickory-cli/src/serve/bisect.rs](../../../support/files/crates/hickory-cli/src/serve/bisect.rs.md)
- [crates/hickory-cli/src/up/mod.rs](../../../shared/files/crates/hickory-cli/src/up/mod.rs.md)

## Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../shared/files/apps/web/e2e/literate-editor.spec.ts.md)
- [crates/hickory-cli/tests/literate_views.rs](../../../support/files/crates/hickory-cli/tests/literate_views.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
