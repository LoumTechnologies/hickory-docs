<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Lineage Browser Shows One Column Per Stage

For: Authors editing executable documents and the files those documents produce.

Given a project of documents connected into a pipeline, when the lineage
browser opens it, then there is one column per **stage** — a document plus the
files it generates — named for the stage, with one of the stage's files opened
in place inside the stage's own file tree; and every link drawn between blocks
is one the weaver computed, never one the view invented.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-lineage-browser-shows-one-column-per-stage.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-67fd95b7486a"></a>
## `apps/web/src/lineage/build.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Lineage Browser Shows One Column Per Stage](contract.md).

<a id="f-67fd95b7486a-SourceDocument"></a>
### `SourceDocument`

Build a lineage model from documents and their woven outputs.
The links here are COMPUTED, never guessed: each one comes from a
provenance entry the weaver produced, so every line the browser draws
corresponds to bytes hick actually carried from one place to another. When
the language gains a way to declare an influence that no weave can compute
— a decision behind a requirement — those arrive as `asserted` links
through `extraLinks`, and they are drawn differently on purpose.

<storycode:copy id="f-67fd95b7486a-SourceDocument">
// Build a lineage model from documents and their woven outputs.
//
// The links here are COMPUTED, never guessed: each one comes from a
// provenance entry the weaver produced, so every line the browser draws
// corresponds to bytes hick actually carried from one place to another. When
// the language gains a way to declare an influence that no weave can compute
// — a decision behind a requirement — those arrive as `asserted` links
// through `extraLinks`, and they are drawn differently on purpose.

import { parseStructure } from "../editor/hickDoc";
import { byteToChar } from "../lib/offsets";
import type { OutputFile, Provenance } from "../api/types";
import type { FileModel, LineageModel, LineageNode, Link, Stage } from "./model";

export interface SourceDocument {
  /** Repo-relative path, which is also how provenance names it. */
  path: string;
  id?: string;
  source: string;
  /** Everything this document weaves. */
  outputs: OutputFile[];
}

</storycode:copy>
<a id="f-67fd95b7486a-lineLookup"></a>
### `lineLookup`

Scan a source once; repeated block offsets must not rescan its prefix.

<storycode:copy id="f-67fd95b7486a-lineLookup">
/** Scan a source once; repeated block offsets must not rescan its prefix. */
function lineLookup(text: string): (offset: number) => number {
  const starts = [0];
  for (let index = 0; index < text.length; index++) if (text[index] === "\n") starts.push(index + 1);
  return offset => {
    let lo = 0, hi = starts.length;
    while (lo < hi) { const mid = (lo + hi) >>> 1; if (starts[mid] <= offset) lo = mid + 1; else hi = mid; }
    return Math.max(0, lo - 1);
  };
}
</storycode:copy>
<a id="f-67fd95b7486a-lineOf"></a>
### `lineOf`

<storycode:copy id="f-67fd95b7486a-lineOf">
function lineOf(text: string, offset: number): number { return lineLookup(text)(offset); }

</storycode:copy>
<a id="f-67fd95b7486a-LINEAGE_MODEL_TEXT_LIMIT"></a>
### `LINEAGE_MODEL_TEXT_LIMIT`

<storycode:copy id="f-67fd95b7486a-LINEAGE_MODEL_TEXT_LIMIT">
export const LINEAGE_MODEL_TEXT_LIMIT = 1_000_000;
</storycode:copy>
<a id="f-67fd95b7486a-LINEAGE_PREVIEW_CHARACTERS"></a>
### `LINEAGE_PREVIEW_CHARACTERS`

<storycode:copy id="f-67fd95b7486a-LINEAGE_PREVIEW_CHARACTERS">
export const LINEAGE_PREVIEW_CHARACTERS = 16_384;
</storycode:copy>
<a id="f-67fd95b7486a-fileModel"></a>
### `fileModel`

<storycode:copy id="f-67fd95b7486a-fileModel">
function fileModel(path: string, text: string, kind: FileModel["kind"], documentId?: string): FileModel {
  if (text.length > LINEAGE_MODEL_TEXT_LIMIT) {
    const shown = text.slice(0, LINEAGE_PREVIEW_CHARACTERS);
    return { path, kind, documentId, lines: shown.split("\n"), preview: { totalCharacters: text.length, shownCharacters: shown.length } };
  }
  return { path, kind, documentId, lines: text.replace(/\n$/, "").split("\n") };
}

</storycode:copy>
<a id="f-67fd95b7486a-stageName"></a>
### `stageName`

<storycode:copy id="f-67fd95b7486a-stageName">
function stageName(docPath: string): string {
  return docPath.split("/").pop()!.replace(/\.md$/, "").replace(/^\d{4}-\d{2}-\d{2}-/, "");
}

</storycode:copy>
<a id="f-67fd95b7486a-documentNodes"></a>
### `documentNodes`

Nodes for a document: the blocks a link can point at.
Fragments and file blocks are what pastes connect, and cells are what a
reader clicks to ask "what does this run?", so all three are selectable.
The tightest block covering a line wins at selection time (`nodeAt`), which
is what lets a paste inside a file block be its own target.

<storycode:copy id="f-67fd95b7486a-documentNodes">
/**
 * Nodes for a document: the blocks a link can point at.
 *
 * Fragments and file blocks are what pastes connect, and cells are what a
 * reader clicks to ask "what does this run?", so all three are selectable.
 * The tightest block covering a line wins at selection time (`nodeAt`), which
 * is what lets a paste inside a file block be its own target.
 */
