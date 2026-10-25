<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Meeting import retains original evidence without invoking a model: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/serve/meeting_import.rs](files/crates/hickory-cli/src/serve/meeting_import.rs.md)

## Tests and fixtures

- [apps/web/src/components/MeetingImport.test.tsx](../../agent/ui-action-dispatch/files/apps/web/src/components/MeetingImport.test.tsx.md)
- [crates/hickory-cli/tests/serve_meeting_import.rs](files/crates/hickory-cli/tests/serve_meeting_import.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
