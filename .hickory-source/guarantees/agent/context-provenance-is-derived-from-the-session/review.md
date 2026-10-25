<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Context Provenance Is Derived From The Session, Never From The Model: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-agent/src/context.rs](../../../shared/files/crates/hickory-agent/src/context.rs.md)
- [crates/hickory-agent/src/session.rs](../../../shared/files/crates/hickory-agent/src/session.rs.md)
- [crates/hickory-agent/src/tools/mod.rs](../../../shared/files/crates/hickory-agent/src/tools/mod.rs.md)
- [crates/hickory-agent/src/tools/read_file.rs](../../../shared/files/crates/hickory-agent/src/tools/read_file.rs.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)
- [crates/hickory-cli/src/serve/acp/read_receipt.rs](../../../shared/files/crates/hickory-cli/src/serve/acp/read_receipt.rs.md)
- [crates/hickory-cli/src/serve/api.rs](../../../shared/files/crates/hickory-cli/src/serve/api.rs.md)

## Tests and fixtures

- [apps/web/src/App.test.tsx](../../../shared/files/apps/web/src/App.test.tsx.md)
- [crates/hickory-agent/tests/tools_edit_session.rs](../../../shared/files/crates/hickory-agent/tests/tools_edit_session.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