function documentNodes(path: string, source: string): LineageNode[] {
  const structure = parseStructure(source);
  const out: LineageNode[] = [];
  const lineAt = lineLookup(source);
  for (const block of structure.blocks) {
    const id = block.attrs.id
      ? `${path}#${block.attrs.id}`
      : block.attrs.path
        ? `${path}#${block.attrs.path}`
        : `${path}@${block.from}`;
    out.push({
      id,
      file: path,
      startLine: lineAt(block.from),
      endLine: lineAt(Math.max(block.from, block.to - 1)),
      label: block.attrs.id ?? block.attrs.path ?? block.name,
      kind: block.name,
    });
  }
  return out;
}

</storycode:copy>
<a id="f-67fd95b7486a-nodeForSpan"></a>
### `nodeForSpan`

The node in `doc` whose lines contain a source span, tightest first.

<storycode:copy id="f-67fd95b7486a-nodeForSpan">
/** The node in `doc` whose lines contain a source span, tightest first. */
function nodeForSpan(nodes: LineageNode[], docPath: string, line: number): LineageNode | undefined {
  let best: LineageNode | undefined;
  for (const node of nodes) {
    if (node.file !== docPath || line < node.startLine || line > node.endLine) continue;
    if (!best || node.endLine - node.startLine < best.endLine - best.startLine) best = node;
  }
  return best;
}

</storycode:copy>
<a id="f-67fd95b7486a-outputRuns"></a>
### `outputRuns`

Group an output's provenance into runs that share an origin.
Provenance is per byte range, and a fragment usually arrives as several
consecutive entries. Drawing one link per entry would put a dozen threads
where the reader sees one paste, so consecutive entries with the same
origin span become one node.

<storycode:copy id="f-67fd95b7486a-outputRuns">
/**
 * Group an output's provenance into runs that share an origin.
 *
 * Provenance is per byte range, and a fragment usually arrives as several
 * consecutive entries. Drawing one link per entry would put a dozen threads
 * where the reader sees one paste, so consecutive entries with the same
 * origin span become one node.
 */
function outputRuns(file: OutputFile): { start: number; end: number; origin: Provenance["origin"] }[] {
  const runs: { start: number; end: number; origin: Provenance["origin"] }[] = [];
  for (const entry of file.provenance) {
    const last = runs[runs.length - 1];
    const same =
      last &&
      last.origin.kind === entry.origin.kind &&
      last.origin.kind !== "synthetic" &&
      entry.origin.kind !== "synthetic" &&
      last.origin.doc_path === entry.origin.doc_path &&
      last.origin.span[0] === entry.origin.span[0];
    if (same) last.end = entry.end;
    else runs.push({ start: entry.start, end: entry.end, origin: entry.origin });
  }
  return runs;
}

</storycode:copy>
<a id="f-67fd95b7486a-buildModel"></a>
### `buildModel`

<storycode:copy id="f-67fd95b7486a-buildModel">
export function buildModel(docs: SourceDocument[], extraLinks: Link[] = []): LineageModel {
  const files = new Map<string, FileModel>();
  const nodes = new Map<string, LineageNode>();
  const links: Link[] = [];
  const stages: Stage[] = [];
  const nodesByDoc = new Map<string, LineageNode[]>();

  for (const doc of docs) {
    files.set(doc.path, fileModel(doc.path, doc.source, "document", doc.id));
    // Keep the stage and an explicit bounded preview, but never parse or
    // fabricate block anchors for source omitted from this graph capture.
    const docNodes = files.get(doc.path)!.preview ? [] : documentNodes(doc.path, doc.source);
    nodesByDoc.set(doc.path, docNodes);
    for (const node of docNodes) nodes.set(node.id, node);
    stages.push({
      name: stageName(doc.path),
      doc: doc.path,
      files: [doc.path, ...doc.outputs.map((o) => o.path)],
    });
  }

  for (const doc of docs) {
    for (const output of doc.outputs) {
      files.set(output.path, fileModel(output.path, output.content, "generated", doc.id));
      if (files.get(output.path)!.preview) continue;
      for (const run of outputRuns(output)) {
        if (run.origin.kind === "synthetic") continue;
        const originDoc = run.origin.doc_path;
        const originSource = docs.find((d) => d.path === originDoc)?.source;
        if (originSource === undefined || files.get(originDoc)?.preview) continue;

        // Provenance offsets are UTF-8 bytes on the wire; lines are characters.
        const originLine = lineOf(originSource, byteToChar(originSource, run.origin.span[0]));
        const from = nodeForSpan(nodesByDoc.get(originDoc) ?? [], originDoc, originLine);
        if (!from) continue;

        const startLine = lineOf(output.content, byteToChar(output.content, run.start));
        const endLine = lineOf(output.content, byteToChar(output.content, Math.max(run.start, run.end - 1)));
        const id = `${output.path}@${startLine}`;
        if (!nodes.has(id)) {
          nodes.set(id, {
            id,
            file: output.path,
            startLine,
            endLine,
            label: `${output.path}:${startLine + 1}`,
            kind: "output",
          });
        }
        // One link per (fragment, output run). A fragment pasted twice into
        // one file is two links, which is the truth about it.
        if (!links.some((l) => l.from === from.id && l.to === id)) {
          links.push({ from: from.id, to: id, kind: "paste" });
        }
      }
    }
  }

  return { files, nodes, links: [...links, ...extraLinks], stages };
}
</storycode:copy>

