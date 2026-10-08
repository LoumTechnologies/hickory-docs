// @vitest-environment jsdom
// Guarantee: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorViewport, linkShapes } from "./linkShapes";
import type { RibbonLink } from "./Ribbons";

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
