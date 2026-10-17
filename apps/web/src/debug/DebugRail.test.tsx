import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { debugRailTop, DebugLaunchRail } from "./DebugRail";
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
  expect(start).toHaveBeenCalledWith("app.py");
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
  expect(start).toHaveBeenCalledWith("app.py");
});
