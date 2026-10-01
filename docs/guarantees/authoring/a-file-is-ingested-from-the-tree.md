# A File Is Ingested From The Tree

Given a plain text file in the tree — not a document, not a file some
document already writes, not a binary — when a person right-clicks it,
then the menu offers **Make literate**, which writes a new `.md` document beside
the file owning its bytes exactly, and, when a document is focused,
**Ingest into <that document>**, which appends the file's bytes to it as a
block; either way the document it landed in opens. Both are the same
adoption `hick ingest --from file` performs, through `POST /api/adopt`,
and both are refused for anything that is not a plain text file, so the
verb never appears where it cannot be honoured.

The verb was reachable only from a plain file's own toolbar and from the
command line. Ingest is a verb of the tree: the file is a row before it is
a pane.

Two properties hold it up:

1. **The row knows what it is.** `fileAction` already classifies every row
   (document, generated, plain file, inert); the menu is built with
   `plainText` only for the third, and "Ingest into …" only when the tree
   can name the focused document (`docPathOf`, found in the tree by id).
2. **The document opens, not the file.** The adopt answer carries the
   document's id, and the pane opens it through the same `onOpen` a click
   on a document row uses.

## Boundary

The inbox — a transcript dropped in a watched folder becoming a note — is
a different ingest (`docs/specs/freeform/ingest.md`) and is not on the
menu; a file marked with dired marks is still ingested one at a time, by
the row it is asked of.

---

Last LLM verification:
- Date: 2026-09-05
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `apps/web/src/shell/treeMenu.ts` (`literate`, `ingest`
  actions and `TreeMenuContext`), `FolderTreePane.tsx` (`runItem`'s adopt
  branch, `docPathOf`, `plainText` on the row menu);
  `crates/hickory-cli/src/serve/refactor.rs` `adopt`.
- Test coverage: `apps/web/src/shell/treeMenu.test.ts` ("offers Make
  literate and Ingest into…"), `FolderTreePane.test.tsx` ("ingest from the
  tree").


Verified 2026-10-01 (Markdown document extension): The new-document path is `<stem>.md`. The HTTP integration in
`tests/serve_files.rs::make_literate_creates_an_openable_markdown_document`
verifies adoption, registration, opening, rendering, and unchanged file bytes.
The frontend tree menu and pane suites pass with Markdown document paths.
