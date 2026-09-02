# A Plain File Has The Same Language Server A Document Has

Given a folder open in the app and a text file in it that is not a `.hick`
document — `src/main.rs`, `app.py`, `index.ts` — when it is opened in a
pane, then every language question the document editor can ask is asked
about this file too: diagnostics, hover, completion, go to definition, type
definition and implementation, find references, document highlight, inlay
hints, semantic colouring, signature help, folding, rename and code actions;
and the answers are in the file's own coordinates, at the file's own path,
with the folder the app opened as the project root. A definition that lands
in another file opens that file's tab at that line, and the file's problems
count in the status bar and appear in the Problems list.

The app was an editor for documents and a viewer for everything else: a
plain file got project-text completions and nothing more, while the whole
language server sat one pane over. A note-taking IDE that cannot rename a
Rust function in the repository it is open on is not a replacement for the
IDE the person is leaving.

Three properties hold it up:

1. **The file is its own virtual file.** `hick-lsp` weaves a document into
   virtual files and maps positions through a `PositionMap`; a file that is
   not a document skips the weave and is handed to its language server AS
   ITSELF, at its real URI, with `PositionMap::identity` — so every request
   handler, every translation and the diagnostics accumulator work
   unchanged. The root is the folder from `initialize` when the file is
   under it, else the nearest enclosing repository.
2. **The pane wires the whole bundle.** `PlainFilePane` mounts the same
   `lspSupport` and `lspFeatures` extensions the document editor mounts,
   fed the live buffer through `useWorkspaceLsp`, over the workspace's own
   socket (`?doc=workspace`) — a plain file has no document room, so it has
   a room-less connection carrying only the language channel.
3. **Cross-file navigation goes through the workspace.** An editor cannot
   open a tab; a definition in another file is announced with
   `openLocation` and the workspace opens or raises the tab and reveals the
   line. The same road serves a document whose definition lands in a plain
   file, which used to select a span in the wrong buffer.

## Boundary

A file whose language `hick-lsp` cannot name, or names but has no server
for, is left as it was: project completions, no diagnostics, and no error —
a `justfile` with no language server is not a problem to report. Nothing
here formats a file; formatting is a separate guarantee.

---

Last LLM verification:
- Date: 2026-09-02
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hick-lsp/src/backend.rs` — `process_document` branches
  on `is_hick_document`; `process_plain_file` registers the file with
  `PositionMap::identity` and sends `didOpen`/`didChange` to the child rooted
  at `plain_root_uri`; `crates/hick-lsp/src/position_map.rs` `identity`.
  `apps/web/src/components/PlainFilePane.tsx` mounts `lspSupport`,
  `lspFeatures` and the LSP completion source over `useWorkspaceLsp`
  (`apps/web/src/lsp/useLsp.ts`), publishes diagnostics to
  `apps/web/src/lib/fileProblems.ts`, and routes other-file targets through
  `openLocation` (`apps/web/src/lib/revealLine.ts`), which
  `apps/web/src/views/WorkspaceView.tsx` answers with `openHit`.
- Test coverage: `crates/hickory-cli/tests/lsp_languages.rs`
  (`every_installed_language_answers_about_a_plain_file`, driven through the
  real `hick-lsp` binary against rust-analyzer and pyright on this machine);
  `apps/web/src/components/PlainFilePane.test.tsx` (the pane opens the file
  with the workspace language client and its diagnostics reach the store).
- Caveats: the browser-side behaviours (Ctrl+click, the reference list, the
  Problems row for a file) are exercised by the pane test through the
  wire, not through a real browser.
