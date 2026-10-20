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

## Scoped UI requests — 2026-10-20

The same validated parent may send structured `ui` requests for the scope minted
by the latest accepted load. The frame fixes agent identity/policy; actor/scope
injection, text commands and disposed scopes are refused. Current explicit
controls provide progressive observation and same-controller action invocation.
A replacement load disposes the old registry and remounts the document. Replies
identify their original scope even if a callback settles late. Requested/result
receipt events go to the concrete owning parent; durable storage remains the
host's responsibility and is not claimed by receipt emission.

LLM implementation review: partially verified for `iframe.tsx`, `uiHostSession.ts`
and `EditingSurface.tsx`; this is an additive shared-controller path, not a grant
of engine/provider/native execution. Three focused real-CodeMirror cases passed
for structured editing/retry, actor/scope/text/disposal refusal, actual read-only
policy and no implicit fetch. One focused real Chrome cross-origin flow passed
against a local Vite server: rejected wrong sender/origin, native edit, scoped
model edit, host save, wrong-scope refusal, replacement-load retirement, dispose,
and zero engine endpoint requests. The first attempt's blocker also caught Vite
source-module imports; it was restricted to actual `/api/` endpoints and rerun.
Targeted TypeScript, whitespace and file-length checks passed.

Other browsers, actual provider callers, durable host receipt retention, late
cross-origin callback completion and complete rendered-asset mapping were not
witnessed here. Earlier three-browser evidence above is historical. Full #72
parity and native/released/deployed acceptance remain incomplete. Human judgment
is unrecorded. Source fingerprints follow; changed inputs invalidate this check.

- `apps/web/src/embed/iframe.tsx`: `ad6e46f073d0ba50299fbc939940233367290084271ea4d59899790d02f7cc99`

- `apps/web/src/embed/uiHostSession.ts`: `e7011b60d2e64b931391984bc7770fe0a1bc67f98b6cff5d52010c43ab8c9711`

- `apps/web/src/embed/EditingSurface.tsx`: `b751d3623dc4cd11dbad5859160c1be94ec78b0aca28e497998e405b105d54f0`

- `apps/web/e2e/browserEmbedding.spec.ts`: `21a12ab1dc3dbebbc3be7b6b0645e6abfb50065bdd0215fff59947ef6fdadca5`
