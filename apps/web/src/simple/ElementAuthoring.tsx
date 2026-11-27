import { useEffect, useMemo, useState } from "react";
import { EditorView } from "@codemirror/view";
import { isolateHistory } from "@codemirror/commands";
import { ElementDialog, elementsByLine } from "../components/ElementAuthoring";
import { readElements, type ElementReading, type ElementTarget, type ElementPlan } from "../components/ElementAuthoringModel";
import { authoredChanges, authoredOffset, authoredSource, editorOffsetIn } from "./SourceText";
import { readSourceEditorIntent } from "./SourceSnippets";
import type { Buffer } from "./buffers";

const insertCommands = new WeakMap<Buffer, () => boolean>();
export function canAuthorElements(buffer: Buffer | undefined): buffer is Buffer {
  return !!buffer && !buffer.readonly && !buffer.saving && !buffer.resourceDeleted && !buffer.transient
    && (!!buffer.file.doc_id || /\.(md|markdown)$/i.test(buffer.file.path));
}
export function insertDocumentElement(buffer: Buffer) { return insertCommands.get(buffer)?.() ?? false; }

type Intent = {
  reading: ElementReading; range: { from: number; to: number }; targets?: ElementTarget[];
  anchor?: HTMLElement; current: () => boolean;
};

export function useElementAuthoring(buffer: Buffer, view: EditorView | null, parserReady: boolean, current?: () => boolean) {
  const [reading, setReading] = useState<ElementReading>();
  const [intent, setIntent] = useState<Intent>();
  const eligible = canAuthorElements(buffer), source = buffer.text;
  useEffect(() => {
    if (!eligible || !parserReady) return;
    const handle = window.setTimeout(() => {
      try { setReading(readElements(source)); }
      catch { setReading(undefined); }
    }, 70);
    return () => window.clearTimeout(handle);
  }, [buffer, source, eligible, parserReady]);
  const fresh = eligible && parserReady && reading?.source === source ? reading : undefined;
  const lines = useMemo(() => fresh ? elementsByLine(fresh) : new Map<number, ElementTarget[]>(), [fresh]);
  const [viewport, setViewport] = useState({ first: 0, last: 0 });
  useEffect(() => {
    if (!view || !eligible) return;
    let frame = 0;
    const measure = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const { from, to } = view.viewport;
        const doc = view.state.doc;
        const first = Math.max(0, doc.lineAt(from).number - 4);
        const last = Math.min(doc.lines - 1, doc.lineAt(to).number + 2);
        setViewport(previous => previous.first === first && previous.last === last ? previous : { first, last });
      });
    };
    measure();
    view.scrollDOM.addEventListener("scroll", measure, { passive: true });
    const resize = typeof ResizeObserver === "undefined" ? undefined : new ResizeObserver(measure);
    resize?.observe(view.scrollDOM);
    resize?.observe(view.contentDOM);
    return () => { cancelAnimationFrame(frame); resize?.disconnect(); view.scrollDOM.removeEventListener("scroll", measure); };
  }, [view, eligible, source]);
  const close = () => { setIntent(undefined); view?.focus(); };
  function open(captured: ElementReading | undefined, targets?: ElementTarget[], anchor?: HTMLElement) {
    if (!view || !view.dom.isConnected || view.composing || view.state.readOnly || !view.state.facet(EditorView.editable)
      || !canAuthorElements(buffer) || !parserReady || current?.() === false || authoredSource(view.state) !== buffer.text) return false;
    const selected = view.state.selection;
    if (!targets && selected.ranges.length !== 1) return false;
    const { from, to } = selected.main;
    const revision = buffer.sourceRevision, path = buffer.file.path, editorIntent = readSourceEditorIntent(view.state);
    let next;
    try { next = captured ?? readElements(buffer.text); }
    catch { return false; }
    if (next.source !== buffer.text) return false;
    const row = anchor?.closest<HTMLElement>(".cm-debug-rail__group") ?? anchor;
    setIntent({ reading: next, range: { from: authoredOffset(view.state, from), to: authoredOffset(view.state, to) }, targets, anchor: row,
      current: () => view.dom.isConnected && !view.composing && !view.state.readOnly && view.state.facet(EditorView.editable)
        && canAuthorElements(buffer) && current?.() !== false && buffer.file.path === path && buffer.sourceRevision === revision
        && buffer.text === next.source && authoredSource(view.state) === next.source && readSourceEditorIntent(view.state) === editorIntent,
    });
    return true;
  }
  useEffect(() => {
    if (!view) return;
    const command = () => open(fresh);
    insertCommands.set(buffer, command);
    const shortcut = (event: KeyboardEvent) => {
      if (!event.defaultPrevented && (event.metaKey || event.ctrlKey) && !event.altKey && !event.shiftKey
        && event.key.toLowerCase() === "i" && command()) event.preventDefault();
    };
    view.dom.addEventListener("keydown", shortcut);
    return () => { insertCommands.delete(buffer); view.dom.removeEventListener("keydown", shortcut); };
  });
  function apply(plan: ElementPlan) {
    if (!view || !intent?.current()) return "The editor changed. Close this form and open it again.";
    const source = intent.reading.source;
    const next = source.slice(0, plan.from) + plan.insert + source.slice(plan.to);
    view.dispatch({
      ...authoredChanges(view.state, [{ from: plan.from, to: plan.to, insert: plan.insert }]),
      selection: { anchor: editorOffsetIn(next, plan.selectFrom), head: editorOffsetIn(next, plan.selectTo) },
      annotations: isolateHistory.of("full"),
      userEvent: "input.element",
    });
    close();
  }
  return {
    actions: view && fresh ? Array.from({ length: viewport.last - viewport.first + 1 }, (_, index) => viewport.first + index).flatMap(line => {
      const targets = lines.get(line);
      if (!targets || line >= view.state.doc.lines) return [];
      return [{ at: view.state.doc.line(line + 1).from, label: `Edit ${targets.length === 1 ? targets[0].option.title : "elements"} on line ${line + 1}`,
        glyph: "✎", run: (anchor?: HTMLElement) => open(fresh, targets, anchor) }];
    }) : [],
    dialog: intent && <ElementDialog reading={intent.reading} range={intent.range} targets={intent.targets}
      anchor={intent.anchor} close={close} apply={apply} />,
  };
}
