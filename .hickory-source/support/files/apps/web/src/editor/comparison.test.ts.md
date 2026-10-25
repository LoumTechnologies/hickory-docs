<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/comparison.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A comparison keeps current code editable](../../../../../../guarantees/editing/a-comparison-keeps-current-code-editable/contract.md)
- [A commit reads as a literate change](../../../../../../guarantees/lenses/a-commit-reads-as-a-literate-change/contract.md)

Captured SHA-256: `739eca28bf68814f1031ae43d976025d823e43b823234ab407a81e7652b37901`.

<book:entry path="apps/web/src/editor/comparison.test.ts">
// @vitest-environment jsdom
// Guarantee: editing/a-comparison-keeps-current-code-editable.md
import { afterEach, describe, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { comparisonChanges, comparisonField, setComparison } from "./comparison";
const views: EditorView[] = [];
afterEach(() => { views.splice(0).forEach(view => view.destroy()); document.body.replaceChildren(); });
describe("editable comparison in the main buffer", () => {
  it("keeps deleted bytes out of current text and recomputes when editing", () => {
    const view = new EditorView({ state:EditorState.create({ doc:"same\nnew\n", extensions:[comparisonField] }), parent:document.body }); views.push(view);
    view.dispatch({ effects:setComparison.of({ base:"same\nold\n",editable:true }) });
    expect(view.state.doc.toString()).toBe("same\nnew\n");
    expect(document.querySelector(".comparison-removed")).toBeNull();
    expect(document.querySelector(".comparison-deletion")?.getAttribute("aria-label")).toBe("Show 1 removed line");
    expect(document.querySelector(".comparison-added")?.textContent).toBe("new");
    view.dispatch({ changes:{ from:5,to:8,insert:"edited" } });
    expect(view.state.doc.toString()).toBe("same\nedited\n");
    expect(document.querySelector(".comparison-added")?.textContent).toBe("edited");
    view.dispatch({effects:setComparison.of({base:null,editable:true})});
    expect(document.querySelector(".comparison-removed")).toBeNull();
  });
  it("restores only explicitly chosen removed lines and hides restoration for history", async () => {
    const view=new EditorView({state:EditorState.create({doc:"same\n",extensions:[comparisonField]}),parent:document.body}); views.push(view);
    view.dispatch({effects:setComparison.of({base:"same\ndeleted\n",editable:false})});
    expect(document.querySelector(".comparison-deletion")).not.toBeNull();
    view.dispatch({effects:setComparison.of({base:"same\ndeleted\n",editable:true})});
    const marker = document.querySelector(".comparison-deletion") as HTMLButtonElement;
    marker.click();
    const tooltip = view.state.field(comparisonField);
    expect(tooltip.comparison.editable).toBe(true);
    await vi.waitFor(() => expect(document.querySelector(".comparison-removed pre")?.textContent).toContain("deleted"));
    const restore = [...document.querySelectorAll(".comparison-removed button")].find(button => button.textContent === "Restore these lines") as HTMLButtonElement;
    restore.click();
    expect(view.state.doc.toString()).toBe("same\ndeleted\n");
    expect(document.querySelector(".comparison-removed")).toBeNull();
  });
  it("handles empty sides, missing final newlines, and UTF-16 coordinates",()=>{
    expect(comparisonChanges("", "é🙂")).toEqual([{from:0,to:3,removed:""}]);
    expect(comparisonChanges("é🙂", "")).toEqual([{from:0,to:0,removed:"é🙂"}]);
    expect(comparisonChanges("a\n", "a")).toEqual([{from:0,to:1,removed:"a\n"}]);
  });
});
</book:entry>

</book:doc>
