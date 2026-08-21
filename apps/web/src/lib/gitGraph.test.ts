import { describe, expect, it } from "vitest";
import { graphWidth, laneColor, layout } from "./gitGraph";

/** `git log` order: newest first, each naming its parents. */
const commits = (spec: Record<string, string[]>) =>
  Object.entries(spec).map(([sha, parents]) => ({ sha, parents }));

const lanes = (rows: ReturnType<typeof layout>) =>
  Object.fromEntries(rows.map((r) => [r.sha, r.lane]));

describe("a straight history", () => {
  it("keeps every commit in one column", () => {
    const rows = layout(commits({ c: ["b"], b: ["a"], a: [] }));
    expect(lanes(rows)).toEqual({ c: 0, b: 0, a: 0 });
    expect(graphWidth(rows)).toBe(1);
  });

  it("draws a line from each commit to the next", () => {
    const rows = layout(commits({ b: ["a"], a: [] }));
    expect(rows[0].through).toContainEqual({ from: 0, to: 0 });
  });

  it("ends the lane at a root commit", () => {
    const rows = layout(commits({ a: [] }));
    expect(rows[0].through).toEqual([]);
    expect(rows[0].width).toBe(1);
  });
});

describe("a branch and a merge", () => {
  // m ── merge of "feature" into master
  // |\
  // | f   (the branch)
  // b |   (master moved on)
  // |/
  // a
  const history = commits({ m: ["b", "f"], f: ["a"], b: ["a"], a: [] });

  it("puts the merge on the mainline and the branch beside it", () => {
    const rows = layout(history);
    expect(lanes(rows).m).toBe(0);
    // The second parent got a lane of its own.
    expect(lanes(rows).f).toBeGreaterThan(0);
  });

  it("leaves the merge sideways for its second parent", () => {
    const rows = layout(history);
    const merge = rows[0];
    expect(merge.through.some((t) => t.from === 0 && t.to > 0)).toBe(true);
  });

  it("brings the branch back into the mainline at their shared parent", () => {
    // Both `f` and `b` name `a`; the second one to reach it must bend in
    // rather than claiming a lane that then dangles.
    const rows = layout(history);
    const bendsIn = rows.some((row) => row.through.some((t) => t.from !== t.to && t.to === 0));
    expect(bendsIn).toBe(true);
    expect(lanes(rows).a).toBe(0);
  });

  it("is two columns wide, not three", () => {
    expect(graphWidth(layout(history))).toBe(2);
  });
});

describe("reusing lanes", () => {
  it("does not drift rightwards forever", () => {
    // Two branches that both end, then more history. Without reuse the graph
    // would keep widening on any repository with merge history.
    const rows = layout(
      commits({
        e: ["d"],
        d: ["c", "x"],
        x: ["c"],
        c: ["b"],
        b: ["a"],
        a: [],
      }),
    );
    expect(graphWidth(rows)).toBe(2);
    // The mainline never leaves column 0.
    expect(lanes(rows).e).toBe(0);
    expect(lanes(rows).a).toBe(0);
  });

  it("gives a tip with nothing waiting for it the lowest free lane", () => {
    // Two unrelated tips: the second takes lane 1, not lane 7.
    const rows = layout(commits({ p: [], q: [] }));
    expect(lanes(rows)).toEqual({ p: 0, q: 0 });
  });
});

describe("what the graph is drawn with", () => {
  it("cycles a small palette, because crossing lines need telling apart", () => {
    expect(laneColor(0)).toBe(0);
    expect(laneColor(6)).toBe(0);
    expect(laneColor(7)).toBe(1);
  });

  it("reports a width for an empty history rather than zero", () => {
    expect(graphWidth([])).toBe(1);
  });
});

describe("an octopus merge", () => {
  it("leaves sideways once per extra parent", () => {
    const rows = layout(commits({ o: ["a", "b", "c"], a: [], b: [], c: [] }));
    const sideways = rows[0].through.filter((t) => t.from === 0 && t.to !== 0);
    expect(sideways).toHaveLength(2);
  });
});
