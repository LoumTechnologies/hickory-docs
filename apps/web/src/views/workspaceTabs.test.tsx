// @vitest-environment jsdom
//
// Document-wide actions live in the pane header's menu and target its session.
// Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md

import { EditorView } from "@codemirror/view";
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render } from "@testing-library/react";

import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import type { Doc } from "../api/types";
import { api } from "../api/client";
import { SessionRegistry, type DocSession } from "./documentSession";
import { DocTabBody, GeneratedTabBody, UntitledTab } from "./workspaceTabs";

const readyEditor = vi.hoisted(() => ({ view: null as import("@codemirror/view").EditorView | null }));

// The editor and debugger chrome are heavyweight and irrelevant here; the
// toolbar sits beside them in DocTabBody, not inside them.
vi.mock("../editor/DocumentEditor", () => ({
  DocumentEditor: ({ onChange, wrapColumn, onWrapColumn, onViewReady }: {
    onViewReady?: (view: EditorView | null) => void;
    onChange?: (source: string) => void;
    wrapColumn?: number;
    onWrapColumn?: (column: number) => void;
  }) => (
    <>
      <button data-testid="ready" onClick={() => onViewReady?.(readyEditor.view)} />
      <button data-testid="editor" onClick={() => onChange?.("first thought")} />
      <button data-testid="wrap" onClick={() => onWrapColumn?.(60)}>{wrapColumn}</button>
    </>
  ),
}));
vi.mock("../debug/DebugStrip", () => ({
  DebugStrip: () => <div data-testid="debug-strip" />,
}));
vi.mock("../shell/views", () => ({
  GeneratedFileView: () => <div data-testid="generated" />,
}));

afterEach(cleanup);

const doc: Doc = {
  id: "d1",
  path: "notes/demo.md",
  source: "<hick:doc></hick:doc>",
} as Doc;

/** A published session with just enough shape for the tab bodies. */
function makeSession(overrides: Partial<DocSession> = {}): DocSession {
  return {
    docId: "d1",
    doc,
    liveSource: doc.source,
    startDebugFile: vi.fn(),
    blocks: [],
    fatalError: null,
    renderError: null,
    banner: null,
    syncState: "idle",
    runningCells: new Set<string>(),
    execBlocks: [],
    outputs: new Map(),
    openOutputs: new Map(),
    docEditor: null,
    debug: {},
    lspExtensions: [],
    runAll: vi.fn(),
    verify: vi.fn(),
    runCell: vi.fn(),
    ...overrides,
  } as unknown as DocSession;
}

function mount(session: DocSession) {
  const registry = new SessionRegistry();
  registry.publish(session);
  return render(<DocTabBody registry={registry} docId={session.docId} />);
}

describe("the document tab's toolbar", () => {
  // Protects docs/guarantees/editor-intelligence/the-toolbar-uses-words-an-ide-user-knows.md

  it("offers Run and Test, and nothing a traditional IDE would misread", () => {
    const { getByRole, queryByRole } = mount(makeSession());
    const toolbar = getByRole("toolbar", { name: "Actions for notes/demo.md" });
    expect(toolbar.className).toBe("doc-tab-toolbar");
    expect(queryByRole("button", { name: "Run" })).toBeNull();
    fireEvent.click(getByRole("button", { name: "More actions for this document" }));
    expect(getByRole("menuitem", { name: "Run" })).toBeTruthy();
    // `hick test` is the command; the button says the same word, so each one
    // is findable from the other.
    expect(getByRole("menuitem", { name: "Test" })).toBeTruthy();
    expect(queryByRole("button", { name: "Verify" })).toBeNull();
    // "Refactor" means rename/extract in every other editor. It named a mode
    // here, and a mode belongs in a menu.
    expect(queryByRole("button", { name: "Refactor" })).toBeNull();
  });

  it("keeps the rare document-wide action in a menu", () => {
    const { getByRole, getByText } = mount(makeSession());
    fireEvent.click(getByRole("button", { name: "More actions for this document" }));
    // Named by what it produces, not by a mode nobody can search for.
    expect(getByText("Pin outputs as a baseline")).toBeTruthy();
  });

  it("calls THIS session's runAll and verify", () => {
    const session = makeSession();
    const { getByRole } = mount(session);
    fireEvent.click(getByRole("button", { name: "More actions for this document" }));
    fireEvent.click(getByRole("menuitem", { name: "Run" }));
    expect(session.runAll).toHaveBeenCalledTimes(1);
    fireEvent.click(getByRole("button", { name: "More actions for this document" }));
    fireEvent.click(getByRole("menuitem", { name: "Test" }));
    expect(session.verify).toHaveBeenCalledTimes(1);
  });

  it("disables both buttons while the document's run is in flight", () => {
    const session = makeSession({ runningCells: new Set(["cell-1"]) });
    const { getByRole } = mount(session);
    fireEvent.click(getByRole("button", { name: "More actions for this document" }));
    const run = getByRole("menuitem", { name: "Running…" }) as HTMLButtonElement;
    const test = getByRole("menuitem", { name: "Test" }) as HTMLButtonElement;
    expect(run.disabled).toBe(true);
    expect(test.disabled).toBe(true);
    fireEvent.click(run);
    fireEvent.click(test);
    expect(session.runAll).not.toHaveBeenCalled();
    expect(session.verify).not.toHaveBeenCalled();
  });

  it("gives a generated-file tab no toolbar — the actions are the document's", () => {
    const registry = new SessionRegistry();
    registry.publish(makeSession());
    const { queryByRole } = render(
      <GeneratedTabBody registry={registry} docId="d1" path="src/hello.py" />,
    );
    expect(queryByRole("toolbar")).toBeNull();
  });
});

