<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/merge.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-39496bbc1d89"></a>
## `apps/web/src/lib/merge.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-39496bbc1d89-splitLines"></a>
### `splitLines`

Three-way (and two-way) merge, as regions a person can look at and choose
between.
This is diff3, the algorithm `git merge` and every merge tool is built on,
and it is worth saying plainly what it buys over the two-way comparison it
would be easier to write. Two-way can only say "these two files differ
here"; every difference is a question for the reader. Three-way knows what
the file looked like BEFORE either side touched it, so it can answer most
of those questions itself: a region only one side changed is not a conflict,
it is that side's edit, and it is taken silently. What is left over — the
regions both sides changed, differently — is the only thing worth a human's
attention.
That is why every draft this app writes records the bytes it was taken
from. Without the base we fall back to `mergeTwoWay`, which is honest about
being worse: it marks every differing run as a conflict, because with no
ancestor there is genuinely no way to tell an edit from a counter-edit.
Pure text in, regions out. Nothing here knows about CodeMirror, a document,
or a file — the merge is the part worth arguing with in a test.
Split keeping the newline attached, so regions rejoin byte-exactly.

<storycode:copy id="f-39496bbc1d89-splitLines">
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

</storycode:copy>
<a id="f-39496bbc1d89-MAX_DP_CELLS"></a>
### `MAX_DP_CELLS`

Above this the DP table is not worth the memory; see `matchedLines`.

<storycode:copy id="f-39496bbc1d89-MAX_DP_CELLS">
/** Above this the DP table is not worth the memory; see `matchedLines`. */
const MAX_DP_CELLS = 4_000_000;

</storycode:copy>
<a id="f-39496bbc1d89-matchedLines"></a>
### `matchedLines`

Indices of lines common to `a` and `b`, as `[indexInA, indexInB]` pairs in
increasing order — the LCS, which is the spine every alignment here hangs
off.
On inputs too large for the table this returns no matches at all. That is
deliberate and safe: with no common spine, every line is "changed", so the
merge degrades into one big conflict covering both files. Slow and blunt,
but never wrong — which is the right way for a merge to fail.

<storycode:copy id="f-39496bbc1d89-matchedLines">
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

</storycode:copy>
<a id="f-39496bbc1d89-MergeSide"></a>
### `MergeSide`

<storycode:copy id="f-39496bbc1d89-MergeSide">
/** Which side a region's text came from. */
export type MergeSide = "ours" | "theirs";

</storycode:copy>
<a id="f-39496bbc1d89-MergeRegion"></a>
### `MergeRegion`

<storycode:copy id="f-39496bbc1d89-MergeRegion">
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

</storycode:copy>
<a id="f-39496bbc1d89-conflictCount"></a>
### `conflictCount`

Whether a set of regions still has something to answer.

<storycode:copy id="f-39496bbc1d89-conflictCount">
/** Whether a set of regions still has something to answer. */
export function conflictCount(regions: readonly MergeRegion[]): number {
  return regions.filter((r) => r.kind === "conflict").length;
}

</storycode:copy>
<a id="f-39496bbc1d89-slice"></a>
### `slice`

<storycode:copy id="f-39496bbc1d89-slice">
/** A line range of `lines`, rejoined. */
function slice(lines: readonly string[], from: number, to: number): string {
  return lines.slice(from, to).join("");
}

</storycode:copy>
<a id="f-39496bbc1d89-alignment"></a>
### `alignment`

For each line of `base`, the line it corresponds to in `other`, or -1 where
`other` does not have it. This is the shape diff3 walks: two such maps let
us step through the base once and see what each side did to every line.

<storycode:copy id="f-39496bbc1d89-alignment">
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

</storycode:copy>
<a id="f-39496bbc1d89-mergeThreeWay"></a>
### `mergeThreeWay`

Merge `ours` and `theirs` given their common ancestor `base`.
Regions come back in document order and always rejoin — concatenating the
text of every non-conflict region, with a choice made for each conflict,
reproduces a whole file with no lines invented or lost.

<storycode:copy id="f-39496bbc1d89-mergeThreeWay">
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

</storycode:copy>
<a id="f-39496bbc1d89-mergeTwoWay"></a>
### `mergeTwoWay`

Compare two files with no common ancestor.
Every differing run becomes a conflict, and that is not a shortcoming to be
fixed later — without a base there is no way to tell "they added this" from
"we deleted it", and a tool that guessed would silently throw away work.

