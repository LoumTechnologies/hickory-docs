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
