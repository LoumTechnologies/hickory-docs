<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/diff.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c4632e343941"></a>
## `apps/web/src/lib/diff.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c4632e343941-TextEdit"></a>
### `TextEdit`

Minimal diff between two text buffers, expressed as replace-range edits for
POST /api/docs/:id/outputs/edit (`{start, end, text}`, byte offsets).
Strategy: line-level LCS on the changed middle (common prefix/suffix lines
stripped first), each changed run trimmed by character prefix/suffix so the
ranges are as tight as possible. Falls back to a single edit when the input
is too large for the DP table — correctness over minimality.

<storycode:copy id="f-c4632e343941-TextEdit">
// Minimal diff between two text buffers, expressed as replace-range edits for
// POST /api/docs/:id/outputs/edit (`{start, end, text}`, byte offsets).
//
// Strategy: line-level LCS on the changed middle (common prefix/suffix lines
// stripped first), each changed run trimmed by character prefix/suffix so the
// ranges are as tight as possible. Falls back to a single edit when the input
// is too large for the DP table — correctness over minimality.

import { charToByte } from "./offsets";

export interface TextEdit {
  /** Replace [start, end) — UTF-16 char offsets into the old text. */
  start: number;
  end: number;
  text: string;
}

</storycode:copy>
<a id="f-c4632e343941-toByteEdits"></a>
### `toByteEdits`

`{start, end}` converted from char offsets in `oldText` to UTF-8 bytes.

<storycode:copy id="f-c4632e343941-toByteEdits">
/** `{start, end}` converted from char offsets in `oldText` to UTF-8 bytes. */
export function toByteEdits(oldText: string, edits: TextEdit[]): TextEdit[] {
  return edits.map((e) => ({
    start: charToByte(oldText, e.start),
    end: charToByte(oldText, e.end),
    text: e.text,
  }));
}

</storycode:copy>
<a id="f-c4632e343941-splitLines"></a>
### `splitLines`

<storycode:copy id="f-c4632e343941-splitLines">
function splitLines(text: string): string[] {
  // Keep the newline attached so offsets reconstruct exactly.
  const lines: string[] = [];
  let start = 0;
  while (start <= text.length) {
    const nl = text.indexOf("\n", start);
    if (nl < 0) {
      if (start < text.length) lines.push(text.slice(start));
      break;
    }
    lines.push(text.slice(start, nl + 1));
    start = nl + 1;
  }
  return lines;
}

</storycode:copy>
<a id="f-c4632e343941-shrink"></a>
### `shrink`

Trim a single replacement to its tight char range.

<storycode:copy id="f-c4632e343941-shrink">
/** Trim a single replacement to its tight char range. */
function shrink(oldText: string, start: number, end: number, text: string): TextEdit | null {
  let a = 0;
  const oldLen = end - start;
  while (a < oldLen && a < text.length && oldText[start + a] === text[a]) a++;
  let b = 0;
  while (
    b < oldLen - a &&
    b < text.length - a &&
    oldText[end - 1 - b] === text[text.length - 1 - b]
  ) {
    b++;
  }
  const s = start + a;
  const e = end - b;
  const t = text.slice(a, text.length - b);
  if (s === e && t === "") return null;
  return { start: s, end: e, text: t };
}

</storycode:copy>
<a id="f-c4632e343941-MAX_DP_CELLS"></a>
### `MAX_DP_CELLS`

<storycode:copy id="f-c4632e343941-MAX_DP_CELLS">
const MAX_DP_CELLS = 4_000_000;

</storycode:copy>
<a id="f-c4632e343941-computeEdits"></a>
### `computeEdits`

Minimal replace-range edits turning `oldText` into `newText` (char offsets;
use `toByteEdits` for the wire format). Empty array when the texts match.

<storycode:copy id="f-c4632e343941-computeEdits">
/**
 * Minimal replace-range edits turning `oldText` into `newText` (char offsets;
 * use `toByteEdits` for the wire format). Empty array when the texts match.
 */
