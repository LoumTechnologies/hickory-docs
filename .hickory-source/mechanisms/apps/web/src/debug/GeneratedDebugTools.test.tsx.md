<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/GeneratedDebugTools.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7003cb507e09"></a>
## `apps/web/src/debug/GeneratedDebugTools.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Choose a launch without replacing the editor being read](../../../../../guarantees/debugging/debug-controls-preserve-editor-space/contract.md).

<a id="f-7003cb507e09-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-7003cb507e09-expression_statement">
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterEach, expect, it, vi } from "vitest";
import type { OutputFile } from "../api/types";
import { IDLE_SESSION } from "./useDebugger";
import { generatedLineMap, GeneratedDebugTools } from "./GeneratedDebugTools";
import { DebugLaunchRail } from "./DebugRail";

vi.mock("./launchProfiles", () => ({launchWithProfiles: (_query: unknown, start: (profile: unknown) => void) => {start({id:"app",fingerprint:"f"});return Promise.resolve();}}));
</storycode:copy>
<a id="f-7003cb507e09-view"></a>
### `view`

<storycode:copy id="f-7003cb507e09-view">
let view:EditorView|undefined;
</storycode:copy>
<a id="f-7003cb507e09-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-7003cb507e09-expression_statement-2">
afterEach(()=>{cleanup();view?.destroy();document.body.replaceChildren();});
</storycode:copy>
<a id="f-7003cb507e09-code"></a>
### `code`

<storycode:copy id="f-7003cb507e09-code">
const code='print("🙂")\nprint(2)\n';
</storycode:copy>
<a id="f-7003cb507e09-source"></a>
### `source`

<storycode:copy id="f-7003cb507e09-source">
const source='# Note 🙂\n<hick:file path="app.py">\n'+code+'</hick:file>';
</storycode:copy>
<a id="f-7003cb507e09-from"></a>
### `from`

<storycode:copy id="f-7003cb507e09-from">
const from=new TextEncoder().encode(source.slice(0,source.indexOf(code))).length;
</storycode:copy>
<a id="f-7003cb507e09-file"></a>
### `file`

<storycode:copy id="f-7003cb507e09-file">
const file:OutputFile={path:"app.py",language:"python",content:code,provenance:[{start:0,end:new TextEncoder().encode(code).length,
  origin:{kind:"literal",doc_path:"note.md",span:[from,from+new TextEncoder().encode(code).length]}}]};
</storycode:copy>
<a id="f-7003cb507e09-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-7003cb507e09-expression_statement-3">
it("maps generated code back to its owner without treating synthetic or foreign bytes as source",()=>{
  expect([...generatedLineMap(file,"note.md",source)]).toEqual([[0,2],[1,3]]);
  expect(generatedLineMap({...file,provenance:[{start:0,end:100,origin:{kind:"synthetic"}}]},"note.md",source).size).toBe(0);
  expect(generatedLineMap(file,"other.md",source).size).toBe(0);
});
</storycode:copy>
<a id="f-7003cb507e09-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-7003cb507e09-expression_statement-4">
it("starts generated code in its document session and maps breakpoints and active controls",()=>{
  const wrapper=document.body.appendChild(document.createElement("div"));
  const host=wrapper.appendChild(document.createElement("div"));host.className="editor-cm-host";
  view=new EditorView({parent:host,state:EditorState.create({doc:code})});
  const start=vi.fn(),toggleBreakpoint=vi.fn(),step=vi.fn();
  const session={...IDLE_SESSION,start,toggleBreakpoint,step};
  const container=wrapper.appendChild(document.createElement("div"));
  const rails=render(<DebugLaunchRail view={view}/>,{container});
  const tools=render(<GeneratedDebugTools view={view} file={file} debug={session} source={source} docPath="note.md" askText={async()=>null}/>);
  fireEvent.click(screen.getByRole("button",{name:"Debug app.py"}));expect(start).toHaveBeenCalledWith("app.py", {}, "new", {id:"app",fingerprint:"f"});
  tools.rerender(<GeneratedDebugTools view={view} file={file} debug={{...session,status:"paused",program:"app.py",pausedLine:3}} source={source} docPath="note.md" askText={async()=>null}/>);
  expect(screen.getByRole("button",{name:"Step over"}).closest("[data-source-offset]")?.getAttribute("data-source-offset")).toBe(String(code.indexOf("print(2)")));
  fireEvent.click(screen.getByRole("button",{name:"Step over"}));expect(step).toHaveBeenCalledWith("over");
  tools.unmount();rails.unmount();
});
</storycode:copy>

### Assemble `apps/web/src/debug/GeneratedDebugTools.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/GeneratedDebugTools.test.tsx"><storycode:paste select="#f-7003cb507e09-expression_statement"/><storycode:paste select="#f-7003cb507e09-view"/><storycode:paste select="#f-7003cb507e09-expression_statement-2"/><storycode:paste select="#f-7003cb507e09-code"/><storycode:paste select="#f-7003cb507e09-source"/><storycode:paste select="#f-7003cb507e09-from"/><storycode:paste select="#f-7003cb507e09-file"/><storycode:paste select="#f-7003cb507e09-expression_statement-3"/><storycode:paste select="#f-7003cb507e09-expression_statement-4"/></storycode:file>

</storycode:doc>
