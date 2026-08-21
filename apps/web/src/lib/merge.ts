// Three-way (and two-way) merge, as regions a person can look at and choose
// between.
//
// This is diff3, the algorithm `git merge` and every merge tool is built on,
// and it is worth saying plainly what it buys over the two-way comparison it
// would be easier to write. Two-way can only say "these two files differ
// here"; every difference is a question for the reader. Three-way knows what
// the file looked like BEFORE either side touched it, so it can answer most
// of those questions itself: a region only one side changed is not a conflict,
// it is that side's edit, and it is taken silently. What is left over — the
// regions both sides changed, differently — is the only thing worth a human's
// attention.
//
// That is why every draft this app writes records the bytes it was taken
// from. Without the base we fall back to `mergeTwoWay`, which is honest about
// being worse: it marks every differing run as a conflict, because with no
// ancestor there is genuinely no way to tell an edit from a counter-edit.
//
// Pure text in, regions out. Nothing here knows about CodeMirror, a document,
// or a file — the merge is the part worth arguing with in a test.

/** Split keeping the newline attached, so regions rejoin byte-exactly. */
export function splitLines(text: string): string[] {
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

/** Above this the DP table is not worth the memory; see `matchedLines`. */
const MAX_DP_CELLS = 4_000_000;

/**
 * Indices of lines common to `a` and `b`, as `[indexInA, indexInB]` pairs in
 * increasing order — the LCS, which is the spine every alignment here hangs
 * off.
 *
 * On inputs too large for the table this returns no matches at all. That is
 * deliberate and safe: with no common spine, every line is "changed", so the
 * merge degrades into one big conflict covering both files. Slow and blunt,
 * but never wrong — which is the right way for a merge to fail.
 */
export function matchedLines(a: readonly string[], b: readonly string[]): [number, number][] {
  const n = a.length;
  const m = b.length;
  if (n === 0 || m === 0 || (n + 1) * (m + 1) > MAX_DP_CELLS) return [];
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
  const pairs: [number, number][] = [];
  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (a[i] === b[j]) {
      pairs.push([i, j]);
      i++;
      j++;
    } else if (dp[(i + 1) * width + j] >= dp[i * width + j + 1]) {
      i++;
    } else {
      j++;
    }
  }
  return pairs;
}

/** Which side a region's text came from. */
export type MergeSide = "ours" | "theirs";

export type MergeRegion =
  /** Neither side touched this. */
  | { kind: "stable"; text: string }
  /** Exactly one side changed it, or both changed it the same way. Taken
   * without asking; `from` records who to credit, and "both" means the two
   * sides happened to make the identical edit. */
  | { kind: "resolved"; text: string; from: MergeSide | "both" }
  /** Both sides changed it, differently. The only kind a person must answer. */
  | {
      kind: "conflict";
      /** What was there before either side touched it; "" in a two-way merge. */
      base: string;
      ours: string;
      theirs: string;
    };

/** Whether a set of regions still has something to answer. */
export function conflictCount(regions: readonly MergeRegion[]): number {
  return regions.filter((r) => r.kind === "conflict").length;
}

/** A line range of `lines`, rejoined. */
function slice(lines: readonly string[], from: number, to: number): string {
  return lines.slice(from, to).join("");
}

/**
 * For each line of `base`, the line it corresponds to in `other`, or -1 where
 * `other` does not have it. This is the shape diff3 walks: two such maps let
 * us step through the base once and see what each side did to every line.
 */
function alignment(base: readonly string[], other: readonly string[]): number[] {
  const map = new Array<number>(base.length).fill(-1);
  for (const [b, o] of matchedLines(base, other)) map[b] = o;
  return map;
}

/**
 * Merge `ours` and `theirs` given their common ancestor `base`.
 *
 * Regions come back in document order and always rejoin — concatenating the
 * text of every non-conflict region, with a choice made for each conflict,
 * reproduces a whole file with no lines invented or lost.
 */
