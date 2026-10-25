<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Unstable Updates Require Consent: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/updater.rs](files/apps/desktop/src-tauri/src/updater.rs.md)
- [scripts/build-unstable-macos.sh](../../../shared/files/scripts/build-unstable-macos.sh.md)

## Tests and fixtures

- [scripts/tests/test_release_unstable.py](../../../shared/files/scripts/tests/test_release_unstable.py.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
