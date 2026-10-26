<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/folding.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-eee9d1963829"></a>
## `apps/web/src/editor/folding.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Claude Code Transcript Imports As A Session](../../../../../guarantees/agent/a-claude-code-transcript-imports-as-a-session/contract.md), [Read the retained branch of an agent conversation](../../../../../guarantees/agent/a-session-is-the-conversation/contract.md), [The Agent pane is a live document](../../../../../guarantees/agent/the-agent-pane-is-a-live-document/contract.md), [An Ingested Scaffold Opens As A Tree](../../../../../guarantees/authoring/an-ingested-scaffold-opens-as-a-tree/contract.md), [Every Hover Hint Is Drawn In The App Theme](../../../../../guarantees/authoring/every-hover-hint-is-drawn-in-the-app-theme/contract.md).

<a id="f-eee9d1963829-FoldRange"></a>
### `FoldRange`

Folding for the Document view, built on CodeMirror's NATIVE fold framework
(@codemirror/language). Native folds are real fold ranges in CM's fold
state, so the height map stays correct — this is deliberately NOT a
hand-rolled hide-decoration scheme (see the layout rules in wysiwyg.ts).
Two families of foldable regions, both computed from the cached hickDoc
structure:
- Markdown headings: the heading line stays visible; everything up to the
next heading of the same or higher level folds away.
- Hick blocks (exec, file, copy, cut, when, container, session turns):
the opening tag line stays visible (with its chip/banner widget above
and, for exec, the cell panel below — widgets sit outside the folded
range), the body and closing tag fold away.
On top of those, one shape rather than one family: an INGESTED scaffold
reads as a tree. `dotnet new webapi` writes a hundred and sixty lines
nobody typed, and today they sit between two paragraphs of prose that
somebody did. Three nested folds — the whole block, a directory of it, one
file — each labelled with what it stands for, and the files start folded
(`ingestedFolds`), so what you see is one line per file: its real
`<hick:file path="…">` tag and how big it is.
Nothing here is a second rendering of those bytes. A fold header is the
document's own text and a fold placeholder is a placeholder, so the caret
still lands in real positions, the reverse edit is untouched, and somebody
who has never heard of this can select the block and delete it. That is
the whole reason this is folding and not a widget: `ingest` won its
argument by making a scaffold ORDINARY BYTES YOU EDIT
(docs/specs/freeform/owning-what-a-scaffolder-wrote.md), and a lens over
them would be the thing that spec refused.

<storycode:copy id="f-eee9d1963829-FoldRange">
// Folding for the Document view, built on CodeMirror's NATIVE fold framework
// (@codemirror/language). Native folds are real fold ranges in CM's fold
// state, so the height map stays correct — this is deliberately NOT a
// hand-rolled hide-decoration scheme (see the layout rules in wysiwyg.ts).
//
// Two families of foldable regions, both computed from the cached hickDoc
// structure:
//  - Markdown headings: the heading line stays visible; everything up to the
//    next heading of the same or higher level folds away.
//  - Hick blocks (exec, file, copy, cut, when, container, session turns):
//    the opening tag line stays visible (with its chip/banner widget above
//    and, for exec, the cell panel below — widgets sit outside the folded
//    range), the body and closing tag fold away.
//
// On top of those, one shape rather than one family: an INGESTED scaffold
// reads as a tree. `dotnet new webapi` writes a hundred and sixty lines
// nobody typed, and today they sit between two paragraphs of prose that
// somebody did. Three nested folds — the whole block, a directory of it, one
// file — each labelled with what it stands for, and the files start folded
// (`ingestedFolds`), so what you see is one line per file: its real
// `<hick:file path="…">` tag and how big it is.
//
// Nothing here is a second rendering of those bytes. A fold header is the
// document's own text and a fold placeholder is a placeholder, so the caret
// still lands in real positions, the reverse edit is untouched, and somebody
// who has never heard of this can select the block and delete it. That is
// the whole reason this is folding and not a widget: `ingest` won its
// argument by making a scaffold ORDINARY BYTES YOU EDIT
// (docs/specs/freeform/owning-what-a-scaffolder-wrote.md), and a lens over
// them would be the thing that spec refused.

import {
  codeFolding,
  foldEffect,
  foldGutter,
  foldKeymap,
  foldService,
} from "@codemirror/language";
import type { EditorState, Extension } from "@codemirror/state";
import { ViewPlugin, keymap, type EditorView } from "@codemirror/view";
import { structureOf } from "./wysiwyg";
import { blocksNamed, type HickBlock, type HickDocStructure } from "./hickDoc";

export interface FoldRange {
  /** Fold start: end of the visible first line (heading / opening tag), or —
   * for a directory of ingested files — the start of the run's first line,
   * since a directory has no line of its own to keep. */
  from: number;
  /** Fold end: end of the section / end of the closing tag. */
  to: number;
  kind: "heading" | "block" | "ingested-dir";
  /** Block tag name, for block folds. */
  name?: string;
  /**
   * What the collapsed placeholder says, for the ingest folds only: "22
   * lines", "8 files", "Controllers/ · 4 files". Everything else keeps the
   * plain ellipsis, because everything else is text this document's author
   * wrote and can already see the shape of.
   */
  label?: string;
}

