import { describe, expect, it } from "vitest";
import {
  activate,
  closeTab,
  collapsePane,
  expandPane,
  focus,
  focused,
  freeform,
  fromRegions,
  matches,
  open,
  paneFor,
  panes,
  regionOf,
  resize,
  split,
  tab,
  type Layout,
} from "./layout";

const doc = (path: string) => tab("document", path);

function openAll(layout: Layout, ...paths: string[]): Layout {
  return paths.reduce((current, path) => open(current, doc(path)), layout);
}

describe("freeform: one pane, tabs, splits", () => {
  it("starts with a single empty pane that has the focus", () => {
    const layout = freeform();
    expect(panes(layout.root)).toHaveLength(1);
    expect(focused(layout)?.tabs).toEqual([]);
  });

  it("opens files as tabs in the focused pane, newest in front", () => {
    const layout = openAll(freeform(), "a.md", "b.md");
    const pane = focused(layout)!;
    expect(pane.tabs.map((t) => t.target)).toEqual(["a.md", "b.md"]);
    expect(pane.tabs[pane.active].target).toBe("b.md");
  });

  it("focuses an already-open file rather than opening it twice", () => {
    // Two tabs of one file is a state with no honest answer to "which one
    // does this edit belong to".
    let layout = openAll(freeform(), "a.md", "b.md");
    layout = open(layout, doc("a.md"));
    const pane = focused(layout)!;
    expect(pane.tabs).toHaveLength(2);
    expect(pane.tabs[pane.active].target).toBe("a.md");
  });

  it("finds the file in another pane instead of duplicating it there", () => {
    let layout = openAll(freeform(), "a.md");
    const first = focused(layout)!.id;
    layout = split(layout, first, "row");
    layout = open(layout, doc("b.md"));
    // `a.md` lives in the other pane: opening it moves the focus there.
    layout = open(layout, doc("a.md"));
    expect(layout.focus).toBe(first);
    expect(panes(layout.root).flatMap((p) => p.tabs).map((t) => t.target)).toEqual([
      "a.md",
      "b.md",
    ]);
  });

  it("splits into two panes and focuses the new one", () => {
    const layout = split(openAll(freeform(), "a.md"), freeform().focus, "row");
    // Splitting an unknown pane is a no-op rather than a crash.
    expect(panes(layout.root)).toHaveLength(1);

    const start = openAll(freeform(), "a.md");
    const after = split(start, start.focus, "column");
    expect(panes(after.root)).toHaveLength(2);
    expect(after.root.type).toBe("split");
    expect(after.focus).not.toBe(start.focus);
    expect(focused(after)?.tabs).toEqual([]);
  });

  it("keeps sizes summing to one, whatever it is handed", () => {
    const start = openAll(freeform(), "a.md");
    const after = split(start, start.focus, "row");
    const splitId = after.root.type === "split" ? after.root.id : "";
    const sized = resize(after, splitId, [3, 1]);
    const sizes = sized.root.type === "split" ? sized.root.sizes : [];
    expect(sizes[0]).toBeCloseTo(0.75);
    expect(sizes.reduce((a, b) => a + b, 0)).toBeCloseTo(1);
  });
});

describe("closing", () => {
  it("keeps the selection where you were looking", () => {
    // Closing a tab BEFORE the active one must not drag the selection along.
    let layout = openAll(freeform(), "a.md", "b.md", "c.md");
    const pane = focused(layout)!;
    layout = activate(layout, pane.id, 2);
    layout = closeTab(layout, pane.id, pane.tabs[0].id);
    const after = focused(layout)!;
    expect(after.tabs.map((t) => t.target)).toEqual(["b.md", "c.md"]);
    expect(after.tabs[after.active].target).toBe("c.md");
  });

  it("removes an emptied pane and gives the focus to what is left", () => {
    let layout = openAll(freeform(), "a.md");
    const first = focused(layout)!.id;
    layout = split(layout, first, "row");
    layout = open(layout, doc("b.md"));
    const second = layout.focus;

    layout = closeTab(layout, second, focused(layout)!.tabs[0].id);
    expect(panes(layout.root)).toHaveLength(1);
    // A split with one child is not a split.
    expect(layout.root.type).toBe("pane");
    expect(layout.focus).toBe(first);
  });

  it("keeps the last pane even when it is empty", () => {
    // There has to be somewhere for the next file to go.
    let layout = openAll(freeform(), "a.md");
    const pane = focused(layout)!;
    layout = closeTab(layout, pane.id, pane.tabs[0].id);
    expect(panes(layout.root)).toHaveLength(1);
    expect(focused(layout)?.tabs).toEqual([]);
  });

  it("ignores a close for a pane that is gone", () => {
    const layout = openAll(freeform(), "a.md");
    expect(closeTab(layout, "pane-nope", "tab-nope")).toBe(layout);
  });
});

describe("globs", () => {
  it("matches within a segment and across segments", () => {
    expect(matches("apps/web/**", "apps/web/src/main.tsx")).toBe(true);
    expect(matches("apps/web/**", "apps/desktop/src-tauri/main.rs")).toBe(false);
    expect(matches("crates/hick-*/**", "crates/hick-dap/src/session.rs")).toBe(true);
    expect(matches("crates/hick-*/**", "crates/hickory-cli/src/lib.rs")).toBe(false);
    expect(matches("*.md", "notes.md")).toBe(true);
    expect(matches("*.md", "docs/notes.md")).toBe(false);
    expect(matches("**/*.md", "docs/deep/notes.md")).toBe(true);
  });

  it("treats a dot as a dot, not as any character", () => {
    expect(matches("a.md", "axhick")).toBe(false);
  });

  it("matches one character with ?", () => {
    expect(matches("v?.md", "v2.md")).toBe(true);
    expect(matches("v?.md", "v10.md")).toBe(false);
  });
});

