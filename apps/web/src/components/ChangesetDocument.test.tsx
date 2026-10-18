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
