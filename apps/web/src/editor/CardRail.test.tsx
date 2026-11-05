// Contract: .hickory-source/guarantees/authoring/the-editor-keeps-its-space/contract.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { CardRail } from "./CardRail";
import { editorChrome } from "./chrome";
import { EnvRegistry, wysiwyg } from "./wysiwyg";
import { DebugLaunchRail } from "../debug/DebugRail";
import type { DocCard } from "./cards";
const views: EditorView[] = [];
afterEach(() => { cleanup(); vi.restoreAllMocks(); for (const view of views.splice(0)) { const host = view.dom.parentElement; view.destroy(); host?.remove(); } });
it("opens excess document actions in a grid and invokes their existing callbacks", async () => {
  const view = new EditorView({ parent: document.body.appendChild(document.createElement("div")), state: EditorState.create({ doc: "task" }) }); views.push(view);
  Object.defineProperty(view.scrollDOM, "clientHeight", { value: 22 });
  const card: DocCard = { key: "exec-0", kind: "exec", index: 0, at: 0, from: 0, to: 4, label: "Task" };
  const action = vi.fn();
  render(<CardRail view={view} cards={[card]} stateOf={() => "idle"} openKey={null} renderedAt={[]} actionsOf={() => ["run", "source", "replay"]} replayingAt={[]} onAction={action} />);
  const more = await screen.findByRole("button", { name: "Show 3 more gutter icons" });
  fireEvent.click(more);
  const dialog = screen.getByRole("dialog", { name: "Gutter icons" });
  expect(dialog.querySelectorAll("button")).toHaveLength(3);
  fireEvent.keyDown(document, { key: "Escape" });
  expect(screen.queryByRole("dialog", { name: "Gutter icons" })).toBeNull(); expect(document.activeElement).toBe(more);
  fireEvent.click(more);
  const source = screen.getByRole("button", { name: /showing its source; click to render it/ });
  await act(async () => fireEvent.click(source));
  await waitFor(() => expect(action).toHaveBeenCalledWith(card, "source", Number.parseFloat(more.style.top)));
  expect(screen.queryByRole("dialog", { name: "Gutter icons" })).toBeNull();
});

it("keeps debugger actions owned by their original groups inside the same overflow grid", async () => {
  const wrapper = document.body.appendChild(document.createElement("div"));
  const host = wrapper.appendChild(document.createElement("div")); host.className = "editor-cm-host";
  const view = new EditorView({ parent: host, state: EditorState.create({
    doc: '<hick:file path="a.py" language="python">print(1)</hick:file><hick:file path="b.py" language="python">print(2)</hick:file>',
    extensions: [editorChrome("code"), wysiwyg(new EnvRegistry())],
  }) }); views.push(view);
  const rect = (left: number, width: number) => ({ left, right: left + width, top: 0, bottom: 22, x: left, y: 0, width, height: 22, toJSON: () => ({}) });
  vi.spyOn(view.scrollDOM, "getBoundingClientRect").mockReturnValue(rect(0, 100));
  Object.defineProperty(view.scrollDOM, "clientHeight", { value: 22 });
  const start = vi.fn();
  render(<DebugLaunchRail view={view} onStart={start} onProfiles={vi.fn()} />, { container: wrapper.appendChild(document.createElement("div")) });
  const rail = wrapper.querySelector<HTMLElement>(".cm-debug-rail")!;
  vi.spyOn(rail, "getBoundingClientRect").mockReturnValue(rect(100, 24));
  Object.defineProperty(rail, "clientHeight", { value: 22 });
  for (const group of rail.querySelectorAll<HTMLElement>(".cm-debug-rail__group")) vi.spyOn(group, "getBoundingClientRect").mockReturnValue(rect(100, 24));
  const more = await screen.findByRole("button", { name: "Show 4 more gutter icons" });
  fireEvent.click(more);
  expect(screen.getByRole("dialog", { name: "Gutter icons" })).toBeTruthy();
  const button = screen.getByRole("button", { name: "Debug a.py" });
  expect(button.closest(".cm-debug-rail__group")).not.toBeNull();
  await act(async () => fireEvent.click(button));
  expect(start).toHaveBeenCalledWith("a.py", 0);
  fireEvent.keyDown(document, { key: "Escape" });
  expect(screen.queryByRole("dialog", { name: "Gutter icons" })).toBeNull();
});
