<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A protected workspace save is validated against the revision it read: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/views/AgentSettings.tsx](../a-desktop-app-bundles-its-python-agent/files/apps/web/src/views/AgentSettings.tsx.md)
- [crates/hickory-cli/examples/fskit_bundle.rs](../../../support/files/crates/hickory-cli/examples/fskit_bundle.rs.md)
- [crates/hickory-collab/src/lib.rs](../../../shared/files/crates/hickory-collab/src/lib.rs.md)
- [scripts/dist-desktop.sh](../../../shared/files/scripts/dist-desktop.sh.md)

## Tests and fixtures

- [crates/hickory-cli/tests/workspace_fs.rs](files/crates/hickory-cli/tests/workspace_fs.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
