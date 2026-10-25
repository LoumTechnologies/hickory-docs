<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The App Installs Its Command For The User: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/command_path.rs](files/apps/desktop/src-tauri/src/command_path.rs.md)
- [apps/desktop/src-tauri/src/command_path_unix.rs](files/apps/desktop/src-tauri/src/command_path_unix.rs.md)
- [scripts/dist-desktop.sh](../../../shared/files/scripts/dist-desktop.sh.md)
- [scripts/install.sh](../../../shared/files/scripts/install.sh.md)

## Tests and fixtures

- [apps/web/src/views/CommandPathSettings.test.tsx](../../../shared/files/apps/web/src/views/CommandPathSettings.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
