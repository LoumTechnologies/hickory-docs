// docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
import { describe, expect, it } from "vitest";

import { lensSources, onLensChange, registerLens, ribbonLinksOf } from "./lensSources";
import type { EditorView } from "@codemirror/view";

describe("a lens's links, as the overlay draws them", () => {
  it("start from the session's lines and end at the path each element named", () => {
    const links = ribbonLinksOf("sessions/s.hick", [
      { family: "context", span: [10, 40], to: { path: "data/x.csv", lines: [1, 8] }, title: "ctx", lines: [4, 4] },
      { family: "lineage", span: [50, 90], to: { path: "notes/today.hick", lines: [72, 75] }, title: "lin", lines: [9, 9] },
      { family: "declared", span: [50, 90], to: { path: "meetings/sync.hick" }, title: "dec", lines: [9, 12] },
    ]);
    expect(links.map((l) => [l.family, l.from.lines, l.to.path, l.to.kind])).toEqual([
      ["context", [4, 4], "data/x.csv", "file"],
      ["lineage", [9, 9], "notes/today.hick", "document"],
      ["declared", [9, 12], "meetings/sync.hick", "document"],
    ]);
    expect(links.every((l) => l.from.path === "sessions/s.hick")).toBe(true);
    expect(new Set(links.map((l) => l.key)).size).toBe(3);
  });

  it("registers and unregisters a lens, telling listeners each time", () => {
    let told = 0;
    const off = onLensChange(() => told++);
    const view = {} as EditorView;
    const unregister = registerLens({ path: "sessions/a.hick", view, source: "", links: [] });
    expect(lensSources().map((l) => l.path)).toContain("sessions/a.hick");
    unregister();
    expect(lensSources().map((l) => l.path)).not.toContain("sessions/a.hick");
    expect(told).toBe(2);
    off();
  });
});
