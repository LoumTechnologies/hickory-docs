import { useEffect, useRef, useState } from "react";
import { EditorView } from "@codemirror/view";

/** Counts refer to the editor's real scroll geometry, including its native block spacers. */
export function SpatialFileEdges() {
  const root = useRef<HTMLDivElement>(null), [counts, setCounts] = useState({ above: 0, below: 0 });
  const editor = () => { const el = root.current?.closest("[data-spatial-files]")?.querySelector<HTMLElement>(".filesystem-editor .cm-editor"); return el ? EditorView.findFromDOM(el) : null; };
  useEffect(() => {
    const pane = root.current?.closest("[data-spatial-files]");
    const measure = () => { const view = editor(); if (!view || !view.scrollDOM.clientHeight) return; const top = view.lineBlockAtHeight(view.scrollDOM.scrollTop), bottom = view.lineBlockAtHeight(view.scrollDOM.scrollTop + view.scrollDOM.clientHeight); const next = { above: view.state.doc.lineAt(top.from).number - 1, below: view.state.doc.lines - view.state.doc.lineAt(bottom.to).number }; setCounts(old => old.above === next.above && old.below === next.below ? old : next); };
    pane?.addEventListener("scroll", measure, true); const timer = window.setInterval(measure, 400); measure();
    return () => { pane?.removeEventListener("scroll", measure, true); clearInterval(timer); };
  }, []);
  const shift = (direction: number) => { const view = editor(); if (view) view.scrollDOM.scrollTop += direction * 108; };
  return <div ref={root} className="spatial-file-edges"><button data-tip="Scroll file names up" aria-label="Scroll file names up" disabled={!counts.above} onClick={() => shift(-1)}>↑ {counts.above} hidden</button><button data-tip="Scroll file names down" aria-label="Scroll file names down" disabled={!counts.below} onClick={() => shift(1)}>↓ {counts.below} hidden</button></div>;
}
