import { IDLE_SESSION } from "../debug/useDebugger";
import { useFollowingDebugSession } from "../debug/plainDebugHosts";
import { DocumentReading } from "../views/DocumentReading";
import { forgetReading, useReading } from "../lib/readingViews";
import { CompletionContext } from "@codemirror/autocomplete";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import type { LspClient } from "../lsp/client";
import { useWorkspaceLsp } from "../lsp/useLsp";
import { completionSources } from "../lsp/completion";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { request } from "../api/client";
import { materializeProject } from "../editor/hickLang";
import { ComparisonTools } from "./ComparisonTools";
import { DebugLaunchRail } from "../debug/DebugRail";
import { ComparisonEditor } from "./ComparisonEditor";

vi.mock("../debug/launchProfiles", () => ({launchWithProfiles: (_query: unknown, start: (profile: unknown) => void) => {start({id:"app",fingerprint:"f"});return Promise.resolve();}}));
vi.mock("../debug/plainDebugHosts", () => ({ useFollowingDebugSession: vi.fn() }));
vi.mock("../api/client", () => ({ api: {}, request: vi.fn() }));
vi.mock("../lsp/useLsp", () => ({ useWorkspaceLsp: vi.fn(() => ({ client: null, uri: "", diagnostics: [], ready: true })) }));
vi.mock("../debug/useDebugger", async importOriginal => {
  const actual = await importOriginal<typeof import("../debug/useDebugger")>();
  return { ...actual, useWorkspaceDebugger: () => actual.IDLE_SESSION };
});
beforeEach(() => {
  vi.mocked(useFollowingDebugSession).mockReturnValue(IDLE_SESSION);
  vi.mocked(useWorkspaceLsp).mockReturnValue({ client: null, uri: "", diagnostics: [], ready: true });
  vi.mocked(request).mockResolvedValue({ id: "captured", uri: "hick-project:///captured/" });
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
it("automatically registers the completed project for a middle patch", async () => {
  const source = `<hick:project/><hick:file id="x" path="x.ts">old\n</hick:file>
<hick:file id="y" path="y.ts">before\n</hick:file>
<hick:patch id="middle" after="x">
--- a/x.ts
+++ b/x.ts
@@ -1 +1 @@
-old
+new
</hick:patch>
<hick:patch id="last" after="y">
--- a/y.ts
+++ b/y.ts
@@ -1 +1 @@
-before
+completed
</hick:patch>`;
  const project = materializeProject("project.md", { "project.md": source });
  vi.mocked(request).mockResolvedValue({ id: "captured", uri: "hick-project:///captured/" });
  render(<ComparisonTools path="x.ts" view={null} source={{ id: "new", label: "New · completed project",
    kind: "authored", text: "new\n", local: project.local_states.middle.new, project: project.new }} />);
  expect(screen.queryByRole("button", { name: "Enable code completion and navigation" })).toBeNull();
  await waitFor(() => expect(request).toHaveBeenCalledWith("POST", "/api/project-contexts", expect.anything()));
  const body = vi.mocked(request).mock.calls.find(([method]) => method === "POST")![2] as { snapshot: { entries: Record<string, {bytes: number[]}> } };
  expect(Object.keys(body.snapshot.entries).sort()).toEqual(["x.ts", "y.ts"]);
  expect(new TextDecoder().decode(new Uint8Array(body.snapshot.entries["y.ts"].bytes))).toBe("completed\n");
});

it("does not offer runnable tools for an empty Old endpoint", () => {
  render(<ComparisonTools path="x.ts" view={null} source={{ id: "old", label: "Old · scope input",
    kind: "authored", text: "old\n", project: [] }} />);
  expect(screen.queryByRole("button", { name: "Enable code completion and navigation" })).toBeNull();
  expect(screen.queryByRole("button", { name: /^Debug / })).toBeNull();
  expect(request).not.toHaveBeenCalled();
});

it("automatically registers both revisions using their own paths across a rename", async () => {
  const entry = (path: string) => ({ path, bytes: [...new TextEncoder().encode("code\n")], kind: "file" as const, executable: false });
  render(<ComparisonEditor path="new.ts" model={{ sources: [
    { id: "old", label: "Old revision", kind: "git", path: "old.ts", text: "code\n", project: [entry("old.ts")] },
    { id: "new", label: "New revision", kind: "git", text: "code\n", project: [entry("new.ts")] },
  ], old: "old", new: "new", policy: "neither", evidence: "unverified" }} />);
  await waitFor(() => expect(vi.mocked(request).mock.calls.filter(([method]) => method === "POST")).toHaveLength(2));
  const paths = vi.mocked(request).mock.calls.filter(([method]) => method === "POST").map(call => Object.keys((call[2] as {snapshot:{entries:object}}).snapshot.entries));
  expect(paths).toEqual([["old.ts"], ["new.ts"]]);
});

const entry = (text = "value", path = "app.ts") => ({ path, bytes: [...new TextEncoder().encode(text)], kind: "file" as const, executable: false });
it("cleans up late registrations when a revision changes or the view closes", async () => {
  let finish!: (value: {id:string;uri:string}) => void;
  vi.mocked(request).mockImplementation((method) => method === "POST" ? new Promise(resolve => { finish = resolve; }) : Promise.resolve(undefined));
  const first = { id: "new", label: "New", kind: "git" as const, text: "first", project: [entry("first")] };
  const mounted = render(<ComparisonTools path="app.ts" source={first} view={null} />);
  const stale = finish;
  mounted.rerender(<ComparisonTools path="app.ts" source={{ ...first, text: "second", project: [entry("second")] }} view={null} />);
  await act(async () => { stale({id:"stale",uri:"hick-project:///stale/"}); });
  expect(request).toHaveBeenCalledWith("DELETE", "/api/project-contexts/stale");
  mounted.unmount();
  await act(async () => { finish({id:"closed",uri:"hick-project:///closed/"}); });
  expect(request).toHaveBeenCalledWith("DELETE", "/api/project-contexts/closed");
});

it("offers a retry only after automatic setup fails", async () => {
  vi.mocked(request).mockRejectedValueOnce(new Error("Engine unavailable"));
  render(<ComparisonTools path="app.ts" source={{ id:"new",label:"New",kind:"git",text:"value",project:[entry()] }} view={null} />);
  await waitFor(() => expect(screen.getByText(/Engine unavailable/)).toBeTruthy());
  expect(screen.getByRole("alert").textContent).toContain("Engine unavailable");
  fireEvent.click(screen.getByRole("button", {name:"Retry code tools"}));
  await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
  expect(screen.queryByRole("button", {name:"Retry code tools"})).toBeNull();
});

it("requests completions from the captured project's mapped file without an enable click", async () => {
  const completion = vi.fn().mockResolvedValue([{label:"value"}]);
  const definition = vi.fn().mockResolvedValue([]);
  vi.mocked(useWorkspaceLsp).mockImplementation(path => ({client: {completion, definition} as unknown as LspClient, uri:path, diagnostics:[], ready:true}));
  const view = new EditorView({parent:document.body,state:EditorState.create({doc:"value"})});
  const file = entry();
  try {
    const mounted = render(<ComparisonTools path="app.ts" source={{id:"new",label:"New",kind:"git",text:"value",project:[file]}} view={view}/>);
    await waitFor(() => expect(view.state.facet(completionSources)).toHaveLength(1));
    const result = await view.state.facet(completionSources)[0](new CompletionContext(view.state, 3, true));
    expect(completion).toHaveBeenCalledWith("hick-project:///captured/app.ts", {line:0,character:3});
    expect(result?.options.map(option => option.label)).toContain("value");
    act(() => view.dispatch({selection:{anchor:3}}));
    fireEvent.keyDown(view.contentDOM, {key:"F12"});
    await waitFor(() => expect(definition).toHaveBeenCalledWith("hick-project:///captured/app.ts", {line:0,character:3}));
    // A changed buffer must not send coordinates through the frozen mapping.
    act(() => view.dispatch({changes:{from:0,to:5,insert:"other"}}));
    const count = completion.mock.calls.length;
    const stale = await view.state.facet(completionSources)[0](new CompletionContext(view.state, 3, true));
    expect(stale).toBeNull(); expect(completion).toHaveBeenCalledTimes(count);
    mounted.unmount();
  } finally { view.destroy(); }
});


// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
it("launches the latest compatible patch code without replacing the code in view", async () => {
  const start = vi.fn();
  vi.mocked(useFollowingDebugSession).mockReturnValue({...IDLE_SESSION,start});
  const project = [entry("final")];
  const versions = [[entry()], project];
  vi.mocked(request).mockImplementation(async method => method === "POST" ? {id:"captured",uri:"hick-project:///captured/"} : method === "GET" ? {path:"app.ts",bytes:entry().bytes,files:versions[0]} : undefined);
  const model = {sources:[{id:"old",label:"Before this patch",kind:"git" as const,text:""},
    {id:"new",label:"After this patch",kind:"git" as const,text:"value",local:[entry()],project,versions}],old:"old",new:"new",policy:"neither" as const,evidence:"unverified" as const};
  const mounted = render(<ComparisonEditor path="app.ts" model={model} compact/>);
  const launch = await screen.findByRole("button", {name:"Debug app.ts · After this patch"});
  await waitFor(()=>expect((launch as HTMLButtonElement).disabled).toBe(false));
  expect(launch.closest(".cm-debug-rail")).not.toBeNull();
  expect(launch.closest("details")).toBeNull();
  expect(mounted.container.querySelector('summary[aria-label^="Code tools"]')).toBeNull();
  expect(screen.queryByText("Code tools enabled")).toBeNull();
  fireEvent.click(launch);
  expect(start).toHaveBeenCalledOnce();
  expect(vi.mocked(request).mock.calls.some(([method]) => method === "GET")).toBe(false);
  const body = vi.mocked(request).mock.calls.find(([method]) => method === "POST")![2] as {snapshot:{entries:Record<string,{bytes:number[]}>}};
  expect(new TextDecoder().decode(new Uint8Array(body.snapshot.entries["app.ts"].bytes))).toBe("value");
  // Generated-code navigation remains explicit, never a side effect of launch.
  const { openCapturedCode } = await import("../lib/capturedCode");
  await openCapturedCode({uri:"hick-project:///captured/app.ts",range:{start:{line:0,character:0},end:{line:0,character:0}}}, "hick-project:///captured/app.ts");
  mounted.unmount();
  const reading = render(<DocumentReading id="captured:hick-project:///captured/app.ts"/>);
  await waitFor(()=>expect(reading.container.querySelectorAll(".cm-editor")).toHaveLength(1));
  expect(reading.container.querySelector(".cm-content")?.textContent).toBe("value");
  expect(vi.mocked(request).mock.calls.filter(([method])=>method==="POST")).toHaveLength(1);
  const rerun=await screen.findByRole("button",{name:"Debug app.ts · app.ts"});
  await waitFor(()=>expect((rerun as HTMLButtonElement).disabled).toBe(false));
  fireEvent.click(rerun); expect(start).toHaveBeenCalledTimes(2);
  reading.unmount(); forgetReading("captured:hick-project:///captured/app.ts");
  expect(request).toHaveBeenCalledWith("DELETE","/api/project-contexts/captured");
});

it("offers all reference results and opens the chosen source at the requested line", async () => {
  const locations=[{uri:"hick-project:///captured/first.ts",range:{start:{line:0,character:0},end:{line:0,character:1}}},
    {uri:"hick-project:///captured/second.ts",range:{start:{line:1,character:2},end:{line:1,character:3}}}];
  const references=vi.fn().mockResolvedValue(locations);
  vi.mocked(useWorkspaceLsp).mockImplementation(path=>({client:{references} as unknown as LspClient,uri:path,diagnostics:[],ready:true}));
  vi.mocked(request).mockImplementation(async method=>method==="POST"?{id:"captured",uri:"hick-project:///captured/"}:method==="GET"?{path:"second.ts",bytes:entry("one\nsecond").bytes,files:[entry("one\nsecond","second.ts")]}:undefined);
  const view=new EditorView({parent:document.body,state:EditorState.create({doc:"value"})});
  try {
    const mounted=render(<ComparisonTools path="app.ts" source={{id:"new",label:"New",kind:"git",text:"value",project:[entry()]}} view={view}/>);
    await waitFor(()=>expect(view.state.facet(completionSources)).toHaveLength(1));
    fireEvent.keyDown(view.contentDOM,{key:"F12",shiftKey:true});
    const dialog=await screen.findByRole("dialog",{name:"References"});
    expect(dialog.querySelectorAll("button")).toHaveLength(3);
    expect(vi.mocked(request).mock.calls.some(([method])=>method==="GET")).toBe(false);
    fireEvent.click(screen.getByRole("button",{name:"second.ts:2"}));
    await waitFor(()=>expect(screen.queryByRole("dialog",{name:"References"})).toBeNull());
    const value=renderHookReading("captured:hick-project:///captured/second.ts");
    expect(value.location).toEqual({line:1,character:2});
    expect(value.codeUri).toBe(locations[1].uri);
    mounted.unmount(); forgetReading("captured:hick-project:///captured/second.ts");
  } finally {view.destroy();}
});

function renderHookReading(id:string) {
  let record!: NonNullable<ReturnType<typeof useReading>>;
  function Capture(){ record=useReading(id)!; return null; }
  const result=render(<Capture/>); result.unmount(); return record;
}

// Guarantee: docs/guarantees/literate-projects/patch-layout-breakpoints.md
it("sets and removes patch-layout breakpoints at the correct endpoint line", async () => {
  const toggle = vi.fn();
  vi.mocked(useFollowingDebugSession).mockReturnValue({ ...IDLE_SESSION, toggleBreakpoint: toggle,
    breakpoints: [{ line: 1, state: "bound" }] });
  render(<ComparisonEditor path="app.ts" model={{ sources: [
    { id: "old", label: "Before", kind: "git", text: "same\nremoved\nend\n", project: [entry("same\nremoved\nend\n")] },
    { id: "new", label: "After", kind: "authored", text: "same\nadded\nextra\nend\n", project: [entry("same\nadded\nextra\nend\n")] },
  ], old: "old", new: "new", policy: "neither", evidence: "unverified" }} />);
  fireEvent.change(screen.getByLabelText("Comparison layout"), { target: { value: "patch" } });
  await waitFor(() => expect(screen.getByRole("button", { name: "Remove breakpoint on old line 2" })).toBeTruthy());
  fireEvent.click(screen.getByRole("button", { name: "Remove breakpoint on old line 2" }));
  expect(toggle).toHaveBeenLastCalledWith(1);
  fireEvent.click(screen.getByRole("button", { name: "Set breakpoint on new line 3" }));
  expect(toggle).toHaveBeenLastCalledWith(2);
  expect(screen.queryByRole("button", { name: "Set breakpoint on old line 4" })).toBeNull();
});


// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
it("refuses a launch if the visible buffer changes before its captured model updates", async () => {
  const start = vi.fn();
  vi.mocked(useFollowingDebugSession).mockReturnValue({...IDLE_SESSION,start});
  const wrapper = document.body.appendChild(document.createElement("div"));
  const host = wrapper.appendChild(document.createElement("div"));
  host.className = "editor-cm-host";
  const view = new EditorView({parent:host,state:EditorState.create({doc:"value"})});
  const railHost = wrapper.appendChild(document.createElement("div"));
  try {
    render(<DebugLaunchRail view={view}/>,{container:railHost});
    render(<ComparisonTools path="app.ts" source={{id:"new",label:"New",kind:"git",text:"value",project:[entry()]}} view={view}/>);
    const launch = await screen.findByRole("button",{name:"Debug app.ts · New"});
    await waitFor(()=>expect((launch as HTMLButtonElement).disabled).toBe(false));
    act(()=>view.dispatch({changes:{from:0,to:5,insert:"changed"}}));
    fireEvent.click(launch);
    expect(start).not.toHaveBeenCalled();
    expect(screen.getByRole("alert").textContent).toContain("displayed code changed");
  } finally { cleanup(); view.destroy(); wrapper.remove(); }
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("does not adopt references that arrive after the captured source changes", async () => {
  let finish!: (locations: import("../lsp/client").LspLocation[]) => void;
  const references = vi.fn(() => new Promise<import("../lsp/client").LspLocation[]>(resolve => { finish = resolve; }));
  vi.mocked(useWorkspaceLsp).mockImplementation(path => ({ client: { references } as unknown as LspClient, uri: path, diagnostics: [], ready: true }));
  const view = new EditorView({ parent: document.body, state: EditorState.create({ doc: "value" }) });
  try {
    const mounted = render(<ComparisonTools path="app.ts" source={{ id: "new", label: "New", kind: "git", text: "value", project: [entry()] }} view={view} />);
    await waitFor(() => expect(view.state.facet(completionSources)).toHaveLength(1));
    fireEvent.keyDown(view.contentDOM, { key: "F12", shiftKey: true });
    await waitFor(() => expect(references).toHaveBeenCalledOnce());
    act(() => { view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "changed" } }); });
    mounted.rerender(<ComparisonTools path="app.ts" source={{ id: "next", label: "Next", kind: "git", text: "changed", project: [entry("changed")] }} view={view} />);
    await act(async () => { finish([{ uri: "hick-project:///captured/old.ts", range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } } }]); });
    expect(screen.queryByRole("dialog", { name: "References" })).toBeNull();
    mounted.unmount();
  } finally { view.destroy(); }
});
