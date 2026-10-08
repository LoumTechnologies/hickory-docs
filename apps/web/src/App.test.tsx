import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { EditorView } from "@codemirror/view";
import { App } from "./App";
import { newDocument } from "./router";
import { api } from "./api/client";
import { installMockApi } from "./mock/mockApi";
import { STARTUP_INTRODUCTION } from "./lib/newDoc";
import { WELCOME_KEY } from "./lib/welcomePref";
import { openChatTab, openDocTab, initialWorkspace } from "./views/workspaceState";
import { tab, withTree } from "./shell/layout";
import { wrapColumnOf } from "./editor/wrapColumn";

// Keep this fixture wrapped so opening it exercises the clean-introduction
// guarantee independently of edits to the bundled startup copy.
vi.mock("./content/startup.md?raw", () => ({ default: "# Hickory Docs\n\nA wrapped introduction\ncontinues on another line.\n" }));

beforeEach(() => {
  window.history.replaceState(null, "", "/");
  localStorage.setItem(WELCOME_KEY, "1");
  installMockApi();
  const previous = openDocTab(openChatTab(withTree(initialWorkspace(), tab("tree", "folder", "Files"))), "old-doc", "old.md");
  vi.spyOn(api, "workspaceUi").mockResolvedValue({ state: { version: 1, layout: previous } });
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  localStorage.clear();
});

function editor(container: HTMLElement): EditorView {
  const element = container.querySelector(".untitled-tab .cm-editor");
  if (!element) throw new Error("no untitled editor");
  return EditorView.findFromDOM(element as HTMLElement)!;
}

