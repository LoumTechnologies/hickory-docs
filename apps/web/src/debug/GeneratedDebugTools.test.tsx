import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterEach, expect, it, vi } from "vitest";
import type { OutputFile } from "../api/types";
import { IDLE_SESSION } from "./useDebugger";
import { generatedLineMap, GeneratedDebugTools } from "./GeneratedDebugTools";
import { DebugLaunchRail } from "./DebugRail";

vi.mock("./launchProfiles", () => ({launchWithProfiles: (_query: unknown, start: (profile: unknown) => void) => {start({id:"app",fingerprint:"f"});return Promise.resolve();}}));
let view:EditorView|undefined;
afterEach(()=>{cleanup();view?.destroy();document.body.replaceChildren();});
const code='print("🙂")\nprint(2)\n';
const source='# Note 🙂\n<hick:file path="app.py">\n'+code+'</hick:file>';
const from=new TextEncoder().encode(source.slice(0,source.indexOf(code))).length;
const file:OutputFile={path:"app.py",language:"python",content:code,provenance:[{start:0,end:new TextEncoder().encode(code).length,
  origin:{kind:"literal",doc_path:"note.md",span:[from,from+new TextEncoder().encode(code).length]}}]};
it("maps generated code back to its owner without treating synthetic or foreign bytes as source",()=>{
  expect([...generatedLineMap(file,"note.md",source)]).toEqual([[0,2],[1,3]]);
  expect(generatedLineMap({...file,provenance:[{start:0,end:100,origin:{kind:"synthetic"}}]},"note.md",source).size).toBe(0);
  expect(generatedLineMap(file,"other.md",source).size).toBe(0);
});
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
