<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/WorkspaceChat.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-41ee32c19327"></a>
## `apps/web/src/views/WorkspaceChat.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Selected reading scope remains separate from complete edit state](../../../../../guarantees/agent/expanded-reading-projection/contract.md), [Select model context without saving open buffers](../../../../../guarantees/agent/the-agent-sees-the-open-editors/contract.md), [Unsent responses survive restart](../../../../../guarantees/agent/unsent-responses-survive-restart/contract.md).

<a id="f-41ee32c19327-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-41ee32c19327-expression_statement">
import { codeFolding, foldEffect } from "@codemirror/language";
// @ts-expect-error Node runtime crypto is available to Vitest without browser Node typings.
import { webcrypto } from "node:crypto";
import { bufferTarget } from "../lib/editorBufferTargets";
import { responseStart } from "../editor/protectedPrefix";
import { afterEach, expect, it, vi } from "vitest";
import { render, screen, fireEvent, waitFor, cleanup, act } from "@testing-library/react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorContext, WorkspaceChat } from "./WorkspaceChat";
import { SessionRegistry } from "./documentSession";
import { initialWorkspace, openUntitledTab } from "./workspaceState";
import { panes, tab } from "../shell/layout";
import { api } from "../api/client";
import { registerSearchEditor } from "../lib/workspaceSearch";
vi.mock("../api/client", () => ({ request: vi.fn().mockImplementation(async (_method, path: string) => path.endsWith("/agent/python/context") ? { items: [], updates: [], coverage: "No retained references in this fixture" } : {}), api: { responseDraft: vi.fn().mockResolvedValue({ scope: "test", state: null }), saveResponseDraft: vi.fn().mockResolvedValue({ ok: true }), agent: vi.fn(), agentTurns: vi.fn(), file: vi.fn(), executor: vi.fn().mockResolvedValue({}), complete: vi.fn().mockResolvedValue({ suggestions: [] }) } }));
</storycode:copy>
<a id="f-41ee32c19327-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-41ee32c19327-expression_statement-2">
vi.mock("../api/acp", () => ({ acpApi: { state: vi.fn(async () => ({backend: "python", ready: true, edits: {mode: "review", changes: []}})), agents: vi.fn().mockResolvedValue({ agents: [] }) } }));
</storycode:copy>
<a id="f-41ee32c19327-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-41ee32c19327-expression_statement-3">
vi.mock("../api/realtime", async importOriginal => ({ ...await importOriginal<typeof import("../api/realtime")>(), getWorkspaceRealtime: () => ({ onRunEvent: () => () => {} }) }));
</storycode:copy>
<a id="f-41ee32c19327-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-41ee32c19327-expression_statement-4">
vi.stubGlobal("crypto", webcrypto);
</storycode:copy>
<a id="f-41ee32c19327-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-41ee32c19327-expression_statement-5">
afterEach(() => { cleanup(); vi.clearAllMocks(); localStorage.clear(); });
</storycode:copy>
<a id="f-41ee32c19327-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-41ee32c19327-expression_statement-6">
// Guarantee: docs/guarantees/agent/the-agent-sees-the-open-editors.md
it("sends a startup untitled editor without a folder, using fresh text", async () => {
  vi.mocked(api.agentTurns).mockResolvedValue({ turns: [], provider: "anthropic", model: "claude-sonnet-5", totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
  vi.mocked(api.agent).mockResolvedValue({ session_id: "turn" });
  const layout = openUntitledTab(initialWorkspace());
  const untitled = panes(layout.root).flatMap(pane => pane.tabs).find(tab => tab.kind === "untitled")!;
  const source = { current: { [untitled.id]: "original" } };
  render(<WorkspaceChat layout={layout} registry={new SessionRegistry()} untitledSources={source}
    plainSources={{ current: new Map() }} folder={null} focusedEditor={untitled.id} onOpenSession={() => {}} />);
  expect(screen.queryByText("Open a document to talk to the agent about it.")).toBeNull();
  expect(screen.getByText(/Context:.*unsaved/)).toBeTruthy();
  source.current[untitled.id] = "latest unsaved text";
  await waitFor(() => expect(api.agentTurns).toHaveBeenCalled());
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const editor = EditorView.findFromDOM(host)!;
  act(() => editor.dispatch({ changes: { from: editor.state.field(responseStart), to: editor.state.doc.length, insert: "summarize" }, userEvent: "input.type" }));
  fireEvent.click(screen.getByRole("button", { name: "Send" }));
  await waitFor(() => expect(api.agent).toHaveBeenCalledWith("workspace", "summarize", null,
    expect.anything(), expect.anything(), "python", { live: true, projection: { version: 1, documents: [], observations: [], estimate: expect.anything() }, buffers: [{ id: untitled.id, kind: "untitled", provenance_uri: bufferTarget(untitled.id), name: untitled.title ?? untitled.target,
      path: null, observations: [], content: "latest unsaved text", focused: true }] }));
});
</storycode:copy>
<a id="f-41ee32c19327-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-41ee32c19327-expression_statement-7">
// Guarantee: docs/guarantees/agent/the-agent-sees-the-open-editors.md
it("includes visible unsaved files, inactive tabs, missing files, and untitled text together", async () => {
  const view = new EditorView({ state: EditorState.create({ doc: "live unsaved" }) });
  const unregister = registerSearchEditor("/outside/file.ts", view);
  vi.mocked(api.file).mockResolvedValue({ path: "inactive.txt", content: "disk fallback", hash: "h", language: "text" });
  try {
    const files = [tab("file", "/outside/file.ts"), tab("file", "deleted.txt"), tab("file", "inactive.txt"), tab("untitled", "Untitled")];
    const result = await editorContext(files, files[0].id, new SessionRegistry(), { [files[3].id]: "draft" },
      new Map([["/outside/file.ts", "stale"], [files[1].id, "still open"]]));
    expect(result.buffers.map(buffer => buffer.content)).toEqual(["live unsaved", "still open", "disk fallback", "draft"]);
    expect(result.buffers[0].focused).toBe(true);
    expect(result.buffers[3].path).toBeNull();
    expect(api.file).toHaveBeenCalledTimes(1);
  } finally { unregister(); view.destroy(); }
});
</storycode:copy>
<a id="f-41ee32c19327-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-41ee32c19327-expression_statement-8">
// Guarantee: docs/guarantees/agent/the-agent-sees-the-open-editors.md
it("reads saved documents and generated files from their latest editors", async () => {
  const registry = new SessionRegistry();
  const doc = tab("document", "notes.md"); doc.docId = "doc";
  const output = tab("generated", "output.py"); output.docId = "doc";
  const docView = new EditorView({ state: EditorState.create({ doc: "unsaved note" }) });
  const outputView = new EditorView({ state: EditorState.create({ doc: "unsaved output" }) });
  vi.spyOn(registry, "get").mockReturnValue({
    docEditor: docView, liveSource: "old note", openOutputs: new Map([["output.py", outputView]]),
  } as unknown as ReturnType<SessionRegistry["get"]>);
  try {
    const result = await editorContext([doc, output], output.id, registry, {}, new Map());
    expect(result.buffers.map(buffer => buffer.content)).toEqual(["unsaved note", "unsaved output"]);
    expect(result.buffers.map(buffer => buffer.focused)).toEqual([false, true]);
    expect(api.file).not.toHaveBeenCalled();
  } finally { docView.destroy(); outputView.destroy(); }
});
</storycode:copy>
<a id="f-41ee32c19327-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-41ee32c19327-expression_statement-9">
// Guarantee: docs/guarantees/agent/the-agent-sees-the-open-editors.md
it("keeps distinct buffers when two panes show the same file", async () => {
  const left = tab("file", "same.txt"), right = tab("file", "same.txt");
  const result = await editorContext([left, right], right.id, new SessionRegistry(), {},
    new Map([[left.id, "left changes"], [right.id, "right changes"]]));
  expect(result.buffers.map(buffer => buffer.content)).toEqual(["left changes", "right changes"]);
  expect(result.buffers.map(buffer => buffer.focused)).toEqual([false, true]);
});

</storycode:copy>
<a id="f-41ee32c19327-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-41ee32c19327-expression_statement-10">
// Guarantee: docs/guarantees/agent/unsent-responses-survive-restart.md
it("restores the folderless workspace response across distinct unnamed editor identities", async () => {
  vi.mocked(api.responseDraft).mockResolvedValueOnce({ scope: "blank-window", state: { version: 1, tip: null, drafts: { new: "Recovered unnamed-editor conversation" } } });
  vi.mocked(api.agentTurns).mockResolvedValue({ turns: [], provider: "anthropic", model: "claude-sonnet-5", totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
  const layout = openUntitledTab(openUntitledTab(initialWorkspace(), true), true);
  const unnamed = panes(layout.root).flatMap(pane => pane.tabs).filter(tab => tab.kind === "untitled");
  expect(unnamed[0].id).not.toBe(unnamed[1].id);
  const props = { layout, registry: new SessionRegistry(), untitledSources: { current: {} }, plainSources: { current: new Map<string, string>() }, folder: null, onOpenSession: () => {} };
  const view = render(<WorkspaceChat {...props} focusedEditor={unnamed[0].id} />);
  await waitFor(() => expect(screen.getByRole("textbox", { name: "Agent conversation and response" }).textContent).toContain("Recovered unnamed-editor conversation"));
  view.rerender(<WorkspaceChat {...props} focusedEditor={unnamed[1].id} />);
  expect(screen.getByRole("textbox", { name: "Agent conversation and response" }).textContent).toContain("Recovered unnamed-editor conversation");
  expect(api.responseDraft).toHaveBeenLastCalledWith("workspace");
});

</storycode:copy>
<a id="f-41ee32c19327-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-41ee32c19327-expression_statement-11">
// Guarantee: docs/guarantees/agent/expanded-reading-projection.md
it("previews only explicitly selected unsaved text and lets a user pin it", async () => {
  vi.mocked(api.agentTurns).mockResolvedValue({ turns: [], provider: "anthropic", model: "", totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
  const layout = openUntitledTab(initialWorkspace());
  const draft = panes(layout.root).flatMap(pane => pane.tabs).find(t => t.kind === "untitled")!;
  render(<WorkspaceChat layout={layout} registry={new SessionRegistry()} untitledSources={{ current: { [draft.id]: "unsaved selected text" } }} plainSources={{ current: new Map() }} folder={null} focusedEditor={draft.id} onOpenSession={() => {}} />);
  fireEvent.click(screen.getByText("Next context projection"));
  fireEvent.click(screen.getByRole("checkbox", { name: "Untitled" }));
  fireEvent.click(screen.getByRole("checkbox", { name: "Pin full text" }));
  fireEvent.click(screen.getByRole("button", { name: "Refresh preview" }));
  await waitFor(() => expect(screen.getByLabelText("Context projection preview").textContent).toContain("unsaved selected text"));
  fireEvent.click(screen.getByRole("checkbox", { name: "Untitled" }));
  fireEvent.click(screen.getByRole("button", { name: "Refresh preview" }));
  await waitFor(() => expect(screen.getByLabelText("Context projection preview").textContent).not.toContain("unsaved selected text"));
});

</storycode:copy>
<a id="f-41ee32c19327-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-41ee32c19327-expression_statement-12">
// Guarantee: docs/guarantees/agent/expanded-reading-projection.md
it("captures folded and pinned documents independently of focus and scrolling", async () => {
  const registry = new SessionRegistry();
  const doc = tab("document", "notes.md"); doc.docId = "doc";
  const draft = tab("untitled", "Draft");
  const content = "heading\nhidden\nvisible";
  const view = new EditorView({ state: EditorState.create({ doc: content, extensions: [codeFolding()] }) });
  view.dispatch({ effects: foldEffect.of({ from: 7, to: 14 }) });
  vi.spyOn(registry, "get").mockReturnValue({ docEditor: view, liveSource: "old", openOutputs: new Map() } as unknown as ReturnType<SessionRegistry["get"]>);
  try {
    const capture = (focus: string) => editorContext([doc, draft], focus, registry, { [draft.id]: "unsaved pin" }, new Map(), { [doc.id]: "expanded", [draft.id]: "pinned" });
    const first = await capture(doc.id);
    view.scrollDOM.dispatchEvent(new Event("scroll"));
    const next = await capture(draft.id);
    expect(next.projection).toEqual(first.projection);
    expect(next.projection!.documents[0].segments.some(s => s.omitted)).toBe(true);
    expect(next.projection!.documents[0].segments.map(s => s.text).join("")).not.toContain("hidden");
    expect(next.projection!.documents[1].segments[0].text).toBe("unsaved pin");
    expect(next.buffers[0].content).toBe(content);
  } finally { view.destroy(); }
});
</storycode:copy>

### Assemble `apps/web/src/views/WorkspaceChat.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/WorkspaceChat.test.tsx"><storycode:paste select="#f-41ee32c19327-expression_statement"/><storycode:paste select="#f-41ee32c19327-expression_statement-2"/><storycode:paste select="#f-41ee32c19327-expression_statement-3"/><storycode:paste select="#f-41ee32c19327-expression_statement-4"/><storycode:paste select="#f-41ee32c19327-expression_statement-5"/><storycode:paste select="#f-41ee32c19327-expression_statement-6"/><storycode:paste select="#f-41ee32c19327-expression_statement-7"/><storycode:paste select="#f-41ee32c19327-expression_statement-8"/><storycode:paste select="#f-41ee32c19327-expression_statement-9"/><storycode:paste select="#f-41ee32c19327-expression_statement-10"/><storycode:paste select="#f-41ee32c19327-expression_statement-11"/><storycode:paste select="#f-41ee32c19327-expression_statement-12"/></storycode:file>

</storycode:doc>
