# An Ingested Scaffold Opens As A Tree

Given a document holding a `<hick:ingested>` block — the forty files
`dotnet new webapi` wrote, sitting between two paragraphs somebody typed —
when it opens in the Document view, then **every ingested file starts
folded**, so what is on screen is one line per file: that file's own
`<hick:file path="…">` tag, the first line of its content, and a placeholder
saying how many lines it holds. The block itself folds to `8 files`, a
directory of two or more files folds to `Controllers/ · 4 files, 96 lines`,
and none of it is a second rendering: the fold headers are the document's own
bytes and the document is unchanged.

Measured on a `dotnet new webapi --use-controllers` ingest of eight files:
**89 document lines, 20 screen rows.**

## Why

`ingest` won its argument by making a scaffold **ordinary `hick:file` bytes
you edit** (`docs/specs/freeform/owning-what-a-scaffolder-wrote.md`) — no
anchor grammar, no patch file, and your four lines are ordinary edits. That
decision is what made the block unreadable: a hundred and sixty lines of
somebody else's `Program.cs` between two paragraphs of yours, correct and
unskimmable.

The fix must not undo the decision that caused it. A tree *widget* — a
control replacing the block, which is the obvious answer — is a lens over
bytes the whole design says are not a lens: it would need its own edit path
back, its own meaning for a caret, and its own behaviour when a re-ingest
three-way merges underneath it. Folding needs none of those, because a fold
hides text without replacing it.

## Three properties hold it up

1. **A fold header is real text, and a fold is not an edit.** The visible
   line per file is that file's actual opening tag; the caret lands in real
   positions; unfolding shows bytes, not a rendering of them. Every fold is
   CodeMirror's native fold state, so the height map stays correct — the same
   rule the rest of `folding.ts` already follows.
2. **The gutter still counts.** A fold header is *a rendered block standing
   for a contiguous run of lines*, which is exactly what
   `docs/guarantees/authoring/the-gutters-never-skip-a-number.md` permits: the
   header carries its own line's number and numbering resumes after the run.
   A directory fold has no line of its own, so it starts at the start of its
   first file's line and that row is the placeholder.
3. **Only what a run wrote.** A `hick:file` the document's author wrote gets
   no label and does not start folded. The signal is containment in a
   `hick:ingested` block, not the tag name — the same distinction
   `.cm-prov-ingested` and the cell bar's ingest chip already draw, and the
   counted placeholder wears their hue.

## Boundary

**Directories are foldable, not folded.** Files start folded because that
turns the wall into a tree while keeping every path on screen — the paths
*are* the tree. Collapsing directories too would hide those paths behind a
second click, and collapsing the block would hide that a scaffold is there at
all; both remain one click away and neither is chosen for the reader.

**Three runs never get a directory fold**: one holding a single file (the fold
would stand for the line beneath it), one holding every file in the block
(that is the block, which folds a line higher), and one at the scaffold's own
root (the same claim again, and split into pieces by whichever subdirectories
sort between them).

A directory is a fold *range* only because an ingest writes its files
path-sorted, so a directory's files are adjacent. If a re-ingest ever emitted
them out of order, directory folds would quietly stop appearing — the file
folds and the labels would not. Nothing enforces the ordering today.

On the first line of a directory run two ranges start, and the outermost wins
(the rule every nested block already uses), so that first file cannot be
folded alone from the gutter while its directory is unfolded. The fold keymap
still reaches it.

Folding is per-view and per-session: it is not persisted, and it is applied
**once**, when the document first has content — a later edit must never
re-hide what a reader opened. That is the same contract `sessionWorkFolds`
has, for the same reason.

---

Last LLM verification:
- Date: 2026-09-02
- Reviewer: Claude Opus 5
- Result: verified by unit tests and by mounting the real editor. A
  `dotnet new webapi --use-controllers` ingest of eight files renders 89
  document lines as 20 screen rows, each file line reading
  `<hick:file path="service/Program.cs">var builder = …` plus its path chip,
  language chip and an `11 lines` placeholder.
- Evidence: `apps/web/src/editor/folding.ts` — `FOLDABLE_BLOCKS` (gained
  `ingested`), `labelIngest`, `filesIngestedBy`, `rootDirOf`,
  `ingestedFileFolds`, `ingestedFolds`, and the `preparePlaceholder` /
  `placeholderDOM` pair in `hickoryFolding`;
  `apps/web/src/editor/DocumentEditor.tsx` (extension wiring);
  `apps/web/src/styles.css` (`.cm-hick-fold-counted`).
- Test coverage: `apps/web/src/editor/folding.test.ts` — labels, the three
  refused directory runs, hand-written files left alone, and the fold the
  gutter hands a click on a run's first line;
  `apps/web/src/editor/DocumentEditor.test.tsx` — the mounted editor opens
  folded with counted placeholders, every path still on screen, no bodies,
  and `state.doc` byte-identical to the source.
