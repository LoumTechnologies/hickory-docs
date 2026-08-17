import { describe, expect, it } from "vitest";
import { anchorEdges, atLeast, bandAround, braceDepth, braceLinkPath, braceLinkToEdgePath, braceNub, bracePath, clampBand, linkArm, ribbonPath, ribbonPathVia, ribbonStubPath, ribbonTerminalPath, terminalEdge, thicknessFor } from "./ribbonGeometry";

/** Parse a single-cubic path `M x y C c1x c1y c2x c2y x y` into its points. */
function cubic(d: string) {
  const m =
    /^M (-?[\d.]+) (-?[\d.]+) C (-?[\d.]+) (-?[\d.]+) (-?[\d.]+) (-?[\d.]+) (-?[\d.]+) (-?[\d.]+)$/.exec(
      d,
    );
  expect(m, `not one cubic: ${d}`).not.toBeNull();
  const [, ...n] = m!.map(Number);
  return {
    start: { x: n[0], y: n[1] },
    c1: { x: n[2], y: n[3] },
    c2: { x: n[4], y: n[5] },
    end: { x: n[6], y: n[7] },
  };
}

describe("clampBand", () => {
  it("passes a fully visible band through untouched", () => {
    expect(clampBand(120, 140, 100, 500)).toEqual({
      yTop: 120,
      yBot: 140,
      clamped: false,
      clampedTop: false,
      clampedBottom: false,
    });
  });

  it("clamps a band that pokes above the pane, and says WHICH end", () => {
    expect(clampBand(80, 140, 100, 500)).toEqual({
      yTop: 100,
      yBot: 140,
      clamped: true,
      clampedTop: true,
      clampedBottom: false,
    });
  });

  it("clamps a band that pokes below the pane, and says WHICH end", () => {
    expect(clampBand(480, 560, 100, 500)).toEqual({
      yTop: 480,
      yBot: 500,
      clamped: true,
      clampedTop: false,
      clampedBottom: true,
    });
  });

  it("reports both ends when the band overflows the pane on both sides", () => {
    expect(clampBand(50, 600, 100, 500)).toEqual({
      yTop: 100,
      yBot: 500,
      clamped: true,
      clampedTop: true,
      clampedBottom: true,
    });
  });

  it("collapses a band fully above to a sliver at the top edge, open at the top", () => {
    expect(clampBand(0, 50, 100, 500)).toEqual({
      yTop: 100,
      yBot: 102,
      clamped: true,
      clampedTop: true,
      clampedBottom: false,
    });
  });

  it("collapses a band fully below to a sliver at the bottom edge, open at the bottom", () => {
    expect(clampBand(600, 700, 100, 500)).toEqual({
      yTop: 498,
      yBot: 500,
      clamped: true,
      clampedTop: false,
      clampedBottom: true,
    });
  });
});

describe("thicknessFor", () => {
  it("is proportional to the byte share", () => {
    expect(thicknessFor(50, 100, 26)).toBeCloseTo(13);
  });

  it("caps at maxPx and floors at minPx", () => {
    expect(thicknessFor(100, 100, 26)).toBe(26);
    expect(thicknessFor(1, 1_000_000, 26, 3)).toBe(3);
  });

  it("degrades safely on zero totals", () => {
    expect(thicknessFor(10, 0)).toBe(3);
    expect(thicknessFor(0, 10)).toBe(3);
  });
});

describe("bandAround", () => {
  it("centers the thickness on the band midpoint", () => {
    expect(bandAround(100, 120, 10)).toEqual({ yTop: 105, yBot: 115 });
  });
});

describe("ribbonPath", () => {
  it("builds a closed two-bezier ribbon between the two bands", () => {
    const d = ribbonPath(10, 100, 110, 200, 300, 340);
    expect(d.startsWith("M 10 100")).toBe(true);
    expect(d.endsWith("Z")).toBe(true);
    // Horizontal tangents: control points at the midpoint x.
    expect(d).toContain("C 105 100 105 300 200 300");
    expect(d).toContain("L 200 340");
    expect(d).toContain("C 105 340 105 110 10 110");
  });

  it("rounds coordinates to a tenth to keep paths compact", () => {
    const d = ribbonPath(0.123, 1.056, 2.849, 3.301, 4.05, 5.999);
    expect(d).toContain("M 0.1 1.1");
    expect(d).toContain("6");
    expect(d).not.toMatch(/\d\.\d\d/);
  });
});

