// One line on the canvas: colour, label, and the path itself.
//
// Rounded elbows read well across open space and badly between neighbours: a
// smoothstep between two boxes sitting next to each other folds its corner
// radii into an S in the little gap between them. So a SHORT line is drawn
// straight — the distance between two adjacent boxes' rims is small by
// definition — and everything longer keeps its rounded corners.

import { BaseEdge, getSmoothStepPath, getStraightPath } from "@xyflow/react";
import type { EdgeProps } from "@xyflow/react";

import type { SceneEdge } from "./scene";

/** Below this many pixels between endpoints, the line goes straight. */
const STRAIGHT_BELOW = 96;

export function SceneEdgeView(props: EdgeProps) {
  const {
    id,
    sourceX,
    sourceY,
    targetX,
    targetY,
    sourcePosition,
    targetPosition,
    markerStart,
    markerEnd,
    style,
    label,
    interactionWidth,
    data,
  } = props;
  const edge = (data as { edge?: SceneEdge } | undefined)?.edge;
  const short = Math.hypot(targetX - sourceX, targetY - sourceY) < STRAIGHT_BELOW;
  const [path, labelX, labelY] = short
    ? getStraightPath({ sourceX, sourceY, targetX, targetY })
    : getSmoothStepPath({
        sourceX,
        sourceY,
        targetX,
        targetY,
        sourcePosition,
        targetPosition,
        borderRadius: 8,
      });
  return (
    <BaseEdge
      id={id}
      path={path}
      markerStart={markerStart}
      markerEnd={markerEnd}
      style={style}
      interactionWidth={interactionWidth}
      label={label}
      labelX={labelX}
      labelY={labelY}
      labelStyle={{
        fill: edge?.color ?? "var(--fg)",
        fontSize: 11,
      }}
      labelBgStyle={{ fill: "var(--bg-sunken)", fillOpacity: 0.85 }}
      labelBgPadding={[4, 2]}
      labelBgBorderRadius={3}
    />
  );
}
