<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/projectDebugSurface.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-777a3021cd65"></a>
## `apps/web/src/debug/projectDebugSurface.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Launch profiles choose targets within the viewed project](../../../../../guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project/contract.md).

<a id="f-777a3021cd65-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-777a3021cd65-expression_statement">
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { renderedField, renderBlock } from "../editor/rendered";
import { materializeProject } from "../editor/hickLang";
import { preferCapturedLiterateSurface, ProjectDebugLinks } from "./projectDebugSurface";
import { clearPreferredDebugSurface, revealPreferredDebugLocation } from "./preferredDebugLocation";
import { IDLE_SESSION } from "./useDebugger";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { openCapturedCode } from "../lib/capturedCode";
vi.mock("../lib/capturedCode", () => ({openCapturedCode:vi.fn().mockResolvedValue(undefined)}));
</storycode:copy>
<a id="f-777a3021cd65-view"></a>
### `view`

<storycode:copy id="f-777a3021cd65-view">
let view: EditorView;
</storycode:copy>
<a id="f-777a3021cd65-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-777a3021cd65-expression_statement-2">
afterEach(() => { clearPreferredDebugSurface("hick-project:///test/app.py"); cleanup(); view?.destroy(); document.body.replaceChildren(); });
</storycode:copy>
<a id="f-777a3021cd65-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-777a3021cd65-expression_statement-3">
it("reveals exact literate source and offers both directions for one captured paused frame", () => {
  const source = '<hick:project/>\n<hick:file path="library.py">\ndef welcome(name):\n    return name\n</hick:file>';
  const project = materializeProject("project.md", {"project.md":source});
  view = new EditorView({parent:document.body, state:EditorState.create({doc:source})});
  const owner = "hick-project:///test/app.py";
  const frame = {id:1,name:"welcome",source:"hick-project:///test/library.py",source_line:1,line:null,in_document:false};
  preferCapturedLiterateSurface(owner,"hick-project:///test/",project.new,{view,entry:"project.md",source});
  expect(revealPreferredDebugLocation(owner,frame)).toBe(true);
  expect(view.state.doc.lineAt(view.state.selection.main.head).text).toBe("    return name");
  render(<ProjectDebugLinks owner={owner} debug={{...IDLE_SESSION,status:"paused",frames:[frame]}}/>);
  fireEvent.click(screen.getByRole("button",{name:"Show generated code"}));
  expect(openCapturedCode).toHaveBeenCalledWith(expect.objectContaining({uri:frame.source}),owner,expect.any(Function));
  fireEvent.click(screen.getByRole("button",{name:"Show literate source"}));
  expect(view.state.doc.lineAt(view.state.selection.main.head).text).toBe("    return name");
});

</storycode:copy>
<a id="f-777a3021cd65-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-777a3021cd65-expression_statement-4">
it("preserves rendered blocks and the outer selection when a breakpoint hits", () => {
  const source = '<hick:project/>\n<hick:file path="library.py">\ndef welcome(name):\n    return name\n</hick:file>';
  const project = materializeProject("project.md", {"project.md":source});
  const at = source.indexOf("<hick:file");
  view = new EditorView({parent:document.body,state:EditorState.create({doc:source,extensions:[renderedField]})});
  view.dispatch({effects:renderBlock.of(at)});
  const frame = {id:1,name:"welcome",source:"hick-project:///test/library.py",source_line:1,line:null,in_document:false};
  preferCapturedLiterateSurface("hick-project:///test/app.py","hick-project:///test/",project.new,{view,entry:"project.md",source});
  expect(revealPreferredDebugLocation("hick-project:///test/app.py",frame)).toBe(true);
  expect(view.state.field(renderedField)).toEqual([at]);
  expect(view.state.selection.main.head).toBe(0);
});

