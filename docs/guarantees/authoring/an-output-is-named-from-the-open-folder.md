# An Output Is Named From The Open Folder

Given a document anywhere under the open folder — `tools/app.hick` writing
`<hick:file path="app.py">` — when the app lists its outputs, opens one in a
generated-file tab, carries an edit back through it, or reads it at a
commit, then the file is named the way the tree names it, relative to the
open folder (`tools/app.py`), on every route and in every answer; and a
request that spells it the document's way (`app.py`) is answered too,
under the folder-relative name.

A document writes its files relative to itself, and the weave keys them
that way. The tree, the tabs and every path a client sends are relative
to the open folder. The two agree only for a document at the root — which
is why generated files worked in the examples and every demo, and why the
first document somebody made literate in a subfolder answered *this
document produces no output named "tools/app.py" — it produces: app.md,
app.py* the moment it was opened.

Two properties hold it up:

1. **One translation, at the boundary.** `serve/api.rs` has `doc_dir`,
   `output_key` (folder-relative → the weave's key, accepting either
   spelling) and `output_path` (key → folder-relative). `list_outputs`,
   `get_output_file` and `edit_outputs` go through them; nothing else in
   the server or the client joins directories to output names by hand.
2. **The miss lists what exists in the tree's spelling.** A 404 names the
   outputs the way a person would look for them, not the way the weave
   stores them.

## Boundary

The history route (`GET /api/git/commit`) reads outputs at a commit by the
path the client sends and is unchanged here; the weave's own keys and
`hick_file` attributes stay document-relative, because that is what the
language means by them.

---

Last LLM verification:
- Date: 2026-09-05
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hickory-cli/src/serve/api.rs` — `doc_dir`, `output_key`,
  `output_path`, and their use in `list_outputs`, `get_output_file`,
  `edit_outputs`.
- Test coverage: `crates/hickory-cli/tests/outputs_in_a_subfolder.rs`
  drives the list, the open by folder-relative and document-relative
  names, and the miss, over real HTTP. Reported by the person using the
  app on 2026-09-05.