### Assemble `apps/web/src/lineage/build.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lineage/build.ts"><storycode:paste select="#f-67fd95b7486a-SourceDocument"/><storycode:paste select="#f-67fd95b7486a-lineLookup"/><storycode:paste select="#f-67fd95b7486a-lineOf"/><storycode:paste select="#f-67fd95b7486a-LINEAGE_MODEL_TEXT_LIMIT"/><storycode:paste select="#f-67fd95b7486a-LINEAGE_PREVIEW_CHARACTERS"/><storycode:paste select="#f-67fd95b7486a-fileModel"/><storycode:paste select="#f-67fd95b7486a-stageName"/><storycode:paste select="#f-67fd95b7486a-documentNodes"/><storycode:paste select="#f-67fd95b7486a-nodeForSpan"/><storycode:paste select="#f-67fd95b7486a-outputRuns"/><storycode:paste select="#f-67fd95b7486a-buildModel"/></storycode:file>

<a id="output-88ca70181fed"></a>
## `apps/web/src/lineage/model.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Lineage Browser Shows One Column Per Stage](contract.md).

<a id="f-88ca70181fed-FileModel"></a>
### `FileModel`

The lineage browser's model: stages, the lines each column is showing, and
the typed links between blocks.
Everything here is pure. The two callers give it very different data — the
desktop app reads documents and provenance from the local server, the
marketing page weaves a document in the visitor's browser — and neither
difference reaches the view, because both arrive as this shape.
See docs/specs/freeform/stages-write-forward.md for what a stage is and the
one rule about direction.
A file a column can open: a `.md` document or something one generated.

<storycode:copy id="f-88ca70181fed-FileModel">
// The lineage browser's model: stages, the lines each column is showing, and
// the typed links between blocks.
//
// Everything here is pure. The two callers give it very different data — the
// desktop app reads documents and provenance from the local server, the
// marketing page weaves a document in the visitor's browser — and neither
// difference reaches the view, because both arrive as this shape.
//
// See docs/specs/freeform/stages-write-forward.md for what a stage is and the
// one rule about direction.

/** A file a column can open: a `.md` document or something one generated. */
export interface FileModel {
  path: string;
  lines: string[];
  kind: "document" | "generated";
  documentId?: string;
  preview?: { totalCharacters: number; shownCharacters: number };
}

</storycode:copy>
<a id="f-88ca70181fed-LineageNode"></a>
### `LineageNode`

A block a link can attach to: a fragment, a file block, a cell, or a run of
generated bytes that came from one of those.

<storycode:copy id="f-88ca70181fed-LineageNode">
/** A block a link can attach to: a fragment, a file block, a cell, or a run of
 *  generated bytes that came from one of those. */
export interface LineageNode {
  id: string;
  file: string;
  /** Inclusive line range, 0-based. */
  startLine: number;
  endLine: number;
  label: string;
  /** `copy`, `file`, `exec`, … for documents; `output` for generated runs. */
  kind: string;
}

</storycode:copy>
<a id="f-88ca70181fed-LinkKind"></a>
### `LinkKind`

How a link came to be. The distinction is the point: one is computed from
the weave and the other is a claim somebody made, and a picture that drew
them alike would be lying about which parts are checked.

<storycode:copy id="f-88ca70181fed-LinkKind">
/**
 * How a link came to be. The distinction is the point: one is computed from
 * the weave and the other is a claim somebody made, and a picture that drew
 * them alike would be lying about which parts are checked.
 */
export type LinkKind = "paste" | "structural" | "asserted";

</storycode:copy>
<a id="f-88ca70181fed-LINK_KINDS"></a>
### `LINK_KINDS`

<storycode:copy id="f-88ca70181fed-LINK_KINDS">
export const LINK_KINDS: Record<LinkKind, { label: string; detail: string }> = {
  paste: {
    label: "paste",
    detail:
      "Computed. These bytes are woven from that fragment, and an edit at either end " +
      "reaches the other.",
  },
  structural: {
    label: "structural",
    detail:
      "Name-matched by tree-sitter: this call and that definition share a name. It ships " +
      "in the download and needs no language server — and it is wrong wherever a name is " +
      "not unique, which it says by counting the candidates.",
  },
  asserted: {
    label: "asserted",
    detail:
      "Declared by a person: this influenced that. Nothing verifies it — the chain " +
      "records the claim, not its truth.",
  },
};

</storycode:copy>
<a id="f-88ca70181fed-Link"></a>
### `Link`

<storycode:copy id="f-88ca70181fed-Link">
export interface Link {
  from: string;
  to: string;
  kind: LinkKind;
}

</storycode:copy>
<a id="f-88ca70181fed-Stage"></a>
### `Stage`

<storycode:copy id="f-88ca70181fed-Stage">
/** A document plus the files it generates. */
export interface Stage {
  /** Display name: the document, without folder or suffix. */
  name: string;
  /** The document that owns the stage. */
  doc: string;
  /** Every file in it, the document included. */
  files: string[];
}

</storycode:copy>
<a id="f-88ca70181fed-LineageModel"></a>
### `LineageModel`

<storycode:copy id="f-88ca70181fed-LineageModel">
export interface LineageModel {
  files: Map<string, FileModel>;
  nodes: Map<string, LineageNode>;
  links: Link[];
  stages: Stage[];
}