describe("declared layouts", () => {
  const REGIONS = [
    { name: "ui", match: ["apps/web/**"] },
    { name: "application", match: ["crates/hickory-*/**"] },
    { name: "domain", match: ["crates/hick-*/**", "*.md"] },
  ];

  it("gives every region a pane, in the order declared", () => {
    const layout = fromRegions(REGIONS);
    expect(panes(layout.root).map((p) => p.region)).toEqual(["ui", "application", "domain"]);
    expect(layout.focus).toBe(panes(layout.root)[0].id);
  });

  it("routes a file to the pane whose region claims it", () => {
    const layout = fromRegions(REGIONS);
    const target = paneFor(layout, REGIONS, "crates/hick-dap/src/session.rs");
    expect(paneById(layout, target)?.region).toBe("domain");
  });

  it("resolves overlaps in the order the document wrote them", () => {
    // First match wins: the order is something a person chose and can see,
    // where "most specific" is a rule they would have to work out.
    const overlapping = [
      { name: "specs", match: ["docs/**"] },
      { name: "everything", match: ["**"] },
    ];
    expect(regionOf(overlapping, "docs/spec.md")).toBe("specs");
    expect(regionOf(overlapping, "src/main.rs")).toBe("everything");
  });

  it("opens an unclaimed file where the focus is, rather than refusing it", () => {
    // Punishing a person for an incomplete declaration is the wrong trade.
    let layout = fromRegions(REGIONS);
    layout = focus(layout, panes(layout.root)[1].id);
    expect(paneFor(layout, REGIONS, "LICENSE.txt")).toBe(layout.focus);
  });

  it("falls back to freeform when nothing is declared", () => {
    const layout = fromRegions([]);
    expect(panes(layout.root)).toHaveLength(1);
    expect(panes(layout.root)[0].region).toBeUndefined();
  });

  it("keeps declared panes usable as ordinary panes", () => {
    // A region pane still splits and takes tabs: the declaration seeds the
    // arrangement, it does not freeze it.
    let layout = fromRegions(REGIONS);
    layout = open(layout, doc("apps/web/src/main.tsx"), paneFor(layout, REGIONS, "apps/web/src/main.tsx"));
    const ui = panes(layout.root).find((p) => p.region === "ui")!;
    expect(ui.tabs).toHaveLength(1);
    layout = split(layout, ui.id, "column");
    expect(panes(layout.root)).toHaveLength(4);
  });
});

// Local helper: the module exports `paneById`, but importing it above would
// shadow the name used inside the tests for clarity.
function paneById(layout: Layout, id: string) {
  return panes(layout.root).find((pane) => pane.id === id) ?? null;
}

describe("collapse: a pane folds to a strip and unfolds intact", () => {
  it("round-trips: collapse then expand restores tabs, active, and sizes", () => {
    let layout = openAll(freeform(), "a.md", "b.md");
    const first = layout.focus;
    layout = split(layout, first, "row");
    layout = open(layout, doc("c.md"));
    const before = paneById(layout, first)!;
    const sizes = layout.root.type === "split" ? layout.root.sizes : [];

    const collapsed = collapsePane(layout, first);
    expect(paneById(collapsed, first)?.collapsed).toBe(true);
    // Collapsing is presentation, not surgery: the tabs never moved, and the
    // split fraction is remembered for the way back.
    expect(paneById(collapsed, first)?.tabs).toEqual(before.tabs);
    expect(collapsed.root.type === "split" ? collapsed.root.sizes : []).toEqual(sizes);

    const expanded = expandPane(collapsed, first);
    expect(paneById(expanded, first)?.collapsed).toBe(false);
    expect(paneById(expanded, first)?.tabs).toEqual(before.tabs);
    expect(paneById(expanded, first)?.active).toBe(before.active);
    expect(expanded.focus).toBe(first);
  });

  it("moves the focus off a collapsing pane, so keys aim at something visible", () => {
    let layout = openAll(freeform(), "a.md");
    const first = layout.focus;
    layout = split(layout, first, "row");
    layout = focus(layout, first);
    const collapsed = collapsePane(layout, first);
    expect(collapsed.focus).not.toBe(first);
    expect(paneById(collapsed, collapsed.focus)?.collapsed).toBeFalsy();
  });

  it("refuses to collapse the last visible pane", () => {
    // A shell that is all strips has nothing left to show and no way back.
    const layout = openAll(freeform(), "a.md");
    expect(collapsePane(layout, layout.focus)).toBe(layout);

    let two = openAll(freeform(), "a.md");
    const first = two.focus;
    two = split(two, first, "row");
    const second = two.focus;
    two = collapsePane(two, first);
    expect(collapsePane(two, second)).toBe(two);
  });

  it("expanding a pane that is not collapsed is a no-op", () => {
    const layout = openAll(freeform(), "a.md");
    expect(expandPane(layout, layout.focus)).toBe(layout);
    expect(expandPane(layout, "no-such-pane")).toBe(layout);
  });
});
