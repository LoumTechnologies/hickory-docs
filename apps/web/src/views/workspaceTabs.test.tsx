// @vitest-environment jsdom
//
// The per-document toolbar: Run and Verify moved OFF the workspace toolbar
// and into each document tab's own strip, so every open document can be run
// or verified independently. These tests pin down:
//  - a document tab renders the toolbar, wired to ITS session's actions;
//  - both buttons are disabled while that document's run is in flight;
//  - a generated-file tab gets NO toolbar (the actions belong to the
//    document, and the workspace toolbar no longer carries them).

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/react";

import type { Doc } from "../api/types";
import { SessionRegistry, type DocSession } from "./documentSession";
import { DocTabBody, GeneratedTabBody } from "./workspaceTabs";

// The editor and debugger chrome are heavyweight and irrelevant here; the
// toolbar sits beside them in DocTabBody, not inside them.
vi.mock("../editor/DocumentEditor", () => ({
  DocumentEditor: () => <div data-testid="editor" />,
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
  path: "notes/demo.hick",
  source: "<hick:doc></hick:doc>",
} as Doc;

/** A published session with just enough shape for the tab bodies. */
function makeSession(overrides: Partial<DocSession> = {}): DocSession {
  return {
    docId: "d1",
    doc,
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
  it("renders Run and Verify at the top of the tab", () => {
    const { getByRole } = mount(makeSession());
    const toolbar = getByRole("toolbar", { name: "Actions for notes/demo.hick" });
    expect(toolbar.className).toBe("doc-tab-toolbar");
    expect(getByRole("button", { name: "Run" })).toBeTruthy();
    expect(getByRole("button", { name: "Verify" })).toBeTruthy();
  });

  it("calls THIS session's runAll and verify", () => {
    const session = makeSession();
    const { getByRole } = mount(session);
    fireEvent.click(getByRole("button", { name: "Run" }));
    expect(session.runAll).toHaveBeenCalledTimes(1);
    fireEvent.click(getByRole("button", { name: "Verify" }));
    expect(session.verify).toHaveBeenCalledTimes(1);
  });

  it("disables both buttons while the document's run is in flight", () => {
    const session = makeSession({ runningCells: new Set(["cell-1"]) });
    const { getByRole } = mount(session);
    const run = getByRole("button", { name: "Running…" }) as HTMLButtonElement;
    const verify = getByRole("button", { name: "Verify" }) as HTMLButtonElement;
    expect(run.disabled).toBe(true);
    expect(verify.disabled).toBe(true);
    fireEvent.click(run);
    fireEvent.click(verify);
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
