<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Language Server Is Found Without Being Configured: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-lsp/src/child_lsp.rs](../../../shared/files/crates/hick-lsp/src/child_lsp.rs.md)
- [crates/hick-lsp/src/discovery.rs](../../../shared/files/crates/hick-lsp/src/discovery.rs.md)
- [crates/hick-plugins/src/lsp_discovery.rs](../../../support/files/crates/hick-plugins/src/lsp_discovery.rs.md)
- [crates/hickory-cli/src/init.rs](../../../shared/files/crates/hickory-cli/src/init.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/lsp_languages.rs](../../../shared/files/crates/hickory-cli/tests/lsp_languages.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
