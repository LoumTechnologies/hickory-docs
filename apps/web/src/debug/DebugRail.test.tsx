import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { debugRailTop, debugRailElement, DebugLaunchRail } from "./DebugRail";
import { DebugStrip, type DebugStripProps } from "./DebugStrip";
import { EnvRegistry, wysiwyg } from "../editor/wysiwyg";

let view: EditorView;
afterEach(() => { cleanup(); view?.destroy(); document.body.replaceChildren(); });
function editor(source: string) {
  const wrapper = document.body.appendChild(document.createElement("div"));
  wrapper.className = "output-editor with-right-rail";
  const host = wrapper.appendChild(document.createElement("div"));
  host.className = "editor-cm-host";
  view = new EditorView({ parent: host, state: EditorState.create({ doc: source, extensions: [wysiwyg(new EnvRegistry())] }) });
  return wrapper.appendChild(document.createElement("div"));
}
it("puts one launch icon per debuggable script in the right action rail", () => {
  const start = vi.fn();
  const container = editor('<hick:file path="app.py" language="python">\nprint(1)\n</hick:file>\n<hick:file path="notes.txt">text</hick:file>');
  render(<DebugLaunchRail view={view} onStart={start} />, { container });
  const button = screen.getByRole("button", { name: "Debug app.py" });
  expect(button.closest(".cm-debug-rail")).not.toBeNull();
  expect(button.closest(".cm-gutters")).toBeNull();
  expect(document.querySelectorAll(".cm-debug-start")).toHaveLength(1);
  fireEvent.click(button);
  expect(start).toHaveBeenCalledWith("app.py", 0);
});
it("puts continue, stepping and stop beside the paused line and keeps their handlers", () => {
  const container = editor("one\ntwo\nthree\n");
  const step = vi.fn(), stop = vi.fn();
  const props: DebugStripProps = {
    editorView: view, pausedLine: 1, status: "paused", program: "app.py", message: null,
    capabilities: null, frames: [], selectedFrame: null, watches: [], exceptionFilters: [],
    onSelectFrame: vi.fn(), onStep: step, onJumpHere: vi.fn(), onStart: vi.fn(), onStop: stop,
    onAddWatch: vi.fn(), onToggleExceptionFilter: vi.fn(), onRemoveWatch: vi.fn(),
  };
  const rendered = render(<DebugLaunchRail view={view} onStart={vi.fn()} plainPath="app.py" />, { container });
  const session = render(<DebugStrip {...props} />);
  const toolbar = screen.getByRole("toolbar", { name: "Debugger" });
  expect(toolbar.closest(".cm-debug-rail")).not.toBeNull();
  expect(toolbar.closest("[data-source-offset]")?.getAttribute("data-source-offset")).toBe("4");
  fireEvent.click(screen.getByRole("button", { name: "Continue" }));
  fireEvent.click(screen.getByRole("button", { name: "Step over" }));
  fireEvent.click(screen.getByRole("button", { name: "Step into" }));
  fireEvent.click(screen.getByRole("button", { name: "Step out" }));
  expect(step.mock.calls.map(call => call[0])).toEqual(["continue", "over", "in", "out"]);
  session.rerender(<DebugStrip {...props} pausedLine={2} />);
  expect(toolbar.closest("[data-source-offset]")?.getAttribute("data-source-offset")).toBe("8");
  session.rerender(<DebugStrip {...props} status="running" pausedLine={null} />);
  expect(screen.queryByRole("button", { name: "Step over" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Stop" }));
  expect(stop).toHaveBeenCalledOnce();
  session.unmount(); rendered.unmount();
});
it("keeps active controls reachable at both viewport edges without moving source lines", () => {
  expect(debugRailTop(-300, 140, 500, true)).toBe(0);
  expect(debugRailTop(480, 140, 500, true)).toBe(360);
  expect(debugRailTop(100, 140, 500, true)).toBe(100);
  expect(debugRailTop(100, 140, 80, true)).toBe(0);
  expect(debugRailTop(-300, 22, 500, false)).toBeNull();
  expect(debugRailTop(600, 22, 500, false)).toBeNull();
});

it("starts the focused script with F5 without opening a menu", () => {
  const container=editor("print(1)\n");
  const start=vi.fn();
  render(<DebugLaunchRail view={view} plainPath="app.py" onStart={start}/>,{container});
  view.focus();
  fireEvent.keyDown(view.contentDOM,{key:"F5"});
  expect(start).toHaveBeenCalledWith("app.py", undefined);
});

it("keeps a document's controls out of the rails of its nested patch editors", () => {
  const container = editor("source");
  const nested = view.dom.appendChild(document.createElement("div"));
  nested.className = "cm-debug-rail";
  const rail = container.appendChild(document.createElement("div"));
  rail.className = "cm-debug-rail";
  expect(debugRailElement(view)).toBe(rail);
});
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
it("shares launch rail callbacks and refuses stale source before a React rerender", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const container = editor("print(1)\n"), first = vi.fn(), second = vi.fn(), profiles = vi.fn();
  const draw = (start: typeof first) => <UiActionContext.Provider value={registry}><DebugLaunchRail view={view} plainPath="app.py" onStart={start} onProfiles={profiles} /></UiActionContext.Provider>;
  const rendered = render(draw(first), { container });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const handle = () => actions().find(action => action.label === "Debug app.py")!.handle;
  const run = async (target: string) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle: target, arguments: {}, request_id: crypto.randomUUID() }); });
    return result;
  };
  expect(await run(handle())).toMatchObject({ status: "result", outcome: { value: { launch_callback_completed: true, launch_completed: false } } });
  expect(first).toHaveBeenCalledExactlyOnceWith("app.py", undefined);
  const stale = handle();
  act(() => view.dispatch({ changes: { from: 0, to: 0, insert: "#changed\n" } }));
  expect(await run(stale)).not.toMatchObject({ status: "result" });
  view.focus(); fireEvent.keyDown(view.contentDOM, { key: "F5" });
  expect(first).toHaveBeenCalledTimes(1);
  rendered.rerender(draw(second));
  expect(await run(handle())).toMatchObject({ status: "result" });
  expect(second).toHaveBeenCalledExactlyOnceWith("app.py", undefined);
  expect(await run(actions().find(action => action.label === "Launch profiles for app.py")!.handle)).toMatchObject({ status: "result" });
  expect(profiles).toHaveBeenCalledExactlyOnceWith("app.py", undefined);
});
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
it("surfaces a launch callback failure and excludes overlapping native requests", async () => {
  const { act } = await import("@testing-library/react");
  let reject!: (error: Error) => void;
  const start = vi.fn(() => new Promise<void>((_resolve, fail) => { reject = fail; }));
  const container = editor("print(1)\n");
  render(<DebugLaunchRail view={view} plainPath="app.py" onStart={start} />, { container });
  const button = screen.getByRole("button", { name: "Debug app.py" });
  fireEvent.click(button); fireEvent.click(button);
  expect(start).toHaveBeenCalledTimes(1);
  await act(async () => { reject(Error("actual debugger refusal")); });
  expect(screen.getByRole("alert").textContent).toContain("actual debugger refusal");
});
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
it("retires projection launch callbacks when an identical-label target is replaced", async () => {
  const { act } = await import("@testing-library/react");
  const { DebugLaunchAction } = await import("./DebugRail");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const container = editor("print(1)\n"), first = vi.fn(), second = vi.fn();
  render(<DebugLaunchRail view={view} />, { container });
  const draw = (start: typeof first, disabled = false) => <UiActionContext.Provider value={registry}><DebugLaunchAction view={view} label="Debug captured app" onStart={start} disabled={disabled} /></UiActionContext.Provider>;
  const rendered = render(draw(first));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const handle = () => actions().find(action => action.label === "Debug captured app")!.handle;
  const stale = handle();
  rendered.rerender(draw(second));
  await act(async () => { expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: stale, arguments: {}, request_id: "old-projection" })).not.toMatchObject({ status: "result" }); });
  expect(first).not.toHaveBeenCalled(); expect(second).not.toHaveBeenCalled();
  await act(async () => { fireEvent.click(screen.getByRole("button", { name: "Debug captured app" })); });
  expect(second).toHaveBeenCalledTimes(1);
  rendered.rerender(draw(second, true));
  await act(async () => { expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: handle(), arguments: {}, request_id: "disabled-projection" })).toMatchObject({ status: "disabled" }); });
  expect(second).toHaveBeenCalledTimes(1);
});
