# Inserting An Element Writes Hick You Could Have Typed Yourself

Given a document open in the app, when a person chooses an element from the
Insert menu and fills in its attributes, then the exact bytes the insert will
write are shown before it happens, and what lands in the buffer is ordinary
`.hick` source — correctly quoted, placed where that element belongs, with the
body left selected so the next keystroke replaces it — indistinguishable from
the same element typed by hand, and undoable with one Ctrl+Z.

The vocabulary of this language is its main barrier: a `.hick` file is XML a
person owns and edits by hand, and until this menu existed the only way to
learn what `<hick:attenuate>` takes was to read the guide in another window.
A menu that wrote XML nobody saw would replace that barrier with a worse one —
a document you cannot edit without the menu that made it. So the menu teaches
rather than hides: every field carries the sentence from the guide that says
what the attribute is for, the attribute's real name sits beside its label,
and the preview assembles as you type.

Four properties hold it up:

1. **One catalogue, and it is the authoring vocabulary only.**
   `apps/web/src/lib/insertCatalog.ts` names every element a *person* writes,
   with its attributes, which are required, and a placeholder that is a real
   example. The agent transcript tags (`hick:user`, `hick:assistant`,
   `hick:action`, `hick:observation`, `hick:tool-result`) are excluded: a
   person hand-writing one is describing a conversation that never happened.
2. **The no-escaping invariant reaches the form.** Hick never escapes an
   attribute value, so a value is quoted with `"` or `'` — whichever the value
   does not contain — and a value containing BOTH is refused with a message
   saying why and what to do instead, rather than smuggled through as
   `&quot;`.
3. **Placement is decided from what surrounds the caret.** A block element
   opens its own paragraph, adding only the newlines that are missing; a
   capability rule starts a line and takes the indentation that line already
   carries; `hick:paste` and `hick:val` go exactly where the caret is. A
   selection becomes the element's body, so selecting a paragraph and choosing
   Copy wraps it.
4. **It lands in the buffer that was last being typed in.** The panel takes
   the focus when it opens, so the target is remembered on focus
   (`editor/activeEditor.ts`), not read at insert time — and a destroyed
   editor is never the target, because a destroyed CodeMirror view accepts a
   dispatch and throws the edit away in silence.

## Boundary

Only document editors are insert targets. A generated file is written *by* a
document, so a hick tag typed into one would sit in woven output where it
means nothing.

The menu does not validate the document it writes into: inserting an
`<hick:expect>` while the caret is in prose produces a well-formed element in
a place the pipeline will reject, and the form says where the element belongs
rather than refusing. Structural validation belongs to the parser and the LSP,
which see the whole document; a menu that guessed would be wrong about
included files and conditional blocks.

Attribute VALUES are not checked against the project — a container name that
does not exist, a `select=` naming no copy, a `timeout="2m"` — because those
are exactly the errors the run and the LSP already report against the real
document.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change; exercised
  end-to-end in the running app)
- Evidence: `apps/web/src/lib/insertCatalog.ts` — the catalogue,
  `normalize` (the `#` a selector needs), `validate` (required fields and the
  both-quotes refusal), `render`/`buildInsertion` (quoting, attribute order,
  padding, where the selection lands).
  `apps/web/src/components/InsertMenu.tsx` — the two-pane panel: searchable
  grouped list, per-attribute label + real name + hint, body field seeded from
  the buffer's selection, live preview, pinned actions.
  `apps/web/src/editor/activeEditor.ts` (focus-remembered target, liveness
  check) and `apps/web/src/editor/insertElement.ts` (the dispatch, carrying
  `userEvent: "input.insertElement"` so DocumentEditor's history filter — which
  keeps un-annotated transactions out of the undo stack — lets Ctrl+Z reach
  it). Registered in `apps/web/src/editor/DocumentEditor.tsx`
  (`EditorView.focusChangeEffect`), opened from
  `apps/web/src/views/WorkspaceView.tsx` (`openInsert`, Mod-I, the
  `hickory-doc-command` route) via `apps/web/src/lib/menuBridge.ts`
  (`insert` and `insert:<element>`) and `apps/web/src/App.tsx`.
  Native menu: `apps/desktop/src-tauri/src/lib.rs` (`insert_menu`,
  `INSERT_GROUPS`, `CmdOrCtrl+I`).
- Test coverage: `apps/web/src/lib/insertCatalog.test.ts` (28 tests: catalogue
  shape, no transcript tags, search ranking, normalization, both-quote
  refusal, attribute order and omission, quote switching, and every placement
  case — block padding, start of document, selection wrapping, inline, child
  indentation including multi-line);
  `apps/web/src/components/InsertMenu.test.tsx` (19 tests: the panel's list,
  preselection from a menu pick and its fallback, filtering, the required-field
  refusal, what it hands back, Escape; plus the target-buffer rules and what
  actually lands in a live CodeMirror buffer, including the `userEvent`
  annotation);
  `apps/web/src/lib/menuBridge.test.ts` (the `insert:<element>` action and its
  malformed forms);
  `apps/desktop/src-tauri/tests/insert_menu_matches_catalogue.rs` (the native
  menu and the page's catalogue name the same elements, both directions).
- Caveat requiring review: the Rust parity test reads the catalogue as text
  (`id: "…"` at the start of a line), so a future reformatting of that file
  could make it silently find nothing — it guards against that with a lower
  bound on the count, which is a smoke alarm rather than a parser. And the
  desktop menu itself is not exercised by a test: `insert_menu()` is built
  from the same table the parity test reads, but that the Tauri submenu
  renders and forwards its ids was verified by reading, not by running the
  packaged app. The web path was driven end to end in a browser.
