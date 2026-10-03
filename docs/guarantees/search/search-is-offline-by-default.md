# Search Works Offline; Only An Explicit Install Touches The Network

Given a machine with no network access, when `hick search "query"` (or the
app's search panel, or the MCP `search` tool) runs, then it answers from a
locally built index with lexical (BM25) ranking, builds and caches that index
under `.hick-cache/search/`, and never attempts a network request.

Given the same project after `hick search --install-model` has been run once
(the only part of search that uses the network, and only when asked), when
any of the three surfaces searches, then results are additionally ranked by
Model2Vec static embeddings loaded from `.hick-cache/models/embed/`, fused
with the lexical ranking — and still with no network request at query time.

This is the `hick lsp install` doctrine applied to search: fetching things is
never automatic, and the tool must be useful before anything is fetched. The
CLI and the MCP tool say when they are ranking lexically only, so the upgrade
is discoverable without being pushed.

## Boundary

The index honours `.gitignore` (even before `git init`), skips hidden files,
`.hick-cache`, binaries, and files over 512 KiB. The three model files are
loaded from disk only; `model2vec-rs` is compiled with `local-only` (no
`hf-hub`, no `ureq`), so the search crate has no network code path at all —
the downloader lives in `hickory-cli::search_install`.

---

**Verification notes (2026-08-16).** Engine:
`crates/hick-search/src/lib.rs` (`SearchEngine`, BM25 + cosine + reciprocal-
rank fusion; `model2vec-rs` dependency declared with
`default-features = false, features = ["local-only", "fancy-regex"]` in
`crates/hick-search/Cargo.toml` — the feature set with no network
dependency). Surfaces: `cmd_search` in `crates/hickory-cli/src/main.rs`,
`GET /api/search` in `crates/hickory-cli/src/serve/api.rs`, `search` in
`crates/hickory-cli/src/mcp.rs::tool_catalogue`/`call_search_tool`.
Downloader: `crates/hickory-cli/src/search_install.rs`, reached only via the
`--install-model` flag. Test coverage: `crates/hick-search/src/lib.rs`
`tests` module (lexical ranking, cache persistence and reuse, gitignore
exclusion, related-code lookup, chunk line bounds). Caveat: "never attempts
a network request" is established by construction (no network code path in
`hick-search`) rather than by a test that observes traffic; the semantic
path is exercised manually (model installed, both rankings fused) but has no
automated test because the model is a 30 MB download CI does not fetch.

**Open-file scope (2026-10-03).** Given open files in the app, with or
without an open folder, when the top search bar or search panel searches,
then it includes the live contents of all open documents, generated-file
editors, and plain-file editors. Open files outside the folder participate;
unsaved contents replace indexed disk hits for the same path. With a folder
open, its index also participates. Without a folder, search does not ask
for a folder index. Open buffers use local lexical ranking.

Verification: `apps/web/src/lib/workspaceSearch.test.ts` covers open-file
search without a folder, combined folder and external-file results,
replacement of stale disk hits, live edits, closed editors, deduplication,
and result limits. Both app controls use `WorkspaceView`'s shared search
callback. CLI and MCP search retain their folder scope.