</storycode:copy>
<a id="f-eee9d1963829-FOLDABLE_BLOCKS"></a>
### `FOLDABLE_BLOCKS`

Hick blocks whose bodies fold. Session turns fold individually; `session`
itself folds as a whole conversation.

<storycode:copy id="f-eee9d1963829-FOLDABLE_BLOCKS">
/** Hick blocks whose bodies fold. Session turns fold individually; `session`
 * itself folds as a whole conversation. */
export const FOLDABLE_BLOCKS = new Set([
  "exec",
  "file",
  // A whole scaffold, collapsed to the line that says where it came from.
  "ingested",
  "copy",
  "cut",
  "when",
  "container",
  "session",
  "user",
  "assistant",
  "observation",
  "tool",
  "tool-result",
  "action",
  "reasoning",
  "context",
]);

</storycode:copy>
<a id="f-eee9d1963829-WORK_BLOCKS"></a>
### `WORK_BLOCKS`

The agent's WORK in a session — what it ran, what it called, what came
back, what it was thinking — as opposed to what was said. These fold
differently from every other block: the closing tag line stays visible
too, so a folded tool call still reads `<hick:tool …>` … `</hick:tool>` —
the whole element is on screen, and a bubble keeps its last line. And they
start folded when a session opens (`sessionWorkFolds`), the way "show
work" hides them in the dock: the answer reads first, the work is a click.

<storycode:copy id="f-eee9d1963829-WORK_BLOCKS">
/**
 * The agent's WORK in a session — what it ran, what it called, what came
 * back, what it was thinking — as opposed to what was said. These fold
 * differently from every other block: the closing tag line stays visible
 * too, so a folded tool call still reads `<hick:tool …>` … `</hick:tool>` —
 * the whole element is on screen, and a bubble keeps its last line. And they
 * start folded when a session opens (`sessionWorkFolds`), the way "show
 * work" hides them in the dock: the answer reads first, the work is a click.
 */
export const WORK_BLOCKS = new Set([
  "tool",
  "tool-result",
  "action",
  "observation",
  "reasoning",
  "context",
]);

</storycode:copy>
<a id="f-eee9d1963829-computeFoldRanges"></a>
### `computeFoldRanges`

All foldable ranges of a document, in order (outermost first at equal
starts). Pure — unit-tested against structures from parseHickDoc.

<storycode:copy id="f-eee9d1963829-computeFoldRanges">
/**
 * All foldable ranges of a document, in order (outermost first at equal
 * starts). Pure — unit-tested against structures from parseHickDoc.
 */
export function computeFoldRanges(
  structure: HickDocStructure,
  text: string,
): FoldRange[] {
  const out: FoldRange[] = [];
  // Which range a block produced, so the ingest pass below can label the
  // ranges this loop already made rather than computing them twice.
  const rangeOf = new Map<HickBlock, FoldRange>();

  for (const b of structure.blocks) {
    if (!FOLDABLE_BLOCKS.has(b.name)) continue;
    // Self-closing / empty blocks have nothing to fold.
    if (b.contentTo <= b.contentFrom) continue;
    const lineEnd = text.indexOf("\n", b.from);
    // Recorded ACP context is often one JSON line. Fold its body natively,
    // keeping both tags on that line instead of replacing it with a card.
    if (lineEnd < 0 || lineEnd >= b.to) {
      if (WORK_BLOCKS.has(b.name)) out.push({ from: b.contentFrom, to: b.contentTo, kind: "block", name: b.name });
      continue;
    }
    let to = Math.min(b.to, text.length);
    // Work keeps its closing tag on screen (see WORK_BLOCKS) — when that tag
    // starts its own line, the fold stops at the end of the line before it.
    if (WORK_BLOCKS.has(b.name) && b.close) {
      const closeLineStart = text.lastIndexOf("\n", b.close.from - 1) + 1;
      if (text.slice(closeLineStart, b.close.from).trim() === "")
        to = closeLineStart - 1;
    }
    if (to > lineEnd) {
      const range: FoldRange = { from: lineEnd, to, kind: "block", name: b.name };
      rangeOf.set(b, range);
      out.push(range);
    }
  }

  for (const ing of blocksNamed(structure, "ingested")) {
    labelIngest(out, rangeOf, ing, filesIngestedBy(structure, ing), text);
  }

  const hs = structure.headings;
  for (let i = 0; i < hs.length; i++) {
    const h = hs[i];
    // Section runs until the next heading of same-or-higher level.
    let end = text.length;
    for (let j = i + 1; j < hs.length; j++) {
      if (hs[j].level <= h.level) {
        end = hs[j].from - 1; // keep the newline that starts the next line
        break;
      }
    }
    // Trim trailing blank lines so the fold ends at real content.
    while (end > h.to && text[end - 1] === "\n") end--;
    if (end > h.to) out.push({ from: h.to, to: end, kind: "heading" });
  }

  out.sort((a, b) => a.from - b.from || b.to - a.to);
  return out;
}

