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

/** `{start, end}` converted from char offsets in `oldText` to UTF-8 bytes. */
export function toByteEdits(oldText: string, edits: TextEdit[]): TextEdit[] {
  return edits.map((e) => ({
    start: charToByte(oldText, e.start),
    end: charToByte(oldText, e.end),
    text: e.text,
  }));
}

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

const MAX_DP_CELLS = 4_000_000;

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
