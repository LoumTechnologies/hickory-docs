# The Meta-LSP Forwards Whatever The Child Supports

Given an editor pointed at `hick-lsp` for `*.hick`, when it makes any request
a mainstream language server answers, then `hick-lsp` forwards it to the
server that owns that block and translates the answer back into document
coordinates. The document is the buffer being edited, so every position in
every reply is a position in the `.hick` file — never in the virtual file the
child actually saw.

The surface is the whole of what popular servers implement:

| | |
|---|---|
| Read | hover, completion (with `completionItem/resolve`), signature help, document symbol, workspace symbol, semantic tokens, inlay hint, code lens |
| Navigate | definition, declaration, type definition, implementation, references, document highlight |
| Change | code action, rename, prepare rename |
| Structure | folding range, selection range |

All of it is **advertised** in `initialize`, because a client only asks for
what the server claims. Under-advertising is how a meta-LSP ends up feeling
worse than the servers behind it: the capability works, nothing requests it,
and it reads as missing.

## Three translations that are not just range-rewriting

- **Semantic tokens** are delta-encoded against the previous token, so the
  deltas are meaningless the moment the tokens move. A document interleaves
  several blocks with prose, so tokens are decoded to absolute positions,
  mapped, **re-sorted into document order**, and re-encoded. Encoding them in
  arrival order would need a negative delta, which the protocol cannot express.
- **Legends are per server.** Two children can disagree about which integer
  means `function`, so tokens carry type *names* internally and are re-indexed
  against the single legend the editor was told about. A type this build's
  legend lacks is dropped rather than emitted as some other colour.
- **`completionItem/resolve` carries no document and no position** — only the
  item, whose `data` is the child's private bookkeeping. It is therefore
  routed to the server that produced the completion, and its resolved
  `textEdit` is mapped back through the virtual file named inside that `data`.
  Otherwise an accepted completion inserts itself lines away from the cursor.

## What it does not promise

It promises **forwarding**, not features the child does not have. A request a
child cannot answer returns nothing, and that is the child's limit showing
through, not a gap here — open-source pyright, for one, implements no
semantic tokens, folding ranges, or inlay hints at all (they are Pylance's).
The same document's Rust block gets all three from `rust-analyzer` in the
same editing session.

---

Last LLM verification:
- Date: 2026-08-14
- Reviewer: Claude (Opus 5)
- Result: verified against two real servers
- Evidence:
  - `crates/hick-lsp/src/backend.rs` — `ServerCapabilities` in `initialize`
    advertises every method in the table. Handlers are built from four
    helpers: `positional` (ranges in the same file), `positional_locations`
    (locations elsewhere, through `translate_locations`),
    `child_document_request` (whole-document), and `fan_out_array` (ask every
    block's server and merge). `translate_edit_uris` maps a rename's
    workspace edit back to the `.hick` document; `map_for_completion_data`
    finds the virtual file named inside an opaque completion `data`.
  - `crates/hick-lsp/src/semantic.rs` — `decode`, `to_source`, `encode`, and
    the advertised legend. Tests: delta decoding, encode/decode round trip,
    re-sorting tokens from two blocks, dropping an unknown token type,
    mapping into document lines, and dropping a token with no source.
  - `crates/hick-lsp/src/dispatcher.rs` — `semantic_legend` parses each
    child's legend from its initialize result and `token_legend` hands it
    back for decoding.
  - `crates/hick-lsp/src/child_lsp.rs` — declared client capabilities cover
    semantic tokens (with a full legend), inlay hints, code actions, rename
    with prepare, folding and selection ranges, code lens, document symbols,
    signature help, and workspace symbols. A child that gates a feature on
    the client asking for it will not gate it here.
  - Observed end to end on this machine, over stdio, against a real
    `.hick` document:
    - **pyright**: hover, definition, references, document highlight,
      document symbol, signature help, completion, `completionItem/resolve`
      (documentation appears only after resolve), prepare rename and rename
      all returned answers in `.hick` coordinates.
    - **rust-analyzer**: `semanticTokens/full` returned tokens landing on the
      block's real document lines, and `foldingRange` returned the two
      function bodies at document lines 6–8 and 10–13.
    - pyright returned nothing for semantic tokens, folding ranges, or inlay
      hints; querying pyright directly confirms it advertises none of the
      three, so this is the child's limit and not a forwarding fault.
- Caveats — what LLM review could NOT establish:
  - **No editor was driven.** Every observation above is a raw JSON-RPC
    session, not VS Code or Helix rendering the result.
  - Only Python and Rust children were exercised. TypeScript, Go and the rest
    are forwarded by the same generic helpers, but unobserved.
  - `codeLens` is advertised with `resolve_provider: false`; a server that
    only fills lenses in on resolve will show empty lenses.
  - Inlay hints were not observed returning data from any installed server —
    pyright has none, and rust-analyzer returned none for the small probe
    document. The forwarding path is therefore code-reviewed, not observed.
- Test coverage: the `semantic.rs` and `dispatcher.rs` unit tests protect the
  encoding and legend logic, which is the part that fails silently and
  invisibly. The forwarding itself has no automated test — it needs a real
  child server, which CI cannot assume — and that is the gap this section
  names.
