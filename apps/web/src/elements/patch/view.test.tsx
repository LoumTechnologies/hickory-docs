import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { patchView } from "./view";
import { parseHickDoc } from "../../editor/hickDoc";
import { materializeProject, materializeLiteralFiles } from "../../editor/hickLang";
import type { RenderedSlot } from "../../editor/rendered";

// Guarantees:
// - docs/guarantees/literate-projects/replay-is-exact-and-atomic.md
// - docs/guarantees/literate-projects/patch-view-uses-the-shared-diff-renderer.md

describe("patch element", () => {
  it("uses the shared renderer and labels atomic endpoints beside the diff", () => {
    const slot = { text: "\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-old\n+new\n" } as RenderedSlot;
    render(patchView.render(slot, {} as never));
    expect(screen.getByText("New · completed project; Old · scope input")).toBeTruthy();
    expect(document.querySelectorAll(".diff-line--add")).toHaveLength(1);
    expect(document.querySelectorAll(".diff-line--del")).toHaveLength(1);
  });
  it("keeps tags in raw patches literal in the actual WASM parser", () => {
    const source = '<hick:patch>\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-<hick:file path="a">\n+<hick:paste select="#p"/>\n</hick:patch>';
    expect(parseHickDoc(source).blocks.map(b => b.name)).toEqual(["patch"]);
    expect(parseHickDoc('```diff\n--- a/x\n+++ b/x\n```').blocks).toHaveLength(0);
  });
  it("evaluates exact complete project bytes in WASM with no native tools", () => {
    const source = '<hick:project/><hick:file id="base" path="x">é\r\n</hick:file><hick:patch after="base">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-é\r\n+🦀\r\n</hick:patch>';
    expect(() => materializeLiteralFiles(source)).toThrow(/complete project evaluation/);
    const project = materializeProject("p.md", { "p.md": source });
    expect(project.old).toEqual([]);
    expect(new TextDecoder().decode(new Uint8Array(project.new[0].bytes))).toBe("🦀\r\n");
    expect(() => materializeProject("p.md", { "p.md": source.replace("-é", "-wrong") })).toThrow(/context/);
  });
});
