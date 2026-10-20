# A viewer snapshot keeps recorded results historical

Given a document that can be woven by the local application, exporting its viewer
snapshot captures source bytes, their SHA-256, capture time and the cell model
with recorded results. Export does not execute checks. The standalone browser
viewer uses the existing document embed and CellPanel to display that source,
stdout/stderr transcripts, statuses and expectation differences without a local
backend. It labels the results historical and does not certify current files.

The viewer checks the source hash and supported snapshot shape before rendering.
Snapshots can be opened from a local JSON file or a same-origin hosted JSON URL.
Neither the source hash nor a result in a supplied JSON file authenticates it.
Export is explicit: source and output may contain private information.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: partially verified
- Originating requirement: publish processed executable documents in a browser
  by reusing the existing viewer rather than creating a second renderer.
- Evidence: `serve/api.rs::snapshot_doc`, `components/SnapshotExport.tsx`,
  `embed/PublishedSnapshot.tsx`, `embed/snapshot.tsx`, `vite.site.config.ts`.
- Assumptions and exclusions: captures one document and its cell results, not
  a signed build attestation or complete dependency/environment manifest. No
  automatic upload, CI index, remote execution, or external asset export.
- Test coverage: decoder/source-tampering tests and browser snapshot acceptance.
- Executed checks: source tampering/decoder unit test passed. Standalone
  document, failed-cell diff and tampered-source rejection passed in Chromium,
  Firefox and WebKit with local APIs blocked. `serve_blame` HTTP integration
  verifies that snapshot export returns unrecorded cells without executing.
- Human judgment: not recorded.

- Dependency fingerprint: `apps/web/src/embed/PublishedSnapshot.tsx` SHA-256 `b3632eb6c3f856e95bb7a581b2b86c51a893735f889cb22c7d04850f4a22ca96`; changes to this implementation or its cited integration paths require review.

## Shared captured-view controls — 2026-10-20

When mounted under an owning UI registry, PublishedSnapshot shares transcript
visibility and explicit recording/environment/command-evidence disclosure. Paged
status preserves the historical/non-authenticating scope. Collapsed evidence
bodies have no reading action; failed command records remain initially open as
before. Replacing a capture retires controls and resets transcript state. An
asset that finishes verification after its capture retires is revoked and refused,
so a late blob URL cannot be adopted into a replacement capture.

LLM review: partially verified for `PublishedSnapshot`, `SnapshotDisclosure` and
capture-owned asset resolution. Two focused `PublishedSnapshot.controls.test.tsx`
cases passed with the nested document mocked and actual registry/controllers:
disclosure, exact evidence bytes, native collapse, stale capture refusal and a
late verified-fetch completion whose blob is revoked. Targeted TypeScript,
whitespace and file-length checks passed. Existing decoder checks were not rerun
because decoding was unchanged. Actual browser asset playback, standalone-page
native-agent attachment and full import/index/link/media mapping remain open;
these are implementation gaps, not newly declared OS exemptions. No publication,
provider/native/release/deployed validation or human acceptance is asserted.

- Dependency `apps/web/src/embed/PublishedSnapshot.tsx` SHA-256: `833964d54137f01c378346246447e0fa314ad534f3364862c49c58c20b4fc8e7`.

- Dependency `apps/web/src/embed/PublishedSnapshot.controls.test.tsx` SHA-256: `2dce032345088d92329d33d669936f93cccf052ee54c31566505ca1eeaa031d8`.
