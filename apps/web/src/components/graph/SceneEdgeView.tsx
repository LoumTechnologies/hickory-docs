// One line on the canvas: colour, label, path — and, on double-click, the
// label editor right on the line.
//
// Rounded elbows read well across open space and badly between neighbours: a
// smoothstep between two boxes sitting next to each other folds its corner
// radii into an S in the little gap between them. So a SHORT line is drawn
// straight — the distance between two adjacent boxes' rims is small by
// definition — and everything longer keeps its rounded corners.

import { useEffect, useRef } from "react";
import { BaseEdge, EdgeLabelRenderer, getSmoothStepPath, getStraightPath } from "@xyflow/react";
import type { EdgeProps } from "@xyflow/react";

import type { SceneEdge } from "./scene";

/** Below this many pixels between endpoints, the line goes straight. */
const STRAIGHT_BELOW = 96;

interface SceneEdgeData {
  edge?: SceneEdge;
  /** This line's label is being edited (double-click put it there). */
  editing?: boolean;
  onLabel?: (label: string) => void;
  onCancel?: () => void;
}

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
  const { edge, editing, onLabel, onCancel } = (data ?? {}) as SceneEdgeData;
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
    <>
      <BaseEdge
        id={id}
        path={path}
        markerStart={markerStart}
        markerEnd={markerEnd}
        style={style}
        interactionWidth={interactionWidth}
        label={editing ? undefined : label}
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
      {editing && (
        <EdgeLabelRenderer>
          {/* The editor sits ON the line, where the words will live —
              not in a toolbar the eye has to travel to. */}
          <LabelInput
            x={labelX}
            y={labelY}
            initial={edge?.label ?? ""}
            onLabel={onLabel}
            onCancel={onCancel}
          />
        </EdgeLabelRenderer>
      )}
    </>
  );
}

function LabelInput({
  x,
  y,
  initial,
  onLabel,
  onCancel,
}: {
  x: number;
  y: number;
  initial: string;
  onLabel?: (label: string) => void;
  onCancel?: () => void;
}) {
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => {
    ref.current?.select();
  }, []);
  return (
    <input
      ref={ref}
      className="graph-edge__label-input nodrag nopan"
      style={{
        position: "absolute",
        transform: `translate(-50%, -50%) translate(${x}px, ${y}px)`,
        pointerEvents: "all",
      }}
      aria-label="Line label"
      placeholder="label…"
      defaultValue={initial}
      autoFocus
      onBlur={(e) => onLabel?.(e.target.value)}
      onKeyDown={(e) => {
        if (e.key === "Enter") (e.target as HTMLInputElement).blur();
        if (e.key === "Escape") {
          e.stopPropagation();
          onCancel?.();
        }
      }}
    />
  );
}
