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
import { parserBecameReady, structureOf } from "./wysiwyg";
import { hickLangReady } from "./hickLang";
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
  if (!hickLangReady()) return [];
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
      foldingChanged: update => update.transactions.some(tr => tr.effects.some(effect => effect.is(parserBecameReady))),
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
    // A late parse adds fold controls while leaving the visible source open.
    let done = !hickLangReady();
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
    let done = !hickLangReady();
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

</storycode:doc>
