<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/graph/layout.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `02b902bcf02725ad95b0b387bca7929344e28d41958fd749aba039c2c575633a`.

<book:entry path="apps/web/src/components/graph/layout.ts">
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
</book:entry>

</book:doc>