<storycode:copy id="f-39496bbc1d89-mergeTwoWay">
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

</storycode:copy>
<a id="f-39496bbc1d89-Resolution"></a>
### `Resolution`

What a conflict was answered with. Absent means still unanswered.

<storycode:copy id="f-39496bbc1d89-Resolution">
/** What a conflict was answered with. Absent means still unanswered. */
export type Resolution = MergeSide | "both" | "base";

</storycode:copy>
<a id="f-39496bbc1d89-mergedText"></a>
### `mergedText`

The merged file, given an answer for each conflict by its index.
An unanswered conflict falls back to `ours` — the reader's own unsaved work
— because this text is what goes into the editor buffer while they are
still deciding, and the one thing that must never happen is their typing
disappearing from the screen while they think.

<storycode:copy id="f-39496bbc1d89-mergedText">
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
</storycode:copy>

### Assemble `apps/web/src/lib/merge.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/merge.ts"><storycode:paste select="#f-39496bbc1d89-splitLines"/><storycode:paste select="#f-39496bbc1d89-MAX_DP_CELLS"/><storycode:paste select="#f-39496bbc1d89-matchedLines"/><storycode:paste select="#f-39496bbc1d89-MergeSide"/><storycode:paste select="#f-39496bbc1d89-MergeRegion"/><storycode:paste select="#f-39496bbc1d89-conflictCount"/><storycode:paste select="#f-39496bbc1d89-slice"/><storycode:paste select="#f-39496bbc1d89-alignment"/><storycode:paste select="#f-39496bbc1d89-mergeThreeWay"/><storycode:paste select="#f-39496bbc1d89-mergeTwoWay"/><storycode:paste select="#f-39496bbc1d89-Resolution"/><storycode:paste select="#f-39496bbc1d89-mergedText"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-0827b061679a"></a>
## `apps/web/src/lib/merge.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-0827b061679a-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-0827b061679a-expression_statement">
import { describe, expect, it } from "vitest";
import {
  conflictCount,
  mergeThreeWay,
  mergeTwoWay,
  mergedText,
  splitLines,
  type Resolution,
} from "./merge";

describe("three-way merge — what it can answer by itself", () => {
  const base = "one\ntwo\nthree\n";

  it("takes an edit only we made, without asking", () => {
    const regions = mergeThreeWay(base, "one\nTWO\nthree\n", base);
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe("one\nTWO\nthree\n");
  });

  it("takes an edit only they made, without asking", () => {
    const regions = mergeThreeWay(base, base, "one\ntwo\nTHREE\n");
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe("one\ntwo\nTHREE\n");
  });

  it("combines edits the two sides made in different places", () => {
    // The whole point over a two-way comparison: neither of these is a
    // question for the reader.
    const regions = mergeThreeWay(base, "ONE\ntwo\nthree\n", "one\ntwo\nTHREE\n");
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe("ONE\ntwo\nTHREE\n");
  });

  it("does not ask when both sides made the identical edit", () => {
    const same = "one\nTWO\nthree\n";
    const regions = mergeThreeWay(base, same, same);
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe(same);
  });

  it("takes a deletion only one side made", () => {
    const regions = mergeThreeWay(base, "one\nthree\n", base);
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe("one\nthree\n");
  });

  it("changes nothing when nobody changed anything", () => {
    const regions = mergeThreeWay(base, base, base);
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe(base);
  });
});

