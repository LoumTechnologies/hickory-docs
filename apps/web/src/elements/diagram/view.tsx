import { workspaceLinkTarget, openDiagramLocation } from "../../lib/revealLine";
import { Suspense, lazy, useEffect, useState } from "react";
import { request } from "../../api/client";

import { DiagramPanel } from "../../components/DiagramPanel";
import { structureOf } from "../../editor/wysiwyg";
import { assertionStates, matchBlock } from "../../lib/blockMatch";
import type { ElementView } from "../types";

// Lazy for the same reason mermaid is: the canvas library must never reach
// the marketing site's bundle, which builds from this tree.
const GraphEditorPanel = lazy(() => import("../../components/graph/GraphEditorPanel"));

function CodeModelDiagram({ path, script, captureSource }: { path: string; script: string; captureSource: () => () => boolean }) {
  const [attempt, refresh] = useState(0);
  const [result, setResult] = useState<{ source?: string; error?: string }>({});
  useEffect(() => {
    let live = true;
    setResult({});
    const uri = "hick:///" + path.split("/").map(encodeURIComponent).join("/");
    let timer: ReturnType<typeof setTimeout>;
    let revision: string | undefined;
    async function read() {
      try {
        const update = await request<{ revision: string; scene: object | null }>(
          "POST", "/api/diagram/model", { uri, script, revision }
        );
        if (live && update.scene) {
          revision = update.revision;
          setResult({ source: JSON.stringify(update.scene) });
        }
      } catch (error) {
        revision = undefined;
        if (live) setResult({ error: String(error) });
      } finally {
        if (live) timer = setTimeout(read, 2500);
      }
    }
    void read();
    return () => { live = false; clearTimeout(timer); };
  }, [path, script, attempt]);
  function follow(id: string) {
    const current = captureSource();
    if (!current() || !result.source) return;
    const scene = JSON.parse(result.source);
    const nodes = (scene.topology?.nodes ?? scene.nodes ?? []) as { id: string; href?: string }[];
    const href = nodes.find(item => item.id === id)?.href;
    if (typeof href === "string") openDiagramLocation(href, path, current);
  }
  return <div className="rendered-diagram" style={{ position: "relative" }}>
    <button type="button" className="debug-icon" aria-label="Refresh diagram" onClick={() => refresh(n => n + 1)}
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
    const view = cx.view, capturedSource = slot.text;
    const sourceCurrent = () => {
      if (!view?.dom.isConnected) return false;
      const block = structureOf(view.state).blocks.find(block => block.from === slot.at);
      if (block?.name !== "diagram" || block.attrs.renderer !== slot.renderer) return false;
      const body = view.state.doc.sliceString(block.contentFrom, block.contentTo);
      return slot.renderer === "code-model" ? body.trim() === capturedSource.trim() : body === capturedSource;
    };
    const captureSource = () => {
      const revision = view?.state.doc;
      return () => view?.state.doc === revision && sourceCurrent();
    };
    if (slot.renderer === "code-model" && cx.path) {
      return <CodeModelDiagram key={JSON.stringify([cx.path, slot.at, slot.text])} path={cx.path} script={slot.text} captureSource={captureSource} />;
    }
    const openLink = (href: string) => { if (cx.path) openDiagramLocation(href, cx.path, captureSource()); };
    if (slot.renderer === "graph") {
      const linkFor = (href: string) => {
        const target = workspaceLinkTarget(href, cx.path ?? "");
        if (!target) return undefined;
        return openLink.bind(undefined, href);
      };
      return (
        <div className="rendered-diagram" onDoubleClickCapture={event => {
          if (!sourceCurrent()) return;
          const element = event.target as Element;
          if (element.closest(".graph-node__open")) return;
          const node = element.closest('[data-testid^="graph-node-"]');
          if (!node) return;
          try {
            const graph = JSON.parse(resolved ?? slot.text), nodes = graph.topology?.nodes ?? graph.nodes ?? [];
            const href = nodes.find((item: { id: string }) => `graph-node-${item.id}` === node.getAttribute("data-testid"))?.href;
            const follow = typeof href === "string" ? linkFor(href) : undefined;
            if (follow) { event.stopPropagation(); follow(); }
          } catch { /* Invalid scenes retain their renderer's source/error fallback. */ }
        }}>
          <Suspense fallback={<div className="graph-editor graph-editor--loading" />}>
            <GraphEditorPanel
              source={slot.text}
              linkFor={linkFor}
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
          openLink={openLink}
          assertions={states}
          systemAppearance={cx.systemAppearance}
        />
      </div>
    );
  },
};
