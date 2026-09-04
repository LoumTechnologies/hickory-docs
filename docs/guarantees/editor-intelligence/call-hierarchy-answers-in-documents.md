# Call Hierarchy Answers In Documents

Given a symbol in a `.hick` document or a plain file, when a client asks
`textDocument/prepareCallHierarchy` and then `callHierarchy/incomingCalls` or
`outgoingCalls`, then `hick-lsp` forwards both to the child language server
and answers in **document coordinates** — `uri`, `range`, and
`selectionRange`, which is the one an editor puts the cursor on.

The capability is advertised, so a client actually asks.

## The item round-trip

Call hierarchy is a three-step protocol, and the third step is the awkward
one: the client hands back an item **it was shown**, in document
coordinates, and the child needs the item **it produced**, in its own.

Rather than translate an item backwards — which means deciding which virtual
file a document range belongs to, and getting `range`, `selectionRange` and
the `data` the child put there all right — the item the child produced is
**kept**. `prepare` stores it under a key planted in the item's `data`, and a
follow-up looks it up. Every item a client can hold came from a `prepare`, so
there is always one to find.

That is the same reasoning `last_completion_language` already uses:
`completionItem/resolve` arrives with no document and no position, only a
thing this server handed out, and the answer is to remember rather than to
reconstruct.

Answers carry further items, and those are remembered too, so a hierarchy can
be walked to any depth.

## Why `selectionRange` is translated by hand

`translate_locations` maps a `{uri, range}` pair and **returns as soon as it
has** — so a `CallHierarchyItem`'s `selectionRange` would come back in the
child's coordinates, pointing at a line of the staged file, which is a line
of nothing. It is the range an editor puts the cursor on when you click a row
in the hierarchy tree, so leaving it would send a person to an arbitrary
line: the exact failure this whole area of the product exists to prevent.

## Type hierarchy is not here, and not because it is hard

It is the same three-step shape and the same store would serve it.
`lsp-types` 0.94.1 has no `type_hierarchy_provider` field on
`ServerCapabilities` at all — not even behind its `proposed` feature — so the
capability cannot be declared, and **a client never asks for what a server
does not advertise**. Handlers for it would be code nothing can reach, which
is the thing this repository keeps finding and calling a bug. It waits on the
dependency.

## Where this is reachable

Any editor pointed at `hick-lsp` — which `hick init` registers for `*.hick`.
**The app's own editor does not draw a hierarchy tree yet**, and that is
stated rather than implied: this is the server half. A panel in the app is
the remaining half, and until it exists a person using the desktop app
cannot reach this from the UI.

---

Last LLM verification:
- Date: 2026-09-04
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hick-lsp/src/backend.rs` — `call_hierarchy_provider` in
  the advertised capabilities; `prepare_call_hierarchy`, `incoming_calls`,
  `outgoing_calls`; `PreparedHierarchyItem` and the `prepared_hierarchy`
  store; `remember_hierarchy` / `remember_one` (which translates
  `selectionRange` explicitly, against the map for the item's own file and
  before `uri` is rewritten) and `hierarchy_followup`, which also remembers
  the `from`/`to` items in each answer.
- Test coverage: `crates/hickory-cli/tests/lsp_languages.rs` — the language
  sweep now asserts `callHierarchyProvider` is advertised, that a prepared
  item names the **document** and not the staged file, that both `range` and
  `selectionRange` are document lines, that something was remembered for the
  item, and that `incomingCalls` answers at all — which it cannot do unless
  the item was recalled. Passes for Python and Rust on this machine;
  TypeScript and Go skip loudly for want of a language server.
- Caveat requiring LLM review: the store grows for the life of the server
  process — one entry per item ever prepared — and nothing evicts it. That is
  bounded by how much hierarchy a person explores in one session and is
  small, but it is unbounded in principle. `incomingCalls` is asserted to
  *answer*, not to name a specific caller: what a child reports depends on
  the child, and pinning one server's answer would make this test about
  rust-analyzer rather than about coordinates.
