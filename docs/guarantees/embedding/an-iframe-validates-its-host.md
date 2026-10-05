# An iframe validates its host

Given a static document iframe configured with a concrete parent origin,
when a versioned embedding message arrives,
then it is accepted only from its actual parent window at that exact origin.
Load supplies source/revision and optional asset URLs; Save returns the current
source for the host to persist. Disposal unmounts the editor and terminates its
debug worker. No credentials are sent to wildcard origins. Storage remains the
host's responsibility when iframe-local persistence is unavailable.

---

Last LLM verification:

- Date: 2026-10-04
- Reviewer: Codex
- Result: verified
- Evidence: `apps/web/src/embed/iframe.tsx` checks `event.source`, `event.origin`,
  channel and protocol version before acting, and uses a concrete target origin.
- Test coverage: the cross-origin sandboxed iframe Playwright flow verifies
  load/save, rejected wrong-window/origin messages and disposal in Chromium,
  Firefox and WebKit.
- Scope: `allow-scripts allow-same-origin` on a separate iframe origin. Opaque
  sandbox origins and remote/hybrid execution are not supported by this wrapper.
