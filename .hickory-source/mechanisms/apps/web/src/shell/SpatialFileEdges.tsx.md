<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/SpatialFileEdges.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-39788750b29a"></a>
## `apps/web/src/shell/SpatialFileEdges.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-39788750b29a-owners"></a>
### `owners`

<storycode:copy id="f-39788750b29a-owners">
import { useUiCommands } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { useEffect, useId, useRef, useState } from "react";
import { EditorView } from "@codemirror/view";

const owners = new WeakMap<EditorView, string>();
</storycode:copy>
<a id="f-39788750b29a-identity"></a>
### `identity`

<storycode:copy id="f-39788750b29a-identity">
function identity(view: EditorView) { let id = owners.get(view); if (!id) { id = crypto.randomUUID(); owners.set(view, id); } return id; }

</storycode:copy>
<a id="f-39788750b29a-SpatialFileEdges"></a>
### `SpatialFileEdges`

Counts refer to the editor's real scroll geometry, including its native block spacers.

<storycode:copy id="f-39788750b29a-SpatialFileEdges">
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
</storycode:copy>

### Assemble `apps/web/src/shell/SpatialFileEdges.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/SpatialFileEdges.tsx"><storycode:paste select="#f-39788750b29a-owners"/><storycode:paste select="#f-39788750b29a-identity"/><storycode:paste select="#f-39788750b29a-SpatialFileEdges"/></storycode:file>

</storycode:doc>
