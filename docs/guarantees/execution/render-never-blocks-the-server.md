# Rendering a Document Never Blocks the Server

Given any number of documents being rendered at once, when a client calls
`GET /api/docs/:id/render`, then the weave and the project-checkout seed run
on the blocking pool — never on an async runtime worker — so unrelated
requests (`GET /api/health` among them) keep answering promptly; and when
nothing the weave depends on has changed, the render is served from a
bounded in-process cache instead of being recomputed.

---

Last LLM verification:
- Date: 2026-08-06
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `apps/server/src/runs.rs::weave_blocks` wraps the whole body
  (tempdir, `seed_checkout`, weave) in `tokio::task::spawn_blocking`, and
  `weave_blocks_json` serializes the block model there too;
  `GitStore::seed_checkout_async` moves the remaining synchronous
  tree copies (run checkouts, agent workspaces, LSP sessions) off the
  runtime. `apps/server/src/render_cache.rs` is a fixed-capacity LRU
  (256 entries, 4 MB per entry) keyed by doc id + `sha256(doc.source)` +
  the project's newest `docs.updated_at` + its newest `runs.finished_at`;
  the last-run status/transcript overlay in `routes/docs.rs::render_doc` is
  always recomputed, so run results are never served stale.
  `GitStore::commit_outputs` copies back only the pipeline's declared
  outputs, so incidental exec artifacts cannot inflate the tree that every
  render copies.
- Test coverage: `apps/server/tests/integration.rs` —
  `health_stays_responsive_while_renders_are_in_flight` (single-worker
  runtime, eight concurrent cache-missing renders over a project carrying a
  48 MB artifact) asserts `GET /api/health` stays under 500 ms and every
  render completes; `render_is_cached_and_recomputed_when_the_source_changes`
  asserts hit/miss accounting, byte-identical hits, a warm render at least
  2x cheaper than the cold one, and a miss after the source is edited.
  Unit tests in `render_cache.rs` cover boundedness, LRU order and the
  per-entry size cap.
- Caveats: the cache is per process, so a multi-instance deploy warms one
  cache per instance; and it assumes a document's weave reads nothing from
  the project checkout beyond other documents and previously committed run
  outputs.
