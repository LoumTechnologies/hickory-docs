<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/SessionLens.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-11b8d4e005a0"></a>
## `apps/web/src/views/SessionLens.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace conversation passages without confusing context with support](../../../../../guarantees/agent/an-answer-in-the-agent-pane-has-ribbons/contract.md), [Captured reading surfaces share owned native actions](../../../../../guarantees/agent/scoped-reading-surfaces-share-native-actions/contract.md), [A view declares where edits land](../../../../../guarantees/lenses/a-view-declares-where-edits-land/contract.md).

<a id="f-11b8d4e005a0-Blank"></a>
### `Blank`

<storycode:copy id="f-11b8d4e005a0-Blank">
import { SurfaceInfo } from "../components/SurfaceInfo";
// The agent pane's history: the session document, with line numbers, the
// conversation's elements drawn as cards in place of their source, and the
// provenance each element declared handed to the ribbon overlay.
//
// A lens, not a document (docs/specs/freeform/lenses.md): read-only, no
// save path, the same cards and the same overlay every other view gets.
// See docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.
import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { EditorState, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType, lineNumbers } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { api } from "../api/client";
import type { Block, SessionLink, SessionViewResponse } from "../api/types";
import { editorChrome } from "../editor/chrome";
import { RenderedRegistry, renderableBlocks, renderedBlocks, setRenderedBlocks } from "../editor/rendered";
import type { RenderedSlot } from "../editor/rendered";
import { structureOf } from "../editor/wysiwyg";
import { elementViews } from "../elements";
import type { SlotContext } from "../elements";
import { conversationAnchor } from "../lib/conversationLineage";
import { registerLens, ribbonLinksOf } from "../lib/lensSources";
import { byteToChar } from "../lib/offsets";

/** Nothing: what the file's own chrome is drawn as. */
class Blank extends WidgetType {
  toDOM() {
    const el = document.createElement("span");
    el.className = "session-lens__chrome";
    return el;
  }
}

</storycode:copy>
<a id="f-11b8d4e005a0-sessionChrome"></a>
### `sessionChrome`

The lines that are the FILE's, not the conversation's — the XML
declaration and the root element's open and close tags — folded away.
The record is still the record; a reader of the conversation is not
reading XML.

<storycode:copy id="f-11b8d4e005a0-sessionChrome">
/**
 * The lines that are the FILE's, not the conversation's — the XML
 * declaration and the root element's open and close tags — folded away.
 * The record is still the record; a reader of the conversation is not
 * reading XML.
 */
const sessionChrome = StateField.define<DecorationSet>({
  create: (state) => chromeDecorations(state),
  update: (value, tr) => (tr.docChanged ? chromeDecorations(tr.state) : value),
  provide: (field) => EditorView.decorations.from(field),
});

</storycode:copy>
<a id="f-11b8d4e005a0-chromeDecorations"></a>
### `chromeDecorations`

<storycode:copy id="f-11b8d4e005a0-chromeDecorations">
function chromeDecorations(state: EditorState): DecorationSet {
  const doc = state.doc;
  const structure = structureOf(state);
  const lines = new Set<number>();
  if (doc.lines > 0 && doc.line(1).text.startsWith("<?xml")) lines.add(1);
  for (const tag of structure.tags) {
    if (tag.name === "session") {
      lines.add(doc.lineAt(tag.from).number);
      lines.add(doc.lineAt(Math.max(tag.to - 1, tag.from)).number);
    }
  }
  const ranges = [...lines]
    .sort((a, b) => a - b)
    .map((n) => doc.line(n))
    .filter((line) => line.to > line.from)
    .map((line) => Decoration.replace({ widget: new Blank(), block: true }).range(line.from, line.to));
  return Decoration.set(ranges, true);
}

</storycode:copy>
<a id="f-11b8d4e005a0-linksByAnswer"></a>
### `linksByAnswer`

The provenance under each answer: every link whose source lies between
this answer's first line and the next speaker's — what the turn read,
wrote and pointed at — keyed by the answer's block start.

