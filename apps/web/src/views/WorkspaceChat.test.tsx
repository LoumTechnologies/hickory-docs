import { afterEach, expect, it, vi } from "vitest";
import { render, screen, fireEvent, waitFor, cleanup } from "@testing-library/react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorContext, WorkspaceChat } from "./WorkspaceChat";
import { SessionRegistry } from "./documentSession";
import { initialWorkspace, openUntitledTab } from "./workspaceState";
import { panes, tab } from "../shell/layout";
import { api } from "../api/client";
import { registerSearchEditor } from "../lib/workspaceSearch";
vi.mock("../api/client", () => ({ api: { agent: vi.fn(), agentTurns: vi.fn(), file: vi.fn() } }));
vi.mock("../api/acp", () => ({ acpApi: { agents: vi.fn().mockResolvedValue({ agents: [] }) } }));
vi.mock("../api/realtime", () => ({ getWorkspaceRealtime: () => ({ onRunEvent: () => () => {} }) }));
afterEach(() => { cleanup(); vi.clearAllMocks(); localStorage.clear(); });
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
  fireEvent.change(screen.getByPlaceholderText("Ask the agent…"), { target: { value: "summarize" } });
  fireEvent.click(screen.getByRole("button", { name: "Send" }));
  await waitFor(() => expect(api.agent).toHaveBeenCalledWith("workspace", "summarize", null,
    expect.anything(), expect.anything(), "builtin", { buffers: [{ name: untitled.title ?? untitled.target,
      path: null, content: "latest unsaved text", focused: true }] }));
});
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
// Guarantee: docs/guarantees/agent/the-agent-sees-the-open-editors.md
it("keeps distinct buffers when two panes show the same file", async () => {
  const left = tab("file", "same.txt"), right = tab("file", "same.txt");
  const result = await editorContext([left, right], right.id, new SessionRegistry(), {},
    new Map([[left.id, "left changes"], [right.id, "right changes"]]));
  expect(result.buffers.map(buffer => buffer.content)).toEqual(["left changes", "right changes"]);
  expect(result.buffers.map(buffer => buffer.focused)).toEqual([false, true]);
});