</storycode:copy>
<a id="f-777a3021cd65-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-777a3021cd65-expression_statement-5">
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
it("keeps a foreign-origin old library stop in its visible comparison pane",async()=>{
  const {registerCapturedComparison}=await import("./projectDebugSurface");
  const source='<hick:project sources="library.md"/><hick:file path="app.py">print(1)\n</hick:file>';
  const project=materializeProject("project.md",{"project.md":source,"library.md":'<hick:file path="library.py">\ndef greet():\n    return "Hello"\n</hick:file>'});
  view=new EditorView({parent:document.body,state:EditorState.create({doc:source})});
  const code=new EditorView({parent:document.body,state:EditorState.create({doc:'def greet():\n    return "Hello"\n'})});
  vi.spyOn(code.dom,"getClientRects").mockReturnValue([new DOMRect(0,0,300,100)] as unknown as DOMRectList);
  const release=registerCapturedComparison("hick-project:///test/library.py",code,code.state.doc.toString(),view);
  try {
    const owner="hick-project:///test/app.py";
    const frame={id:1,name:"greet",source:"hick-project:///test/library.py",source_line:1,line:null,in_document:false};
    preferCapturedLiterateSurface(owner,"hick-project:///test/",project.new,{view,entry:"project.md",source});
    expect(revealPreferredDebugLocation(owner,frame)).toBe(true);
    expect(code.state.doc.lineAt(code.state.selection.main.head).text).toBe('    return "Hello"');
  } finally {release();code.destroy();}
});

</storycode:copy>
<a id="f-777a3021cd65-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-777a3021cd65-expression_statement-6">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares captured frame navigation and retires an old stop before late code delivery", async () => {
  const source = '<hick:project/>\n<hick:file path="library.py">\ndef welcome(name):\n    return name\n</hick:file>';
  const project = materializeProject("project.md", { "project.md": source });
  view = new EditorView({ parent: document.body, state: EditorState.create({ doc: source }) });
  const owner = "hick-project:///test/app.py";
  const frame = { id: 1, name: "welcome", source: "hick-project:///test/library.py", source_line: 1, line: null, in_document: false };
  preferCapturedLiterateSurface(owner, "hick-project:///test/", project.new, { view, entry: "project.md", source });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const ui = (id: number) => <UiActionContext.Provider value={registry}><ProjectDebugLinks owner={owner} debug={{ ...IDLE_SESSION, status: "paused", frames: [{ ...frame, id }], selectedFrame: id }} /></UiActionContext.Provider>;
  const rendered = render(ui(1));
  const action = (label: string) => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === label)!;
  const literal = action("Show captured debugger literate source");
  await act(async () => expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: literal.handle, arguments: {}, request_id: "source" })).toMatchObject({ outcome: { value: { source_selection_requested: true } } }));
  expect(view.state.doc.lineAt(view.state.selection.main.head).text).toBe("    return name");
  let release!: () => void, gate!: () => boolean;
  vi.mocked(openCapturedCode).mockImplementationOnce(async (_target, _owner, current) => { gate = current!; await new Promise<void>(resolve => { release = resolve; }); if (!current!()) throw Error("retired code navigation"); });
  const generated = action("Show captured debugger generated code");
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: generated.handle, arguments: {}, request_id: "generated" }); });
  expect(gate()).toBe(true);
  rendered.rerender(ui(2));
  expect(gate()).toBe(false);
  await act(async () => { release(); expect(await pending).toMatchObject({ status: "error", reason: expect.stringContaining("retired code navigation") }); });
  expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: generated.handle, arguments: {}, request_id: "old-stop" })).toMatchObject({ status: "stale" });
  rendered.unmount();
});
</storycode:copy>

### Assemble `apps/web/src/debug/projectDebugSurface.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/projectDebugSurface.test.tsx"><storycode:paste select="#f-777a3021cd65-expression_statement"/><storycode:paste select="#f-777a3021cd65-view"/><storycode:paste select="#f-777a3021cd65-expression_statement-2"/><storycode:paste select="#f-777a3021cd65-expression_statement-3"/><storycode:paste select="#f-777a3021cd65-expression_statement-4"/><storycode:paste select="#f-777a3021cd65-expression_statement-5"/><storycode:paste select="#f-777a3021cd65-expression_statement-6"/></storycode:file>

</storycode:doc>
