<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Persist local document edits in ordinary files: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

The shared room coalesces changes and asks FileDocStore to persist them. The socket-close path flushes before dropping the room. The directory lock prevents hick up and the desktop process from independently owning the same directory.

## Implementation

- [crates/hickory-cli/src/serve/socket.rs](../../../shared/files/crates/hickory-cli/src/serve/socket.rs.md)
- [crates/hickory-cli/src/serve/store.rs](../../../shared/files/crates/hickory-cli/src/serve/store.rs.md)
- [crates/hickory-collab/src/lib.rs](../../../shared/files/crates/hickory-collab/src/lib.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/serve_local.rs](../../../shared/files/crates/hickory-cli/tests/serve_local.rs.md)

## Focused witnesses

- `an_edit_in_one_editor_reaches_the_other_and_the_file_on_disk` in `crates/hickory-cli/tests/serve_local.rs`.

## Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
