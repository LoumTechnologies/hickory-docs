// One node on the graph canvas: the shape, the label, and — on double click,
// when the topology is the author's to edit — the label as an input.
//
// A derived node's label is NOT editable: the topology came from a generator
// through a paste, and an edit here would be silently thrown away by the next
// re-run. The panel's banner says where to change it instead. Resizing is
// allowed on BOTH kinds — a size is layout, the person's half.

import { useEffect, useRef, useState } from "react";
import { Handle, NodeResizer, Position } from "@xyflow/react";
import type { NodeProps, ResizeParams } from "@xyflow/react";

import { sideRef, slotLayout } from "./scene";
import type { SceneNode, Side } from "./scene";

export interface SceneNodeData extends Record<string, unknown> {
  node: SceneNode;
  derived: boolean;
  onRename: (id: string, label: string) => void;
  /** A resize ended: the node's new place and size, to snap and commit. */
  onResize: (id: string, layout: { x: number; y: number; w: number; h: number }) => void;
  /** Which slots each side has lines attached to — decides how many bubbles
   * the side offers (every occupied one, plus one free). */
  slots: Partial<Record<Side, number[]>>;
}

export function SceneNodeView({ data, selected }: NodeProps) {
  const { node, derived, onRename, onResize, slots } = data as SceneNodeData;
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(node.label ?? node.id);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (editing) inputRef.current?.select();
  }, [editing]);

  const finish = (keep: boolean) => {
    setEditing(false);
    if (keep && draft.trim() && draft !== (node.label ?? node.id)) {
      onRename(node.id, draft.trim());
    } else {
      setDraft(node.label ?? node.id);
    }
  };

  const shape = node.shape ?? "rect";
  return (
    <div
      className={`graph-node graph-node--${shape}${selected ? " graph-node--selected" : ""}`}
      style={{
        ...(node.fill ? { background: node.fill } : {}),
        ...(node.stroke ? { borderColor: node.stroke } : {}),
        ...(node.text ? { color: node.text } : {}),
      }}
      onDoubleClick={() => {
        if (!derived) setEditing(true);
      }}
      data-testid={`graph-node-${node.id}`}
    >
      {/* Resize is a layout act, so it is never locked — a derived node may
          be resized like any other. The commit snaps to the grid. */}
      <NodeResizer
        isVisible={selected ?? false}
        minWidth={96}
        minHeight={48}
        onResizeEnd={(_event: unknown, params: ResizeParams) =>
          onResize(node.id, {
            x: params.x,
            y: params.y,
            w: params.width,
            h: params.height,
          })
        }
      />
      {/* Connection points live on SIDES, never corners — corners belong to
          resizing. Each side offers a centred row of bubbles: one per line
          already attached, plus one free, so a side that holds something
          still has an open point right beside it. Every bubble is usable in
          either direction (loose connection mode): choosing where a line
          meets a box is choosing a place, not a polarity. The target handle
          under each source handle keeps side-less edges — scenes drawn
          before sides existed — rendering by default. */}
      {SIDES.map(([side, position]) =>
        slotLayout(slots?.[side] ?? []).map(({ slot, offset }) => {
          const id = sideRef(side, slot);
          const style =
            side === "top" || side === "bottom"
              ? { left: `calc(50% + ${offset}px)` }
              : { top: `calc(50% + ${offset}px)` };
          return (
            <span key={id}>
              <Handle type="target" position={position} id={id} style={style} />
              <Handle type="source" position={position} id={id} style={style} />
            </span>
          );
        }),
      )}
      {editing ? (
        <input
          ref={inputRef}
          className="graph-node__rename"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={() => finish(true)}
          onKeyDown={(e) => {
            if (e.key === "Enter") finish(true);
            if (e.key === "Escape") finish(false);
          }}
        />
      ) : (
        <span className="graph-node__label">{node.label ?? node.id}</span>
      )}
    </div>
  );
}

const SIDES = [
  ["top", Position.Top],
  ["right", Position.Right],
  ["bottom", Position.Bottom],
  ["left", Position.Left],
] as const;