</storycode:copy>
<a id="f-eee9d1963829-filesIngestedBy"></a>
### `filesIngestedBy`

The `hick:file` blocks one `hick:ingested` block owns, in document order.

<storycode:copy id="f-eee9d1963829-filesIngestedBy">
/** The `hick:file` blocks one `hick:ingested` block owns, in document order. */
function filesIngestedBy(
  structure: HickDocStructure,
  ingested: HickBlock,
): HickBlock[] {
  return structure.blocks.filter(
    (b) =>
      b.name === "file" &&
      b.from >= ingested.contentFrom &&
      b.to <= ingested.contentTo,
  );
}

</storycode:copy>
<a id="f-eee9d1963829-contentLines"></a>
### `contentLines`

Lines of actual content in a block, not counting a trailing newline.

<storycode:copy id="f-eee9d1963829-contentLines">
/** Lines of actual content in a block, not counting a trailing newline. */
function contentLines(text: string, b: HickBlock): number {
  const body = text.slice(b.contentFrom, b.contentTo).replace(/\n$/, "");
  return body === "" ? 0 : body.split("\n").length;
}

</storycode:copy>
<a id="f-eee9d1963829-plural"></a>
### `plural`

<storycode:copy id="f-eee9d1963829-plural">
function plural(n: number, one: string): string {
  return `${n} ${one}${n === 1 ? "" : "s"}`;
}

</storycode:copy>
<a id="f-eee9d1963829-dirOf"></a>
### `dirOf`

The directory part of a path, "" for a path with no directory at all.

<storycode:copy id="f-eee9d1963829-dirOf">
/** The directory part of a path, "" for a path with no directory at all. */
function dirOf(path: string): string {
  const cut = path.lastIndexOf("/");
  return cut < 0 ? "" : path.slice(0, cut + 1);
}

</storycode:copy>
<a id="f-eee9d1963829-rootDirOf"></a>
### `rootDirOf`

The directory every file in the block sits under — the scaffold's own root
(`service/` for a `dotnet new -o service`), "" when they share nothing.

<storycode:copy id="f-eee9d1963829-rootDirOf">
/**
 * The directory every file in the block sits under — the scaffold's own root
 * (`service/` for a `dotnet new -o service`), "" when they share nothing.
 */
function rootDirOf(files: HickBlock[]): string {
  const paths = files.map((f) => f.attrs.path ?? "");
  if (paths.length === 0) return "";
  let root = dirOf(paths[0]).split("/");
  for (const p of paths.slice(1)) {
    const segs = dirOf(p).split("/");
    let i = 0;
    while (i < root.length && i < segs.length && root[i] === segs[i]) i++;
    root = root.slice(0, i);
  }
  const joined = root.join("/");
  return joined === "" || joined.endsWith("/") ? joined : joined + "/";
}

</storycode:copy>
<a id="f-eee9d1963829-labelIngest"></a>
### `labelIngest`

Label the block and its files, and add a fold per directory.
A directory is only ever a fold RANGE because the block is path-sorted, so
a directory's files are adjacent: this walks contiguous runs rather than
grouping, and a run that is broken by a subdirectory in between is two
runs, which is the truth about where those lines are.
Three runs are deliberately not folded. One file is not a directory worth a
click — the fold would stand for exactly the thing on the line beneath it;
a run holding every file in the block is the block, which already folds one
line higher; and a run at the scaffold's own ROOT is that same claim under
another name, told twice and in pieces, since the root's files are split
into runs by whichever subdirectories happen to sort between them.

<storycode:copy id="f-eee9d1963829-labelIngest">
/**
 * Label the block and its files, and add a fold per directory.
 *
 * A directory is only ever a fold RANGE because the block is path-sorted, so
 * a directory's files are adjacent: this walks contiguous runs rather than
 * grouping, and a run that is broken by a subdirectory in between is two
 * runs, which is the truth about where those lines are.
 *
 * Three runs are deliberately not folded. One file is not a directory worth a
 * click — the fold would stand for exactly the thing on the line beneath it;
 * a run holding every file in the block is the block, which already folds one
 * line higher; and a run at the scaffold's own ROOT is that same claim under
 * another name, told twice and in pieces, since the root's files are split
 * into runs by whichever subdirectories happen to sort between them.
 */
function labelIngest(
  out: FoldRange[],
  rangeOf: Map<HickBlock, FoldRange>,
  ingested: HickBlock,
  files: HickBlock[],
  text: string,
): void {
  const whole = rangeOf.get(ingested);
  if (whole) whole.label = plural(files.length, "file");
  for (const f of files) {
    const r = rangeOf.get(f);
    if (r) r.label = plural(contentLines(text, f), "line");
  }

  const root = rootDirOf(files);
  for (let i = 0; i < files.length; ) {
    const dir = dirOf(files[i].attrs.path ?? "");
    let j = i;
    while (j + 1 < files.length && dirOf(files[j + 1].attrs.path ?? "") === dir)
      j++;
    const run = files.slice(i, j + 1);
    i = j + 1;
    if (run.length < 2 || run.length === files.length) continue;
    if (dir === "" || dir === root) continue;
    // A directory has no line of its own, so its fold starts where its first
    // file's line starts: the row is the placeholder, and the gutter still
    // numbers it, standing for the run the way any fold header does.
    const from = text.lastIndexOf("\n", run[0].from - 1) + 1;
    const to = Math.min(run[run.length - 1].to, text.length);
    const lines = run.reduce((n, f) => n + contentLines(text, f), 0);
    const leaf = dir.replace(/\/$/, "").split("/").pop() ?? dir;
    if (to > from)
      out.push({
        from,
        to,
        kind: "ingested-dir",
        label: `${leaf}/ · ${plural(run.length, "file")}, ${plural(lines, "line")}`,
      });
  }
}