</storycode:copy>
<a id="f-88ca70181fed-Range"></a>
### `Range`

Visible line ranges. The gaps between them are the holes, so growing a hole
until it meets its neighbour and merging them are the same operation.

<storycode:copy id="f-88ca70181fed-Range">
// ---------------------------------------------------------------------------
// Visible line ranges. The gaps between them are the holes, so growing a hole
// until it meets its neighbour and merging them are the same operation.
// ---------------------------------------------------------------------------

export type Range = [number, number];

</storycode:copy>
<a id="f-88ca70181fed-normalizeRanges"></a>
### `normalizeRanges`

<storycode:copy id="f-88ca70181fed-normalizeRanges">
export function normalizeRanges(ranges: Range[], max: number): Range[] {
  const clipped = ranges
    .map(([a, b]): Range => [Math.max(0, a), Math.min(max, b)])
    .filter(([a, b]) => a <= b)
    .sort((x, y) => x[0] - y[0]);
  const out: Range[] = [];
  for (const range of clipped) {
    const last = out[out.length - 1];
    // Touching counts as adjacent: with no line between them, a hole would
    // hide nothing, and a hole that hides nothing must not be drawn.
    if (last && range[0] <= last[1] + 1) last[1] = Math.max(last[1], range[1]);
    else out.push([...range]);
  }
  return out;
}

</storycode:copy>
<a id="f-88ca70181fed-revealRange"></a>
### `revealRange`

<storycode:copy id="f-88ca70181fed-revealRange">
export function revealRange(ranges: Range[], from: number, to: number, max: number): Range[] {
  return normalizeRanges([...ranges, [from, to]], max);
}

</storycode:copy>
<a id="f-88ca70181fed-foldRange"></a>
### `foldRange`

Fold lines away. Subtracting a range can split one range in two, which is
how a hole appears in the middle of a file.

<storycode:copy id="f-88ca70181fed-foldRange">
/** Fold lines away. Subtracting a range can split one range in two, which is
 *  how a hole appears in the middle of a file. */
export function foldRange(ranges: Range[], from: number, to: number, max: number): Range[] {
  const out: Range[] = [];
  for (const [a, b] of ranges) {
    if (to < a || from > b) {
      out.push([a, b]);
      continue;
    }
    if (from > a) out.push([a, from - 1]);
    if (to < b) out.push([to + 1, b]);
  }
  return normalizeRanges(out, max);
}

</storycode:copy>
<a id="f-88ca70181fed-holesOf"></a>
### `holesOf`

The holes implied by a set of visible ranges.

<storycode:copy id="f-88ca70181fed-holesOf">
/** The holes implied by a set of visible ranges. */
export function holesOf(ranges: Range[], max: number): Range[] {
  const out: Range[] = [];
  let cursor = 0;
  for (const [a, b] of ranges) {
    if (a > cursor) out.push([cursor, a - 1]);
    cursor = b + 1;
  }
  if (cursor <= max) out.push([cursor, max]);
  return out;
}

</storycode:copy>
<a id="f-88ca70181fed-fullRange"></a>
### `fullRange`

<storycode:copy id="f-88ca70181fed-fullRange">
export function fullRange(file: FileModel): Range[] {
  return [[0, Math.max(0, file.lines.length - 1)]];
}

</storycode:copy>
<a id="f-88ca70181fed-matchingLines"></a>
### `matchingLines`

Lines matching `query`, case-insensitively.

<storycode:copy id="f-88ca70181fed-matchingLines">
/** Lines matching `query`, case-insensitively. */
export function matchingLines(file: FileModel, query: string): Set<number> {
  const needle = query.trim().toLowerCase();
  const out = new Set<number>();
  if (!needle) return out;
  file.lines.forEach((line, i) => {
    if (line.toLowerCase().includes(needle)) out.add(i);
  });
  return out;
}

</storycode:copy>
<a id="f-88ca70181fed-rangesForLines"></a>
### `rangesForLines`

Fold a file down to the given lines, keeping `context` lines around each.
The hits come from the ranked index rather than from a substring scan, so
"which lines" and "in what order they matter" are decided in one place
(see bm25.ts) instead of drifting between the fold and the ranking.

<storycode:copy id="f-88ca70181fed-rangesForLines">
/**
 * Fold a file down to the given lines, keeping `context` lines around each.
 *
 * The hits come from the ranked index rather than from a substring scan, so
 * "which lines" and "in what order they matter" are decided in one place
 * (see bm25.ts) instead of drifting between the fold and the ranking.
 */
export function rangesForLines(
  file: FileModel,
  lines: Iterable<number>,
  context = 2,
): Range[] {
  const max = Math.max(0, file.lines.length - 1);
  const ranges = [...lines].map((i): Range => [i - context, i + context]);
  if (!ranges.length) return [];
  return normalizeRanges(ranges, max);
}

</storycode:copy>
<a id="f-88ca70181fed-searchRanges"></a>
### `searchRanges`

Fold a file to the lines literally containing `query`. Kept for callers
with no index to hand; the browser uses the ranked path.

<storycode:copy id="f-88ca70181fed-searchRanges">
/** Fold a file to the lines literally containing `query`. Kept for callers
 *  with no index to hand; the browser uses the ranked path. */
export function searchRanges(file: FileModel, query: string, context = 2): Range[] {
  if (!query.trim()) return fullRange(file);
  return rangesForLines(file, matchingLines(file, query), context);
}

