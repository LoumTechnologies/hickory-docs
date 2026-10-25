<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/embedding/an-embed-has-no-implicit-host.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `4842072d055d9cec6d757cdb6785a1d6fbbc05a53f0dc7494a5987dbe5e70cbe`.

<book:entry path="docs/guarantees/embedding/an-embed-has-no-implicit-host.md">
# An embed has no implicit host

Given a host-controlled `DocumentEmbed` supplied with source, revision and path,
when it mounts, edits, receives host updates, or unmounts,
then it opens no API connection, socket, executor or analytics client. Its parser
loads as a static WASM asset. Changes carry the host's base revision; a host-driven
replacement is not echoed as a user edit. Two instances keep independent buffers,
registries, selections and disposal. Unsupported elements remain in the source.
Assets are resolved only through an explicit host callback.

---

Last LLM verification:

- Date: 2026-10-04
- Reviewer: Codex
- Result: verified
- Evidence: `apps/web/src/embed/DocumentEmbed.tsx`, `EditingSurface.tsx`,
  `embed.css`; `DemoEditor` now imports this shared surface. Parser initialization
  is shared and idempotent; editing state and registries are instance-scoped.
- Test coverage: `DocumentEmbed.test.tsx` preserves the fixture corpus and checks
  update/disposal/isolation. `e2e/browserEmbedding.spec.ts` checks real editing,
  assets, read-only, save, mount/unmount with API requests and WebSockets blocked
  in Chromium, Firefox and WebKit.
- Scope: source-tree React API, not a published npm package. Supported document
  rendering is shared source decoration; a complete native block/result model
  and all native element actions are still being extracted.
</book:entry>

</book:doc>
