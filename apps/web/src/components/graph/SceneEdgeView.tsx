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

import { straightConnector } from "./scene";
import type { SceneEdge } from "./scene";

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
  const straight = straightConnector(
    sourcePosition ?? "right",
    targetPosition ?? "left",
    targetX - sourceX,
    targetY - sourceY,
  );
  const [path, labelX, labelY] = straight
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
  // While the label is being edited the line STOPS at the editor and picks up
  // again on the other side, instead of running behind it. The input is opaque,
  // so this is not about seeing through it — a line crossing the middle of a
  // text box reads as a strikethrough, which says "deleted" about the words you
  // are typing. A gap says "this label belongs to this line" instead.
  //
  // A mask rather than a patch of background colour: the canvas has its own
  // pattern behind it, and a filled rectangle would sit on it as a visible
  // hole. The mask travels with the path, so it is right at every zoom.
  const gapId = `edge-label-gap-${id}`;
  return (
    <>
      {editing && (
        <mask id={gapId} maskUnits="userSpaceOnUse">
          {/* White shows the path, black hides it. The cover is deliberately
              enormous: a mask's default region is the object's bounding box,
              and a path that is a horizontal line has zero height there. */}
          <rect x={-100000} y={-100000} width={200000} height={200000} fill="white" />
          <rect
            x={labelX - LABEL_GAP_WIDTH / 2}
            y={labelY - LABEL_GAP_HEIGHT / 2}
            width={LABEL_GAP_WIDTH}
            height={LABEL_GAP_HEIGHT}
            rx={4}
            fill="black"
          />
        </mask>
      )}
      <BaseEdge
        id={id}
        path={path}
        mask={editing ? `url(#${gapId})` : undefined}
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

/** The hole the label editor gets in the line, in flow units.
 *
 * Sized from `.graph-edge__label-input` — `width: 8rem` (128px, border-box)
 * and a ~17px tall box — plus a little clearance so the line does not kiss
 * the border. The label layer is transformed by the same viewport the edges
 * are, so these units track the input at any zoom. */
const LABEL_GAP_WIDTH = 136;
const LABEL_GAP_HEIGHT = 24;

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
