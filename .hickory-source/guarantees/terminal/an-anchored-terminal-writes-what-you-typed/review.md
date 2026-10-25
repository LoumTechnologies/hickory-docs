<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Anchored Terminal Writes What You Typed: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/terminal/AnchorBar.tsx](files/apps/web/src/terminal/AnchorBar.tsx.md)
- [crates/hick-term/src/anchor.rs](files/crates/hick-term/src/anchor.rs.md)
- [crates/hick-term/src/command.rs](files/crates/hick-term/src/command.rs.md)
- [crates/hick-term/src/session.rs](../../../shared/files/crates/hick-term/src/session.rs.md)
- [crates/hick-term/src/shell_integration.rs](../../../shared/files/crates/hick-term/src/shell_integration.rs.md)
- [crates/hickory-cli/src/anchor.rs](files/crates/hickory-cli/src/anchor.rs.md)
- [crates/hickory-cli/src/serve/anchored.rs](files/crates/hickory-cli/src/serve/anchored.rs.md)

## Tests and fixtures

- [crates/hick-term/tests/typed_commands.rs](files/crates/hick-term/tests/typed_commands.rs.md)
- [crates/hickory-cli/tests/anchored_terminal.rs](files/crates/hickory-cli/tests/anchored_terminal.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
