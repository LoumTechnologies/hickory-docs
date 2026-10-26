<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/linkShapes.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5b6df5b99815"></a>
## `apps/web/src/shell/linkShapes.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Show the requested connections without changing their meaning](../../../../../guarantees/lineage/a-ribbon-is-drawn-where-it-is-asked-for/contract.md).

<a id="f-5b6df5b99815-Geometry"></a>
### `Geometry`

Context and declared links use the same rails and geometry as byte lineage.

<storycode:copy id="f-5b6df5b99815-Geometry">
// Context and declared links use the same rails and geometry as byte lineage.
import { coalesceLinks } from "./coalesceLinks";
import type { EditorView } from "@codemirror/view";
import type { Draft, Terminal, RibbonFamily, RibbonLink, RibbonSource, RibbonFile } from "./Ribbons";
import { samePath } from "../lib/paths";
import { anchorEdges, bandVisible, clampBand, braceNub, bracePath, braceLinkPath, ribbonPath } from "../lib/ribbonGeometry";
import { visibleElementRect } from "../lib/visibleElementRect";

type Geometry = Pick<typeof import("./Ribbons"), "bandBetween" | "paneEdges" | "findTerminal" | "attachTerminal" | "revealStrip">;
</storycode:copy>
<a id="f-5b6df5b99815-linkShapes"></a>
### `linkShapes`

