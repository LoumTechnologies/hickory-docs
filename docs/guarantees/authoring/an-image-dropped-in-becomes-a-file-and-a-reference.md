# An Image Dropped In Becomes A File And A Reference

Given an image dragged into an open document, or pasted from the clipboard,
when it lands then the bytes are written into an `assets/` directory beside
that document and the buffer gains an ordinary markdown reference to it —
`![a chart](assets/a-chart.png)` — at the point of the drop. The picture is
drawn under the line that references it. Nothing is embedded in the document,
nothing is uploaded anywhere, and the woven markdown needs no special support
for either, because there is nothing special in the document to support.

## Why a file and not a blob

A note that carries its picture as a base64 blob is a note nobody can read in
a diff, nothing else can open, and git cannot store twice without storing it
twice. A file beside the note is a file: `git add` picks up both, another
editor shows the markdown, a static-site generator resolves the path, and the
image can be replaced without touching the note.

## The rules

1. **Beside the document, not at the root.** `notes/weekly/mon.hick` puts its
   images in `notes/weekly/assets/`, and the markdown destination is
   `assets/…` — relative to the note, so the folder can be moved.
2. **Images only, refused by name.** The types a woven `.md` can actually
   display in a browser, GitHub, or a preview pane. A `.pdf` is refused with
   the accepted list in the message rather than written and then silently not
   rendered. `PUT /api/file` remains the only way to overwrite existing text.
3. **The same bytes twice is one file.** A name already taken by identical
   content is reused; a name taken by different content gets `-2`.
4. **A name is made safe before it is used.** The extension decides whether
   the image renders, so it is kept and lowercased; everything else becomes
   `-`, is trimmed, and is capped. A clipboard image has no name and gets one.
5. **The position is held in a StateField, not a variable.** Writing the file
   is a round trip and the buffer is a CRDT — a collaborator, the up-loop, or
   the agent may edit during it. The pending position is mapped through every
   change so the picture lands where the pointer was.
6. **A failure says so.** A write that could not happen puts the server's own
   message above the ruler rather than leaving a note that references a file
   that was never created.
7. **The picture is drawn UNDER the line, never instead of it.** The markdown
   stays visible and editable, which is the rule the rest of this editor's
   markdown display follows — there is no state where the text is hidden
   behind a widget you have to work out how to get back out of.
8. **`GET /api/asset` serves images and nothing else**, refuses a path that
   names its way out of the folder (including through a symlink), and is the
   route the editor's `<img>` reads — so what the editor shows is what the
   markdown says, rather than approximately that.

## Boundary

25 MB is the ceiling. Past it, the thing being dropped is a file that belongs
beside the repository rather than inside it, and the refusal says so.

Nothing rewrites an image destination at weave time: `assets/c.png` is already
correct in both the source and the weave, because both sit in the same
directory. Only `.hick` destinations are rewritten
(`a-link-between-documents-lands-in-the-weave.md`).

Dropping an image into a fenced block or a generated file's body still writes
the file and still inserts markdown — the drop position is where the pointer
is, and this does not second-guess it. Only the URL-over-selection paste
consults the prose ranges, because that one changes text you already had.

---

Last LLM verification:
- Date: 2026-08-21
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hickory-cli/src/serve/asset.rs` — `post_asset`, `decode`
  (a `data:` prefix tolerated, empty and oversize refused), `clean_name`
  (rules 2 and 4), `asset_dir` and `relative_to_doc` (rule 1), `unique_name`
  (rule 3), `get_asset` / `mime_of` / `safe_existing` (rule 8). Routed at
  `/api/asset` in `serve/mod.rs`.
  `apps/web/src/editor/mdPaste.ts` — `mdPaste`, `imageFilesOf` (files and
  clipboard items, because a screenshot is not a file entry everywhere),
  `pendingField` and `pendingPos` (rule 5), `base64Of`, `pastedImageName`,
  `altFor`, and the `dragover` handler without which the browser navigates the
  window to the dropped image.
  `apps/web/src/editor/mdLinks.ts` — `ImageWidget` and `assetUrl` (rule 7), and
  its `eq` so the picture does not reload on every keystroke.
  `apps/web/src/editor/DocumentEditor.tsx` — both extensions wired, `pathRef`
  (an untitled buffer gains a path when it is first saved), and the error
  notice for rule 6.
- Test coverage: `crates/hickory-cli/src/serve/asset.rs::tests` (8) — the
  `data:` prefix, empty and undecodable refusals, name cleaning, the non-image
  refusal naming both the type and the accepted list, the directory rule, the
  relative destination, identical-bytes de-duplication and the `-2` fallback,
  and the read route's image-only and outside-the-folder refusals.
  `apps/web/src/editor/mdPaste.test.ts` (10) — an image paste writing and
  referencing, the pending position surviving an insertion ahead of it
  (rule 5, driven by holding the upload open and dispatching a change), a
  failed write reporting and inserting nothing, and what counts as an image on
  a clipboard.
  `apps/web/src/editor/mdLinks.test.ts` — the picture drawn under its line,
  through `assetUrl`, with the markdown still in the buffer.
- Caveat requiring review: the DROP path (as opposed to paste) is wired and
  reviewed but not driven by a test — jsdom's `DataTransfer` and
  `posAtCoords` make a faithful drag hard to stage, and the two paths share
  `insertImages` below the event. The 25 MB ceiling is asserted in code and
  not exercised. Nothing yet cleans up an asset whose reference is deleted;
  that is a deliberate omission, because a file in the user's folder is the
  user's.
