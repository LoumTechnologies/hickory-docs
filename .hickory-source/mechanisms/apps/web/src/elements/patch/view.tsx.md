<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/elements/patch/view.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d6e75baad108"></a>
## `apps/web/src/elements/patch/view.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Patch elements use the shared diff renderer](../../../../../../guarantees/literate-projects/patch-view-uses-the-shared-diff-renderer/contract.md).

<a id="f-d6e75baad108-patchView"></a>
### `patchView`

<storycode:copy id="f-d6e75baad108-patchView">
import { PatchEditor } from "./Editor";
import type { ElementView } from "../types";

export const patchView: ElementView = {
  kind: "patch",
  draws: block => block.name === "patch",
  render: (slot, cx) => <PatchEditor slot={slot} cx={cx} />,
};
</storycode:copy>

### Assemble `apps/web/src/elements/patch/view.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/patch/view.tsx"><storycode:paste select="#f-d6e75baad108-patchView"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-059795295090"></a>
## `apps/web/src/elements/patch/view.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Literal files use native bytes](../../../../../../guarantees/embedding/literal-files-use-native-bytes/contract.md), [Patch elements use the shared diff renderer](../../../../../../guarantees/literate-projects/patch-view-uses-the-shared-diff-renderer/contract.md), [Reject a patch that does not fit its exact input](../../../../../../guarantees/literate-projects/replay-is-exact-and-atomic/contract.md).

<a id="f-059795295090-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-059795295090-expression_statement">
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
</storycode:copy>

### Assemble `apps/web/src/elements/patch/view.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/patch/view.test.tsx"><storycode:paste select="#f-059795295090-expression_statement"/></storycode:file>


</storycode:doc>