describe("ribbons routed through the file-tree waypoint", () => {
  it("threads source → node → output as one closed path", () => {
    const d = ribbonPathVia(100, 10, 30, 200, 260, 40, 50, 400, 80, 120);
    // Starts at the source edge, reaches the node's left and right edges, ends
    // closed so the ribbon fills rather than strokes.
    expect(d.startsWith("M 100 10")).toBe(true);
    expect(d).toContain("200 40");
    expect(d).toContain("L 260 40");
    expect(d).toContain("400 80");
    expect(d.endsWith("Z")).toBe(true);
  });

  it("stubs stop at the node and never reach the output edge", () => {
    const d = ribbonStubPath(100, 10, 30, 200, 40, 50);
    expect(d.startsWith("M 100 10")).toBe(true);
    expect(d).toContain("200 40");
    expect(d).toContain("L 200 50");
    expect(d.endsWith("Z")).toBe(true);
  });

  it("gives a zero-height band a grabbable minimum without moving its center", () => {
    const b = atLeast(100, 100, 4);
    expect(b.yBot - b.yTop).toBe(4);
    expect((b.yTop + b.yBot) / 2).toBe(100);
    // A band that is already tall enough is returned untouched.
    expect(atLeast(10, 40, 4)).toEqual({ yTop: 10, yBot: 40 });
  });
});

describe("ribbons terminating on chrome (tabs and divider ports)", () => {
  // Which edge takes the band: a terminal is a tab or a button in a thin
  // vertical divider, so attachment is always to a horizontal edge — the
  // side of a 4px-wide button gives a ribbon nothing to land on.
  it("attaches a band above the terminal's centre to its top edge", () => {
    expect(terminalEdge(100, 140, 300, 320)).toBe("top");
  });

  it("attaches a band below the terminal's centre to its bottom edge", () => {
    expect(terminalEdge(500, 540, 300, 320)).toBe("bottom");
  });

  it("splits ties by the midpoints, not the extremes", () => {
    // Band midpoint 310 equals the terminal's centre: top wins the tie.
    expect(terminalEdge(300, 320, 300, 320)).toBe("top");
    expect(terminalEdge(311, 311, 300, 320)).toBe("bottom");
  });

  it("lands a source LEFT of the terminal flat on the edge, near corner first", () => {
    const d = ribbonTerminalPath(100, 10, 30, 400, 460, 200);
    // Starts at the source band's top boundary…
    expect(d.startsWith("M 100 10")).toBe(true);
    // …elbows to the NEAR corner (left, 400) with a vertical arrival…
    expect(d).toContain("400 200");
    // …runs the full edge to the far corner…
    expect(d).toContain("L 460 200");
    // …and returns to the band's bottom boundary, closed for filling.
    expect(d).toContain("100 30");
    expect(d.endsWith("Z")).toBe(true);
  });

  it("mirrors when the source is RIGHT of the terminal", () => {
    const d = ribbonTerminalPath(800, 10, 30, 400, 460, 200);
    // The near corner is now the right one, so the edge runs right→left.
    expect(d).toContain("L 400 200");
    expect(d.endsWith("Z")).toBe(true);
  });

  it("has horizontal tangents at the source and vertical at the terminal", () => {
    const d = ribbonTerminalPath(100, 10, 30, 400, 460, 200);
    // First control point shares the source's y (horizontal start), second
    // shares the terminal corner's x (vertical landing).
    expect(d).toContain("C 250 10 400 105 400 200");
  });

  it("works upward too: a terminal above the band attaches from below", () => {
    const d = ribbonTerminalPath(100, 500, 540, 400, 460, 60);
    expect(d.startsWith("M 100 500")).toBe(true);
    expect(d).toContain("400 60");
    expect(d).toContain("L 460 60");
    expect(d.endsWith("Z")).toBe(true);
  });
});

describe("anchorEdges", () => {
  const doc = { left: 0, right: 400 };
  const out = { left: 500, right: 900 };

  it("anchors the left pane's right rail edge and the right pane's left gutter edge", () => {
    expect(anchorEdges(doc, out)).toEqual({ forward: true, x0: 400, x1: 500 });
  });

  it("swaps edges when the panes swap sides", () => {
    expect(anchorEdges(out, doc)).toEqual({ forward: false, x0: 500, x1: 400 });
  });

  it("decides by midpoints, so overlapping rects mid-drag still pick one pair", () => {
    // a's midpoint (200) is left of b's (450) even though the rects overlap.
    expect(anchorEdges({ left: 0, right: 400 }, { left: 300, right: 600 })).toEqual({
      forward: true,
      x0: 400,
      x1: 300,
    });
  });
});