</storycode:copy>
<a id="f-eee9d1963829-foldRangeForLine"></a>
### `foldRangeForLine`

The fold range starting on the line [lineFrom, lineTo], or null. When
several ranges start on one line (nested blocks opening together), the
outermost wins. Pure — this is the foldService predicate.

<storycode:copy id="f-eee9d1963829-foldRangeForLine">
/**
 * The fold range starting on the line [lineFrom, lineTo], or null. When
 * several ranges start on one line (nested blocks opening together), the
 * outermost wins. Pure — this is the foldService predicate.
 */
export function foldRangeForLine(
  ranges: FoldRange[],
  lineFrom: number,
  lineTo: number,
): { from: number; to: number } | null {
  let best: FoldRange | null = null;
  for (const r of ranges) {
    if (r.from < lineFrom) continue;
    if (r.from > lineTo) break; // ranges are sorted by from
    if (r.to > r.from && (!best || r.to > best.to)) best = r;
  }
  return best ? { from: best.from, to: best.to } : null;
}

</storycode:copy>
<a id="f-eee9d1963829-foldCache"></a>
### `foldCache`

One fold-range computation per document version (same pattern as the
structure cache in wysiwyg.ts).

<storycode:copy id="f-eee9d1963829-foldCache">
// One fold-range computation per document version (same pattern as the
// structure cache in wysiwyg.ts).
const foldCache = new WeakMap<object, FoldRange[]>();

</storycode:copy>
<a id="f-eee9d1963829-foldRangesOf"></a>
### `foldRangesOf`

<storycode:copy id="f-eee9d1963829-foldRangesOf">
export function foldRangesOf(state: EditorState): FoldRange[] {
  const key = state.doc as unknown as object;
  let ranges = foldCache.get(key);
  if (!ranges) {
    ranges = computeFoldRanges(structureOf(state), state.doc.toString());
    foldCache.set(key, ranges);
  }
  return ranges;
}

</storycode:copy>
<a id="f-eee9d1963829-hickoryFolding"></a>
### `hickoryFolding`

The Document-view folding extension set: native fold state + gutter with
hickory-styled chevrons + the standard fold keymap.

<storycode:copy id="f-eee9d1963829-hickoryFolding">
/** The Document-view folding extension set: native fold state + gutter with
 * hickory-styled chevrons + the standard fold keymap. */
export function hickoryFolding(): Extension {
  return [
    codeFolding({
      // What a fold stands for, resolved from the same ranges the fold
      // service offered: a collapsed scaffold that said only "…" would be
      // asking the reader to unfold it to find out whether it was worth it.
      preparePlaceholder: (state, range) =>
        foldRangesOf(state).find(
          (r) => r.from === range.from && r.to === range.to,
        )?.label ?? null,
      placeholderDOM(_view, onclick, prepared) {
        const label = typeof prepared === "string" ? prepared : null;
        const el = document.createElement("span");
        el.className = `cm-hick-fold-placeholder${label ? " cm-hick-fold-counted" : ""}`;
        el.textContent = label ?? "…";
        el.dataset.tip = "Unfold";
        el.setAttribute(
          "aria-label",
          label
            ? `folded: ${label} — click to unfold`
            : "folded content — click to unfold",
        );
        el.onclick = onclick;
        return el;
      },
    }),
    foldService.of((state, lineFrom, lineTo) =>
      foldRangeForLine(foldRangesOf(state), lineFrom, lineTo),
    ),
    foldGutter({
      markerDOM(open) {
        const el = document.createElement("span");
        el.className = `cm-fold-marker${open ? " cm-fold-open" : ""}`;
        el.textContent = "›"; // ›
        el.dataset.tip = open ? "Fold" : "Unfold";
        return el;
      },
    }),
    keymap.of(foldKeymap),
  ];
}

</storycode:copy>
<a id="f-eee9d1963829-workFoldRanges"></a>
### `workFoldRanges`

The work folds a session starts with: every WORK_BLOCKS range. Pure.

<storycode:copy id="f-eee9d1963829-workFoldRanges">
/** The work folds a session starts with: every WORK_BLOCKS range. Pure. */
export function workFoldRanges(
  ranges: FoldRange[],
): { from: number; to: number }[] {
  return ranges
    .filter(
      (r) =>
        r.kind === "block" && r.name !== undefined && WORK_BLOCKS.has(r.name),
    )
    .map((r) => ({ from: r.from, to: r.to }));
}

