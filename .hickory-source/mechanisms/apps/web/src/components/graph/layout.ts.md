<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/graph/layout.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8543fec70dc7"></a>
## `apps/web/src/components/graph/layout.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-8543fec70dc7-autoLayout"></a>
### `autoLayout`

Auto-layout: dagre's layered layout over the whole scene.
A deliberate VERB — the toolbar button — never something a re-run or a
paste applies to a layout somebody arranged. See scene.ts::placeMissing for
what happens to nodes that merely lack a position.

<storycode:copy id="f-8543fec70dc7-autoLayout">
// Auto-layout: dagre's layered layout over the whole scene.
//
// A deliberate VERB — the toolbar button — never something a re-run or a
// paste applies to a layout somebody arranged. See scene.ts::placeMissing for
// what happens to nodes that merely lack a position.

import dagre from "@dagrejs/dagre";

import { DEFAULT_H, DEFAULT_W, type NodeLayout, type Scene } from "./scene";

export function autoLayout(scene: Scene): Record<string, NodeLayout> {
  const g = new dagre.graphlib.Graph();
  g.setGraph({ rankdir: "TB", nodesep: 48, ranksep: 64 });
  g.setDefaultEdgeLabel(() => ({}));
  for (const node of scene.nodes) {
    const at = scene.layout[node.id];
    g.setNode(node.id, {
      width: at?.w ?? DEFAULT_W,
      height: at?.h ?? DEFAULT_H,
    });
  }
  for (const edge of scene.edges) {
    if (g.hasNode(edge.from) && g.hasNode(edge.to)) g.setEdge(edge.from, edge.to);
  }
  dagre.layout(g);
  const out: Record<string, NodeLayout> = {};
  for (const node of scene.nodes) {
    const placed = g.node(node.id);
    if (!placed) continue;
    // dagre reports centers; the scene records top-left corners.
    out[node.id] = {
      x: Math.round(placed.x - placed.width / 2),
      y: Math.round(placed.y - placed.height / 2),
      ...(scene.layout[node.id]?.w !== undefined ? { w: scene.layout[node.id].w } : {}),
      ...(scene.layout[node.id]?.h !== undefined ? { h: scene.layout[node.id].h } : {}),
    };
  }
  return out;
}
</storycode:copy>

### Assemble `apps/web/src/components/graph/layout.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/graph/layout.ts"><storycode:paste select="#f-8543fec70dc7-autoLayout"/></storycode:file>

</storycode:doc>
