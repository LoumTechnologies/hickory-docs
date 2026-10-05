# S3 publishes one manifest

Given an S3-compatible host callback that passes the adapter's conditional-write
and visible-ETag probe, a workspace batch uploads immutable content-addressed
objects and a manifest before conditionally updating its one head object.
Readers follow a single published manifest. A failed upload does not publish
part of a batch; a stale file revision or head cannot silently overwrite another
session's publication. Object reads verify content hashes; ETags remain opaque
concurrency tokens. No object revision claims git history or verified execution.

---

Last LLM verification:

- Date: 2026-10-04
- Reviewer: Codex
- Result: partially verified
- Evidence: `apps/web/src/embed/s3Storage.ts` probes create-if-absent and stale
  If-Match rejection before opening; batch upload precedes the conditional head.
- Test coverage: `s3Storage.test.ts` covers conflicting sessions, racing head
  writes, failed upload, denied credentials/renewal, rename/delete, binary export,
  incompatible conditional writes and corrupt objects. Browser acceptance uses
  a cross-origin HTTP fixture with CORS and opaque ETags.
- Caveat: no real bucket/provider credentials were supplied or tested. There
  are no advertised verified providers. The host supplies authenticated HTTPS
  requests and preserves/signs the conditional headers; the probe checks the
  configured connection when the application opens it. No cleanup policy ships.