</storycode:copy>
<a id="f-eee9d1963829-ingestedFileFolds"></a>
### `ingestedFileFolds`

The folds an ingested scaffold opens with: one per ingested FILE. Pure.
Files, not directories and not the whole block. Folding the files is what
turns the wall into a tree — every path stays on screen, which IS the
tree — while folding the directories too would hide those paths behind a
second click, and folding the block would hide the fact that a scaffold is
there at all. Both of those remain a click away for a reader who wants
them; neither is chosen for them.

<storycode:copy id="f-eee9d1963829-ingestedFileFolds">
/**
 * The folds an ingested scaffold opens with: one per ingested FILE. Pure.
 *
 * Files, not directories and not the whole block. Folding the files is what
 * turns the wall into a tree — every path stays on screen, which IS the
 * tree — while folding the directories too would hide those paths behind a
 * second click, and folding the block would hide the fact that a scaffold is
 * there at all. Both of those remain a click away for a reader who wants
 * them; neither is chosen for them.
 */
export function ingestedFileFolds(
  ranges: FoldRange[],
): { from: number; to: number }[] {
  return ranges
    .filter((r) => r.kind === "block" && r.name === "file" && r.label)
    .map((r) => ({ from: r.from, to: r.to }));
}

</storycode:copy>
<a id="f-eee9d1963829-ingestedFolds"></a>
### `ingestedFolds`

Fold an ingested scaffold's files ONCE, when the document first has
content — the same once-only contract `sessionWorkFolds` has, and for the
same reason: unfolding is the reader's choice, and a later edit must not
re-hide what they opened.

<storycode:copy id="f-eee9d1963829-ingestedFolds">
/**
 * Fold an ingested scaffold's files ONCE, when the document first has
 * content — the same once-only contract `sessionWorkFolds` has, and for the
 * same reason: unfolding is the reader's choice, and a later edit must not
 * re-hide what they opened.
 */
export function ingestedFolds(): Extension {
  return ViewPlugin.define((view: EditorView) => {
    let done = false;
    const attempt = (state: EditorState) => {
      if (done || state.doc.length === 0) return;
      done = true;
      const folds = ingestedFileFolds(foldRangesOf(state));
      if (folds.length === 0) return;
      queueMicrotask(() => {
        if (!view.dom.isConnected) return;
        view.dispatch({ effects: folds.map((r) => foldEffect.of(r)) });
      });
    };
    attempt(view.state);
    return {
      update(u) {
        if (u.docChanged) attempt(u.state);
      },
    };
  });
}

</storycode:copy>
<a id="f-eee9d1963829-sessionWorkFolds"></a>
### `sessionWorkFolds`

Fold the agent's work ONCE, when the document first has content. A session
opens with what was said on screen and what was done a click away —
exactly the dock's "show work". Only once: unfolding is the reader's
choice, and a later edit must not re-hide what they opened. Only for a
session: a note with no `hick:session` root has no work to hide.

<storycode:copy id="f-eee9d1963829-sessionWorkFolds">
/**
 * Fold the agent's work ONCE, when the document first has content. A session
 * opens with what was said on screen and what was done a click away —
 * exactly the dock's "show work". Only once: unfolding is the reader's
 * choice, and a later edit must not re-hide what they opened. Only for a
 * session: a note with no `hick:session` root has no work to hide.
 */
export function sessionWorkFolds(): Extension {
  return ViewPlugin.define((view: EditorView) => {
    let done = false;
    const attempt = (state: EditorState) => {
      if (done || state.doc.length === 0) return;
      done = true;
      if (!structureOf(state).blocks.some((b) => b.name === "session")) return;
      const folds = workFoldRanges(foldRangesOf(state));
      if (folds.length === 0) return;
      // Not inside an update: dispatching from one is forbidden, and the
      // room's first sync is an update.
      queueMicrotask(() => {
        if (!view.dom.isConnected) return;
        view.dispatch({ effects: folds.map((r) => foldEffect.of(r)) });
      });
    };
    attempt(view.state);
    return {
      update(u) {
        if (u.docChanged) attempt(u.state);
      },
    };
  });
}
</storycode:copy>

### Assemble `apps/web/src/editor/folding.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/folding.ts"><storycode:paste select="#f-eee9d1963829-FoldRange"/><storycode:paste select="#f-eee9d1963829-FOLDABLE_BLOCKS"/><storycode:paste select="#f-eee9d1963829-WORK_BLOCKS"/><storycode:paste select="#f-eee9d1963829-computeFoldRanges"/><storycode:paste select="#f-eee9d1963829-filesIngestedBy"/><storycode:paste select="#f-eee9d1963829-contentLines"/><storycode:paste select="#f-eee9d1963829-plural"/><storycode:paste select="#f-eee9d1963829-dirOf"/><storycode:paste select="#f-eee9d1963829-rootDirOf"/><storycode:paste select="#f-eee9d1963829-labelIngest"/><storycode:paste select="#f-eee9d1963829-foldRangeForLine"/><storycode:paste select="#f-eee9d1963829-foldCache"/><storycode:paste select="#f-eee9d1963829-foldRangesOf"/><storycode:paste select="#f-eee9d1963829-hickoryFolding"/><storycode:paste select="#f-eee9d1963829-workFoldRanges"/><storycode:paste select="#f-eee9d1963829-ingestedFileFolds"/><storycode:paste select="#f-eee9d1963829-ingestedFolds"/><storycode:paste select="#f-eee9d1963829-sessionWorkFolds"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-92a9cd526370"></a>
## `apps/web/src/editor/folding.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Read the retained branch of an agent conversation](../../../../../guarantees/agent/a-session-is-the-conversation/contract.md), [The Agent pane is a live document](../../../../../guarantees/agent/the-agent-pane-is-a-live-document/contract.md), [An Ingested Scaffold Opens As A Tree](../../../../../guarantees/authoring/an-ingested-scaffold-opens-as-a-tree/contract.md).

