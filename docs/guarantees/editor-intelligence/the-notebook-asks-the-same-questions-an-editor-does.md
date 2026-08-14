# The Notebook Asks The Same Questions An Editor Does

Given a document open in the notebook — the editor the desktop app hosts —
when the editor asks a language question (hover, definition, references,
completion) or the document changes, then the answer comes from the same
`hick-lsp` that serves Zed, VS Code, Helix, and Neovim, delegating to the same
child language servers, and arrives in **document coordinates**.

There is one meta-LSP, not two. A second implementation of "which language is
this block, where does this position land in the generated file, what did the
child server say about it" would drift from the first, and the notebook would
disagree with the user's own editor about code they can see in both.

The transport is channel `0x02` on the document WebSocket
(`docs/specs/freeform/api.md`): one JSON-RPC 2.0 message per binary frame,
UTF-8, no `Content-Length` headers.

Four properties hold at that boundary:

- **In-process, not a subprocess.** The bridge drives `hick-lsp` as a library
  over an in-memory pipe. The desktop app is one binary and needs nothing on
  the user's `PATH` to give a document diagnostics — which is what makes the
  app installable on its own, without the CLI.
- **The browser never learns a filesystem path.** The client names documents
  `hick:///<doc-path>`; every `uri` field is rewritten in both directions at
  this boundary. A generated file that has no path on disk comes back as
  `hick-output:///<output-path>`, which the client opens in the Output view.
- **A path that escapes the project root is refused, not translated.** The
  browser is not a trusted source of paths: `hick:///../../etc/passwd` is
  dropped rather than handed to a language server.
- **The session is lazy and connection-scoped.** No `0x02` frame, no
  `hick-lsp`, no child language servers. Closing the socket drops all three.

**The handshake belongs to the server.** `initialize` is performed by the
bridge, not the client, and the client's traffic is held until the response
comes back. `tower-lsp` silently discards every notification that arrives
before it has answered `initialize` — a notification has no reply to carry an
error — so a bridge that sent `initialize`, `initialized`, and the first
`didOpen` back to back would connect, accept everything, and answer nothing.

Availability of any child server stays best-effort, exactly as in an editor:
see [lsp-channel-degrades-never-errors](lsp-channel-degrades-never-errors.md).
A failure to start the bridge at all ends the language channel and never the
session carrying the user's edits.

---

Last LLM verification:
- Date: 2026-08-13
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - `crates/hickory-cli/src/serve/lsp_bridge.rs` — framing, URI rewriting,
    handshake gate, and lazy start. `crates/hickory-cli/src/serve/socket.rs`
    dispatches `CHANNEL_LSP`, starting the bridge on first use and logging
    (never propagating) a start failure.
  - `crates/hickory-cli/tests/serve_local.rs::the_language_channel_answers_in_document_coordinates`
    drives it over a real WebSocket the way the notebook does: `didOpen` on
    `hick:///demo.hick` produces a `publishDiagnostics` addressed in the same
    scheme, and a `textDocument/hover` gets a reply. Run on this machine with
    no `rust-analyzer` installed — the log shows the spawn failing and the
    channel answering anyway, which is the degradation guarantee holding in
    the same test.
  - Unit tests in `lsp_bridge.rs` cover round-tripping a document URI, the
    virtual-output translation, percent escapes, refusing `..` traversal, and
    that rewriting touches `uri`/`targetUri` fields only — never a URI that
    appears inside a diagnostic message, i.e. inside the user's own text.
  - The handshake ordering was found empirically, not assumed: driving
    `hick-lsp` over stdio with the three messages sent back to back produced
    the initialize response and then silence; inserting a wait produced
    `window/logMessage` and `publishDiagnostics`.
- Caveats — what LLM review could NOT establish:
  - **No browser has driven this.** The client half (`apps/web/src/lsp/`) is
    exercised by its own unit tests and the server half by the test above, but
    the two have not been run against each other in a real window.
  - Child-server answers — real rust-analyzer or pyright diagnostics inside a
    `hick:file` block, and definition results mapped back through provenance —
    are covered only by `hick-lsp`'s own tests. This machine has no child
    servers installed, so the integration test observes the degraded path.
  - One bridge per connection means N connections to the same project start N
    `hick-lsp` sessions and N sets of child servers. That is correct but not
    cheap; nothing here shares them, and nothing yet measures the cost.
- Test coverage: `the_language_channel_answers_in_document_coordinates` in
  `crates/hickory-cli/tests/serve_local.rs`, plus the unit tests in
  `crates/hickory-cli/src/serve/lsp_bridge.rs`.