export function computeEdits(oldText: string, newText: string): TextEdit[] {
  if (oldText === newText) return [];

  const oldLines = splitLines(oldText);
  const newLines = splitLines(newText);

  // Strip common prefix/suffix lines.
  let pre = 0;
  while (pre < oldLines.length && pre < newLines.length && oldLines[pre] === newLines[pre]) pre++;
  let suf = 0;
  while (
    suf < oldLines.length - pre &&
    suf < newLines.length - pre &&
    oldLines[oldLines.length - 1 - suf] === newLines[newLines.length - 1 - suf]
  ) {
    suf++;
  }

  const oldMid = oldLines.slice(pre, oldLines.length - suf);
  const newMid = newLines.slice(pre, newLines.length - suf);
  const midStart = oldLines.slice(0, pre).reduce((n, l) => n + l.length, 0);
  const midEndOld = oldText.length - oldLines.slice(oldLines.length - suf).reduce((n, l) => n + l.length, 0);
  const newMidStart = newLines.slice(0, pre).reduce((n, l) => n + l.length, 0);
  const newMidEnd = newText.length - newLines.slice(newLines.length - suf).reduce((n, l) => n + l.length, 0);

  const n = oldMid.length;
  const m = newMid.length;

  if (n === 0 || m === 0 || (n + 1) * (m + 1) > MAX_DP_CELLS) {
    const single = shrink(oldText, midStart, midEndOld, newText.slice(newMidStart, newMidEnd));
    return single ? [single] : [];
  }

  // LCS table over the middle lines.
  const width = m + 1;
  const dp = new Uint32Array((n + 1) * width);
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i * width + j] =
        oldMid[i] === newMid[j]
          ? dp[(i + 1) * width + j + 1] + 1
          : Math.max(dp[(i + 1) * width + j], dp[i * width + j + 1]);
    }
  }

  // Walk the table, emitting a replace edit per changed run.
  const edits: TextEdit[] = [];
  let i = 0;
  let j = 0;
  let oldPos = midStart;
  let newPos = newMidStart;
  let runOldStart = -1;
  let runNewStart = -1;

  const flush = (oldEnd: number, newEnd: number) => {
    if (runOldStart < 0) return;
    const e = shrink(oldText, runOldStart, oldEnd, newText.slice(runNewStart, newEnd));
    if (e) edits.push(e);
    runOldStart = -1;
    runNewStart = -1;
  };

  while (i < n || j < m) {
    if (i < n && j < m && oldMid[i] === newMid[j]) {
      flush(oldPos, newPos);
      oldPos += oldMid[i].length;
      newPos += newMid[j].length;
      i++;
      j++;
    } else {
      if (runOldStart < 0) {
        runOldStart = oldPos;
        runNewStart = newPos;
      }
      if (j >= m || (i < n && dp[(i + 1) * width + j] >= dp[i * width + j + 1])) {
        oldPos += oldMid[i].length;
        i++;
      } else {
        newPos += newMid[j].length;
        j++;
      }
    }
  }
  flush(oldPos, newPos);
  return edits;
}

</storycode:copy>
<a id="f-c4632e343941-DiffLine"></a>
### `DiffLine`

Display diff (expected vs actual output in a failed cell panel).

<storycode:copy id="f-c4632e343941-DiffLine">
// ---------------------------------------------------------------------------
// Display diff (expected vs actual output in a failed cell panel).
// ---------------------------------------------------------------------------

export interface DiffLine {
  /** "same" | "del" (expected line not produced) | "ins" (actual-only line). */
  kind: "same" | "del" | "ins";
  text: string;
}

</storycode:copy>
<a id="f-c4632e343941-diffLines"></a>
### `diffLines`

Line-level diff of `expected` → `actual` for display (the failed-cell
panel). Same LCS strategy as `computeEdits`, but emitting per-line rows
instead of replace-range edits. Trailing newlines are normalised away so a
missing final "\n" never shows as a phantom change.

<storycode:copy id="f-c4632e343941-diffLines">
/**
 * Line-level diff of `expected` → `actual` for display (the failed-cell
 * panel). Same LCS strategy as `computeEdits`, but emitting per-line rows
 * instead of replace-range edits. Trailing newlines are normalised away so a
 * missing final "\n" never shows as a phantom change.
 */