<storycode:copy id="f-5b6df5b99815-linkShapes">
export function linkShapes(container: HTMLElement, sources: readonly RibbonSource[], files: readonly RibbonFile[], links: readonly RibbonLink[], on: (family: RibbonFamily) => boolean, braces: boolean, geometry: Geometry): Draft[] {
  const { bandBetween, paneEdges, findTerminal, attachTerminal, revealStrip } = geometry;
  const box = container.getBoundingClientRect();
  const out: Draft[] = [];
    for (const link of coalesceLinks(links, sources)) {
      if (!on(link.family)) continue;
      const source = sources.find(
        (s) => s.view && samePath(s.docPath, link.from.path),
      );
      const view = source?.view;
      if (!source || !view) {
        const reverse = link.to.lines && (sources.some(s => s.view && samePath(s.docPath, link.to.path)) || files.some(f => f.view && samePath(f.file.path, link.to.path)));
        if (reverse) out.push(...linkShapes(container, [...sources, ...files.filter(f => f.view).map(f => ({ docPath: f.file.path, docSource: f.file.content, view: f.view }))], [], [{ ...link, key: `${link.key}:reverse`, from: { path: link.to.path, lines: link.to.lines! }, to: { path: link.from.path, lines: link.from.lines }, anchor: undefined }], on, braces, geometry));
        continue;
      }
      const kinds: ("document" | "generated" | "file")[] = link.to.kind
        ? [link.to.kind]
        : ["document", "generated", "file"];
      let terminal: Terminal | null = null;
      for (const kind of kinds) {
        terminal = findTerminal(container, kind, link.to.path);
        if (terminal) break;
      }
      const targetView = link.to.lines ? sources.find(s => s.view && samePath(s.docPath, link.to.path))?.view
        ?? files.find(f => f.view && samePath(f.file.path, link.to.path))?.view : undefined;
      if (!terminal && !targetView) continue;
      const total = view.state.doc.lines;
      const a = Math.min(Math.max(link.from.lines[0], 1), total);
      const b = Math.min(Math.max(link.from.lines[1], a), total);
      const from = view.state.doc.line(a).from;
      const to = view.state.doc.line(b).to;
      // An anchored source (an agent message card) reports its visible rect
      // only when it is on screen; a scrolled-out anchor produces no band at
      // all. A line-anchored source (a document section) is measured from the
      // height map and then checked against the editor's viewport the same
      // way, so invisible prose stops drawing its connection.
      const pane = editorViewport(view);
      const anchorRect = link.anchor ? visibleElementRect(link.anchor) : undefined;
      let band: [number, number] | null = anchorRect
        ? [anchorRect.top, anchorRect.bottom]
        : bandBetween(view, from, to);
      if (band && !anchorRect && !bandVisible(band[0], band[1], pane.top, pane.bottom)) band = null;
      if (!band || band[1] <= band[0]) continue;
      const edges = paneEdges(view);
      if (targetView && targetView !== view && link.to.lines) {
        const range = lineRange(targetView, link.to.lines);
        // Do not clamp an invalid citation onto some other paragraph.
        if (!range) continue;
        const targetBand = bandBetween(targetView, range.from, range.to);
        if (!targetBand) continue;
        const targetPane = editorViewport(targetView);
        const targetEdges = paneEdges(targetView);
        const { forward, x0, x1 } = anchorEdges(
          { left: edges.left - box.left, right: edges.right - box.left },
          { left: targetEdges.left - box.left, right: targetEdges.right - box.left });
        const fromBand = clampBand(band[0] - box.top, band[1] - box.top, pane.top - box.top, pane.bottom - box.top);
        const into = clampBand(targetBand[0] - box.top, targetBand[1] - box.top, targetPane.top - box.top, targetPane.bottom - box.top);
        const shape: Draft = { key: `${link.family}:${link.key}:text`, family: link.family, review: link.review,
          color: link.family === "context" ? 0 : 1, alwaysVisible: link.alwaysVisible,
          ends: "text", clamped: fromBand.clamped || into.clamped,
          hl: [{ view, from, to }, { view: targetView, ...range }],
          target: { kind: "path", path: link.to.path, lines: link.to.lines, family: link.family, linkKey: link.key } };
        if (braces) {
          const dir = forward ? 1 : -1, back = forward ? -1 : 1;
          const a = bracePath(x0, fromBand.yTop, fromBand.yBot, dir, undefined,
            { top: fromBand.clampedTop, bottom: fromBand.clampedBottom }, forward ? edges.rightRailW : edges.leftRailW);
          const b = bracePath(x1, into.yTop, into.yBot, back, undefined,
            { top: into.clampedTop, bottom: into.clampedBottom }, forward ? targetEdges.leftRailW : targetEdges.rightRailW);
          const na = braceNub(x0, fromBand.yTop, fromBand.yBot, dir), nb = braceNub(x1, into.yTop, into.yBot, back);
          const join = braceLinkPath(na.x, na.y, dir, nb.x, nb.y, back);
          shape.brace = { a, b, link: join, hit: `${a} ${join} ${b}` };
        } else shape.band = ribbonPath(x0, fromBand.yTop, fromBand.yBot, x1, into.yTop, into.yBot);
        out.push(shape);
        continue;
      }
      if (!terminal) continue;
      const tMid = (terminal.rect.left + terminal.rect.right) / 2 - box.left;
      const docL = edges.left - box.left;
      const docR = edges.right - box.left;
      const x0 = tMid >= (docL + docR) / 2 ? docR : docL;
      const clamped = clampBand(
        band[0] - box.top,
        band[1] - box.top,
        pane.top - box.top,
        pane.bottom - box.top,
      );
      const dir: 1 | -1 = x0 === docR ? 1 : -1;
      const horn = x0 === docR ? edges.rightRailW : edges.leftRailW;
      const nub = braces ? braceNub(x0, clamped.yTop, clamped.yBot, dir) : null;
      const attach = attachTerminal(
        terminal,
        box,
        x0,
        clamped,
        braces,
        nub,
        dir,
      );
      const shape: Draft = {
        key: `${link.family}:${link.key}:${terminal.ends}`,
        review: link.review,
        alwaysVisible: link.alwaysVisible,
        family: link.family,
        color: link.family === "context" ? 0 : 1,
        clamped: clamped.clamped,
        ends: terminal.ends,
        reveal: revealStrip(x0, dir, clamped),
        connector: attach.connector,
        hl: [{ view, from, to }],
        target: {
          kind: "path",
          path: link.to.path,
          lines: link.to.lines,
          family: link.family,
          linkKey: link.key,
        },
      };
      if (braces && attach.link) {
        const brace = bracePath(
          x0,
          clamped.yTop,
          clamped.yBot,
          dir,
          undefined,
          { top: clamped.clampedTop, bottom: clamped.clampedBottom },
          horn,
        );
        shape.brace = {
          a: brace,
          link: attach.link,
          hit: `${brace} ${attach.link}`,
        };
      } else {
        shape.band = attach.band;
      }
      out.push(shape);
    }

  return out;
}

