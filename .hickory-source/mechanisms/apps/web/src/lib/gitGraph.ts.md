<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/gitGraph.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-574b24466b05"></a>
## `apps/web/src/lib/gitGraph.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-574b24466b05-GraphCommit"></a>
### `GraphCommit`

Laying commits out in lanes, the way a commit graph is read.
The picture answers one question — "what came from where" — and it answers
it with two things: which column a commit sits in, and which lines run
between columns. Everything here computes those; nothing here draws.
The algorithm is the standard one, and its shape follows from the input.
`git log` gives commits newest first, and each names its PARENTS (older).
So walking top to bottom, a commit is always reached before the commits it
came from — which means a lane can be reserved for a parent at the moment
its child is placed, and every commit knows its lane before it is drawn.
Two decisions worth stating, because both are the difference between a
graph that reads and one that is a plate of spaghetti:
* **A commit takes its first child's lane.** Following the first parent
keeps mainline history in one column all the way down, which is the
column people scan.
* **A lane is released the moment nothing is waiting in it**, and reused
left-most-first. A graph that never reused lanes would drift rightwards
forever on a repository with any merge history.
A commit, as the graph needs it.

<storycode:copy id="f-574b24466b05-GraphCommit">
// Laying commits out in lanes, the way a commit graph is read.
//
// The picture answers one question — "what came from where" — and it answers
// it with two things: which column a commit sits in, and which lines run
// between columns. Everything here computes those; nothing here draws.
//
// The algorithm is the standard one, and its shape follows from the input.
// `git log` gives commits newest first, and each names its PARENTS (older).
// So walking top to bottom, a commit is always reached before the commits it
// came from — which means a lane can be reserved for a parent at the moment
// its child is placed, and every commit knows its lane before it is drawn.
//
// Two decisions worth stating, because both are the difference between a
// graph that reads and one that is a plate of spaghetti:
//
//  * **A commit takes its first child's lane.** Following the first parent
//    keeps mainline history in one column all the way down, which is the
//    column people scan.
//  * **A lane is released the moment nothing is waiting in it**, and reused
//    left-most-first. A graph that never reused lanes would drift rightwards
//    forever on a repository with any merge history.

/** A commit, as the graph needs it. */
export interface GraphCommit {
  sha: string;
  parents: string[];
}

</storycode:copy>
<a id="f-574b24466b05-GraphRow"></a>
### `GraphRow`

Where one commit sits, and what runs past it.

<storycode:copy id="f-574b24466b05-GraphRow">
/** Where one commit sits, and what runs past it. */
export interface GraphRow {
  sha: string;
  /** Zero-based column. */
  lane: number;
  /** Lines passing through this row: `from` lane at the top, `to` lane at the
   * bottom. A line from a lane to itself is a straight edge; anything else
   * bends. */
  through: { from: number; to: number }[];
  /** How wide the graph is at this row, for the SVG's own width. */
  width: number;
}

</storycode:copy>
<a id="f-574b24466b05-claim"></a>
### `claim`

The lowest free lane, so the graph does not drift rightwards forever.

<storycode:copy id="f-574b24466b05-claim">
/** The lowest free lane, so the graph does not drift rightwards forever. */
function claim(lanes: (string | null)[], sha: string): number {
  const free = lanes.indexOf(null);
  if (free >= 0) {
    lanes[free] = sha;
    return free;
  }
  lanes.push(sha);
  return lanes.length - 1;
}

</storycode:copy>
<a id="f-574b24466b05-layout"></a>
### `layout`

Lay out `commits` — newest first, as `git log` gives them.
Returns one row per commit, in the same order.

<storycode:copy id="f-574b24466b05-layout">
/**
 * Lay out `commits` — newest first, as `git log` gives them.
 *
 * Returns one row per commit, in the same order.
 */