export function diffLines(expected: string, actual: string, preserveNewlines = false): DiffLine[] {
  const strip = (l: string) => (l.endsWith("\n") ? l.slice(0, -1) : l);
  const a = splitLines(preserveNewlines ? expected : expected.replace(/\n$/, "")).map(l => preserveNewlines ? l : strip(l));
  const b = splitLines(preserveNewlines ? actual : actual.replace(/\n$/, "")).map(l => preserveNewlines ? l : strip(l));
  const n = a.length;
  const m = b.length;
  const rows: DiffLine[] = [];

  if ((n + 1) * (m + 1) > MAX_DP_CELLS) {
    for (const text of a) rows.push({ kind: "del", text });
    for (const text of b) rows.push({ kind: "ins", text });
    return rows;
  }

  const width = m + 1;
  const dp = new Uint32Array((n + 1) * width);
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i * width + j] =
        a[i] === b[j]
          ? dp[(i + 1) * width + j + 1] + 1
          : Math.max(dp[(i + 1) * width + j], dp[i * width + j + 1]);
    }
  }
  let i = 0;
  let j = 0;
  while (i < n || j < m) {
    if (i < n && j < m && a[i] === b[j]) {
      rows.push({ kind: "same", text: a[i] });
      i++;
      j++;
    } else if (j >= m || (i < n && dp[(i + 1) * width + j] >= dp[i * width + j + 1])) {
      rows.push({ kind: "del", text: a[i] });
      i++;
    } else {
      rows.push({ kind: "ins", text: b[j] });
      j++;
    }
  }
  return rows;
}

</storycode:copy>
<a id="f-c4632e343941-applyEdits"></a>
### `applyEdits`

Apply replace-range edits (char offsets, non-overlapping) to `text`.

<storycode:copy id="f-c4632e343941-applyEdits">
/** Apply replace-range edits (char offsets, non-overlapping) to `text`. */
export function applyEdits(text: string, edits: TextEdit[]): string {
  const sorted = [...edits].sort((a, b) => a.start - b.start);
  let out = "";
  let pos = 0;
  for (const e of sorted) {
    out += text.slice(pos, e.start) + e.text;
    pos = e.end;
  }
  return out + text.slice(pos);
}

</storycode:copy>
<a id="f-c4632e343941-unifiedDiff"></a>
### `unifiedDiff`

Unified display using the same renderer as Git changes, including final-newline evidence.

<storycode:copy id="f-c4632e343941-unifiedDiff">
/** Unified display using the same renderer as Git changes, including final-newline evidence. */
export function unifiedDiff(path: string, before: string, after: string): string {
  if (before === after) return "";
  const count = (text: string) => splitLines(text).length;
  const rows = diffLines(before, after, true).flatMap(row => {
    const prefix = row.kind === "ins" ? "+" : row.kind === "del" ? "-" : " ";
    const text = row.text.replace(/\n$/, "");
    return row.text.endsWith("\n") ? [`${prefix}${text}`] : [`${prefix}${text}`, "\\ No newline at end of file"];
  });
  return `--- a/${path}\n+++ b/${path}\n@@ -${count(before) ? 1 : 0},${count(before)} +${count(after) ? 1 : 0},${count(after)} @@\n${rows.join("\n")}\n`;
}
</storycode:copy>

### Assemble `apps/web/src/lib/diff.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/diff.ts"><storycode:paste select="#f-c4632e343941-TextEdit"/><storycode:paste select="#f-c4632e343941-toByteEdits"/><storycode:paste select="#f-c4632e343941-splitLines"/><storycode:paste select="#f-c4632e343941-shrink"/><storycode:paste select="#f-c4632e343941-MAX_DP_CELLS"/><storycode:paste select="#f-c4632e343941-computeEdits"/><storycode:paste select="#f-c4632e343941-DiffLine"/><storycode:paste select="#f-c4632e343941-diffLines"/><storycode:paste select="#f-c4632e343941-applyEdits"/><storycode:paste select="#f-c4632e343941-unifiedDiff"/></storycode:file>

</storycode:doc>