</storycode:copy>
<a id="f-5b6df5b99815-lineRange"></a>
### `lineRange`

<storycode:copy id="f-5b6df5b99815-lineRange">
export function lineRange(view: EditorView, lines: [number, number]) {
  const [a, b] = lines;
  if (a < 1 || b < a || b > view.state.doc.lines) return null;
  return { from: view.state.doc.line(a).from, to: view.state.doc.line(b).to };
}

</storycode:copy>
<a id="f-5b6df5b99815-editorViewport"></a>
### `editorViewport`

A conversation can scroll in a parent while CodeMirror's own scroller grows.

<storycode:copy id="f-5b6df5b99815-editorViewport">
/** A conversation can scroll in a parent while CodeMirror's own scroller grows. */
export function editorViewport(view: EditorView): { top: number; bottom: number } {
  let { top, bottom } = view.scrollDOM.getBoundingClientRect();
  for (let parent = view.scrollDOM.parentElement; parent; parent = parent.parentElement) {
    if (!/^(auto|scroll|hidden|clip)$/.test(getComputedStyle(parent).overflowY)) continue;
    const bounds = parent.getBoundingClientRect();
    top = Math.max(top, bounds.top);
    bottom = Math.min(bottom, bounds.bottom);
  }
  return { top, bottom: Math.max(top, bottom) };
}
</storycode:copy>

### Assemble `apps/web/src/shell/linkShapes.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/linkShapes.ts"><storycode:paste select="#f-5b6df5b99815-Geometry"/><storycode:paste select="#f-5b6df5b99815-linkShapes"/><storycode:paste select="#f-5b6df5b99815-lineRange"/><storycode:paste select="#f-5b6df5b99815-editorViewport"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-04fb400a1dba"></a>
## `apps/web/src/shell/linkShapes.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace conversation passages without confusing context with support](../../../../../guarantees/agent/an-answer-in-the-agent-pane-has-ribbons/contract.md), [Execution logs preserve contributor relationships and historical event identity](../../../../../guarantees/execution/runtime-logs-preserve-contributor-relationships/contract.md), [Spatial Views Preserve The Editor](../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md).

<a id="f-04fb400a1dba-views"></a>
### `views`

<storycode:copy id="f-04fb400a1dba-views">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorViewport, linkShapes } from "./linkShapes";
import type { RibbonLink } from "./Ribbons";
import { attachTerminal, revealStrip } from "./Ribbons";

const views: EditorView[] = [];
</storycode:copy>
<a id="f-04fb400a1dba-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-04fb400a1dba-expression_statement">
afterEach(() => views.splice(0).forEach(view => view.destroy()));
</storycode:copy>
<a id="f-04fb400a1dba-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-04fb400a1dba-expression_statement-2">
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

</storycode:copy>
<a id="f-04fb400a1dba-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-04fb400a1dba-expression_statement-3">
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

</storycode:copy>
<a id="f-04fb400a1dba-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-04fb400a1dba-expression_statement-4">
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

</storycode:copy>
<a id="f-04fb400a1dba-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-04fb400a1dba-expression_statement-5">
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
</storycode:copy>

### Assemble `apps/web/src/shell/linkShapes.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/linkShapes.test.ts"><storycode:paste select="#f-04fb400a1dba-views"/><storycode:paste select="#f-04fb400a1dba-expression_statement"/><storycode:paste select="#f-04fb400a1dba-expression_statement-2"/><storycode:paste select="#f-04fb400a1dba-expression_statement-3"/><storycode:paste select="#f-04fb400a1dba-expression_statement-4"/><storycode:paste select="#f-04fb400a1dba-expression_statement-5"/></storycode:file>


</storycode:doc>
