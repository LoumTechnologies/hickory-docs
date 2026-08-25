// One node on the graph canvas: the shape, the label, and — on double click,
// when the topology is the author's to edit — the label as an input.
//
// A derived node's label is NOT editable: the topology came from a generator
// through a paste, and an edit here would be silently thrown away by the next
// re-run. The panel's banner says where to change it instead.

import { useEffect, useRef, useState } from "react";
import { Handle, Position } from "@xyflow/react";
import type { NodeProps } from "@xyflow/react";

import type { SceneNode } from "./scene";

export interface SceneNodeData extends Record<string, unknown> {
  node: SceneNode;
  derived: boolean;
  onRename: (id: string, label: string) => void;
}

export function SceneNodeView({ data, selected }: NodeProps) {
  const { node, derived, onRename } = data as SceneNodeData;
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
      }}
      onDoubleClick={() => {
        if (!derived) setEditing(true);
      }}
      data-testid={`graph-node-${node.id}`}
    >
      <Handle type="target" position={Position.Top} id="top" />
      <Handle type="target" position={Position.Left} id="left" />
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
      <Handle type="source" position={Position.Right} id="right" />
      <Handle type="source" position={Position.Bottom} id="bottom" />
    </div>
  );
}
