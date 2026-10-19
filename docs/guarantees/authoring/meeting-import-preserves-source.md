# Meeting import retains original evidence without invoking a model

Given an open local workspace, the meeting import control accepts UTF-8 VTT,
SRT, SBV, Markdown and text transcripts up to 1 MB. Import invokes the existing
offline transcript parser and ingestion engine. It writes the source into the
configured inbox, retains ingested originals, and returns the existing note for
identical bytes. Both legacy Markdown and current Hickory note fingerprints
participate in deduplication. Changed bytes create a distinct note.

The import preview reports supplied speaker/timestamp values and original UTF-8
utterance offsets. Missing speakers remain unknown. Utterance links open the
retained source at the actual source line. Note corrections do not modify the
retained original. Summary/action-item placeholders remain separate generated
declarations; neither importing, citing nor passing execution proves accuracy.
Import does not call a model or implicitly refresh summaries.

Unsupported media, empty/unparseable input, invalid raw transcript delimiters,
path traversal and inbox symlink escapes are refused. Cancel before submission
does not write. Once submitted, the interface explicitly states that hiding it
does not cancel or undo a server import; it does not falsely claim cancellation.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: partially verified
- Originating requirement: #59 and #47; ai-engineer-review-workflow.md.
- Evidence: meeting_import::import/post, ingest::ingest_one/existing_note,
  hick_transcript::detect/parse, MeetingImport and WorkspaceView.openHit.
- Assumptions: browser upload provides original bytes but cannot authenticate the
  exporter's identity or timestamp accuracy. Retained files are ordinary local
  files; external changes are not prevented or represented as immutable custody.
- Witnesses: meeting_import focused Rust tests cover four formats, duplicates,
  changed-source imports, exact original preservation and refused inputs.
  MeetingImport.test.tsx covers exact CRLF upload, unknown speakers, source-line
  navigation, cancel-before-submission, media rejection, duplicate and errors.
- Challenges: dedup scanner previously omitted .hick notes; test caught duplicate
  creation and scanner now checks both .md/.hick. No new transcript parser added.
- Exclusions: importing does not create a context delivery receipt or claim model
  exposure. Summary refresh remains the existing separate workflow. Server-side
  cancellation after submission is not offered. Concurrent outside filesystem
  replacement during an import remains a general filesystem race boundary.
- Dependencies: files above in this commit; changes invalidate the review.
- Executed checks: three focused Rust unit tests, one real HTTP import test and
  three focused UI tests passed; TypeScript check passed. Broad regression checks left to user.
- Human judgment: not recorded; no native interaction independently observed.
