# A local save is revision checked

Given a file read from browser-local storage with an opaque revision token,
when another tab changes it before a save or delete,
then the stale operation fails and preserves the newer stored bytes and the
editor's unsaved draft. Success is reported only after the IndexedDB transaction
commits. Quota/transaction failures do not mark the draft saved. Memory storage
identifies itself as memory only. Workspace imports either create every file or create none; conflicts never
partly replace a workspace. Workspace exports capture a consistent set
of source, binary assets and evidence without reclassifying their provenance.

---

Last LLM verification:

- Date: 2026-10-04
- Reviewer: Codex
- Result: partially verified
- Evidence: `apps/web/src/embed/storage.ts` compares revisions inside a single
  read/write transaction, waits for completion, and takes an atomic snapshot for
  export. `BrowserWorkspace.tsx` keeps draft state after errors and distinguishes
  local saves from memory-only saves.
- Test coverage: memory conflict/export tests in `DocumentEmbed.test.tsx`; real
  IndexedDB reload, two-tab conflict, atomic import/export, text search and
  injected QuotaExceededError in `browserEmbedding.spec.ts`, verified
  in Chromium, Firefox and WebKit.
- Caveat: forced browser quota exhaustion has not been measured. The injected error path
  is verified, but no universal quota limit is advertised.
