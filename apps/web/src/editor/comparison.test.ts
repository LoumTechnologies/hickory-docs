// @vitest-environment jsdom
// Guarantee: editing/a-comparison-keeps-current-code-editable.md
import { afterEach, describe, expect, it } from "vitest";
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
    expect(document.querySelector(".comparison-removed pre")?.textContent).toBe("old\n");
    expect(document.querySelector(".comparison-added")?.textContent).toBe("new");
    view.dispatch({ changes:{ from:5,to:8,insert:"edited" } });
    expect(view.state.doc.toString()).toBe("same\nedited\n");
    expect(document.querySelector(".comparison-added")?.textContent).toBe("edited");
    view.dispatch({effects:setComparison.of({base:null,editable:true})});
    expect(document.querySelector(".comparison-removed")).toBeNull();
  });
  it("restores only explicitly chosen removed lines and hides restoration for history", () => {
    const view=new EditorView({state:EditorState.create({doc:"same\n",extensions:[comparisonField]}),parent:document.body}); views.push(view);
    view.dispatch({effects:setComparison.of({base:"same\ndeleted\n",editable:false})});
    expect(document.querySelector(".comparison-removed button")).toBeNull();
    view.dispatch({effects:setComparison.of({base:"same\ndeleted\n",editable:true})});
    (document.querySelector(".comparison-removed button") as HTMLButtonElement).click();
    expect(view.state.doc.toString()).toBe("same\ndeleted\n");
    expect(document.querySelector(".comparison-removed")).toBeNull();
  });
  it("handles empty sides, missing final newlines, and UTF-16 coordinates",()=>{
    expect(comparisonChanges("", "é🙂")).toEqual([{from:0,to:3,removed:""}]);
    expect(comparisonChanges("é🙂", "")).toEqual([{from:0,to:0,removed:"é🙂"}]);
    expect(comparisonChanges("a\n", "a")).toEqual([{from:0,to:1,removed:"a\n"}]);
  });
});
