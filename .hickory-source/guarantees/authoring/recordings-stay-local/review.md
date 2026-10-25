<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Meeting recordings remain recoverable local evidence: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/MeetingImport.tsx](../../../support/files/apps/web/src/components/MeetingImport.tsx.md)
- [apps/web/src/components/MeetingRecording.tsx](../../../support/files/apps/web/src/components/MeetingRecording.tsx.md)
- [crates/hickory-cli/src/serve/meeting_recording.rs](files/crates/hickory-cli/src/serve/meeting_recording.rs.md)
- [scripts/transcribe-meeting-local.py](files/scripts/transcribe-meeting-local.py.md)

## Tests and fixtures

- [apps/web/src/components/MeetingRecording.test.tsx](../../../shared/files/apps/web/src/components/MeetingRecording.test.tsx.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
