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
function lineOf(text: string, offset: number): number { return lineLookup(text)(offset); }

export const LINEAGE_MODEL_TEXT_LIMIT = 1_000_000;
export const LINEAGE_PREVIEW_CHARACTERS = 16_384;
function fileModel(path: string, text: string, kind: FileModel["kind"], documentId?: string): FileModel {
  if (text.length > LINEAGE_MODEL_TEXT_LIMIT) {
    const shown = text.slice(0, LINEAGE_PREVIEW_CHARACTERS);
    return { path, kind, documentId, lines: shown.split("\n"), preview: { totalCharacters: text.length, shownCharacters: shown.length } };
  }
  return { path, kind, documentId, lines: text.replace(/\n$/, "").split("\n") };
}

function stageName(docPath: string): string {
  return docPath.split("/").pop()!.replace(/\.md$/, "").replace(/^\d{4}-\d{2}-\d{2}-/, "");
}

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

/** The node in `doc` whose lines contain a source span, tightest first. */
function nodeForSpan(nodes: LineageNode[], docPath: string, line: number): LineageNode | undefined {
  let best: LineageNode | undefined;
  for (const node of nodes) {
    if (node.file !== docPath || line < node.startLine || line > node.endLine) continue;
    if (!best || node.endLine - node.startLine < best.endLine - best.startLine) best = node;
  }
  return best;
}

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