<storycode:copy id="f-11b8d4e005a0-linksByAnswer">
/**
 * The provenance under each answer: every link whose source lies between
 * this answer's first line and the next speaker's — what the turn read,
 * wrote and pointed at — keyed by the answer's block start.
 */
export function linksByAnswer(
  state: EditorState,
  links: readonly SessionLink[],
): Map<number, SessionLink[]> {
  const doc = state.doc;
  const speakers = structureOf(state)
    .blocks.filter((b) => b.name === "assistant" || b.name === "user")
    .sort((a, b) => a.from - b.from);
  const out = new Map<number, SessionLink[]>();
  const users = speakers.filter(block => block.name === "user");
  users.forEach((user, i) => {
    const end = users[i + 1]?.from ?? doc.length;
    const answer = speakers.filter(block => block.name === "assistant" && block.from > user.from && block.from < end).at(-1);
    if (!answer) return;
    const startLine = doc.lineAt(user.from).number;
    const endLine = end === doc.length ? doc.lines + 1 : doc.lineAt(end).number;
    const mine = links.filter(link => link.lines[0] >= startLine && link.lines[0] < endLine);
    if (mine.length > 0) out.set(answer.from, mine);
  });
  return out;
}

</storycode:copy>
<a id="f-11b8d4e005a0-SessionLensProps"></a>
### `SessionLensProps`

<storycode:copy id="f-11b8d4e005a0-SessionLensProps">
export interface SessionLensProps {
  /** The session file, folder-relative. */
  path: string;
  /** Anything that means "the file may have changed" — a finished turn. */
  stamp?: string;
  showCollapsedLineage?: boolean;
}

</storycode:copy>
<a id="f-11b8d4e005a0-SessionLens"></a>
### `SessionLens`

<storycode:copy id="f-11b8d4e005a0-SessionLens">
export function SessionLens(props: SessionLensProps) {
  return <OwnedSessionLens key={props.path} {...props} />;
}

</storycode:copy>
<a id="f-11b8d4e005a0-OwnedSessionLens"></a>
### `OwnedSessionLens`