<a id="f-92a9cd526370-rangesOf"></a>
### `rangesOf`

<storycode:copy id="f-92a9cd526370-rangesOf">
import { describe, expect, it } from "vitest";
import { parseHickDoc } from "./hickDoc";
import {
  computeFoldRanges,
  foldRangeForLine,
  FOLDABLE_BLOCKS,
  ingestedFileFolds,
  workFoldRanges,
} from "./folding";
import { WEAVE_SOURCE, CLI_SOURCE } from "../mock/mockData";

function rangesOf(text: string) {
  return computeFoldRanges(parseHickDoc(text), text);
}

</storycode:copy>
<a id="f-92a9cd526370-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-92a9cd526370-expression_statement">
describe("computeFoldRanges — hick blocks", () => {
  it("folds an exec block body, keeping the opening tag line visible", () => {
    const text = 'intro\n<hick:exec container="shell">\nls -la\n</hick:exec>\ntail\n';
    const [r] = rangesOf(text).filter((r) => r.kind === "block");
    expect(r.name).toBe("exec");
    // Fold starts at the end of the opening-tag line…
    expect(r.from).toBe(text.indexOf("\n", text.indexOf("<hick:exec")));
    // …and swallows the body plus the closing tag.
    expect(r.to).toBe(text.indexOf("</hick:exec>") + "</hick:exec>".length);
  });

  it("folds copy/cut/file/when/container/session-turn blocks", () => {
    const text = [
      '<hick:copy id="a">',
      "x = 1",
      "</hick:copy>",
      '<hick:cut id="b">',
      "y = 2",
      "</hick:cut>",
      '<hick:file path="f.py" language="python">',
      "z = 3",
      "</hick:file>",
      '<hick:when test="ci">',
      "gated",
      "</hick:when>",
      '<hick:container name="env">',
      '<hick:allow network="github.com:443" />',
      "</hick:container>",
      "<hick:user>",
      "hello",
      "</hick:user>",
    ].join("\n");
    const blocks = rangesOf(text).filter((r) => r.kind === "block");
    const names = blocks.map((r) => r.name).sort();
    expect(names).toEqual(["container", "copy", "cut", "file", "user", "when"]);
    for (const r of blocks) expect(r.to).toBeGreaterThan(r.from);
  });

  it("folds the agent's work but keeps its closing tag line visible", () => {
    // docs/guarantees/agent/a-session-is-the-conversation.md — a folded tool
    // call still reads `<hick:tool …>` … `</hick:tool>`.
    const text = [
      "<hick:session>",
      "<hick:assistant>",
      "Looking.",
      '<hick:tool name="read_doc">',
      "<hick:input>a.md</hick:input>",
      "</hick:tool>",
      "</hick:assistant>",
      '<hick:observation source="action-0" exit="0">',
      "3",
      "</hick:observation>",
      "</hick:session>",
    ].join("\n");
    const all = rangesOf(text);
    const tool = all.find((r) => r.name === "tool")!;
    expect(tool.from).toBe(text.indexOf("\n", text.indexOf("<hick:tool")));
    expect(tool.to).toBe(text.indexOf("\n</hick:tool>"));
    const obs = all.find((r) => r.name === "observation")!;
    expect(obs.to).toBe(text.indexOf("\n</hick:observation>"));
    // Assistant turns still fold through their closing tag: they are said,
    // not done, and a reader folding one wants it gone.
    const assistant = all.find((r) => r.name === "assistant")!;
    expect(assistant.to).toBe(text.indexOf("</hick:assistant>") + "</hick:assistant>".length);
    // The initial folds are exactly the work.
    expect(workFoldRanges(all).map((r) => r.from)).toEqual([tool.from, obs.from]);
  });

  it("skips self-closing and single-line blocks", () => {
    const text =
      '<hick:container name="shell" image="debian:12" />\n' +
      "<hick:exec>echo hi</hick:exec>\n";
    expect(rangesOf(text).filter((r) => r.kind === "block")).toEqual([]);
  });

  // Guarantee: docs/guarantees/agent/the-agent-pane-is-a-live-document.md
  it("folds a single-line ACP context body while keeping both tags", () => {
    const text = '<hick:context kind="editor-buffers">{"context":"snapshot"}</hick:context>\n';
    const [range] = workFoldRanges(rangesOf(text));
    expect(text.slice(range.from, range.to)).toBe('{"context":"snapshot"}');
    expect(text.slice(0, range.from)).toBe('<hick:context kind="editor-buffers">');
    expect(text.slice(range.to)).toBe('</hick:context>\n');
  });

  it("folds an unclosed block to the end of the document", () => {
    const text = '<hick:exec container="shell">\nnever closed\n';
    const [r] = rangesOf(text).filter((r) => r.kind === "block");
    expect(r.to).toBe(text.length);
  });

  it("covers every foldable multi-line block of the weave demo (d3)", () => {
    const blocks = rangesOf(WEAVE_SOURCE).filter((r) => r.kind === "block");
    // Two copy slots, one file, one exec.
    expect(blocks.map((r) => r.name).sort()).toEqual(["copy", "copy", "exec", "file"]);
    for (const r of blocks) expect(FOLDABLE_BLOCKS.has(r.name!)).toBe(true);
  });
});

