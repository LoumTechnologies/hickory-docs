// @vitest-environment jsdom
// Guarantee: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorViewport, linkShapes } from "./linkShapes";
import type { RibbonLink } from "./Ribbons";
import { attachTerminal, revealStrip } from "./Ribbons";

const views: EditorView[] = [];
afterEach(() => views.splice(0).forEach(view => view.destroy()));
it.each([false, true])("joins the cited passage in an open editor, braces=%s", braces => {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const source = new EditorView({ parent: container, state: EditorState.create({ doc: "Claim\nOther claim" }) });
  const target = new EditorView({ parent: container, state: EditorState.create({ doc: "Title\n\nSupporting paragraph\ncontinued\nOther material" }) });
  views.push(source, target);
  const box = (left: number) => ({ left, right: left + 100, top: 0, bottom: 200, width: 100, height: 200, x: left, y: 0, toJSON() {} });
  vi.spyOn(container, "getBoundingClientRect").mockReturnValue(box(0));
  const link: RibbonLink = { key: "claim", family: "declared", from: { path: "sessions/s.md", lines: [1,1] }, to: { path: "note.md", lines: [3,4] }, title: "citation" };
  const geometry = {
    bandBetween: (_view: EditorView, from: number, to: number): [number,number] => [from + 10, to + 30],
    paneEdges: (view: EditorView) => ({ left: view === source ? 0 : 300, right: view === source ? 100 : 400, leftRailW: 12, rightRailW: 12 }),
    findTerminal: vi.fn(() => null), attachTerminal: vi.fn(), revealStrip: vi.fn(),
  };
  vi.spyOn(source.scrollDOM, "getBoundingClientRect").mockReturnValue(box(0));
  vi.spyOn(target.scrollDOM, "getBoundingClientRect").mockReturnValue(box(300));
  const sources = [{ docPath: "sessions/s.md", docSource: "", view: source }, { docPath: "note.md", docSource: "", view: target }];
  const shapes = linkShapes(container, sources, [], [link], () => true, braces, geometry);
  expect(shapes).toHaveLength(1);
  const bufferShapes = linkShapes(container, [sources[0], { ...sources[1], docPath: "hickory-buffer://u1" }], [],
    [{ ...link, to: { ...link.to, path: "hickory-buffer://u1" } }], () => true, braces, geometry);
  expect(bufferShapes).toHaveLength(1);
  expect(bufferShapes[0].hl[1]).toEqual(shapes[0].hl[1]);
  // Guarantee: docs/guarantees/execution/runtime-logs-preserve-contributor-relationships.md
  const runtimeLink: RibbonLink = { ...link, key: "event-one", family: "execution", review: {
    kind: "runtime-relationship", path: "note.md", quote: "Supporting paragraph", source_hash: "a".repeat(64), lines: [3, 3], relationship: { event: "event-one" },
  } };
  const execution = linkShapes(container, sources, [], [runtimeLink, { ...runtimeLink, key: "event-two" }], () => true, braces, geometry);
  expect(execution).toHaveLength(2);
  expect(execution[0].review).toEqual(runtimeLink.review);
  expect(execution.map(shape => shape.target.kind === "path" ? shape.target.linkKey : null)).toEqual(["event-one", "event-two"]);
  expect(shapes[0].ends).toBe("text");
  expect(shapes[0].family).toBe("declared");
  expect(shapes[0].hl[1]).toEqual({ view: target, from: target.state.doc.line(3).from, to: target.state.doc.line(4).to });
  expect(braces ? shapes[0].brace?.b : shapes[0].band).toBeTruthy();
  expect(linkShapes(container, sources, [], [{ ...link, to: { path: "note.md", lines: [99,100] } }], () => true, braces, geometry)).toHaveLength(0);
  expect(linkShapes(container, sources, [], [link], () => false, braces, geometry)).toHaveLength(0);
  container.remove();
});

it("clips a conversation's connections to its actual scrolling parent", () => {
  const log = document.createElement("div"); document.body.appendChild(log);
  log.style.overflowY = "auto";
  const view = new EditorView({ parent: log, state: EditorState.create({ doc: "Final claim" }) });
  views.push(view);
  vi.spyOn(view.scrollDOM, "getBoundingClientRect").mockReturnValue({ top: -300, bottom: 500 } as DOMRect);
  vi.spyOn(log, "getBoundingClientRect").mockReturnValue({ top: 100, bottom: 400 } as DOMRect);
  expect(editorViewport(view)).toEqual({ top: 100, bottom: 400 });
  log.remove();
});

it("skips a connection whose source passage is scrolled out of the editor viewport", () => {
  const container = document.createElement("div"); document.body.appendChild(container);
  const source = new EditorView({ parent: container, state: EditorState.create({ doc: "Claim\nOther claim" }) });
  const target = new EditorView({ parent: container, state: EditorState.create({ doc: "Title\n\nSupporting paragraph\ncontinued\nOther material" }) });
  views.push(source, target);
  const box = (left: number) => ({ left, right: left + 100, top: 0, bottom: 200, width: 100, height: 200, x: left, y: 0, toJSON() {} });
  vi.spyOn(container, "getBoundingClientRect").mockReturnValue(box(0));
  vi.spyOn(source.scrollDOM, "getBoundingClientRect").mockReturnValue(box(0));
  vi.spyOn(target.scrollDOM, "getBoundingClientRect").mockReturnValue(box(300));
  const link: RibbonLink = { key: "claim", family: "declared", from: { path: "sessions/s.md", lines: [1, 1] }, to: { path: "note.md", lines: [3, 4] }, title: "citation" };
  const geometry = {
    bandBetween: () => [-500, -400] as [number, number],
    paneEdges: (view: EditorView) => ({ left: view === source ? 0 : 300, right: view === source ? 100 : 400, leftRailW: 12, rightRailW: 12 }),
    findTerminal: vi.fn(() => null), attachTerminal: vi.fn(), revealStrip: vi.fn(),
  };
  const sources = [{ docPath: "sessions/s.md", docSource: "", view: source }, { docPath: "note.md", docSource: "", view: target }];
  expect(linkShapes(container, sources, [], [link], () => true, false, geometry)).toHaveLength(0);
  container.remove();
});

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
it("draws from a centered cited document back to a collapsed conversation header", () => {
  const container = document.createElement("div"); document.body.append(container);
  const view = new EditorView({ parent: container, state: EditorState.create({ doc: "Title\nSupporting paragraph\nOther" }) }); views.push(view);
  vi.spyOn(view.scrollDOM, "getBoundingClientRect").mockReturnValue(new DOMRect(300, 0, 400, 300));
  const link: RibbonLink = { key: "receipt", family: "context", from: { path: "sessions/agent.md", lines: [10, 12] }, to: { path: "notes.md", lines: [2, 2] }, title: "context receipt" };
  const shapes = linkShapes(container, [{ docPath: "sessions/agent.md", docSource: "" }, { docPath: "notes.md", docSource: "", view }], [], [link], () => true, true, {
    bandBetween: () => [80, 110], paneEdges: () => ({ left: 300, right: 700, leftRailW: 10, rightRailW: 10 }),
    findTerminal: () => ({ rect: new DOMRect(0, 78, 150, 36), ends: "tab", vertical: true }), attachTerminal, revealStrip,
  });
  expect(shapes).toHaveLength(1); expect(shapes[0].family).toBe("context");
  expect(shapes[0].target).toMatchObject({ path: "sessions/agent.md", lines: [10, 12] });
  expect(shapes[0].hl[0].view).toBe(view); expect(shapes[0].brace?.link).toBeTruthy(); container.remove();
});
