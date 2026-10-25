<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/authoring/recordings-stay-local.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `75ae7b6054cebd716bb12fcce39c7b8d646486319d8f0e1f41f7072fca2796da`.

<book:entry path="docs/guarantees/authoring/recordings-stay-local.md">
# Meeting recordings remain recoverable local evidence

Given an explicitly started browser microphone recording,
when capture yields chunks, then they are retained with offset checks, size
limits and content hashes in application data outside the workspace. Stopping
finalizes the recording; interrupted chunks can be recovered, exported and
explicitly deleted. Audio is never automatically committed to Git.

Transcription uses saved machine-local adapter settings. A changed adapter
configuration invalidates an earlier approval. Remote transcription requires
fresh per-recording upload/charge consent, uses HTTPS and follows no redirects.
Cancellation ends local waiting/work; remote processing and billing may already
have occurred. Failure leaves audio available for retry. Local process execution
uses explicit argv without a shell and bounded output/time.

Audio, original transcription, imported corrected text and provider/model settings
retain separate identities. Provider timestamps and speaker labels are supplied
evidence, not authenticated identities. Corrections do not overwrite the original
transcription. Reviewed transcripts enter the existing deterministic importer.

---

Last LLM verification:

- Date: 2026-10-19
- Reviewer: Codex
- Result: partially verified; real microphone/ASR acceptance remains unverified.
- Originating requirement: #60/#47 and user approval of local retention until
  explicit deletion, visible size/location and explicit remote choice.
- Evidence: `serve/meeting_recording.rs`, `MeetingRecording.tsx`,
  `MeetingImport.tsx`, `scripts/transcribe-meeting-local.py`.
- Assumptions/exclusions: first source is browser microphone capture using
  MediaRecorder; system audio is explicitly unavailable. Browser permission,
  hardware and codec availability vary. No microphone exists in this execution
  environment. Faster Whisper and a complete local model are optional explicit
  installations; no real ASR model or provider was run here. Arbitrary local
  adapter executables are user-selected programs, not network-confined code.
- Witnesses: focused retained-byte/offset/hash/stopped-state test and UI tests
  for permission refusal, remote consent and failure retention. These fixtures
  do not establish real microphone or speech recognition support.
- Challenges/falsifier: automatic upload, capture continuing after unmount,
  changed settings reusing consent, failed transcription deleting audio, or
  corrections replacing original evidence would violate the stated behavior.
- Dependencies: existing #59 import pipeline; model settings/executable paths are
  recorded, but their mutable on-disk bytes are not a model attestation.
- Executed checks: focused Rust storage test and five recording/import UI tests
  passed; TypeScript compilation passed. Real capture + local model round trip,
  remote service integration and full native permissions remain unverified.
- Human judgment: retention policy approved; no remote upload/credits authorized.
</book:entry>

</book:doc>