describe("bracePath", () => {
  it("touches the anchor column at exactly the first and last pixel rows", () => {
    const path = bracePath(100, 40, 140, 1, 6);
    expect(path.startsWith("M 100 40")).toBe(true);
    expect(path.endsWith("100 140")).toBe(true);
  });

  it("bulges toward the channel: right for dir 1, left for dir -1", () => {
    // Spine at x+6, nub at x+12 for a span tall enough for full depth.
    expect(bracePath(100, 0, 100, 1, 6)).toContain("Q 106 50 112 50");
    expect(bracePath(100, 0, 100, -1, 6)).toContain("Q 94 50 88 50");
  });

  it("shrinks its depth on a short span instead of folding over itself", () => {
    // One 16px line: depth clamps to 4, so the nub sits 8 out, not 12.
    expect(braceDepth(0, 16, 6)).toBe(4);
    expect(braceNub(100, 0, 16, 1, 6)).toEqual({ x: 108, y: 8 });
  });

  it("never loses the brace entirely, even on a degenerate span", () => {
    expect(braceDepth(50, 50, 6)).toBe(1);
  });

  // Clamped ends: when the underlying range continues past the viewport edge,
  // the brace must NOT curl a horn inward at that edge — the spine runs
  // straight to the clamp y, visually off the edge.
  describe("open (clamped) ends", () => {
    it("open top: starts flat ON the spine, no curve toward the anchor column", () => {
      const d = bracePath(100, 40, 140, 1, 6, { top: true });
      // Begins at the spine x (106), at the clamp y — never touching x=100.
      expect(d.startsWith("M 106 40 L 106 84")).toBe(true);
      // No horn: the first command after M is a straight L, not a Q.
      expect(d.split(" ")[3]).toBe("L");
      // Bottom horn still closes onto the anchor column.
      expect(d.endsWith("Q 106 140 100 140")).toBe(true);
    });

    it("open bottom: the spine runs straight off the clamp, no bottom horn", () => {
      const d = bracePath(100, 40, 140, 1, 6, { bottom: true });
      // Top horn intact.
      expect(d.startsWith("M 100 40 Q 106 40 106 46")).toBe(true);
      // Ends with a straight line to the clamp y at the spine x — no Q, no
      // return to the anchor column.
      expect(d.endsWith("L 106 140")).toBe(true);
      expect(d).not.toContain("100 140");
    });

    it("both ends open: a straight spine with only the centre nub", () => {
      const d = bracePath(100, 40, 140, 1, 6, { top: true, bottom: true });
      expect(d).toBe(
        "M 106 40 L 106 84 Q 106 90 112 90 Q 106 90 106 96 L 106 140",
      );
    });

    it("keeps the nub centred on the VISIBLE extent, agreeing with braceNub", () => {
      const d = bracePath(100, 40, 140, 1, 6, { top: true });
      const nub = braceNub(100, 40, 140, 1, 6);
      expect(d).toContain(`Q 106 ${nub.y} ${nub.x} ${nub.y}`);
    });

    it("stays degenerate-safe on a one-line clamped sliver", () => {
      // A 2px sliver: depth clamps to 1; the open path must still be valid
      // and stay on its spine.
      const d = bracePath(100, 498, 500, 1, 6, { bottom: true });
      expect(d.endsWith("L 101 500")).toBe(true);
    });

    it("mirrors for dir -1", () => {
      const d = bracePath(100, 40, 140, -1, 6, { top: true });
      expect(d.startsWith("M 94 40 L 94 84")).toBe(true);
    });

    it("closed braces are unchanged by an empty open spec", () => {
      expect(bracePath(100, 40, 140, 1, 6, {})).toBe(bracePath(100, 40, 140, 1, 6));
    });
  });

  // Horn reach: the horn arms extend INWARD across the line-number rail —
  // opposite the bulge direction — so the tips land on the rail's inner
  // (text-side) edge and the brace visibly wraps the included line numbers.
  describe("horns reaching across the line-number rail", () => {
    it("puts the horn tips one rail-width in from the anchor column", () => {
      const d = bracePath(100, 40, 140, 1, 6, {}, 30);
      // Top tip at x - horn, straight across the gutter, then the curl.
      expect(d.startsWith("M 70 40 L 100 40 Q 106 40 106 46")).toBe(true);
      // Bottom mirrors: curl back to the column, then across to the tip.
      expect(d.endsWith("Q 106 140 100 140 L 70 140")).toBe(true);
    });

    it("mirrors for dir -1: the rail is on the brace's other side", () => {
      const d = bracePath(100, 40, 140, -1, 6, {}, 30);
      expect(d.startsWith("M 130 40 L 100 40 Q 94 40 94 46")).toBe(true);
      expect(d.endsWith("Q 94 140 100 140 L 130 140")).toBe(true);
    });

    it("an open end draws no horn at all — nothing reaches into the gutter", () => {
      const d = bracePath(100, 40, 140, 1, 6, { top: true }, 30);
      // Open top: straight down the spine from the clamp, no tip, no L
      // across the gutter at the top.
      expect(d.startsWith("M 106 40 L 106 84")).toBe(true);
      // The closed bottom still gets its full horn.
      expect(d.endsWith("Q 106 140 100 140 L 70 140")).toBe(true);
    });

    it("zero horn keeps the tips on the anchor column, byte-for-byte", () => {
      expect(bracePath(100, 40, 140, 1, 6, {}, 0)).toBe(bracePath(100, 40, 140, 1, 6));
    });

    it("stays valid on a short span: depth shrinks, the horn does not fold", () => {
      // One 16px line, depth clamps to 4; the horn is horizontal so its
      // length never competes with the vertical curl budget.
      const d = bracePath(100, 0, 16, 1, 6, {}, 40);
      expect(d.startsWith("M 60 0 L 100 0 Q 104 0 104 4")).toBe(true);
      expect(d.endsWith("Q 104 16 100 16 L 60 16")).toBe(true);
    });
  });
});