</storycode:copy>
<a id="f-88ca70181fed-walk"></a>
### `walk`

Traversal
Everything reachable from `id`, following only the kinds in `kinds`.
Filtering by kind changes the GRAPH, not just the drawing: turning off the
kinds nothing verifies has to make the chain as short as the verified links
really make it, or the filter would be decoration.

<storycode:copy id="f-88ca70181fed-walk">
// ---------------------------------------------------------------------------
// Traversal
// ---------------------------------------------------------------------------

/**
 * Everything reachable from `id`, following only the kinds in `kinds`.
 *
 * Filtering by kind changes the GRAPH, not just the drawing: turning off the
 * kinds nothing verifies has to make the chain as short as the verified links
 * really make it, or the filter would be decoration.
 */
export function walk(
  links: Link[],
  kinds: Set<LinkKind>,
  id: string,
  direction: "up" | "down",
): Set<string> {
  const seen = new Set<string>();
  const queue = [id];
  while (queue.length) {
    const current = queue.shift()!;
    for (const link of links) {
      if (!kinds.has(link.kind)) continue;
      const next =
        direction === "up"
          ? link.to === current
            ? link.from
            : null
          : link.from === current
            ? link.to
            : null;
      if (next && !seen.has(next)) {
        seen.add(next);
        queue.push(next);
      }
    }
  }
  return seen;
}

</storycode:copy>
<a id="f-88ca70181fed-stageOf"></a>
### `stageOf`

The stage that owns a file, or undefined for a file outside the pipeline.

<storycode:copy id="f-88ca70181fed-stageOf">
/** The stage that owns a file, or undefined for a file outside the pipeline. */
export function stageOf(model: LineageModel, path: string): Stage | undefined {
  return model.stages.find((s) => s.files.includes(path));
}

</storycode:copy>
<a id="f-88ca70181fed-nodeAt"></a>
### `nodeAt`

<storycode:copy id="f-88ca70181fed-nodeAt">
/** The node covering a line, if any. */
export function nodeAt(model: LineageModel, path: string, line: number): LineageNode | undefined {
  let best: LineageNode | undefined;
  for (const node of model.nodes.values()) {
    if (node.file !== path || line < node.startLine || line > node.endLine) continue;
    // The tightest enclosing node wins: a paste inside a file block should
    // select the paste, not the file that contains it.
    if (!best || node.endLine - node.startLine < best.endLine - best.startLine) best = node;
  }
  return best;
}
</storycode:copy>

### Assemble `apps/web/src/lineage/model.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lineage/model.ts"><storycode:paste select="#f-88ca70181fed-FileModel"/><storycode:paste select="#f-88ca70181fed-LineageNode"/><storycode:paste select="#f-88ca70181fed-LinkKind"/><storycode:paste select="#f-88ca70181fed-LINK_KINDS"/><storycode:paste select="#f-88ca70181fed-Link"/><storycode:paste select="#f-88ca70181fed-Stage"/><storycode:paste select="#f-88ca70181fed-LineageModel"/><storycode:paste select="#f-88ca70181fed-Range"/><storycode:paste select="#f-88ca70181fed-normalizeRanges"/><storycode:paste select="#f-88ca70181fed-revealRange"/><storycode:paste select="#f-88ca70181fed-foldRange"/><storycode:paste select="#f-88ca70181fed-holesOf"/><storycode:paste select="#f-88ca70181fed-fullRange"/><storycode:paste select="#f-88ca70181fed-matchingLines"/><storycode:paste select="#f-88ca70181fed-rangesForLines"/><storycode:paste select="#f-88ca70181fed-searchRanges"/><storycode:paste select="#f-88ca70181fed-walk"/><storycode:paste select="#f-88ca70181fed-stageOf"/><storycode:paste select="#f-88ca70181fed-nodeAt"/></storycode:file>

<a id="output-c5b2cd2e7e50"></a>
## `crates/hick-structure/src/lib.rs`

Provide offline structural navigation from bundled parsers. Name-based links are navigation guesses and must remain distinguishable from semantic resolution.

Related contracts: [The Lineage Browser Shows One Column Per Stage](contract.md).

<a id="f-c5b2cd2e7e50-Definition"></a>
### `Definition`

Structural navigation: definitions and references, without a language
server.
## Why this exists
Precise navigation is a solved problem — LSP and SCIP both do it properly
— and neither can ship in a download. A language server is a per-language
program with a per-language runtime; a SCIP index is a per-language build
step. Both are right when the user already has the toolchain, and both are
absent on the machine of someone who just installed `hick` to read a
document.
Tree-sitter is the one option that travels: the grammars are C, compiled
into this binary, so structure works offline on a machine with nothing
installed.
## What it is honest about
Resolution here is **by name**. A reference to `load` links to every
definition named `load` that this crate can see. That is right often
enough to navigate by and wrong in exactly the ways you would expect:
overloads, shadowing, methods on different types sharing a name, dynamic
dispatch, and anything imported from outside the files given to it.
So the links it produces are labelled `structural`, and the layer above
draws them differently from links that were computed exactly
(`docs/guarantees/authoring/the-lineage-browser-shows-one-column-per-stage.md`).
A guess presented as a fact is worse than no answer; a guess labelled as
one is a good place to start looking.