export function mergeThreeWay(base: string, ours: string, theirs: string): MergeRegion[] {
  const b = splitLines(base);
  const o = splitLines(ours);
  const t = splitLines(theirs);
  const toOurs = alignment(b, o);
  const toTheirs = alignment(b, t);

  const regions: MergeRegion[] = [];
  // The three cursors walk together. `bi` steps through the base; `oi`/`ti`
  // trail behind in each side, so the text each side has accumulated since
  // the last agreed line is exactly what that side did to this region.
  let bi = 0;
  let oi = 0;
  let ti = 0;
  let stableFrom = -1;

  const flushStable = (bEnd: number) => {
    if (stableFrom < 0) return;
    regions.push({ kind: "stable", text: slice(b, stableFrom, bEnd) });
    stableFrom = -1;
  };

  const flushChanged = (bEnd: number, oEnd: number, tEnd: number) => {
    const baseText = slice(b, bi, bEnd);
    const ourText = slice(o, oi, oEnd);
    const theirText = slice(t, ti, tEnd);
    if (ourText === theirText) {
      // Both sides did the same thing — including "both deleted it".
      if (ourText !== baseText) regions.push({ kind: "resolved", text: ourText, from: "both" });
      else if (baseText) regions.push({ kind: "stable", text: baseText });
    } else if (ourText === baseText) {
      regions.push({ kind: "resolved", text: theirText, from: "theirs" });
    } else if (theirText === baseText) {
      regions.push({ kind: "resolved", text: ourText, from: "ours" });
    } else {
      regions.push({ kind: "conflict", base: baseText, ours: ourText, theirs: theirText });
    }
  };

  while (bi < b.length) {
    const oAt = toOurs[bi];
    const tAt = toTheirs[bi];
    // A base line both sides still have, in the same relative place, is an
    // anchor: everything before it belongs to the region we were building.
    if (oAt >= 0 && tAt >= 0 && oAt >= oi && tAt >= ti) {
      if (oAt > oi || tAt > ti) {
        // Both sides inserted before this anchor, or one did.
        flushStable(bi);
        flushChanged(bi, oAt, tAt);
        oi = oAt;
        ti = tAt;
      }
      if (stableFrom < 0) stableFrom = bi;
      bi++;
      oi++;
      ti++;
      continue;
    }
    // Not an anchor: find the next one, and treat everything up to it as one
    // changed region. Grouping rather than emitting line-by-line is what makes
    // a conflict readable — three lines replaced by two is one decision.
    flushStable(bi);
    let bEnd = bi;
    while (bEnd < b.length) {
      const oA = toOurs[bEnd];
      const tA = toTheirs[bEnd];
      if (oA >= 0 && tA >= 0 && oA >= oi && tA >= ti) break;
      bEnd++;
    }
    const oEnd = bEnd < b.length ? toOurs[bEnd] : o.length;
    const tEnd = bEnd < b.length ? toTheirs[bEnd] : t.length;
    flushChanged(bEnd, Math.max(oi, oEnd), Math.max(ti, tEnd));
    bi = bEnd;
    oi = Math.max(oi, oEnd);
    ti = Math.max(ti, tEnd);
  }
  flushStable(bi);
  // Whatever either side appended past the end of the base.
  if (oi < o.length || ti < t.length) flushChanged(b.length, o.length, t.length);
  return regions;
}

/**
 * Compare two files with no common ancestor.
 *
 * Every differing run becomes a conflict, and that is not a shortcoming to be
 * fixed later — without a base there is no way to tell "they added this" from
 * "we deleted it", and a tool that guessed would silently throw away work.
 */
export function mergeTwoWay(ours: string, theirs: string): MergeRegion[] {
  const o = splitLines(ours);
  const t = splitLines(theirs);
  const pairs = matchedLines(o, t);
  const regions: MergeRegion[] = [];
  let oi = 0;
  let ti = 0;
  let stableFrom = -1;

  const flushStable = (end: number) => {
    if (stableFrom < 0) return;
    regions.push({ kind: "stable", text: slice(o, stableFrom, end) });
    stableFrom = -1;
  };

  for (const [oAt, tAt] of pairs) {
    if (oAt > oi || tAt > ti) {
      flushStable(oi);
      regions.push({
        kind: "conflict",
        base: "",
        ours: slice(o, oi, oAt),
        theirs: slice(t, ti, tAt),
      });
    }
    if (stableFrom < 0) stableFrom = oAt;
    oi = oAt + 1;
    ti = tAt + 1;
  }
  flushStable(oi);
  if (oi < o.length || ti < t.length) {
    regions.push({
      kind: "conflict",
      base: "",
      ours: slice(o, oi, o.length),
      theirs: slice(t, ti, t.length),
    });
  }
  return regions;
}

/** What a conflict was answered with. Absent means still unanswered. */
export type Resolution = MergeSide | "both" | "base";

/**
 * The merged file, given an answer for each conflict by its index.
 *
 * An unanswered conflict falls back to `ours` — the reader's own unsaved work
 * — because this text is what goes into the editor buffer while they are
 * still deciding, and the one thing that must never happen is their typing
 * disappearing from the screen while they think.
 */
export function mergedText(
  regions: readonly MergeRegion[],
  resolutions: ReadonlyMap<number, Resolution> = new Map(),
): string {
  let out = "";
  let conflict = 0;
  for (const region of regions) {
    if (region.kind === "conflict") {
      const choice = resolutions.get(conflict) ?? "ours";
      out +=
        choice === "theirs"
          ? region.theirs
          : choice === "base"
            ? region.base
            : choice === "both"
              ? region.ours + region.theirs
              : region.ours;
      conflict++;
    } else {
      out += region.text;
    }
  }
  return out;
}