</storycode:copy>
<a id="f-92a9cd526370-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-92a9cd526370-expression_statement-2">
describe("computeFoldRanges — markdown headings", () => {
  it("folds a section up to the next same-or-higher heading", () => {
    const text = "# A\naaa\n## B\nbbb\n# C\nccc";
    const rs = rangesOf(text).filter((r) => r.kind === "heading");
    expect(rs).toHaveLength(3);
    const a = rs.find((r) => r.from === text.indexOf("\naaa"))!;
    // A folds aaa + the B subsection, stopping before "# C".
    expect(text.slice(a.from, a.to)).toBe("\naaa\n## B\nbbb");
    const b = rs.find((r) => r.from === text.indexOf("\nbbb"))!;
    expect(text.slice(b.from, b.to)).toBe("\nbbb");
    const c = rs.find((r) => text.slice(r.from, r.to) === "\nccc");
    expect(c).toBeTruthy();
  });

  it("trims trailing blank lines and skips empty sections", () => {
    const text = "# A\nbody\n\n\n# B\n# C\ntail\n";
    const rs = rangesOf(text).filter((r) => r.kind === "heading");
    // A ends after "body" (blank lines trimmed); B has no content → no fold.
    const a = rs[0];
    expect(text.slice(a.from, a.to)).toBe("\nbody");
    expect(rs.some((r) => r.from === "# A\nbody\n\n\n# B".length)).toBe(false);
  });

  it("headings inside verbatim block bodies are not headings (CLI doc)", () => {
    const rs = rangesOf(CLI_SOURCE);
    // Exactly one heading in the quickstart doc.
    expect(rs.filter((r) => r.kind === "heading")).toHaveLength(1);
  });
});

</storycode:copy>
<a id="f-92a9cd526370-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-92a9cd526370-expression_statement-3">
describe("foldRangeForLine (the foldService predicate)", () => {
  const text = "# A\naaa\n<hick:exec>\nls\n</hick:exec>\n";
  const ranges = rangesOf(text);

  it("returns the range starting on the queried line", () => {
    const headLine: [number, number] = [0, 3]; // "# A"
    const r = foldRangeForLine(ranges, ...headLine);
    expect(r).toBeTruthy();
    expect(r!.from).toBe(3);
  });

  it("returns null for lines that start nothing", () => {
    const aaaFrom = text.indexOf("aaa");
    expect(foldRangeForLine(ranges, aaaFrom, aaaFrom + 3)).toBeNull();
  });

  it("prefers the outermost range when several start on one line", () => {
    const rs = [
      { from: 10, to: 20, kind: "block" as const },
      { from: 10, to: 40, kind: "block" as const },
    ];
    expect(foldRangeForLine(rs, 5, 10)!.to).toBe(40);
  });
});

</storycode:copy>
<a id="f-92a9cd526370-expression_statement-4"></a>
### `expression_statement`

Protects docs/guarantees/authoring/an-ingested-scaffold-opens-as-a-tree.md