<storycode:copy id="f-c5b2cd2e7e50-Definition">
//! Structural navigation: definitions and references, without a language
//! server.
//!
//! ## Why this exists
//!
//! Precise navigation is a solved problem — LSP and SCIP both do it properly
//! — and neither can ship in a download. A language server is a per-language
//! program with a per-language runtime; a SCIP index is a per-language build
//! step. Both are right when the user already has the toolchain, and both are
//! absent on the machine of someone who just installed `hick` to read a
//! document.
//!
//! Tree-sitter is the one option that travels: the grammars are C, compiled
//! into this binary, so structure works offline on a machine with nothing
//! installed.
//!
//! ## What it is honest about
//!
//! Resolution here is **by name**. A reference to `load` links to every
//! definition named `load` that this crate can see. That is right often
//! enough to navigate by and wrong in exactly the ways you would expect:
//! overloads, shadowing, methods on different types sharing a name, dynamic
//! dispatch, and anything imported from outside the files given to it.
//!
//! So the links it produces are labelled `structural`, and the layer above
//! draws them differently from links that were computed exactly
//! (`docs/guarantees/authoring/the-lineage-browser-shows-one-column-per-stage.md`).
//! A guess presented as a fact is worse than no answer; a guess labelled as
//! one is a good place to start looking.

use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser, Tree};

/// A named thing a file defines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
    pub name: String,
    /// `function`, `class`, `method`, `struct`, `variable`, …
    pub kind: String,
    /// 0-based, inclusive.
    pub start_line: usize,
    pub end_line: usize,
}

</storycode:copy>
<a id="f-c5b2cd2e7e50-Reference"></a>
### `Reference`

<storycode:copy id="f-c5b2cd2e7e50-Reference">
/// A place a name is used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    pub name: String,
    pub line: usize,
}

</storycode:copy>
<a id="f-c5b2cd2e7e50-FileStructure"></a>
### `FileStructure`

<storycode:copy id="f-c5b2cd2e7e50-FileStructure">
/// One file's structure.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileStructure {
    pub path: String,
    pub language: String,
    pub definitions: Vec<Definition>,
    pub references: Vec<Reference>,
}

</storycode:copy>
<a id="f-c5b2cd2e7e50-StructuralLink"></a>
### `StructuralLink`

A reference resolved to the definitions that share its name.

<storycode:copy id="f-c5b2cd2e7e50-StructuralLink">
/// A reference resolved to the definitions that share its name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralLink {
    pub from_path: String,
    pub from_line: usize,
    pub to_path: String,
    pub to_line: usize,
    pub name: String,
    /// How many definitions share this name across the analysed files.
    ///
    /// One is the ordinary case. More than one means the link is a guess
    /// among several, and the reader deserves to know that before following
    /// it.
    pub candidates: usize,
}

</storycode:copy>
<a id="f-c5b2cd2e7e50-language_of"></a>
### `language_of`

Languages this build can parse. Anything else yields no structure rather
than a wrong one.

<storycode:copy id="f-c5b2cd2e7e50-language_of">
/// Languages this build can parse. Anything else yields no structure rather
/// than a wrong one.
pub fn language_of(path: &str) -> Option<&'static str> {
    let language = hick_plugins::routing::language_id(path)?;
    match hick_plugins::registry().interface(language, hick_plugins::InterfaceKind::Structure)? {
        hick_plugins::Interface::Structure {
            language: label, ..
        } => Some(label(language)),
        _ => None,
    }
}

</storycode:copy>
<a id="f-c5b2cd2e7e50-parser_for"></a>
### `parser_for`

<storycode:copy id="f-c5b2cd2e7e50-parser_for">
fn parser_for(language: &str) -> Option<Parser> {
    let canonical = hick_plugins::routing::language_id(language).unwrap_or(language);
    let hick_plugins::Interface::Structure { grammar, .. } =
        hick_plugins::registry().interface(canonical, hick_plugins::InterfaceKind::Structure)?
    else {
        return None;
    };
    let mut parser = Parser::new();
    parser.set_language(&grammar(canonical)?).ok()?;
    Some(parser)
}

</storycode:copy>
<a id="f-c5b2cd2e7e50-definition_kind"></a>
### `definition_kind`

Node kinds that introduce a name, per language family.
Kept as a table rather than a query file: the set is small, it is the same
shape in every grammar tree-sitter ships, and a `.scm` query file would be
one more thing to keep in step with a grammar upgrade.

<storycode:copy id="f-c5b2cd2e7e50-definition_kind">
/// Node kinds that introduce a name, per language family.
///
/// Kept as a table rather than a query file: the set is small, it is the same
/// shape in every grammar tree-sitter ships, and a `.scm` query file would be
/// one more thing to keep in step with a grammar upgrade.
fn definition_kind(node_kind: &str) -> Option<&'static str> {
    Some(match node_kind {
        "function_definition" | "function_declaration" | "function_item" => "function",
        "class_definition" | "class_declaration" => "class",
        "method_definition" | "method_declaration" => "method",
        "struct_item" => "struct",
        "enum_item" | "enum_declaration" => "enum",
        "trait_item" => "trait",
        "impl_item" => "impl",
        "interface_declaration" => "interface",
        "type_alias_declaration" | "type_item" => "type",
        "const_item" | "static_item" => "const",
        _ => return None,
    })
}

</storycode:copy>
<a id="f-c5b2cd2e7e50-name_of"></a>
### `name_of`

