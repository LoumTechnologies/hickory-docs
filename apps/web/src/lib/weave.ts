// In-browser weaver: turns a .hick source into its generated output files with
// real provenance ranges, and maps output edits back to source-document edits.
// This mirrors what the server's /outputs endpoints do (hick-literate weave),
// scoped to the tags a client-side document uses: hick:file bodies with
// hick:paste slots filled from hick:copy blocks.
//
// Two callers, both without a server behind them: mock mode (src/mock/mockApi)
// and the landing page's demos (src/landing/demos), which weave and re-weave
// entirely in the visitor's browser so a stranger can drive the lineage
// picture without an account.
//
// All offsets on the wire are UTF-8 BYTES (the contract in api/types.ts) —
// and the mock honours that for real: both the weave-demo source and the
// woven banner contain an em dash, so byte and char offsets genuinely
// diverge. Content offsets and source spans are computed in bytes here; the
// client converts via lib/offsets exactly as it must against the real server.

import { parseHickDoc } from "../editor/hickDoc";
import { byteLength, byteToChar, charToByte } from "../lib/offsets";
import type {
  OutputEdit,
  OutputFile,
  Provenance,
  SourceEdit,
} from "../api/types";

/** Trim a single leading/trailing newline off a content range (tag on its own
 * line), keeping the range pointing into the source. */
function trimContent(source: string, from: number, to: number): [number, number] {
  if (from < to && source[from] === "\n") from++;
  if (to > from && source[to - 1] === "\n") to--;
  return [from, to];
}

/**
 * The synthetic first line of a woven file, commented the way that file's
 * language comments. A `//` on line 1 of a generated Markdown ticket is
 * visible prose, not a comment — the banner has to speak each language or it
 * corrupts the output it is annotating.
 */
function banner(language: string, docPath: string): string {
  const text = `woven by hickory from ${docPath} — edit the doc or the slots below`;
  if (language === "python" || language === "shell") return `# ${text}\n`;
  if (language === "markdown" || language === "html") return `<!-- ${text} -->\n`;
  return `// ${text}\n`;
}

/** Weave all hick:file outputs of `source`, with provenance into the doc. */
export function weaveOutputs(source: string, docPath: string): OutputFile[] {
  const structure = parseHickDoc(source);
  // Source spans on the wire are bytes into the doc source.
  const byteSpan = (from: number, to: number): [number, number] => [
    charToByte(source, from),
    charToByte(source, to),
  ];
  const copies = new Map<string, [number, number]>();
  for (const b of structure.blocks) {
    if (b.name === "copy" && b.attrs.id) {
      copies.set(b.attrs.id, trimContent(source, b.contentFrom, b.contentTo));
    }
  }

  const files: OutputFile[] = [];
  for (const block of structure.blocks) {
    if (block.name !== "file" || !block.attrs.path) continue;
    const [bodyFrom, bodyTo] = trimContent(source, block.contentFrom, block.contentTo);
    const language = block.attrs.language ?? "text";

    let content = "";
    let contentBytes = 0; // provenance offsets are UTF-8 bytes (the contract)
    const provenance: Provenance[] = [];
    const push = (text: string, origin: Provenance["origin"]) => {
      if (text.length === 0) return;
      const bytes = byteLength(text);
      provenance.push({ start: contentBytes, end: contentBytes + bytes, origin });
      content += text;
      contentBytes += bytes;
    };

    // Synthetic banner the weaver adds — not present in any source span.
    push(banner(language, docPath), { kind: "synthetic" });

    // Paste tags inside the file body, in order.
    const pastes = structure.tags.filter(
      (t) =>
        t.name === "paste" &&
        !t.closing &&
        t.from >= bodyFrom &&
        t.to <= bodyTo &&
        typeof t.attrs.select === "string",
    );
    let pos = bodyFrom;
    for (const paste of pastes) {
      push(source.slice(pos, paste.from), {
        kind: "literal",
        doc_path: docPath,
        span: byteSpan(pos, paste.from),
      });
      const id = paste.attrs.select.replace(/^#/, "");
      const copy = copies.get(id);
      if (copy) {
        push(source.slice(copy[0], copy[1]), {
          kind: "paste",
          doc_path: docPath,
          span: byteSpan(copy[0], copy[1]),
        });
      }
      pos = paste.to;
      // The newline right after a paste tag on its own line belongs to the
      // literal text that follows.
    }
    push(source.slice(pos, bodyTo), {
      kind: "literal",
      doc_path: docPath,
      span: byteSpan(pos, bodyTo),
    });
    if (!content.endsWith("\n")) {
      push("\n", { kind: "synthetic" });
    }

    files.push({ path: block.attrs.path, language, content, provenance });
  }
  return files;
}

export class SyntheticRangeViolation extends Error {
  constructor(public range: { start: number; end: number }) {
    super(
      `edit overlaps a synthetic range ${range.start}..${range.end} (weaver-generated text has no source to edit)`,
    );
  }
}

/**
 * Map output-buffer edits through provenance to source-document edits.
 * Throws SyntheticRangeViolation when an edit touches weaver-generated text.
 * Edits crossing two editable ranges are split at the boundary (inserted text
 * goes with the first part).
 */
export function mapEditsToSource(file: OutputFile, edits: OutputEdit[]): SourceEdit[] {
  const sourceEdits: SourceEdit[] = [];
  for (const edit of edits) {
    const overlapping = file.provenance.filter(
      (p) =>
        (edit.start < p.end && edit.end > p.start) ||
        // Pure insertions at a boundary belong to the range they sit inside
        // (or the one ending exactly here).
        (edit.start === edit.end && edit.start >= p.start && edit.start <= p.end),
    );
    const synthetic = overlapping.find(
      (p) => p.origin.kind === "synthetic" && (edit.start < p.end && edit.end > p.start),
    );
    if (synthetic) {
      throw new SyntheticRangeViolation({ start: synthetic.start, end: synthetic.end });
    }
    const editable = overlapping.filter((p) => p.origin.kind !== "synthetic");
    if (editable.length === 0) {
      // Insertion exactly on a synthetic-only boundary.
      const at = file.provenance.find(
        (p) => p.origin.kind === "synthetic" && edit.start >= p.start && edit.start <= p.end,
      );
      throw new SyntheticRangeViolation(
        at ? { start: at.start, end: at.end } : { start: edit.start, end: edit.end },
      );
    }
    editable.sort((a, b) => a.start - b.start);
    let first = true;
    for (const p of editable) {
      if (p.origin.kind === "synthetic") continue;
      const s = Math.max(edit.start, p.start);
      const e = Math.min(edit.end, p.end);
      sourceEdits.push({
        doc_path: p.origin.doc_path,
        span: [p.origin.span[0] + (s - p.start), p.origin.span[0] + (e - p.start)],
        text: first ? edit.text : "",
      });
      first = false;
    }
  }
  return sourceEdits;
}

/** Apply source edits (byte spans, per the contract) to the doc source. */
export function applySourceEdits(source: string, edits: SourceEdit[]): string {
  const sorted = [...edits].sort((a, b) => b.span[0] - a.span[0]);
  let out = source;
  for (const e of sorted) {
    // Convert against the ORIGINAL source: spans reference it, and edits are
    // applied back-to-front so earlier offsets stay valid.
    out = out.slice(0, byteToChar(source, e.span[0])) + e.text + out.slice(byteToChar(source, e.span[1]));
  }
  return out;
}