export function layout(commits: readonly GraphCommit[]): GraphRow[] {
  // lanes[i] is the sha that lane is currently waiting for, or null.
  let lanes: (string | null)[] = [];
  const rows: GraphRow[] = [];

  for (const commit of commits) {
    // What enters this row from above.
    const before = [...lanes];

    // The lane reserved for this commit by whichever child reached it first,
    // preferring the LEFTMOST — several children converging is a merge, and
    // the merge belongs in the column its mainline child is in.
    let lane = before.indexOf(commit.sha);
    if (lane < 0) {
      // Nothing is waiting for it: a branch tip, or the first commit seen.
      lane = claim(lanes, commit.sha);
    }
    // Every lane waiting for this commit is satisfied by it.
    for (let i = 0; i < lanes.length; i++) {
      if (lanes[i] === commit.sha) lanes[i] = null;
    }

    // Place the parents. The first takes this commit's own lane, which is
    // what keeps mainline history in one column all the way down.
    const [first, ...rest] = commit.parents;
    if (first !== undefined) {
      const waiting = lanes.indexOf(first);
      if (waiting >= 0 && waiting < lane) {
        // A lane further left is already waiting for this parent: leave it
        // there and let this lane close. Claiming a second lane for one
        // commit is how a graph grows a column that goes nowhere — and the
        // line into it is drawn below, out of this node.
      } else {
        if (waiting >= 0) lanes[waiting] = null;
        lanes[lane] = first;
      }
    } else {
      lanes[lane] = null;
    }
    for (const parent of rest) {
      // A second parent is the merge's other side. It takes a lane already
      // waiting for it, or a new one.
      if (!lanes.includes(parent)) claim(lanes, parent);
    }

    // The lines. Every lane that entered this row goes somewhere: to wherever
    // its sha is waited for now, or into this commit's node.
    const through: { from: number; to: number }[] = [];
    for (let i = 0; i < before.length; i++) {
      const sha = before[i];
      if (sha === null) continue;
      if (sha === commit.sha) {
        // It was waiting for THIS commit: the line ends at the node.
        if (i !== lane) through.push({ from: i, to: lane });
        continue;
      }
      const to = lanes.indexOf(sha);
      if (to >= 0) through.push({ from: i, to });
    }
    // ...and one line out of the node per parent.
    for (const parent of commit.parents) {
      const to = lanes.indexOf(parent);
      if (to >= 0) through.push({ from: lane, to });
    }

    // Trim trailing empties so `width` is the real extent rather than the
    // high-water mark.
    while (lanes.length > 0 && lanes[lanes.length - 1] === null) lanes.pop();
    lanes = [...lanes];
    rows.push({
      sha: commit.sha,
      lane,
      through,
      width: Math.max(lanes.length, lane + 1, ...through.map((t) => Math.max(t.from, t.to) + 1)),
    });
  }
  return rows;
}

</storycode:copy>
<a id="f-574b24466b05-laneColor"></a>
### `laneColor`

The palette index for a lane. Colour is the only thing distinguishing two
lines that cross, and six is as many as stay told apart.

<storycode:copy id="f-574b24466b05-laneColor">
/** The palette index for a lane. Colour is the only thing distinguishing two
 * lines that cross, and six is as many as stay told apart. */
export function laneColor(lane: number): number {
  return lane % 6;
}

</storycode:copy>
<a id="f-574b24466b05-graphWidth"></a>
### `graphWidth`

How wide the graph column has to be for a layout.

<storycode:copy id="f-574b24466b05-graphWidth">
/** How wide the graph column has to be for a layout. */
export function graphWidth(rows: readonly GraphRow[]): number {
  return rows.reduce((widest, row) => Math.max(widest, row.width), 1);
}
</storycode:copy>

### Assemble `apps/web/src/lib/gitGraph.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/gitGraph.ts"><storycode:paste select="#f-574b24466b05-GraphCommit"/><storycode:paste select="#f-574b24466b05-GraphRow"/><storycode:paste select="#f-574b24466b05-claim"/><storycode:paste select="#f-574b24466b05-layout"/><storycode:paste select="#f-574b24466b05-laneColor"/><storycode:paste select="#f-574b24466b05-graphWidth"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-fe23bec10eb0"></a>
## `apps/web/src/lib/gitGraph.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-fe23bec10eb0-commits"></a>
### `commits`

<storycode:copy id="f-fe23bec10eb0-commits">
import { describe, expect, it } from "vitest";
import { graphWidth, laneColor, layout } from "./gitGraph";

/** `git log` order: newest first, each naming its parents. */
const commits = (spec: Record<string, string[]>) =>
  Object.entries(spec).map(([sha, parents]) => ({ sha, parents }));

</storycode:copy>
<a id="f-fe23bec10eb0-lanes"></a>
### `lanes`

<storycode:copy id="f-fe23bec10eb0-lanes">
const lanes = (rows: ReturnType<typeof layout>) =>
  Object.fromEntries(rows.map((r) => [r.sha, r.lane]));

</storycode:copy>
<a id="f-fe23bec10eb0-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-fe23bec10eb0-expression_statement">
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

</storycode:copy>
<a id="f-fe23bec10eb0-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-fe23bec10eb0-expression_statement-2">
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

</storycode:copy>
<a id="f-fe23bec10eb0-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-fe23bec10eb0-expression_statement-3">
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

</storycode:copy>
<a id="f-fe23bec10eb0-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-fe23bec10eb0-expression_statement-4">
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

</storycode:copy>
<a id="f-fe23bec10eb0-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-fe23bec10eb0-expression_statement-5">
describe("an octopus merge", () => {
  it("leaves sideways once per extra parent", () => {
    const rows = layout(commits({ o: ["a", "b", "c"], a: [], b: [], c: [] }));
    const sideways = rows[0].through.filter((t) => t.from === 0 && t.to !== 0);
    expect(sideways).toHaveLength(2);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/gitGraph.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/gitGraph.test.ts"><storycode:paste select="#f-fe23bec10eb0-commits"/><storycode:paste select="#f-fe23bec10eb0-lanes"/><storycode:paste select="#f-fe23bec10eb0-expression_statement"/><storycode:paste select="#f-fe23bec10eb0-expression_statement-2"/><storycode:paste select="#f-fe23bec10eb0-expression_statement-3"/><storycode:paste select="#f-fe23bec10eb0-expression_statement-4"/><storycode:paste select="#f-fe23bec10eb0-expression_statement-5"/></storycode:file>


</storycode:doc>