<storycode:copy id="f-c5b2cd2e7e50-name_of">
/// The identifier a definition node names.
fn name_of<'a>(node: Node<'a>, source: &'a str) -> Option<String> {
    // Every grammar here puts the declared name in a `name` field; falling
    // back to the first identifier child covers the few that do not.
    if let Some(child) = node.child_by_field_name("name") {
        return Some(source[child.byte_range()].to_string());
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind().contains("identifier") {
            return Some(source[child.byte_range()].to_string());
        }
    }
    None
}

</storycode:copy>
<a id="f-c5b2cd2e7e50-walk"></a>
### `walk`

<storycode:copy id="f-c5b2cd2e7e50-walk">
fn walk(node: Node, source: &str, out: &mut FileStructure, depth: usize) {
    // A depth bound rather than recursion the tree could exhaust: a generated
    // file with pathological nesting must not take the process down.
    if depth > 200 {
        return;
    }
    if let Some(kind) = definition_kind(node.kind())
        && let Some(name) = name_of(node, source)
    {
        out.definitions.push(Definition {
            name,
            kind: kind.to_string(),
            start_line: node.start_position().row,
            end_line: node.end_position().row,
        });
    }

    // References: the callee of a call, and field/attribute access. Bare
    // identifiers are deliberately NOT collected — every parameter, local and
    // keyword would become a "reference", and a navigation list where
    // everything is a hit is a list nobody reads.
    if (node.kind() == "call" || node.kind() == "call_expression")
        && let Some(callee) = node.child_by_field_name("function")
    {
        {
            let text = source[callee.byte_range()].to_string();
            // `a.b.c()` refers to `c`.
            let name = text.rsplit(['.', ':']).next().unwrap_or(&text).to_string();
            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                out.references.push(Reference {
                    name,
                    line: callee.start_position().row,
                });
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(child, source, out, depth + 1);
    }
}

</storycode:copy>
<a id="f-c5b2cd2e7e50-analyze"></a>
### `analyze`

Parse one file. Returns `None` for a language this build cannot read —
never a guess.

<storycode:copy id="f-c5b2cd2e7e50-analyze">
/// Parse one file. Returns `None` for a language this build cannot read —
/// never a guess.
pub fn analyze(path: &str, source: &str) -> Option<FileStructure> {
    let language = language_of(path)?;
    let mut parser = parser_for(language)?;
    let tree: Tree = parser.parse(source, None)?;
    let mut out = FileStructure {
        path: path.to_string(),
        language: language.to_string(),
        ..Default::default()
    };
    walk(tree.root_node(), source, &mut out, 0);
    Some(out)
}

</storycode:copy>
<a id="f-c5b2cd2e7e50-resolve"></a>
### `resolve`

Resolve references to definitions by name across a set of files.
Self-links are dropped: a recursive function referring to itself is true
and useless to draw. Everything else is kept, with the number of
candidates carried along so the caller can say "one of three" rather than
implying certainty.

<storycode:copy id="f-c5b2cd2e7e50-resolve">
/// Resolve references to definitions by name across a set of files.
///
/// Self-links are dropped: a recursive function referring to itself is true
/// and useless to draw. Everything else is kept, with the number of
/// candidates carried along so the caller can say "one of three" rather than
/// implying certainty.
pub fn resolve(files: &[FileStructure]) -> Vec<StructuralLink> {
    let mut by_name: std::collections::HashMap<&str, Vec<(&str, &Definition)>> =
        std::collections::HashMap::new();
    for file in files {
        for def in &file.definitions {
            by_name
                .entry(def.name.as_str())
                .or_default()
                .push((file.path.as_str(), def));
        }
    }

    let mut out = Vec::new();
    for file in files {
        for reference in &file.references {
            let Some(targets) = by_name.get(reference.name.as_str()) else {
                continue;
            };
            for (path, def) in targets {
                let same_place = *path == file.path
                    && reference.line >= def.start_line
                    && reference.line <= def.end_line;
                if same_place {
                    continue;
                }
                out.push(StructuralLink {
                    from_path: file.path.clone(),
                    from_line: reference.line,
                    to_path: (*path).to_string(),
                    to_line: def.start_line,
                    name: reference.name.clone(),
                    candidates: targets.len(),
                });
            }
        }
    }
    out
}

</storycode:copy>
<a id="f-c5b2cd2e7e50-tests"></a>
### `tests`

<storycode:copy id="f-c5b2cd2e7e50-tests">
#[cfg(test)]
mod tests {
    use super::*;

    const PY: &str = r#"
def load(path):
    return open(path).read()

def summarise(path):
    text = load(path)
    return len(text)

class Report:
    def render(self):
        return summarise("x")
"#;

    #[test]
    fn finds_python_definitions_with_their_extent() {
        let s = analyze("a.py", PY).unwrap();
        let names: Vec<_> = s.definitions.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"load"), "{names:?}");
        assert!(names.contains(&"summarise"), "{names:?}");
        assert!(names.contains(&"Report"), "{names:?}");
        assert!(names.contains(&"render"), "{names:?}");

        let load = s.definitions.iter().find(|d| d.name == "load").unwrap();
        assert_eq!(load.kind, "function");
        assert!(
            load.end_line > load.start_line,
            "a definition spans its body"
        );
    }

    #[test]
    fn collects_calls_and_not_every_bare_identifier() {
        let s = analyze("a.py", PY).unwrap();
        let names: Vec<_> = s.references.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"load"));
        assert!(names.contains(&"summarise"));
        // `path` and `text` are locals, not references worth navigating to.
        assert!(!names.contains(&"path"), "{names:?}");
        assert!(!names.contains(&"text"), "{names:?}");
    }

    #[test]
    fn resolves_a_call_to_the_definition_that_shares_its_name() {
        let s = analyze("a.py", PY).unwrap();
        let links = resolve(&[s]);
        let to_load: Vec<_> = links.iter().filter(|l| l.name == "load").collect();
        assert_eq!(to_load.len(), 1, "{links:?}");
        assert_eq!(to_load[0].candidates, 1);
        assert_eq!(to_load[0].to_line, 1, "the def line");
    }

    #[test]
    fn resolves_across_files() {
        let a = analyze("lib.py", "def helper():\n    return 1\n").unwrap();
        let b = analyze("main.py", "def main():\n    return helper()\n").unwrap();
        let links = resolve(&[a, b]);
        let link = links.iter().find(|l| l.name == "helper").unwrap();
        assert_eq!(link.from_path, "main.py");
        assert_eq!(link.to_path, "lib.py");
    }

    #[test]
    fn says_how_many_definitions_a_name_could_mean() {
        // The honest failure mode of name matching: two `render`s, and the
        // link cannot tell which. It reports the ambiguity rather than
        // picking one and looking certain.
        let a = analyze("a.py", "def render():\n    return 1\n").unwrap();
        let b = analyze("b.py", "def render():\n    return 2\n").unwrap();
        let c = analyze("c.py", "def go():\n    return render()\n").unwrap();
        let links = resolve(&[a, b, c]);
        let from_c: Vec<_> = links.iter().filter(|l| l.from_path == "c.py").collect();
        assert_eq!(from_c.len(), 2, "both candidates are offered");
        for link in from_c {
            assert_eq!(link.candidates, 2, "and each says the name is ambiguous");
        }
    }

    #[test]
    fn a_recursive_call_does_not_link_to_itself() {
        let s = analyze("r.py", "def loop(n):\n    return loop(n - 1)\n").unwrap();
        let links = resolve(&[s]);
        assert!(links.is_empty(), "{links:?}");
    }

    #[test]
    fn reads_rust_and_typescript_too() {
        let rust = analyze("m.rs", "fn helper() -> u8 { 1 }\nfn main() { helper(); }\n").unwrap();
        assert_eq!(rust.language, "rust");
        let links = resolve(&[rust]);
        assert_eq!(links.iter().filter(|l| l.name == "helper").count(), 1);

        let ts = analyze(
            "m.ts",
            "export function helper(): number { return 1; }\nfunction main() { helper(); }\n",
        )
        .unwrap();
        assert_eq!(ts.language, "typescript");
        assert!(ts.definitions.iter().any(|d| d.name == "helper"));
    }

    #[test]
    fn an_unknown_language_gets_no_structure_rather_than_a_wrong_one() {
        assert!(analyze("notes.md", "# hello\n").is_none());
        assert!(analyze("Makefile", "all:\n\techo hi\n").is_none());
    }

    #[test]
    fn a_file_that_does_not_parse_still_yields_what_it_can() {
        // Tree-sitter recovers from errors, which is the property that makes
        // this usable while somebody is typing.
        let s = analyze("broken.py", "def good():\n    return 1\n\ndef bad(:\n").unwrap();
        assert!(s.definitions.iter().any(|d| d.name == "good"));
    }
}
</storycode:copy>