describe("an untitled document", () => {
  // Guarantee: docs/guarantees/authoring/the-measure-says-what-it-measures.md
  it("passes the stored prose measure and ruler changes through to its editor", () => {
    const onWrapColumn = vi.fn();
    const { getByTestId, rerender } = render(
      <UntitledTab tabId="new-note" wrapColumn={72} onWrapColumn={onWrapColumn} />,
    );
    expect(getByTestId("wrap").textContent).toBe("72");
    fireEvent.click(getByTestId("wrap"));
    expect(onWrapColumn).toHaveBeenCalledWith(60);
    rerender(<UntitledTab tabId="new-note" wrapColumn={60} onWrapColumn={onWrapColumn} />);
    expect(getByTestId("wrap").textContent).toBe("60");
  });

  // Guarantee: docs/guarantees/authoring/new-document-is-an-act.md
  it("does not create a project file on its first keystroke", () => {
    const create = vi.spyOn(api, "createDoc");
    const { getByTestId } = render(<UntitledTab tabId="new-note" />);
    fireEvent.click(getByTestId("editor"));
    expect(create).not.toHaveBeenCalled();
    create.mockRestore();
  });

  it("never restores a prior unnamed draft into a new document", () => {
    const drafts = vi.spyOn(api, "drafts").mockResolvedValue({
      drafts: [
        {
          path: "untitled:closed-tab",
          contents: "a thought that still matters",
          base: "",
          saved_at: 42,
        },
      ],
    });
    render(<UntitledTab tabId="reopened-tab" />);

    expect(drafts).not.toHaveBeenCalled();

    drafts.mockRestore();
  });
});

// Guarantee: docs/guarantees/authoring/a-folder-remembers-its-window.md
it("does not focus an unnamed editor that finishes loading in a hidden pane", () => {
  const ui = render(<div hidden><UntitledTab tabId="background-note" /></div>);
  const parent = document.createElement("div");
  ui.container.querySelector(".untitled-tab")!.appendChild(parent);
  const view = new EditorView({ parent });
  const focus = vi.spyOn(view, "focus");
  readyEditor.view = view;
  fireEvent.click(ui.getByTestId("ready"));
  expect(focus).not.toHaveBeenCalled();
  ui.rerender(<div><UntitledTab tabId="background-note" /></div>);
  fireEvent.click(ui.getByTestId("ready"));
  expect(focus).toHaveBeenCalledOnce();
  readyEditor.view = null;
  view.destroy();
});

it("shares the document header launcher and disclosed menu callback with agent requests", async () => {
  const session = makeSession(), sessions = new SessionRegistry(); sessions.publish(session);
  const registry = new UiActions({ workspace: "p", window: "doc-menu" }, () => true, async () => {});
  const mounted = render(<UiActionContext.Provider value={registry}><DocTabBody registry={sessions} docId="d1" /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "acp:tabs", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: {}, request_id: String(++id) }); }); return reply; };
  expect(actions().some(action => action.label === "Run")).toBe(false);
  await run("More actions for this document");
  expect(mounted.getByRole("menuitem", { name: "Run" })).toBeTruthy();
  const old = actions().find(action => action.label === "Run")!;
  expect(await run("Run")).toMatchObject({ outcome: { status: "completed" } });
  expect(session.runAll).toHaveBeenCalledTimes(1);
  expect(await registry.act(registry.scope, { id: "acp:tabs", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old" })).not.toMatchObject({ outcome: { status: "completed" } });
});

it("reads actual loading/fatal states before the document editor exists without including source", async () => {
  const sessions = new SessionRegistry(), registry = new UiActions({ workspace: "p", window: "loading" }, () => true, async () => {});
  const mounted = render(<UiActionContext.Provider value={registry}><DocTabBody registry={sessions} docId="d1" /></UiActionContext.Provider>);
  let id = 0;
  const read = async () => { const action = registry.groups().groups.flatMap(group => registry.observe(group, 64).actions).find(action => action.label === "Read document tab load state bytes")!; let reply: any; await act(async () => { reply = await registry.act(registry.scope, { id: "acp:status", kind: "agent" }, { handle: action.handle, arguments: { offset: 0, limit: 8192 }, request_id: String(++id) }); }); return new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes)); };
  expect(JSON.parse(await read())).toMatchObject({ phase: "loading", message: "Loading document…" });
  await act(async () => sessions.publish(makeSession({ fatalError: "Document cannot be opened", liveSource: "NEVER INCLUDE SOURCE" })));
  expect(mounted.getByText("Document cannot be opened")).toBeTruthy();
  const failed = await read(); expect(JSON.parse(failed)).toMatchObject({ phase: "failed", message: "Document cannot be opened" });
  expect(failed).not.toContain("NEVER INCLUDE SOURCE");
});