describe("the startup workspace", () => {
  // Guarantee: docs/guarantees/collaboration/a-missing-merge-driver-is-a-button.md
  it("does not check merge-driver setup when no folder is open", async () => {
    const files = await api.files();
    vi.spyOn(api, "files").mockResolvedValue({ ...files, folder_open: false });
    const check = vi.spyOn(api, "mergeDriver");
    const { container, getByRole } = render(<App />);
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "show-agent" }));
    await waitFor(() => expect(getByRole("tab", { name: /Agent/ })).toBeTruthy());
    expect(check).not.toHaveBeenCalled();
    expect(container.textContent).not.toContain("Run hick init");
  });

  // Guarantee: docs/guarantees/authoring/the-measure-says-what-it-measures.md
  it("keeps a dragged prose margin in the initial untitled document and stores it", async () => {
    vi.mocked(api.workspaceUi).mockResolvedValue({ state: { version: 1, wrap: { untitled: 72 } } });
    const save = vi.spyOn(api, "saveWorkspaceUi");
    const { container, getByRole } = render(<App />);
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    // The Files pane rearranges the layout during hydration and remounts the
    // initial editor. Wait for that pane before retaining a live view.
    const live = await waitFor(() => {
      getByRole("tab", { name: /Files/ });
      const view = editor(container);
      expect(view.hasFocus).toBe(true);
      return view;
    });
    const marker = getByRole("slider", { name: "Where prose wraps" });
    await waitFor(() => expect(marker.getAttribute("aria-valuenow")).toBe("72"));
    // jsdom has no layout: supply the geometry read by the real ruler.
    Object.defineProperty(live, "defaultCharacterWidth", { value: 8, configurable: true });
    const requestMeasure = live.requestMeasure;
    live.requestMeasure = (<T,>(request: {
      read: (view: EditorView) => T;
      write?: (value: T, view: EditorView) => void;
    }) => request.write?.(request.read(live), live)) as typeof live.requestMeasure;
    fireEvent(window, new Event("resize"));
    live.requestMeasure = requestMeasure;
    fireEvent(marker, new MouseEvent("pointerdown", { bubbles: true, clientX: 576 }));
    fireEvent(marker, new MouseEvent("pointermove", { bubbles: true, clientX: 480 }));
    expect(marker.getAttribute("aria-valuenow")).toBe("60");
    fireEvent(marker, new MouseEvent("pointerup", { bubbles: true, clientX: 480 }));
    expect(marker.getAttribute("aria-valuenow")).toBe("60");
    expect(wrapColumnOf(live.state)).toBe(60);
    expect(live.state.doc.toString()).toBe(STARTUP_INTRODUCTION);
    await waitFor(() => expect(save).toHaveBeenLastCalledWith(
      expect.objectContaining({ wrap: { untitled: 60 } }),
    ));
  });

  it("opens a focused unsaved introduction despite stored panes and the welcome preference", async () => {
    const create = vi.spyOn(api, "createDoc");
    const saveDialog = vi.spyOn(api, "saveFileDialog").mockResolvedValue({ path: null });
    const { container, getAllByRole, getByRole } = render(<App />);
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    await waitFor(() => expect(editor(container).hasFocus).toBe(true));
    expect(getAllByRole("tab")).toHaveLength(2);
    expect(getByRole("tab", { name: /Files/ })).toBeTruthy();
    expect(getByRole("tab", { name: /Untitled/ }).textContent).toContain("Untitled");
    expect(getByRole("tab", { name: /Untitled/ }).textContent).not.toContain("*");
    expect(container.querySelector(".filesystem-editor")).not.toBeNull();
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "save" }));
    await waitFor(() => expect(saveDialog).toHaveBeenCalledWith("Hickory Docs.md"));
    act(() => editor(container).dispatch({ changes: { from: editor(container).state.doc.length, insert: "My note" } }));
    expect(create).not.toHaveBeenCalled();
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "files" }));
    await waitFor(() => expect(getAllByRole("tab").some((el) => el.textContent?.includes("Files"))).toBe(true));
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "show-agent" }));
    await waitFor(() => expect(getAllByRole("tab").some((el) => el.textContent?.includes("Agent"))).toBe(true));
    expect(editor(container).state.doc.toString()).toContain("My note");
    expect(create).not.toHaveBeenCalled();
  });

  it("creates independent unnamed tabs on every New request and recovers them together", async () => {
    const drafts = new Map<string, { path: string; contents: string; base: string; saved_at: number }>();
    vi.spyOn(api, "drafts").mockImplementation(async () => ({ drafts: [...drafts.values()] }));
    vi.spyOn(api, "saveDraft").mockImplementation(async draft => { drafts.set(draft.path, draft); return { ok: true }; });
    vi.spyOn(api, "discardDraft").mockImplementation(async path => { drafts.delete(path); return { ok: true }; });
    const first = render(<App />);
    await waitFor(() => expect(first.getByRole("tab", { name: /Files/ })).toBeTruthy());
    await waitFor(() => expect(editor(first.container).hasFocus).toBe(true));
    act(() => editor(first.container).dispatch({ changes: { from: 0, to: editor(first.container).state.doc.length, insert: "First note" } }));
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "new" }));
    await waitFor(() => expect(first.getAllByRole("tab", { name: /Untitled/ })).toHaveLength(2));
    await waitFor(() => expect(editor(first.container).state.doc.toString()).toBe(""));
    act(() => editor(first.container).dispatch({ changes: { from: 0, insert: "Second note" } }));
    act(() => newDocument());
    await waitFor(() => expect(first.getAllByRole("tab", { name: /Untitled/ })).toHaveLength(3));
    fireEvent(window, new Event("pagehide"));
    await waitFor(() => expect([...drafts.values()].map(draft => draft.contents).sort()).toEqual(["First note", "Second note"]));
    first.unmount();
    window.history.replaceState(null, "", "/");
    const second = render(<App />);
    await waitFor(() => expect(second.getAllByRole("tab", { name: /Untitled/ })).toHaveLength(2));
    await waitFor(() => expect(second.getByRole("tab", { name: /Files/ })).toBeTruthy());
    const restored = new Set<string>();
    for (const tab of second.getAllByRole("tab", { name: /Untitled/ })) {
      fireEvent.click(tab);
      await waitFor(() => expect(editor(second.container)).toBeTruthy());
      restored.add(editor(second.container).state.doc.toString());
    }
    expect(restored.has("First note")).toBe(true);
    expect(restored.has("Second note")).toBe(true);
    await waitFor(() => expect(second.getByRole("tab", { name: /Files/ })).toBeTruthy());
  });

  it("restores unnamed documents without adding the startup introduction on each launch", async () => {
    vi.spyOn(api, "files").mockResolvedValue({ folder_open: false, root: "parent", tree: [] });
    const drafts = new Map([
      ["untitled:first", { path: "untitled:first", contents: "A note begun blank", base: "", saved_at: 1 }],
      ["untitled:second", { path: "untitled:second", contents: "# Older introduction\nMy edits", base: "# Older introduction\n", saved_at: 2 }],
    ]);
    vi.spyOn(api, "drafts").mockImplementation(async () => ({ drafts: [...drafts.values()] }));
    vi.spyOn(api, "saveDraft").mockImplementation(async draft => { drafts.set(draft.path, draft); return { ok: true }; });
    vi.spyOn(api, "discardDraft").mockImplementation(async path => { drafts.delete(path); return { ok: true }; });
    for (let launch = 0; launch < 3; launch++) {
      window.history.replaceState(null, "", "/");
      const app = render(<App />);
      await waitFor(() => expect(app.getAllByRole("tab", { name: /Untitled/ })).toHaveLength(2));
      const restored = new Set<string>();
      for (const tab of app.getAllByRole("tab", { name: /Untitled/ })) {
        fireEvent.click(tab);
        await waitFor(() => expect(editor(app.container).state.doc.toString()).not.toBe(""));
        restored.add(editor(app.container).state.doc.toString());
      }
      expect(restored).toEqual(new Set(["A note begun blank", "# Older introduction\nMy edits"]));
      fireEvent(window, new Event("pagehide"));
      app.unmount();
      expect(drafts.size).toBe(2);
    }
  });

  it("opens a blank document through New after the startup document is discarded", async () => {
    const { container, getByRole } = render(<App />);
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    fireEvent.click(getByRole("button", { name: "Close Untitled" }));
    await waitFor(() => expect(container.querySelector(".untitled-tab")).toBeNull());
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "new" }));
    await waitFor(() => expect(editor(container).state.doc.toString()).toBe(""));
  });
  // Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it("closes the untouched introduction without prompting and clears dirty state on undo", async () => {
    const close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true });
    const saveDraft = vi.spyOn(api, "saveDraft");
    const { container, getByRole, queryByRole } = render(<App />);
    // Folder hydration remounts the editor; exercise the settled layout.
    await waitFor(() => expect(getByRole("tab", { name: /Files/ })).toBeTruthy());
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    act(() => editor(container).dispatch({ changes: { from: 0, insert: "edit" } }));
    await waitFor(() => expect(getByRole("tab", { name: /Untitled/ }).textContent).toContain("*"));
    act(() => editor(container).dispatch({ changes: { from: 0, to: 4 } }));
    await waitFor(() => expect(getByRole("tab", { name: /Untitled/ }).textContent).not.toContain("*"));
    fireEvent(window, new Event("hickory-workspace-close-request"));
    await waitFor(() => expect(close).toHaveBeenCalled());
    expect(queryByRole("button", { name: "Save all" })).toBeNull();
    expect(saveDraft).not.toHaveBeenCalled();
  });

  // Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it("writes a recovery draft on pagehide and deletes it when edits are undone", async () => {
    const save = vi.spyOn(api, "saveDraft");
    const discard = vi.spyOn(api, "discardDraft");
    const { container, getByRole } = render(<App />);
    // Folder hydration remounts the editor; exercise the settled layout.
    await waitFor(() => expect(getByRole("tab", { name: /Files/ })).toBeTruthy());
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    act(() => editor(container).dispatch({ changes: { from: 0, insert: "edit" } }));
    fireEvent(window, new Event("pagehide"));
    await waitFor(() => expect(save).toHaveBeenCalledWith(expect.objectContaining({
      path: expect.stringMatching(/^untitled:/), contents: "edit" + STARTUP_INTRODUCTION,
    })));
    act(() => editor(container).dispatch({ changes: { from: 0, to: 4 } }));
    await waitFor(() => expect(discard).toHaveBeenCalledWith(expect.stringMatching(/^untitled:/)));
  });

  // Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it("discards Untitled recovery before approving window close", async () => {
    let finishWrite!: () => void;
    const writing = new Promise<void>(resolve => { finishWrite = resolve; });
    const save = vi.spyOn(api, "saveDraft").mockImplementation(async () => { await writing; return { ok: true }; });
    const discard = vi.spyOn(api, "discardDraft");
    const close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true });
    const { container, getByRole } = render(<App />);
    // Folder hydration adds a split and remounts the introduction editor.
    // Wait for that arrangement before testing the close gesture.
    await waitFor(() => expect(getByRole("tab", { name: /Files/ })).toBeTruthy());
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    act(() => editor(container).dispatch({ changes: { from: 0, insert: "edit" } }));
    fireEvent(window, new Event("pagehide"));
    await waitFor(() => expect(save).toHaveBeenCalled());
    discard.mockClear();
    fireEvent(window, new Event("hickory-workspace-close-request"));
    await waitFor(() => expect(getByRole("button", { name: "Close without saving" })).toBeTruthy());
    fireEvent.click(getByRole("button", { name: "Close without saving" }));
    expect(close).not.toHaveBeenCalled();
    expect(discard).not.toHaveBeenCalled();
    await act(async () => { finishWrite(); });
    await waitFor(() => expect(close).toHaveBeenCalled());
    fireEvent(window, new Event("pagehide"));
    expect(save).toHaveBeenCalledTimes(1);
    expect(discard).toHaveBeenCalledWith(expect.stringMatching(/^untitled:/));
    expect(discard.mock.invocationCallOrder.at(-1)).toBeLessThan(close.mock.invocationCallOrder[0]);
  });

  // Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it.each([true, false])("discards closed Untitled work when folder_open is %s", async (folderOpen) => {
    vi.spyOn(api, "files").mockResolvedValue({ folder_open: folderOpen, root: "parent", tree: [] });
    let draft: { path: string; contents: string; base: string; saved_at: number } | undefined;
    vi.spyOn(api, "drafts").mockImplementation(async () => ({ drafts: draft ? [draft] : [] }));
    vi.spyOn(api, "saveDraft").mockImplementation(async (value) => { draft = value; return { ok: true }; });
    vi.spyOn(api, "discardDraft").mockImplementation(async () => { draft = undefined; return { ok: true }; });
    const first = render(<App />);
    if (folderOpen) await waitFor(() => expect(first.getByRole("tab", { name: /Files/ })).toBeTruthy());
    await waitFor(() => expect(editor(first.container).state.doc.toString()).toBe(STARTUP_INTRODUCTION));
    act(() => editor(first.container).dispatch({ changes: { from: 0, to: editor(first.container).state.doc.length, insert: "My unsaved note" } }));
    fireEvent.click(first.getByRole("button", { name: "Close Untitled" }));
    await waitFor(() => expect(first.getByRole("button", { name: "Discard" })).toBeTruthy());
    expect(first.getByText(/Save changes to Untitled/).textContent).toContain("discards these changes");
    fireEvent.click(first.getByRole("button", { name: "Discard" }));
    await waitFor(() => expect(first.container.querySelector(".untitled-tab")).toBeNull());
    fireEvent(window, new Event("pagehide"));
    expect(draft).toBeUndefined();
    first.unmount();
    const second = render(<App />);
    await waitFor(() => expect(editor(second.container).state.doc.toString()).toBe(STARTUP_INTRODUCTION));
    expect(second.getByRole("tab", { name: /Untitled/ }).textContent).not.toContain("*");
  });

});