### Assemble `crates/hick-structure/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-structure/src/lib.rs"><storycode:paste select="#f-c5b2cd2e7e50-Definition"/><storycode:paste select="#f-c5b2cd2e7e50-Reference"/><storycode:paste select="#f-c5b2cd2e7e50-FileStructure"/><storycode:paste select="#f-c5b2cd2e7e50-StructuralLink"/><storycode:paste select="#f-c5b2cd2e7e50-language_of"/><storycode:paste select="#f-c5b2cd2e7e50-parser_for"/><storycode:paste select="#f-c5b2cd2e7e50-definition_kind"/><storycode:paste select="#f-c5b2cd2e7e50-name_of"/><storycode:paste select="#f-c5b2cd2e7e50-walk"/><storycode:paste select="#f-c5b2cd2e7e50-analyze"/><storycode:paste select="#f-c5b2cd2e7e50-resolve"/><storycode:paste select="#f-c5b2cd2e7e50-tests"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/lineage/build.ts](contract.md#output-67fd95b7486a)
- [apps/web/src/lineage/model.ts](contract.md#output-88ca70181fed)
- [crates/hick-structure/src/lib.rs](contract.md#output-c5b2cd2e7e50)

### Tests and fixtures

- [apps/web/src/landing/demos/demos.test.tsx](../../../mechanisms/apps/web/src/landing/demos/demos.test.tsx.md#output-e2fe96c33971)
- [apps/web/src/lineage/LineageColumns.test.tsx](../../agent/scoped-reading-surfaces-share-native-actions/implementation/apps/web/src/lineage/LineageColumns.test.tsx.md#output-0ea1a191da2e)
- [apps/web/src/lineage/build.test.ts](../../../mechanisms/apps/web/src/lineage/build.test.ts.md#output-c8188ea98404)
- [apps/web/src/lineage/model.test.ts](../../../mechanisms/apps/web/src/lineage/model.test.ts.md#output-a5a955857ba4)
- [apps/web/src/router.test.ts](../../../mechanisms/apps/web/src/router.test.ts.md#output-415811f3ab59)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</storycode:doc>