</storycode:copy>
<a id="f-0827b061679a-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-0827b061679a-expression_statement-2">
describe("three-way merge — what it has to ask about", () => {
  it("conflicts when both sides changed the same line differently", () => {
    const regions = mergeThreeWay("one\ntwo\nthree\n", "one\nOURS\nthree\n", "one\nTHEIRS\nthree\n");
    expect(conflictCount(regions)).toBe(1);
    const conflict = regions.find((r) => r.kind === "conflict");
    expect(conflict).toMatchObject({ base: "two\n", ours: "OURS\n", theirs: "THEIRS\n" });
  });

  it("honours the answer given for each conflict", () => {
    const regions = mergeThreeWay("one\ntwo\nthree\n", "one\nOURS\nthree\n", "one\nTHEIRS\nthree\n");
    const take = (choice: Resolution) => mergedText(regions, new Map([[0, choice]]));
    expect(take("ours")).toBe("one\nOURS\nthree\n");
    expect(take("theirs")).toBe("one\nTHEIRS\nthree\n");
    expect(take("base")).toBe("one\ntwo\nthree\n");
    expect(take("both")).toBe("one\nOURS\nTHEIRS\nthree\n");
  });

  it("keeps the reader's own text on screen while a conflict is unanswered", () => {
    // This text goes into the editor buffer while they are deciding. Their
    // typing vanishing mid-thought is the one unacceptable outcome.
    const regions = mergeThreeWay("one\ntwo\n", "one\nOURS\n", "one\nTHEIRS\n");
    expect(mergedText(regions)).toBe("one\nOURS\n");
  });

  it("groups a multi-line rewrite into one decision, not one per line", () => {
    const regions = mergeThreeWay("a\nb\nc\nd\n", "a\nX\nY\nd\n", "a\nP\nd\n");
    expect(conflictCount(regions)).toBe(1);
  });
});

</storycode:copy>
<a id="f-0827b061679a-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-0827b061679a-expression_statement-3">
describe("three-way merge — the laws that must always hold", () => {
  // These are diff3's own identities. They are worth asserting over a spread
  // of awkward shapes (empty files, no trailing newline, whole-file deletion)
  // because every one of them is a place an off-by-one in the walk shows up
  // as silently dropped text rather than as a crash.
  const shapes: [string, string, string][] = [
    ["one\ntwo\nthree\n", "one\nTWO\nthree\n", "one\ntwo\nTHREE\n"],
    ["", "added\n", "other\n"],
    ["a\nb\n", "", "a\nb\nc\n"],
    ["a\nb\nc\n", "a\nb\nc\nd\n", "z\na\nb\nc\n"],
    ["x\n", "y\n", "z\n"],
    ["no trailing newline", "no trailing newline!", "no trailing newline?"],
    ["a\nb\nc\nd\ne\n", "a\ne\n", "a\nb\nX\nd\ne\n"],
  ];

  it.each(shapes)("keeps OUR file when they changed nothing (%j)", (base, ours) => {
    expect(mergedText(mergeThreeWay(base, ours, base))).toBe(ours);
  });

  it.each(shapes)("keeps THEIR file when we changed nothing (%j)", (base, _ours, theirs) => {
    expect(mergedText(mergeThreeWay(base, base, theirs))).toBe(theirs);
  });

  it.each(shapes)("asks nothing when both sides agree (%j)", (base, ours) => {
    const regions = mergeThreeWay(base, ours, ours);
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe(ours);
  });

  it.each(shapes)("never invents a line (%j)", (base, ours, theirs) => {
    // Every line of the result has to have come from somewhere.
    const known = new Set([...splitLines(base), ...splitLines(ours), ...splitLines(theirs)]);
    for (const line of splitLines(mergedText(mergeThreeWay(base, ours, theirs)))) {
      expect(known).toContain(line);
    }
  });
});

</storycode:copy>
<a id="f-0827b061679a-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-0827b061679a-expression_statement-4">
describe("two-way merge — honest about having no ancestor", () => {
  it("marks a differing run as a conflict rather than guessing", () => {
    // With no base there is no way to tell "they added it" from "we deleted
    // it", and a tool that guessed would throw away somebody's work.
    const regions = mergeTwoWay("one\ntwo\n", "one\ntwo\nthree\n");
    expect(conflictCount(regions)).toBe(1);
    expect(mergedText(regions, new Map([[0, "theirs"]]))).toBe("one\ntwo\nthree\n");
  });

  it("asks nothing about two identical files", () => {
    expect(conflictCount(mergeTwoWay("same\n", "same\n"))).toBe(0);
  });

  it("carries a base of empty string, so the UI can hide that button", () => {
    const conflict = mergeTwoWay("a\n", "b\n").find((r) => r.kind === "conflict");
    expect(conflict).toMatchObject({ base: "" });
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/merge.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/merge.test.ts"><storycode:paste select="#f-0827b061679a-expression_statement"/><storycode:paste select="#f-0827b061679a-expression_statement-2"/><storycode:paste select="#f-0827b061679a-expression_statement-3"/><storycode:paste select="#f-0827b061679a-expression_statement-4"/></storycode:file>


</storycode:doc>
