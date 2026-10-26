<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/elements/diagram/view.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9a9c534677d6"></a>
## `apps/web/src/elements/diagram/view.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-9a9c534677d6-GraphEditorPanel"></a>
### `GraphEditorPanel`

<storycode:copy id="f-9a9c534677d6-GraphEditorPanel">
import { Suspense, lazy } from "react";

import { DiagramPanel } from "../../components/DiagramPanel";
import { structureOf } from "../../editor/wysiwyg";
import { assertionStates, matchBlock } from "../../lib/blockMatch";
import type { ElementView } from "../types";

// Lazy for the same reason mermaid is: the canvas library must never reach
// the marketing site's bundle, which builds from this tree.
const GraphEditorPanel = lazy(() => import("../../components/graph/GraphEditorPanel"));

</storycode:copy>
<a id="f-9a9c534677d6-diagramView"></a>
### `diagramView`

`<hick:diagram>`: a picture drawn from text, by mermaid or the graph editor.

<storycode:copy id="f-9a9c534677d6-diagramView">
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
    if (slot.renderer === "graph") {
      const view = cx.view, capturedSource = slot.text;
      const sourceCurrent = () => {
        if (!view?.dom.isConnected) return false;
        const block = structureOf(view.state).blocks.find(block => block.from === slot.at);
        return !!block && view.state.doc.sliceString(block.contentFrom, block.contentTo) === capturedSource;
      };
      return (
        <div className="rendered-diagram">
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
          renderer={slot.renderer}
          source={resolved ?? slot.text}
          domId={`hick-diagram-${slot.index}`}
          assertions={states}
        />
      </div>
    );
  },
};
</storycode:copy>

### Assemble `apps/web/src/elements/diagram/view.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/diagram/view.tsx"><storycode:paste select="#f-9a9c534677d6-GraphEditorPanel"/><storycode:paste select="#f-9a9c534677d6-diagramView"/></storycode:file>

</storycode:doc>
