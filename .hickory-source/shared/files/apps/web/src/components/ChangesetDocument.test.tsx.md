<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/ChangesetDocument.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Changeset completion is negotiated and checked](../../../../../../guarantees/agent/changeset-completion-is-negotiated-and-checked/contract.md)
- [A final changeset revision applies B to C](../../../../../../guarantees/agent/final-changeset-revisions-apply-b-to-c/contract.md)
- [Patch regeneration revalidates downstream construction](../../../../../../guarantees/literate-projects/patch-edits-revalidate-downstream-construction/contract.md)

Captured SHA-256: `f3b9f7df63e88c2ff81dff66ae17593a76c2342fbb860c555721d9a8dc58f82b`.

<book:entry path="apps/web/src/components/ChangesetDocument.test.tsx">
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { installMockHandler } from "../api/client";
import { ChangesetDocument } from "./ChangesetDocument";
import { AgentProject } from "./AgentProject";
import { ComparisonTools } from "./ComparisonTools";
import type { ProjectEntry } from "../editor/hickLang";
vi.mock("./ComparisonTools", () => ({ ComparisonTools: vi.fn(() => null) }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
const file = (path:string,text:string):ProjectEntry => ({path,bytes:[...new TextEncoder().encode(text)],kind:"file",executable:false});
const input=[file("app.py","old\n"),file("library.py","dependency\n")];
const source=`Successful change.\n<hick:project/>\n<hick:patch id="completed">
--- a/app.py
+++ b/app.py
@@ -1 +1 @@
-old
+new
</hick:patch>`;
installMockHandler(async () => ({}));
// Guarantee: docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md
it("renders a final patch against captured A and passes complete evaluated projects to language/debug tools", async () => {
  const view=render(<ChangesetDocument entry="response.md" sources={{"response.md":source}} input={input} />);
  await waitFor(() => expect(view.container.querySelector(".comparison-editor")).toBeTruthy());
  expect(view.container.textContent).toContain("Successful change.");
  expect(screen.queryByRole("alert")).toBeNull();
  const calls=vi.mocked(ComparisonTools).mock.calls;
  expect(calls.some(([props]) => props.source.project?.some(e => e.path==="library.py"))).toBe(true);
  expect(calls.some(([props]) => props.source.project?.some(e => e.path==="app.py" && new TextDecoder().decode(new Uint8Array(e.bytes))==="new\n"))).toBe(true);
  view.container.querySelectorAll(".comparison-side .cm-content").forEach(editor => expect(editor.getAttribute("contenteditable")).toBe("false"));
});
it("refuses a final artifact whose required closure was not supplied", () => {
  render(<ChangesetDocument entry="response.md" sources={{}} input={input} />);
  expect(screen.getByRole("alert").textContent).toContain("Could not render final changeset");
});
it("automatically shows the accepted final artifact while incorporation still awaits attribution", async () => {
  const snapshot=(files:ProjectEntry[])=>({entries:Object.fromEntries(files.map(e=>[e.path,{...e,kind:"File"}]))});
  installMockHandler(async (_method,path)=>path.includes("/agent/projects/") ? {entry:"project.md",starting:snapshot(input),completed:snapshot([file("app.py","new\n"),input[1]]),response:source,artifact:{entry:"response.md",sources:{"response.md":source}},ambiguous:["app.py"],error:null} : {});
  render(<AgentProject doc="doc" turn="turn" />);
  await screen.findByRole("region",{name:"Final literate changeset"});
  expect((screen.getByRole("button",{name:"Apply to literate source"}) as HTMLButtonElement).disabled).toBe(true);
  expect(screen.getByText(/do not have an exclusive agent identity/)).toBeTruthy();
});

it("renders included artifact documents against the same frozen project", async () => {
  const sources={"response.md":"<hick:project sources=\"patches.md\"/>\nSuccessful change.","patches.md":source.replace("<hick:project/>", "")};
  const view=render(<ChangesetDocument entry="response.md" sources={sources} input={input} />);
  await waitFor(() => expect(view.container.querySelector(".comparison-editor")).toBeTruthy());
  expect(screen.getByRole("heading",{name:"patches.md"})).toBeTruthy();
  await waitFor(() => expect(vi.mocked(ComparisonTools).mock.calls.some(([props]) => props.source.debugSurface?.entry === "patches.md")).toBe(true));
  expect(screen.queryByRole("alert")).toBeNull();
});

it("renders editor captures with frozen context but exposes no native project tools", async () => {
  const view = render(<ChangesetDocument entry="response.md" sources={{"response.md":source}} input={input} tools={false} />);
  await waitFor(() => expect(view.container.querySelector(".comparison-editor")).toBeTruthy());
  expect(vi.mocked(ComparisonTools).mock.calls.every(([props]) => !props.source.project && !props.source.versions && !props.source.debugSurface)).toBe(true);
  expect(view.container.textContent).toContain("new");
});

it("offers tools for disk patches using captured filesystem dependencies, while keeping buffer addresses virtual", async () => {
  const address = "__hickory_editor_buffers__/6162";
  const captured = [...input, file(address,"old\n")];
  const view = render(<ChangesetDocument entry="response.md" sources={{"response.md":source}} input={captured} filesystemOnly />);
  await waitFor(() => expect(view.container.querySelector(".comparison-editor")).toBeTruthy());
  await waitFor(() => expect(vi.mocked(ComparisonTools).mock.calls.some(([props]) => props.source.project?.some(e => e.path === "library.py"))).toBe(true));
  expect(vi.mocked(ComparisonTools).mock.calls.every(([props]) => !props.source.project?.some(e => e.path === address))).toBe(true);
  expect(vi.mocked(ComparisonTools).mock.calls.every(([props]) => !props.source.debugSurface)).toBe(true);
  view.unmount(); vi.clearAllMocks();
  const editorSource = source.replaceAll("app.py",address);
  const editor = render(<ChangesetDocument entry="response.md" sources={{"response.md":editorSource}} input={captured} filesystemOnly />);
  await waitFor(() => expect(editor.container.querySelector(".comparison-editor")).toBeTruthy());
  expect(vi.mocked(ComparisonTools).mock.calls.every(([props]) => !props.source.project && !props.source.versions)).toBe(true);
});

// Guarantee: docs/guarantees/agent/final-changeset-revisions-apply-b-to-c.md
it("forks only New through ordinary regeneration with captured A, retaining the accepted source", async () => {
  const { EditorView } = await import("@codemirror/view");
  const { act } = await import("@testing-library/react");
  const onEdit = vi.fn(), onPending = vi.fn();
  const sources = {"response.md":source};
  const view = render(<ChangesetDocument entry="response.md" sources={sources} input={input} onEdit={onEdit} onPending={onPending} />);
  await waitFor(() => expect(view.container.querySelectorAll(".comparison-side .cm-content")).toHaveLength(2));
  const panes = view.container.querySelectorAll<HTMLElement>(".comparison-side .cm-editor");
  expect(panes[0].querySelector(".cm-content")?.getAttribute("contenteditable")).toBe("false");
  expect(panes[1].querySelector(".cm-content")?.getAttribute("contenteditable")).toBe("true");
  expect(screen.queryByText("Old · authored predecessor")).toBeNull();
  const editor = EditorView.findFromDOM(panes[1])!;
  act(() => editor.dispatch({changes:{from:0,to:3,insert:"revised"}}));
  await waitFor(() => expect(onEdit).toHaveBeenCalled());
  const { materializeProject } = await import("../editor/hickLang");
  const revised = materializeProject("response.md",onEdit.mock.calls[0][0],input);
  expect(new TextDecoder().decode(new Uint8Array(revised.new.find(e => e.path === "app.py")!.bytes))).toBe("revised\n");
  expect(sources["response.md"]).toBe(source);
  await waitFor(() => expect(vi.mocked(ComparisonTools).mock.calls.some(([props]) => props.source.project?.some(e => e.path === "app.py" && new TextDecoder().decode(new Uint8Array(e.bytes)) === "revised\n"))).toBe(true));

  expect(onPending).toHaveBeenCalledWith(true);
  expect(onPending).toHaveBeenLastCalledWith(false);
});

it("retains downstream-breaking draft bytes without publishing a constructed draft", async () => {
  const { EditorView } = await import("@codemirror/view");
  const { act } = await import("@testing-library/react");
  const later = `${source}\n<hick:patch id="later" after="completed">\n--- a/app.py\n+++ b/app.py\n@@ -1 +1 @@\n-new\n+final\n</hick:patch>`;
  const onEdit = vi.fn(), onPending = vi.fn();
  const view = render(<ChangesetDocument entry="response.md" sources={{"response.md":later}} input={input} onEdit={onEdit} onPending={onPending} />);
  await waitFor(() => expect(view.container.querySelectorAll(".comparison-side .cm-editor")).toHaveLength(4));
  const editor = EditorView.findFromDOM(view.container.querySelectorAll<HTMLElement>(".comparison-side .cm-editor")[1])!;
  act(() => editor.dispatch({changes:{from:0,to:3,insert:"broken"}}));
  await waitFor(() => expect(screen.getByRole("alert").textContent).toMatch(/invalidates/));
  expect(onEdit).not.toHaveBeenCalled();
  expect(onPending).toHaveBeenLastCalledWith(true);
});

it.each([false,true])("keeps editing separate from explicit B-to-C editor application and records its outcome (concurrent=%s)", async concurrent => {
  const { EditorView } = await import("@codemirror/view");
  const { act, fireEvent } = await import("@testing-library/react");
  const address = "__hickory_editor_buffers__/627566";
  const sources = {"response.md":source.replaceAll("app.py",address)};
  const snapshot = (text:string) => ({entries:{[address]:{bytes:[...new TextEncoder().encode(text)],kind:"File",executable:false}}});
  const posts: {path:string;body:unknown}[] = [];
  let release!: () => void;
  const application = new Promise<void>(resolve => { release = resolve; });
  installMockHandler(async (method,path,body) => {
    if (method === "POST") posts.push({path,body});
    if (path.endsWith("/draft/apply")) { await application; return {constructed:true,runtime:"unrun",applied:[],error:null,undo:[],editors:[{id:"revision",buffer:"buf",name:"buf",path:null,oldText:"new\n",newText:"C\n",editor:true,status:"applying"}]}; }
    if (path.endsWith("/draft/result")) return {recorded:true};
    return {entry:"response.md",artifact:{entry:"response.md",sources},response:sources["response.md"],starting:snapshot("old\n"),completed:snapshot("new\n"),workspace:{root:null,identities:{buf:null}},ambiguous:[],error:null};
  });
  const applyChange = vi.fn(() => { if (concurrent) throw new Error("Concurrent editor edit; preserve newer work"); }), getContext = vi.fn(async () => ({buffers:[{id:"buf",name:"Untitled",path:null,content:"new\n",focused:true}]}));
  const view = render(<AgentProject doc="workspace" turn="turn" getContext={getContext} applyChange={applyChange} />);
  fireEvent.click(await screen.findByRole("button",{name:"Revise New"}));
  const draft = screen.getByRole("region",{name:"Authored final changeset draft"});
  await waitFor(() => expect(draft.querySelectorAll(".comparison-side .cm-editor")).toHaveLength(2));
  const code = EditorView.findFromDOM(draft.querySelectorAll<HTMLElement>(".comparison-side .cm-editor")[1])!;
  act(() => code.dispatch({changes:{from:0,to:3,insert:"C"}}));
  await waitFor(() => expect(draft.textContent).toContain("C construction succeeded"));
  expect(posts).toHaveLength(0);
  expect(applyChange).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button",{name:"Apply revision"}));
  await waitFor(() => expect(posts.some(p => p.path.endsWith("/draft/apply"))).toBe(true));
  expect(draft.querySelectorAll(".comparison-side .cm-content")[1].getAttribute("contenteditable")).toBe("false");
  act(() => release());
  await waitFor(() => expect(applyChange).toHaveBeenCalledWith(expect.objectContaining({oldText:"new\n",newText:"C\n",path:null})));
  await waitFor(() => expect(posts.some(p => p.path.endsWith("/draft/result"))).toBe(true));
  if (concurrent) expect(screen.getByRole("alert").textContent).toContain("Concurrent editor edit");
  else expect(screen.getByText(/Revision application succeeded/).textContent).toContain("Runtime checks remain unrun");
  expect(sources["response.md"]).toContain("+new");
  view.unmount();
});

// Guarantee: docs/guarantees/agent/final-changeset-revisions-apply-b-to-c.md
it("regenerates an included patch in its own draft source while preserving the complete closure", async () => {
  const { EditorView } = await import("@codemirror/view");
  const { act } = await import("@testing-library/react");
  const sources = {"response.md":"<hick:project sources=\"patches.md\"/>","patches.md":source.replace("<hick:project/>","")};
  const onEdit = vi.fn();
  const view = render(<ChangesetDocument sources={sources} entry="response.md" input={input} onEdit={onEdit} />);
  await waitFor(() => expect(view.container.querySelectorAll(".comparison-side .cm-editor")).toHaveLength(2));
  const editor = EditorView.findFromDOM(view.container.querySelectorAll<HTMLElement>(".comparison-side .cm-editor")[1])!;
  act(() => editor.dispatch({changes:{from:0,to:3,insert:"included C"}}));
  await waitFor(() => expect(onEdit).toHaveBeenCalled());
  const revised = onEdit.mock.calls[0][0];
  expect(revised["response.md"]).toBe(sources["response.md"]);
  expect(revised["patches.md"]).toContain("+included C");
  expect(sources["patches.md"]).toContain("+new");
  const { materializeProject } = await import("../editor/hickLang");
  expect(materializeProject("response.md",revised,input).new.map(e=>e.path)).toEqual(["app.py","library.py"]);
  expect(screen.queryByRole("alert")).toBeNull();
});
</book:entry>

</book:doc>
