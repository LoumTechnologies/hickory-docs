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
