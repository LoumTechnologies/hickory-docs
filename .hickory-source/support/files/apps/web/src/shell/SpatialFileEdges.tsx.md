<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/SpatialFileEdges.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `4a2279a3e0fbbfc1ddd4f651f80bd62dc51150e95769da75ca52b3c69f986191`.

<book:entry path="apps/web/src/shell/SpatialFileEdges.tsx">
import { useUiCommands } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { useEffect, useId, useRef, useState } from "react";
import { EditorView } from "@codemirror/view";

const owners = new WeakMap<EditorView, string>();
function identity(view: EditorView) { let id = owners.get(view); if (!id) { id = crypto.randomUUID(); owners.set(view, id); } return id; }

/** Counts refer to the editor's real scroll geometry, including its native block spacers. */
export function SpatialFileEdges() {
  const parent = `spatial-file-edges:${useId()}`;
  const [error, setError] = useState("");
  const root = useRef<HTMLDivElement>(null), [counts, setCounts] = useState({ above: 0, below: 0 });
  const editor = () => { const el = root.current?.closest("[data-spatial-files]")?.querySelector<HTMLElement>(".filesystem-editor .cm-editor"); return el ? EditorView.findFromDOM(el) : null; };
  useEffect(() => {
    const pane = root.current?.closest("[data-spatial-files]");
    const refresh = () => { const next = measure() ?? { above: 0, below: 0 }; setCounts(old => old.above === next.above && old.below === next.below ? old : next); };
    pane?.addEventListener("scroll", refresh, true); const timer = window.setInterval(refresh, 400); refresh();
    return () => { pane?.removeEventListener("scroll", refresh, true); clearInterval(timer); };
  }, []);
  const measure = () => {
    const view = editor(); if (!view?.dom.isConnected || !view.scrollDOM.clientHeight) return null;
    const top = view.lineBlockAtHeight(view.scrollDOM.scrollTop), bottom = view.lineBlockAtHeight(view.scrollDOM.scrollTop + view.scrollDOM.clientHeight);
    return { above: Math.max(0, view.state.doc.lineAt(top.from).number - 1), below: Math.max(0, view.state.doc.lines - view.state.doc.lineAt(bottom.to).number) };
  };
  useUiReading("file-list scroll status", { ...counts, error, scope: "Viewport geometry only; scrolling does not disclose file contents" });
  const invoke = useUiCommands(([-1, 1] as const).map(direction => ({ key: `${parent}:${direction}`, command: {
    parent, subject: parent, label: direction < 0 ? "Scroll file names up" : "Scroll file names down", parameters: {}, prerequisites: ["Mounted visible file-list editor with hidden lines in this direction"],
    read: () => { const view = editor(), current = measure(); return { revision: view ? JSON.stringify([identity(view), view.state.doc.toString(), view.scrollDOM.scrollTop, view.scrollDOM.clientHeight, current]) : "unavailable", enabled: !!current && (direction < 0 ? current.above : current.below) > 0, reason: "No measured hidden file names in this direction" }; },
    execute: async () => {
      const view = editor(), current = measure();
      if (!view || !current || !(direction < 0 ? current.above : current.below)) return { status: "unavailable", reason: "The measured file-list viewport has no hidden lines in this direction" };
      const before = view.scrollDOM.scrollTop; view.scrollDOM.scrollTop += direction * 108;
      return { status: "completed", value: { before, after: view.scrollDOM.scrollTop, scope: "Actual scroll position; no file contents disclosed" } };
    },
  } })));
  const act = (direction: number) => { void invoke(`${parent}:${direction}`).catch(reason => setError(String(reason))); };
  return <div ref={root} className="spatial-file-edges">{error && <span role="alert">{error}</span>}<button data-tip="Scroll file names up" aria-label="Scroll file names up" disabled={!counts.above} onClick={() => act(-1)}>↑ {counts.above} hidden</button><button data-tip="Scroll file names down" aria-label="Scroll file names down" disabled={!counts.below} onClick={() => act(1)}>↓ {counts.below} hidden</button></div>;
}
</book:entry>

</book:doc>
