import { workspaceLinkTarget, openLocation } from "../../lib/revealLine";
import { Suspense, lazy, useEffect, useState } from "react";
import { request } from "../../api/client";

import { DiagramPanel } from "../../components/DiagramPanel";
import { structureOf } from "../../editor/wysiwyg";
import { assertionStates, matchBlock } from "../../lib/blockMatch";
import type { ElementView } from "../types";

// Lazy for the same reason mermaid is: the canvas library must never reach
// the marketing site's bundle, which builds from this tree.
const GraphEditorPanel = lazy(() => import("../../components/graph/GraphEditorPanel"));

function ArchitectureDiagram({ path, current }: { path: string; current: () => boolean }) {
  const [attempt, refresh] = useState(0);
  const [result, setResult] = useState<{ source?: string; error?: string }>({});
  useEffect(() => {
    let live = true;
    setResult({});
    const uri = "hick:///" + path.split("/").map(encodeURIComponent).join("/");
    void request<object>("POST", "/api/architecture", { uri }).then(
      scene => { if (live) setResult({ source: JSON.stringify(scene) }); },
      error => { if (live) setResult({ error: String(error) }); },
    );
    return () => { live = false; };
  }, [path, attempt]);
  function follow(id: string) {
    if (!current() || !result.source) return;
    const nodes = JSON.parse(result.source).topology.nodes as { id: string; href?: string }[];
    const href = nodes.find(item => item.id === id)?.href;
    const target = href && workspaceLinkTarget(href, path);
    if (target) openLocation(target.path, target.line);
  }
  return <div className="rendered-diagram" style={{ position: "relative" }}>
    <button type="button" className="debug-icon" aria-label="Refresh architecture" onClick={() => refresh(n => n + 1)}
      style={{ position: "absolute", right: 4, top: 4, zIndex: 2 }}>↻</button>
    {result.source ? <Suspense fallback={null}><GraphEditorPanel source={result.source} onOpenNode={follow} sourceWritable={() => false} onCommit={() => false} /></Suspense>
      : <p role="status">{result.error ?? "Reading the code model…"}</p>}
  </div>;
}

/** `<hick:diagram>`: a picture drawn from text, by mermaid or the graph editor. */
export const diagramView: ElementView = {
  kind: "diagram",
  draws: (block) => block.name === "diagram",
  render(slot, cx) {
    // The asserting cells' live run state, so the panel can say "out of
    // date" the moment a named cell fails — not just at weave.
    const states = cx.view
      ? assertionStates(structureOf(cx.view.state), cx.execBlocks, slot.asserts, cx.runningCells)
      : slot.asserts.map((id) => ({ id, state: "unknown" as const }));
    // A derived diagram's source holds a `<hick:paste>` where its edges
    // should be; the server's block carries the body with the fragment
    // inlined. Only then — the raw text is live while the user types, and
    // the resolved copy is only as fresh as the last render.
    const resolved = slot.text.includes("<hick:paste")
      ? (matchBlock({ span: slot.span, index: slot.index }, cx.diagramBlocks)?.body ?? null)
      : null;
    if (slot.renderer === "architecture" && cx.path) {
      return <ArchitectureDiagram key={cx.path} path={cx.path} current={() => {
        const view = cx.view;
        return !!view?.dom.isConnected && structureOf(view.state).blocks.some(block =>
          block.from === slot.at && block.name === "diagram" && block.attrs.renderer === "architecture");
      }} />;
    }
    if (slot.renderer === "graph") {
      const view = cx.view, capturedSource = slot.text;
      const sourceCurrent = () => {
        if (!view?.dom.isConnected) return false;
        const block = structureOf(view.state).blocks.find(block => block.from === slot.at);
        return !!block && view.state.doc.sliceString(block.contentFrom, block.contentTo) === capturedSource;
      };
      return (
        <div className="rendered-diagram" onDoubleClickCapture={event => {
          if (!sourceCurrent()) return;
          const node = (event.target as Element).closest('[data-testid^="graph-node-"]');
          if (!node) return;
          try {
            const graph = JSON.parse(resolved ?? slot.text), nodes = graph.topology?.nodes ?? graph.nodes ?? [];
            const href = nodes.find((item: { id: string }) => `graph-node-${item.id}` === node.getAttribute("data-testid"))?.href;
            const target = typeof href === "string" ? workspaceLinkTarget(href, cx.path ?? "") : null;
            if (target) { event.stopPropagation(); openLocation(target.path, target.line); }
          } catch { /* Invalid scenes retain their renderer's source/error fallback. */ }
        }}>
          <Suspense fallback={<div className="graph-editor graph-editor--loading" />}>
            <GraphEditorPanel
              source={slot.text}
              sourceCurrent={sourceCurrent}
              sourceWritable={() => !!view && !view.state.readOnly}
              sourceOwner={view ?? undefined}
              sourceRevision={() => view?.state.doc ?? null}
              resolved={resolved}
              assertions={states}
              onCommit={(text, actor) => sourceCurrent() && !view?.state.readOnly ? cx.replaceBlockContent(slot, text, actor?.kind === "agent" ? "input.uiAgentDiagram" : "input.diagram") : false}
            />
          </Suspense>
        </div>
      );
    }
    return (
      <div className="rendered-diagram">
        <DiagramPanel
          path={cx.path ?? undefined}
          renderer={slot.renderer}
          source={resolved ?? slot.text}
          domId={`hick-diagram-${slot.index}`}
          assertions={states}
          systemAppearance={cx.systemAppearance}
        />
      </div>
    );
  },
};