describe("braceNub", () => {
  it("sits two depths out on the midline, on the dir side", () => {
    expect(braceNub(100, 40, 140, 1, 6)).toEqual({ x: 112, y: 90 });
    expect(braceNub(100, 40, 140, -1, 6)).toEqual({ x: 88, y: 90 });
  });
});

describe("linkArm", () => {
  it("is proportional to the endpoint distance", () => {
    expect(linkArm(0, 0, 300, 400)).toBeCloseTo(200); // 0.4 · 500
  });

  it("never shrinks below the minimum, so nearby endpoints do not kink", () => {
    expect(linkArm(100, 100, 104, 103)).toBe(12);
  });
});

describe("braceLinkPath", () => {
  // The maintainer's complaint was a sudden direction change at the brace
  // tip: the link must LEAVE each nub with C1 continuity along the nub's own
  // outward direction — horizontal, away from the text — and curve smoothly
  // the whole way. The tangent at a cubic's endpoint runs through its
  // adjacent control point, so these tests read the control points.
  it("departs each nub horizontally, along that nub's outward direction", () => {
    const arm = Math.round(linkArm(112, 90, 488, 210) * 10) / 10;
    const p = cubic(braceLinkPath(112, 90, 1, 488, 210, -1));
    expect(p.start).toEqual({ x: 112, y: 90 });
    expect(p.end).toEqual({ x: 488, y: 210 });
    // Horizontal tangents: each control shares its endpoint's y…
    expect(p.c1.y).toBe(90);
    expect(p.c2.y).toBe(210);
    // …one arm out along the endpoint's own outward direction.
    expect(p.c1.x).toBeCloseTo(112 + arm, 1);
    expect(p.c2.x).toBeCloseTo(488 - arm, 1);
  });

  it("keeps the departure outward even when the target sits behind the nub", () => {
    // A nub bulging right whose target is LEFT of it: the old midpoint
    // control flipped the tangent back toward the text — a kink at the tip.
    const p = cubic(braceLinkPath(400, 100, 1, 200, 300, 1));
    expect(p.c1.x).toBeGreaterThan(400);
    expect(p.c2.x).toBeGreaterThan(200);
  });

  it("gives near-touching nubs a real arm instead of a corner", () => {
    const p = cubic(braceLinkPath(100, 100, 1, 106, 104, -1));
    expect(p.c1.x - 100).toBeGreaterThanOrEqual(12);
    expect(106 - p.c2.x).toBeGreaterThanOrEqual(12);
  });
});

describe("braceLinkToEdgePath", () => {
  it("is one continuous cubic: horizontal out of the nub, vertical onto the edge", () => {
    const arm = Math.round(linkArm(112, 90, 300, 30) * 10) / 10;
    const p = cubic(braceLinkToEdgePath(112, 90, 1, 300, 30));
    expect(p.start).toEqual({ x: 112, y: 90 });
    expect(p.end).toEqual({ x: 300, y: 30 });
    // Horizontal departure along the nub's outward direction…
    expect(p.c1.y).toBe(90);
    expect(p.c1.x).toBeCloseTo(112 + arm, 1);
    // …and a vertical arrival: the last control shares the terminal's x and
    // approaches the edge from the source's side, so there is no corner.
    expect(p.c2.x).toBe(300);
    expect(p.c2.y).toBeCloseTo(30 + arm, 1);
  });

  it("approaches from above when the terminal edge is below the nub", () => {
    const arm = Math.round(linkArm(112, 90, 300, 500) * 10) / 10;
    const p = cubic(braceLinkToEdgePath(112, 90, -1, 300, 500));
    // Outward is leftward here; the arrival control sits above the edge.
    expect(p.c1.x).toBeCloseTo(112 - arm, 1);
    expect(p.c2.x).toBe(300);
    expect(p.c2.y).toBeCloseTo(500 - arm, 1);
  });
});
