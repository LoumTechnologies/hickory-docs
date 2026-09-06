// The selection inspector's half of "a box leads somewhere".
//
// The gesture is a double-click on the node; this is the affordance that says
// so, because a gesture nobody can see is a gesture nobody uses. It sits in
// the inspector row rather than on the shape: the row is already where a
// selected thing's facts and verbs live, it has room for the PATH itself —
// which is the fact worth reading before you follow it — and it costs the
// canvas no extra ink. The node draws only a small corner glyph, enough to
// say the box is a door (SceneNodeView).
//
// Shown for derived scenes too. `path` is topology, so a generator wrote it
// and the canvas never edits it; a read-only picture being navigable is the
// whole point rather than an exception.

import { openTarget } from "./openNode";
import type { SceneNode } from "./scene";

export function NodeLink({
  node,
  docPath,
}: {
  node: SceneNode;
  docPath: string | null;
}) {
  const path = node.path?.trim() || undefined;
  const doc = node.doc?.trim() || undefined;
  if (!path && !doc) return null;
  return (
    <span className="graph-inspector__group" role="group" aria-label="Open">
      {path && (
        <button
          type="button"
          className="graph-inspector__open"
          aria-label={`Open ${path}`}
          data-tip={`Open ${path} — or double-click the box`}
          onClick={() => openTarget(docPath, path)}
        >
          ↗ {path}
        </button>
      )}
      {doc && (
        <button
          type="button"
          className="graph-inspector__open"
          aria-label={`Open document ${doc}`}
          data-tip={`Open ${doc} — the document about this node`}
          onClick={() => openTarget(docPath, doc)}
        >
          ▤ {doc}
        </button>
      )}
    </span>
  );
}
