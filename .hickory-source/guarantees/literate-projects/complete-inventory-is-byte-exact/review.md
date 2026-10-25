<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Check every source file, including assets and permissions: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

ProjectEntry retains portable metadata beside its bytes. ProjectTree compares entries in both directions. Revision and directory adapters apply the same inventory policy before comparison.

## Implementation

- [crates/hick-lang/src/project/evaluate.rs](../../../shared/files/crates/hick-lang/src/project/evaluate.rs.md)
- [crates/hick-lang/src/project/tree.rs](../../../shared/files/crates/hick-lang/src/project/tree.rs.md)
- [crates/hickory-cli/src/project_revision.rs](../../../shared/files/crates/hickory-cli/src/project_revision.rs.md)

## Tests and fixtures

- [crates/hick-lang/tests/project.rs](../../../shared/files/crates/hick-lang/tests/project.rs.md)
- [crates/hick-literate/tests/project.rs](../../../shared/files/crates/hick-literate/tests/project.rs.md)

## Focused witnesses

- `inventory_checks_binary_modes_symlinks_and_portable_paths` in `crates/hick-lang/tests/project.rs`.

## Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
