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

/** The palette index for a lane. Colour is the only thing distinguishing two
 * lines that cross, and six is as many as stay told apart. */
export function laneColor(lane: number): number {
  return lane % 6;
}

/** How wide the graph column has to be for a layout. */
export function graphWidth(rows: readonly GraphRow[]): number {
  return rows.reduce((widest, row) => Math.max(widest, row.width), 1);
}
