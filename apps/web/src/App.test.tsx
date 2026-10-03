import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { EditorView } from "@codemirror/view";
import { App } from "./App";
import { api } from "./api/client";
import { installMockApi } from "./mock/mockApi";
import { STARTUP_INTRODUCTION } from "./lib/newDoc";
import { WELCOME_KEY } from "./lib/welcomePref";
import { openChatTab, openDocTab, initialWorkspace } from "./views/workspaceState";
import { tab, withTree } from "./shell/layout";
import { wrapColumnOf } from "./editor/wrapColumn";

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
  // Guarantee: docs/guarantees/authoring/the-measure-says-what-it-measures.md
  it("keeps a dragged prose margin in the initial untitled document and stores it", async () => {
    vi.mocked(api.workspaceUi).mockResolvedValue({ state: { version: 1, wrap: { untitled: 72 } } });
    const save = vi.spyOn(api, "saveWorkspaceUi");
    const { container, getByRole } = render(<App />);
    await waitFor(() => expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION));
    const live = editor(container);
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
    await waitFor(() => expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION));
    await waitFor(() => expect(editor(container).hasFocus).toBe(true));
    expect(getAllByRole("tab")).toHaveLength(1);
    expect(getByRole("tab").textContent).toContain("Untitled");
    expect(getByRole("tab").textContent).toContain("*");
    expect(container.querySelector(".filesystem-editor")).toBeNull();
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

  it("opens a blank document through New after the startup document is discarded", async () => {
    const { container, getByRole } = render(<App />);
    await waitFor(() => expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION));
    fireEvent.click(getByRole("button", { name: "Close Untitled" }));
    await waitFor(() => expect(getByRole("button", { name: /Discard|Close and retain/ })).toBeTruthy());
    fireEvent.click(getByRole("button", { name: /Discard|Close and retain/ }));
    await waitFor(() => expect(container.querySelector(".untitled-tab")).toBeNull());
    fireEvent(window, new CustomEvent("hickory-menu", { detail: "new" }));
    await waitFor(() => expect(editor(container).state.doc.toString()).toBe(""));
  });
});


describe("windows without an open folder", () => {
  it("keeps Files closed while editing and saving still work", async () => {
    vi.spyOn(api, "files").mockResolvedValue({ folder_open: false, root: "parent", tree: [] });
    const save = vi.spyOn(api, "saveFileDialog").mockResolvedValue({ path: null });
    const { container, queryByRole } = render(<App />);
    await waitFor(() => expect(editor(container).state.doc.toString()).toBe(STARTUP_INTRODUCTION));
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
