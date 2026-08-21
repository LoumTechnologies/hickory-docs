import { describe, expect, it } from "vitest";

import {
  UI_STATE_VERSION,
  emptyUi,
  normalizeUi,
  withWrap,
  worthStoring,
  wrapFor,
} from "./uiState";
import { WRAP_DEFAULT, WRAP_MAX, WRAP_MIN } from "../editor/wrapColumn";
import { freeform, open as openInLayout, panes, tab, withTree } from "../shell/layout";

const stored = (layout: unknown, wrap: Record<string, unknown> = {}) => ({
  version: UI_STATE_VERSION,
  layout,
  wrap,
});

const onePane = {
  root: {
    type: "pane",
    id: "pane-1",
    tabs: [{ id: "tab-1", kind: "document", target: "notes.hick", docId: "d1" }],
    active: 0,
  },
  focus: "pane-1",
};

describe("reading a stored layout back", () => {
  it("restores tabs, in their panes", () => {
    const ui = normalizeUi(stored(onePane));
    const restored = panes(ui.layout!.root);
    expect(restored).toHaveLength(1);
    expect(restored[0].tabs.map((t) => t.target)).toEqual(["notes.hick"]);
    expect(restored[0].tabs[0].docId).toBe("d1");
  });

  it("mints fresh ids, so the session's counter cannot hand one out twice", () => {
    // Ids come from a counter that restarts at zero every session. A restored
    // `pane-1` plus the next split's `pane-1` is two panes with one id, and
    // every lookup finding whichever came first.
    const ui = normalizeUi(stored(onePane));
    const restored = panes(ui.layout!.root)[0];
    expect(restored.id).not.toBe("pane-1");
    expect(restored.tabs[0].id).not.toBe("tab-1");
    expect(ui.layout!.focus).toBe(restored.id);
  });

  it("restores a split with its proportions", () => {
    const ui = normalizeUi(
      stored({
        root: {
          type: "split",
          id: "s1",
          direction: "row",
          sizes: [0.3, 0.7],
          children: [
            { type: "pane", id: "p1", tabs: [], active: 0 },
            { type: "pane", id: "p2", tabs: [], active: 0 },
          ],
        },
        focus: "p2",
      }),
    );
    const root = ui.layout!.root;
    expect(root.type).toBe("split");
    expect(root.type === "split" && root.sizes).toEqual([0.3, 0.7]);
  });
});

describe("reading a stored layout that is WRONG", () => {
  // Every case here is a window that must still open. A damaged layout is an
  // annoyance; a window that will not start because of one is a disaster.
  it("opens default tabs for a blob that is not an object", () => {
    for (const junk of [null, undefined, 42, "layout", []]) {
      expect(normalizeUi(junk)).toEqual(emptyUi());
    }
  });

  it("opens default tabs for a version this build does not know", () => {
    // The same field names may mean something else there; guessing is how a
    // layout gets silently mangled.
    expect(normalizeUi({ version: 99, layout: onePane })).toEqual(emptyUi());
  });

  it("drops a tab whose kind this build cannot draw", () => {
    const ui = normalizeUi(
      stored({
        root: {
          type: "pane",
          id: "p1",
          active: 0,
          tabs: [
            { id: "t1", kind: "hologram", target: "x" },
            { id: "t2", kind: "file", target: "README.md" },
          ],
        },
        focus: "p1",
      }),
    );
    expect(panes(ui.layout!.root)[0].tabs.map((t) => t.target)).toEqual(["README.md"]);
  });

  it("clamps an active index past the end of a shortened tab list", () => {
    // The exact shape of a crash on first render.
    const ui = normalizeUi(
      stored({
        root: { type: "pane", id: "p1", active: 9, tabs: [{ id: "t", kind: "file", target: "a" }] },
        focus: "p1",
      }),
    );
    expect(panes(ui.layout!.root)[0].active).toBe(0);
  });

  it("collapses a split left with one child rather than drawing a lone divider", () => {
    const ui = normalizeUi(
      stored({
        root: {
          type: "split",
          id: "s",
          direction: "row",
          sizes: [0.5, 0.5],
          children: [
            { type: "pane", id: "p1", tabs: [], active: 0 },
            { nonsense: true },
          ],
        },
        focus: "p1",
      }),
    );
    expect(ui.layout!.root.type).toBe("pane");
  });

  it("renormalises sizes so a layout is never 70% wide", () => {
    const ui = normalizeUi(
      stored({
        root: {
          type: "split",
          id: "s",
          direction: "column",
          sizes: [3, 1],
          children: [
            { type: "pane", id: "p1", tabs: [], active: 0 },
            { type: "pane", id: "p2", tabs: [], active: 0 },
          ],
        },
        focus: "p1",
      }),
    );
    const root = ui.layout!.root;
    const sizes = root.type === "split" ? root.sizes : [];
    expect(sizes.reduce((a, b) => a + b, 0)).toBeCloseTo(1);
  });

  it("repairs a focus naming a pane that is not there", () => {
    // Otherwise every "open here" has nowhere to go.
    const ui = normalizeUi(stored({ ...onePane, focus: "pane-gone" }));
    expect(panes(ui.layout!.root).some((p) => p.id === ui.layout!.focus)).toBe(true);
  });
});

describe("the prose measure, per tab", () => {
  it("keys on the path, because tab ids do not survive a session", () => {
    const ui = withWrap(emptyUi(), "notes.hick", 64);
    expect(wrapFor(ui, "notes.hick")).toBe(64);
    expect(wrapFor(ui, "other.hick")).toBe(WRAP_DEFAULT);
  });

  it("clamps a stored measure that would not lay out", () => {
    const ui = normalizeUi(stored(onePane, { "a.hick": 1, "b.hick": 9999, "c.hick": "wide" }));
    expect(ui.wrap["a.hick"]).toBe(WRAP_MIN);
    expect(ui.wrap["b.hick"]).toBe(WRAP_MAX);
    expect(ui.wrap["c.hick"]).toBeUndefined();
  });
});

describe("what is worth writing down", () => {
  it("does not store a workspace holding nothing but furniture", () => {
    // A fresh project opens like this anyway; storing it only turns "never
    // seen" into "left empty", which reads the same and costs a write.
    expect(worthStoring(withTree(freeform(), tab("tree", "folder", "Files")))).toBe(false);
  });

  it("stores a workspace with a document open", () => {
    const layout = withTree(freeform(), tab("tree", "folder", "Files"));
    const withDoc = openInLayout(layout, tab("document", "notes.hick", undefined, "d1"), layout.focus);
    expect(worthStoring(withDoc)).toBe(true);
  });
});
