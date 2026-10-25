<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/elements/patch/view.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Literal files use native bytes](../../../../../../../guarantees/embedding/literal-files-use-native-bytes/contract.md)
- [Patch elements use the shared diff renderer](../../../../../../../guarantees/literate-projects/patch-view-uses-the-shared-diff-renderer/contract.md)
- [Reject a patch that does not fit its exact input](../../../../../../../guarantees/literate-projects/replay-is-exact-and-atomic/contract.md)

Captured SHA-256: `d9f0a5e4b7c7200dc8dcc9efa566af187c3a34795821414f7047a5674443eea6`.

<book:entry path="apps/web/src/elements/patch/view.test.tsx">
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
</book:entry>

</book:doc>