<storycode:copy id="f-11b8d4e005a0-OwnedSessionLens">
function OwnedSessionLens({ path, stamp, showCollapsedLineage = false }: SessionLensProps) {
  const [retry, setRetry] = useState(0);
  const hostRef = useRef<HTMLDivElement | null>(null);
  const viewRef = useRef<EditorView | null>(null);
  const registry = useMemo(() => new RenderedRegistry(), []);
  const [slots, setSlots] = useState<RenderedSlot[]>([]);
  const [data, setData] = useState<SessionViewResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  // What the cards read, settled once the editor exists: the server's blocks
  // with their spans in the editor's own units (characters, not bytes —
  // the difference is every em dash the model wrote), and the links under
  // each answer.
  const [facts, setFacts] = useState<{
    blocks: Block[];
    linksAt: Map<number, SessionLink[]>;
    view: EditorView;
  } | null>(null);

  useEffect(() => registry.subscribe(() => setSlots(registry.list())), [registry]);

  useEffect(() => {
    let live = true;
    let pending = false;
    const refresh = () => {
      if (pending || document.hidden) return;
      pending = true;
      api.sessionView(path).then(answer => {
        if (!live) return;
        setError(null);
        setData(previous => JSON.stringify(previous) === JSON.stringify(answer) ? previous : answer);
      }, (e: unknown) => live && setError(e instanceof Error ? e.message : String(e)))
        .finally(() => { pending = false; });
    };
    refresh();
    const timer = window.setInterval(refresh, 5000);
    return () => { live = false; window.clearInterval(timer); };
  }, [path, stamp, retry]);

  const statusRevision = useUiRevision(() => [path, stamp, retry, error, !!data]);
  useUiReading("conversation history status", { path, loaded: !!data, error, scope: "Read-only session lens. Message cards and disclosed work have separate reading controls." });
  useUiCommands(error ? [{ key: `session-retry:${path}`, command: {
    label: "Retry reading conversation history", parent: `session:${path}`, subject: `session:${path}`, parameters: {}, prerequisites: [],
    read: () => ({ revision: statusRevision(), enabled: true }), execute: async () => {
      setRetry(value => value + 1); return { status: "completed", value: { read_requested: true, read_completed: false } };
    },
  } }] : []);

  // One editor for the lens's life; a new version of the file replaces its
  // text rather than remounting, so the cards already on screen stay put.
  useEffect(() => {
    const host = hostRef.current;
    if (!host || !data) return;
    let view = viewRef.current;
    const changed = !view || view.state.doc.toString() !== data.source;
    if (!view) {
      view = new EditorView({
        parent: host,
        state: EditorState.create({
          doc: data.source,
          extensions: [
            editorChrome("document"),
            lineNumbers(),
            // A conversation reads as prose: it wraps at the pane's edge.
            EditorView.lineWrapping,
            EditorState.readOnly.of(true),
            EditorView.editable.of(false),
            sessionChrome,
            renderedBlocks(registry),
          ],
        }),
      });
      viewRef.current = view;
    } else if (view.state.doc.toString() !== data.source) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: data.source },
      });
    }
    // Every element renders as its card: the lens is the conversation, and
    // the source is one click away on any card, as in a document.
    const blocks = renderableBlocks(structureOf(view.state));
    view.dispatch({ effects: setRenderedBlocks.of(blocks.map((b) => b.from)) });
    // A conversation opens at its newest turn, the way the cards always
    // did; the log is the pane's, so it is the log that scrolls.
    const scrollTimers: number[] = [];
    let scrollFrame: number | undefined;
    const log = host.closest<HTMLElement>(".chat-log");
    if (log && changed) {
      // Once now, and again after the cards have measured: the editor
      // virtualises against the log, so its height settles over a few
      // frames as widgets mount.
      const toEnd = () => log.scrollTo({ top: log.scrollHeight });
      scrollFrame = requestAnimationFrame(toEnd);
      for (const ms of [120, 400, 900]) scrollTimers.push(window.setTimeout(toEnd, ms));
    }
    setFacts({
      blocks: data.blocks.map((b) => ({
        ...b,
        span: [byteToChar(data.source, b.span[0]), byteToChar(data.source, b.span[1])] as [number, number],
      })),
      linksAt: linksByAnswer(view.state, data.links),
      view,
    });
    return () => {
      if (scrollFrame !== undefined) cancelAnimationFrame(scrollFrame);
      scrollTimers.forEach(timer => window.clearTimeout(timer));
    };
  }, [data, registry]);

  // Recompute links as folds change. Hidden work cannot leave a ribbon
  // implying that its evidence is presently on screen.
  useEffect(() => {
    const host = hostRef.current;
    if (!host || !data || !facts) return;
    let unregister = () => {};
    const sync = () => {
      const visible = data.links.flatMap(link => {
        const anchor = conversationAnchor(host, byteToChar(data.source, link.span[0]), byteToChar(data.source, link.span[1]), showCollapsedLineage);
        return anchor ? ribbonLinksOf(data.path, [link]).map(ribbon => ({ ...ribbon,
          key: `lens:${data.path}:${link.span.join(":")}:${link.family}:${link.to.path}:${link.to.lines?.join(":")}:${link.evidence?.path}:${link.evidence?.lines.join(":")}`, anchor, alwaysVisible: true })) : [];
      });
      unregister();
      unregister = registerLens({ path: data.path, view: facts.view, source: data.source, links: visible });
    };
    sync();
    host.addEventListener("toggle", sync, true);
    return () => { host.removeEventListener("toggle", sync, true); unregister(); };
  }, [data, facts, slots, showCollapsedLineage]);

  useEffect(
    () => () => {
      viewRef.current?.destroy();
      viewRef.current = null;
    },
    [],
  );

  const cx: SlotContext = useMemo(
    () => ({
      view: facts?.view ?? null,
      path,
      execBlocks: [],
      diagramBlocks: [],
      sessionBlocks: facts?.blocks ?? [],
      sessionLinksAt: facts?.linksAt,
      runningCells: new Set(),
      replaying: [],
      replaceBlockContent: () => {},
    }),
    [path, facts],
  );

  return (
    <div className="session-lens" data-session-lens={path}>
      <SurfaceInfo name="Conversation history" editable={false} destination="Reads the session document. This view writes nothing; replies belong in the conversation editor." />
      {error && <div><p className="error">{error}</p><button type="button" onClick={() => setRetry(value => value + 1)}>Retry reading conversation history</button></div>}
      <div ref={hostRef} className="editor-cm-host session-lens__editor" />
      {slots.map((slot) => {
        const element = elementViews[slot.kind];
        return element ? createPortal(<div data-session-from={slot.span[0]} data-session-to={slot.span[1]}>{element.render(slot, cx)}</div>, slot.el, slot.key) : null;
      })}
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/views/SessionLens.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/SessionLens.tsx"><storycode:paste select="#f-11b8d4e005a0-Blank"/><storycode:paste select="#f-11b8d4e005a0-sessionChrome"/><storycode:paste select="#f-11b8d4e005a0-chromeDecorations"/><storycode:paste select="#f-11b8d4e005a0-linksByAnswer"/><storycode:paste select="#f-11b8d4e005a0-SessionLensProps"/><storycode:paste select="#f-11b8d4e005a0-SessionLens"/><storycode:paste select="#f-11b8d4e005a0-OwnedSessionLens"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-3a59c565f3d4"></a>
## `apps/web/src/views/SessionLens.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace conversation passages without confusing context with support](../../../../../guarantees/agent/an-answer-in-the-agent-pane-has-ribbons/contract.md), [Captured reading surfaces share owned native actions](../../../../../guarantees/agent/scoped-reading-surfaces-share-native-actions/contract.md).

<a id="f-3a59c565f3d4-SOURCE"></a>
### `SOURCE`

agent pane's history is the session document, drawn as cards with line
numbers, and its links reach the overlay through the lens store.

<storycode:copy id="f-3a59c565f3d4-SOURCE">
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
// docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md: the
// agent pane's history is the session document, drawn as cards with line
// numbers, and its links reach the overlay through the lens store.
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { installMockHandler } from "../api/client";
import { lensSources } from "../lib/lensSources";
import { SessionLens } from "./SessionLens";

const SOURCE = `<?xml version="1.0" encoding="UTF-8"?>
<hick:session xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:user turn="t1">Why is checkout slow?</hick:user>
<hick:usage turn="0" input="10" cache-write="0" cache-read="0" output="2"/>
<hick:read file="data/latency.csv" sha256="ff" lines="1-8"/>
<hick:assistant>
<hick:reasoning>
Let me think about the pool.
</hick:reasoning>
It slowed after the pool change; see data/latency.csv:3.
</hick:assistant>
<hick:wrote file="notes/today.md" lines="72-75" hashes="a b c d"/>
</hick:session>
`;

</storycode:copy>
<a id="f-3a59c565f3d4-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-3a59c565f3d4-expression_statement">
afterEach(cleanup);

</storycode:copy>
<a id="f-3a59c565f3d4-span"></a>
### `span`

Byte span of the element that opens with `open` and ends with `close`
(the source is ASCII, so bytes are characters).

<storycode:copy id="f-3a59c565f3d4-span">
/** Byte span of the element that opens with `open` and ends with `close`
 * (the source is ASCII, so bytes are characters). */
function span(open: string, close: string): [number, number] {
  const from = SOURCE.indexOf(open);
  const to = SOURCE.indexOf(close, from) + close.length;
  return [from, to];
}

</storycode:copy>
<a id="f-3a59c565f3d4-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-3a59c565f3d4-expression_statement-2">
describe("the session lens", () => {
  it("draws the conversation's cards over the file, and registers its links", async () => {
    installMockHandler(async (method, path) => {
      if (method === "GET" && path.startsWith("/api/sessions/view")) {
        return {
          path: "sessions/s.md",
          source: SOURCE,
          view: { start: null, doc: null, turns: [] },
          blocks: [
            { kind: "session-user", turn: "t1", body: "Why is checkout slow?", span: span('<hick:user', "</hick:user>") },
            { kind: "session-meta", element: "usage", span: span("<hick:usage", "/>") },
            { kind: "session-read", file: "data/latency.csv", lines: "1-8", sha256: "ff", span: span("<hick:read", "/>") },
            { kind: "session-assistant", body: "It slowed after the pool change; see data/latency.csv:3.", span: span("<hick:assistant>", "</hick:assistant>") },
            { kind: "session-reasoning", body: "Let me think about the pool.", span: span("<hick:reasoning>", "</hick:reasoning>") },
            { kind: "session-wrote", file: "notes/today.md", lines: "72-75", span: span("<hick:wrote", "hashes=\"a b c d\"/>") },
          ],
          links: [
            { family: "context", span: span("<hick:read", "/>"), to: { path: "data/latency.csv", lines: [1, 8] }, title: "ctx", lines: [5, 5] },
            { family: "declared", span: span("<hick:assistant>", "</hick:assistant>"), to: { path: "data/latency.csv", lines: [3, 3] }, title: "dec", lines: [6, 11] },
            { family: "lineage", span: span("<hick:wrote", "hashes=\"a b c d\"/>"), to: { path: "notes/today.md", lines: [72, 75] }, title: "lin", lines: [12, 12] },
          ],
        };
      }
      throw new Error(`unexpected ${method} ${path}`);
    });
    render(<SessionLens path="sessions/s.md" />);
    await waitFor(() => expect(screen.getByText("Why is checkout slow?")).toBeTruthy());
    expect(screen.getByText(/It slowed after the pool change/)).toBeTruthy();
    expect(screen.getByText("data/latency.csv")).toBeTruthy();
    expect(screen.getByText("notes/today.md")).toBeTruthy();
    // Line numbers are the file's.
    expect(document.querySelector(".cm-lineNumbers")).toBeTruthy();
    // The file's own chrome and the record's bookkeeping are not shown as
    // source; the reasoning is there, folded and labelled as such.
    const shown = document.querySelector(".session-lens")!.textContent ?? "";
    expect(shown).not.toContain("<?xml");
    expect(shown).not.toContain("hick:usage");
    expect(shown).not.toContain("</hick:session>");
    const reasoning = document.querySelector("details.chat-reasoning") as HTMLDetailsElement;
    expect(reasoning).toBeTruthy();
    expect(reasoning.open).toBe(false);
    expect(reasoning.querySelector("summary")?.textContent).toBe("reasoning");
    // And the overlay can find the lens: its links, from the session's lines.
    const lens = await waitFor(() => {
      const registered = lensSources().find((l) => l.path === "sessions/s.md");
      expect(registered).toBeTruthy();
      return registered!;
    });
    expect(lens?.links.map((l) => [l.family, l.from.lines, l.to.path])).toEqual([
      ["context", [5, 5], "data/latency.csv"],
      ["declared", [6, 11], "data/latency.csv"],
      ["lineage", [12, 12], "notes/today.md"],
    ]);
  });
  it("removes a tool ribbon while collapsed and restores it when expanded or opted in", async () => {
    const source = '<hick:session>\n<hick:user>Read it</hick:user>\n<hick:context kind="acp-activity">{"kind":"read","title":"Read source","sessionUpdate":"tool_call"}</hick:context>\n<hick:assistant>Done</hick:assistant>\n</hick:session>';
    const tool: [number, number] = [source.indexOf('<hick:context'), source.indexOf('</hick:context>') + '</hick:context>'.length];
    const answer: [number, number] = [source.indexOf('<hick:assistant>'), source.indexOf('</hick:assistant>') + '</hick:assistant>'.length];
    installMockHandler(async () => ({ path: "sessions/fold.md", source, view: { turns: [] },
      blocks: [{ kind: "session-context", context_kind: "acp-activity", body: '{"kind":"read","title":"Read source","sessionUpdate":"tool_call"}', span: tool },
        { kind: "session-assistant", body: "Done", span: answer }],
      links: [{ family: "context", span: tool, lines: [3, 3], to: { path: "source.md" }, title: "read" }],
    }));
    const result = render(<SessionLens path="sessions/fold.md" />);
    await screen.findByText(/Adapter report · Read source/);
    await waitFor(() => expect(lensSources().find(lens => lens.path === "sessions/fold.md")?.links).toEqual([]));
    const fold = screen.getByText(/Adapter report · Read source/).closest("details")!;
    fold.open = true; fireEvent(fold, new Event("toggle"));
    await waitFor(() => expect(lensSources().find(lens => lens.path === "sessions/fold.md")?.links).toHaveLength(1));
    fold.open = false; fireEvent(fold, new Event("toggle"));
    await waitFor(() => expect(lensSources().find(lens => lens.path === "sessions/fold.md")?.links).toEqual([]));
    result.rerender(<SessionLens path="sessions/fold.md" showCollapsedLineage />);
    await waitFor(() => expect(lensSources().find(lens => lens.path === "sessions/fold.md")?.links).toHaveLength(1));
  });

});

</storycode:copy>
<a id="f-3a59c565f3d4-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-3a59c565f3d4-expression_statement-3">
it("retires a previous session's pending response on path replacement", async () => {
  let resolve!: (value: unknown) => void;
  const answer = (path: string, text: string) => ({ path, source: `<hick:session>\n<hick:user>${text}</hick:user>\n</hick:session>`, blocks: [], links: [], view: { turns: [] } });
  installMockHandler(async (_method, path) => path.includes("first.md") ? new Promise(done => { resolve = done; }) : answer("second.md", "second session"));
  const view = render(<SessionLens path="first.md" />);
  await waitFor(() => expect(resolve).toBeTruthy());
  view.rerender(<SessionLens path="second.md" />);
  await screen.findByText("second session");
  await act(async () => { resolve(answer("first.md", "retired session")); });
  expect(screen.queryByText("retired session")).toBeNull();
  expect(document.querySelector('[data-session-lens="first.md"]')).toBeNull();
});
</storycode:copy>
<a id="f-3a59c565f3d4-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-3a59c565f3d4-expression_statement-4">
it("shares recovery after a session read failure", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let reads = 0;
  installMockHandler(async () => { if (++reads === 1) throw Error("read denied"); return { path: "s.md", source: "<hick:session>\n<hick:user>recovered</hick:user>\n</hick:session>", blocks: [], links: [], view: { turns: [] } }; });
  render(<UiActionContext.Provider value={registry}><SessionLens path="s.md" /></UiActionContext.Provider>);
  await screen.findByText("read denied");
  const retry = registry.groups().groups.flatMap(group => registry.observe(group, 64).actions).find(action => action.label === "Retry reading conversation history")!;
  await act(async () => { expect(await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle: retry.handle, arguments: {}, request_id: "retry" })).toMatchObject({ status: "result", outcome: { value: { read_requested: true, read_completed: false } } }); });
  await screen.findByText("recovered");
  expect(screen.queryByText("read denied")).toBeNull();
  expect(reads).toBe(2);
});
</storycode:copy>

### Assemble `apps/web/src/views/SessionLens.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/SessionLens.test.tsx"><storycode:paste select="#f-3a59c565f3d4-SOURCE"/><storycode:paste select="#f-3a59c565f3d4-expression_statement"/><storycode:paste select="#f-3a59c565f3d4-span"/><storycode:paste select="#f-3a59c565f3d4-expression_statement-2"/><storycode:paste select="#f-3a59c565f3d4-expression_statement-3"/><storycode:paste select="#f-3a59c565f3d4-expression_statement-4"/></storycode:file>


</storycode:doc>