describe("windows without an open folder", () => {
  // Guarantee: docs/guarantees/authoring/file-menu-preserves-the-originating-window.md
  it("keeps each unsaved editor when the first folder is attached", async () => {
    const files = vi.spyOn(api, "files").mockResolvedValue({ folder_open: false, root: "parent", tree: [] });
    const app = render(<App />);
    await waitFor(() => expect(editor(app.container).state.doc.toString()).toBe(STARTUP_INTRODUCTION));
    act(() => editor(app.container).dispatch({ changes: { from: 0, to: editor(app.container).state.doc.length, insert: "First unsaved note" } }));
    act(() => newDocument());
    await waitFor(() => expect(editor(app.container).state.doc.toString()).toBe(""));
    act(() => editor(app.container).dispatch({ changes: { from: 0, insert: "Second unsaved note" } }));
    const ids = app.getAllByRole("tab", { name: /Untitled/ }).map(tab => tab.id);
    files.mockResolvedValue({ folder_open: true, root: "selected", tree: [] });
    fireEvent(window, new Event("hickory:files-changed"));
    await waitFor(() => expect(app.getByRole("tab", { name: /Files/ })).toBeTruthy());
    expect(app.getAllByRole("tab", { name: /Untitled/ }).map(tab => tab.id)).toEqual(ids);
    const contents = new Set<string>();
    for (const tab of app.getAllByRole("tab", { name: /Untitled/ })) {
      fireEvent.click(tab);
      await waitFor(() => expect(editor(app.container).state.doc.toString()).not.toBe(""));
      contents.add(editor(app.container).state.doc.toString());
    }
    expect(contents).toEqual(new Set(["First unsaved note", "Second unsaved note"]));
  });

  it("keeps Files closed while editing and saving still work", async () => {
    vi.spyOn(api, "files").mockResolvedValue({ folder_open: false, root: "parent", tree: [] });
    const save = vi.spyOn(api, "saveFileDialog").mockResolvedValue({ path: null });
    const { container, queryByRole } = render(<App />);
    await waitFor(() => {
      expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
      expect(editor(container).hasFocus).toBe(true);
    });
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "files" }));
    await waitFor(() => expect(queryByRole("status")?.textContent).toContain("Open a folder"));
    expect(container.querySelector(".folder-tree")).toBeNull();
    expect(container.querySelectorAll('[role="tab"]').length).toBe(1);
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "save" }));
    await waitFor(() => expect(save).toHaveBeenCalled());
    expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION);
  });

  it("explains unavailable tools in a blank window without calling their APIs", async () => {
    window.history.replaceState(null, "", "/#/blank");
    const settings = vi.spyOn(api, "settingsKeys");
    const { getByRole, queryByRole } = render(<App />);
    for (const action of ["files", "show-agent", "terminal", "settings", "new", "new-project"]) {
      fireEvent(window, new CustomEvent("hickory-menu", { detail: action }));
      await waitFor(() => expect(getByRole("status").textContent).toContain("Open a"));
    }
    expect(queryByRole("tab")).toBeNull();
    expect(settings).not.toHaveBeenCalled();
    expect(window.location.hash).toBe("#/blank");
  });
});