<storycode:copy id="f-92a9cd526370-expression_statement-4">
// Protects docs/guarantees/authoring/an-ingested-scaffold-opens-as-a-tree.md
describe("an ingested scaffold folds as a tree", () => {
  // A dotnet-shaped ingest: path-sorted, one directory holding two files and
  // one holding a single file, plus files at the block's own root.
  const INGEST = [
    '<hick:exec container="sdk" mount="project:out">',
    '<hick:copy id="scaffold">dotnet new webapi -o out</hick:copy>',
    '<hick:ingested from="#scaffold" sha256="9f2c" at="2026-09-01" files="5" skipped="6">',
    '<hick:file path="service/Controllers/HomeController.cs">using Microsoft.AspNetCore.Mvc;',
    "",
    "public class HomeController : ControllerBase { }",
    "</hick:file>",
    '<hick:file path="service/Controllers/WeatherController.cs">using Microsoft.AspNetCore.Mvc;',
    "",
    "public class WeatherController : ControllerBase { }",
    "</hick:file>",
    '<hick:file path="service/Program.cs">var builder = WebApplication.CreateBuilder(args);',
    "builder.Build().Run();",
    "</hick:file>",
    '<hick:file path="service/Properties/launchSettings.json">{',
    '  "profiles": {}',
    "}",
    "</hick:file>",
    '<hick:file path="service/appsettings.json">{',
    '  "AllowedHosts": "*"',
    "}",
    "</hick:file>",
    "</hick:ingested>",
    "</hick:exec>",
    "",
  ].join("\n");

  const ingestRanges = () => rangesOf(INGEST);
  const fileFolds = () =>
    ingestRanges().filter((r) => r.kind === "block" && r.name === "file");

  it("labels each ingested file with how many lines it holds", () => {
    // Content starts on the tag's own line, so the label counts the file, not
    // the rows the fold hides.
    expect(fileFolds().map((r) => r.label)).toEqual([
      "3 lines",
      "3 lines",
      "2 lines",
      "3 lines",
      "3 lines",
    ]);
  });

  it("labels the block itself with the number of files it carries", () => {
    const whole = ingestRanges().find(
      (r) => r.kind === "block" && r.name === "ingested",
    );
    // Counted from the file blocks actually present, not from `files=`: the
    // attribute records what the run wrote, and the document can be edited.
    expect(whole?.label).toBe("5 files");
  });

  it("folds a directory of two or more files, naming it", () => {
    const dirs = ingestRanges().filter((r) => r.kind === "ingested-dir");
    expect(dirs.map((r) => r.label)).toEqual(["Controllers/ · 2 files, 6 lines"]);
    // It starts where the first file's LINE starts — a directory has no line
    // of its own to keep visible — and ends with the last file's closing tag.
    const first = INGEST.indexOf('<hick:file path="service/Controllers/Home');
    expect(dirs[0].from).toBe(first);
    expect(dirs[0].to).toBe(
      INGEST.indexOf("</hick:file>", INGEST.indexOf("WeatherController : ")) +
        "</hick:file>".length,
    );
  });

  it("does not fold a run at the scaffold's own root", () => {
    // Two files sit directly in service/, split by Properties/ between them —
    // and even adjacent they would not fold: a root run is the block's claim
    // told again, in pieces.
    const adjacent = [
      '<hick:ingested from="#s" sha256="a" at="2026-09-01" files="3" skipped="0">',
      '<hick:file path="service/a.txt">a',
      "</hick:file>",
      '<hick:file path="service/b.txt">b',
      "</hick:file>",
      '<hick:file path="service/deep/c.txt">c',
      "</hick:file>",
      "</hick:ingested>",
      "",
    ].join("\n");
    expect(rangesOf(adjacent).some((r) => r.kind === "ingested-dir")).toBe(false);
  });

  it("does not fold a directory holding one file, nor one holding them all", () => {
    const dirs = ingestRanges().filter((r) => r.kind === "ingested-dir");
    // Properties/ has a single file: the fold would stand for the line under it.
    expect(dirs.some((r) => r.label?.startsWith("Properties/"))).toBe(false);
    // And a run covering every file is the block, which folds one line higher.
    const all = [
      '<hick:ingested from="#s" sha256="a" at="2026-09-01" files="2" skipped="0">',
      '<hick:file path="app/a.txt">a',
      "</hick:file>",
      '<hick:file path="app/b.txt">b',
      "</hick:file>",
      "</hick:ingested>",
      "",
    ].join("\n");
    expect(rangesOf(all).some((r) => r.kind === "ingested-dir")).toBe(false);
  });

  it("leaves a hand-written hick:file alone — no label, no fold on open", () => {
    const own = [
      '<hick:file path="notes.txt">mine',
      "still mine",
      "</hick:file>",
      "",
    ].join("\n");
    const [r] = rangesOf(own).filter((x) => x.name === "file");
    expect(r.label).toBeUndefined();
    expect(ingestedFileFolds(rangesOf(own))).toEqual([]);
  });

  it("opens with every ingested file folded, and nothing else", () => {
    const folds = ingestedFileFolds(ingestRanges());
    expect(folds).toHaveLength(5);
    // Each fold starts at the end of its own tag line, so the path stays on
    // screen — that visible line per file IS the tree.
    for (const f of folds) expect(INGEST[f.from]).toBe("\n");
    expect(folds.map((f) => f.to)).toEqual(
      fileFolds().map((r) => r.to),
    );
  });

  it("hands the directory fold to a click on the run's first line", () => {
    const dir = ingestRanges().find((r) => r.kind === "ingested-dir")!;
    const lineEnd = INGEST.indexOf("\n", dir.from);
    // Two ranges start on that line — the directory and the first file's own.
    // The outermost wins, which is the rule every nested block already uses.
    expect(foldRangeForLine(ingestRanges(), dir.from, lineEnd)).toEqual({
      from: dir.from,
      to: dir.to,
    });
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/folding.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/folding.test.ts"><storycode:paste select="#f-92a9cd526370-rangesOf"/><storycode:paste select="#f-92a9cd526370-expression_statement"/><storycode:paste select="#f-92a9cd526370-expression_statement-2"/><storycode:paste select="#f-92a9cd526370-expression_statement-3"/><storycode:paste select="#f-92a9cd526370-expression_statement-4"/></storycode:file>


</storycode:doc>
