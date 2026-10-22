import type { Connection } from "@xyflow/react";
import type { UiField, UiOutcome, UiValue } from "../../lib/uiActions";
import { FILLS, SHAPES, STROKES } from "./palette";
import { keyedEdges, occupiedSlots, sideHandles, type Scene } from "./scene";
import type { GraphAction, GraphControls } from "./useGraphActions";

/** Finite native palette and actual connector handles, never arbitrary scene patches. */
export function graphInspectorActions(scene: Scene | null, controls: GraphControls,
  receipt: (result: boolean | undefined) => UiOutcome): GraphAction[] {
  if (!scene) return [];
  const actions: GraphAction[] = [];
  const ids = scene.nodes.map(node => node.id);
  const edges = keyedEdges(scene.edges);
  const choice = (choices: string[]): UiField => ({ type: "string", maxLength: Math.max(1, ...choices.map(s => s.length)), choices });
  const list = (choices: string[]): UiField => ({ type: "array", items: choice(choices), maxItems: choices.length });
  actions.push({ id: "selection", viewOnly: true, label: "Select graph objects", parameters: {
    nodes: list(ids), edges: list(edges.map(edge => edge.key)),
  }, execute: async args => {
    const selected = { nodes: [...new Set(args.nodes as string[])], edges: [...new Set(args.edges as string[])] };
    controls.select(selected);
    return { status: "completed", value: selected };
  } });
  for (const node of scene.nodes) {
    for (const [field, choices] of Object.entries({ shape: [...SHAPES], fill: ["", ...FILLS.map(c => c.value)],
      stroke: ["", ...STROKES.map(c => c.value)], text: ["", ...STROKES.map(c => c.value)] })) {
      actions.push({ id: `node:${node.id}:${field}`, label: `Set graph node ${node.id} ${field}`, topology: true,
        parameters: { value: choice(choices) }, execute: async (args, actor) => receipt(controls.patchNodes([node.id], {
          [field]: args.value === "" || (field === "shape" && args.value === "rect") ? undefined : args.value,
        }, actor)) });
    }
    actions.push({ id: `delete-node:${node.id}`, label: `Delete graph node ${node.id}`, topology: true,
      execute: async (_args, actor) => receipt(controls.deleteNodes([node.id], actor)) });
  }
  for (const { key } of edges) {
    actions.push({ id: `label-edge:${key}`, label: `Open graph edge label ${key}`, viewOnly: true, topology: true,
      execute: async () => { controls.openEdgeLabel(key); return { status: "completed", value: { editing_requested: true } }; } });
    for (const [field, choices] of Object.entries({ color: ["", ...STROKES.map(c => c.value)],
      arrow: ["end", "both", "start", "none"], style: ["solid", "dashed", "dotted"] })) {
      actions.push({ id: `edge:${key}:${field}`, label: `Set graph edge ${key} ${field}`, topology: true,
        parameters: { value: choice(choices) }, execute: async (args, actor) => receipt(controls.patchEdges([key], {
          [field]: args.value === "" ? undefined : args.value,
        }, actor)) });
    }
    actions.push({ id: `edge:${key}:label`, label: `Set graph edge ${key} label`, topology: true,
      parameters: { label: { type: "string", maxLength: 8192 } },
      execute: async (args, actor) => receipt(controls.patchEdges([key], { label: (args.label as string).trim() || undefined }, actor)) },
    { id: `delete-edge:${key}`, label: `Delete graph edge ${key}`, topology: true,
      execute: async (_args, actor) => receipt(controls.deleteEdges([key], actor)) });
  }
  if (!ids.length) return actions;
  const slots = occupiedSlots(scene.edges);
  const handles = (id: string) => ["", ...(["top", "right", "bottom", "left"] as const)
    .flatMap(side => sideHandles(side, slots[id]?.[side] ?? []).map(handle => handle.id))];
  // Sides and slots are validated against the addressed node before calling the shared handler.
  const connection = (args: Record<string, UiValue>): Connection | null => {
    const from = args.from as string, to = args.to as string;
    if (!handles(from).includes(args.from_handle as string) || !handles(to).includes(args.to_handle as string)) return null;
    return { source: from, target: to, sourceHandle: args.from_handle as string || null, targetHandle: args.to_handle as string || null };
  };
  const parameters = { from: choice(ids), to: choice(ids),
    from_handle: { type: "string", maxLength: 128 } as UiField, to_handle: { type: "string", maxLength: 128 } as UiField };
  actions.push({ id: "connect", label: "Connect graph nodes", topology: true, parameters,
    execute: async (args, actor) => {
      const value = connection(args);
      return value ? receipt(controls.connect(value, actor)) : { status: "unavailable", reason: "Read the current node connector handles before connecting" };
    } });
  for (const { key } of edges) actions.push({ id: `reconnect:${key}`, label: `Reconnect graph edge ${key}`, topology: true, parameters,
    execute: async (args, actor) => {
      const value = connection(args);
      return value ? receipt(controls.reconnect(key, value, actor)) : { status: "unavailable", reason: "Read the current node connector handles before reconnecting" };
    } });
  return actions;
}
