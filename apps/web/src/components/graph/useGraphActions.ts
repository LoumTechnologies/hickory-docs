import { useId } from "react";
import { useUiCommands, useUiRevision } from "../../lib/uiActionBridge";
import { useUiReading } from "../../lib/uiReading";
import type { UiField, UiCommand, UiOutcome } from "../../lib/uiActions";
import type { Scene } from "./scene";

/** Controller-owned scene data and the same toolbar callbacks as native input. */
export function useGraphActions(input: {
  scene: Scene | null; current: () => boolean; source: string; resolved?: string | null;
  selection: { nodes: string[]; edges: string[] }; assertions: unknown; error: string | null;
  active: boolean; commit: (scene: Scene) => boolean;
  rename: (id: string, label: string) => boolean | undefined;
  resize: (id: string, at: { x: number; y: number; w: number; h: number }) => boolean | undefined;
  move: (id: string, at: { x: number; y: number }) => boolean | undefined;
  add: () => boolean | undefined; layout: () => boolean | undefined; fit: () => Promise<boolean>;
}) {
  const parent = `graph:${useId()}`;
  const revision = useUiRevision(() => [input.source, input.resolved, input.scene, input.selection, input.error, input.active]);
  useUiReading("graph scene", {
    scene: input.scene, selection: input.selection, assertions: input.assertions ?? [],
    error: input.error, canvas_owns_wheel: input.active,
    topology_read_only: input.scene?.paste != null,
    evidence: "Controller scene and reported assertions; no execution or save acknowledgement",
  }, [input.source, input.resolved]);
  const available = () => !!input.scene && input.current();
  const coordinate: UiField = { type: "number", minimum: -1000000, maximum: 1000000 };
  const size: UiField = { type: "number", minimum: 1, maximum: 1000000 };
  const receipt = (committed: boolean | undefined): UiOutcome => committed
    ? { status: "completed", value: { commit_callback_requested: true, saved: false } }
    : { status: "unavailable", reason: "Scene refused the edit; read current graph status" };
  const actions: { id: string; label: string; topology?: boolean; parameters?: UiCommand["parameters"]; execute: UiCommand["execute"] }[] = [
    { id: "add", label: "Add graph node", topology: true, execute: async () => {
      return input.add() ? { status: "completed", value: { commit_callback_requested: true, saved: false } } : { status: "unavailable", reason: "Scene refused the edit; read current graph status" };
    } },
    { id: "layout", label: "Auto-layout graph", execute: async () => {
      return input.layout() ? { status: "completed", value: { commit_callback_requested: true, saved: false } } : { status: "unavailable", reason: "Scene refused the layout; read current graph status" };
    } },
    { id: "fit", label: "Zoom graph to fit", execute: async () => ({ status: "completed", value: { viewport_changed: await input.fit() } }) },
  ];
  for (const node of input.scene?.nodes ?? []) {
    actions.push(
      { id: `rename:${node.id}`, label: `Rename graph node ${node.id}`, topology: true,
        parameters: { label: { type: "string", maxLength: 8192 } },
        execute: async args => {
          const label = (args.label as string).trim();
          if (!label) return { status: "unavailable", reason: "Node labels must contain text" };
          return receipt(input.rename(node.id, label));
        } },
      { id: `move:${node.id}`, label: `Move graph node ${node.id}`,
        parameters: { x: coordinate, y: coordinate },
        execute: async args => receipt(input.move(node.id, { x: args.x as number, y: args.y as number })) },
      { id: `resize:${node.id}`, label: `Resize graph node ${node.id}`,
        parameters: { x: coordinate, y: coordinate, w: size, h: size },
        execute: async args => receipt(input.resize(node.id, {
          x: args.x as number, y: args.y as number, w: args.w as number, h: args.h as number,
        })) },
    );
  }
  const invoke = useUiCommands(actions.map(action => ({ key: `${parent}:${action.id}`, command: {
    parent, subject: parent, label: action.label, parameters: action.parameters ?? {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: available() && !(action.topology && input.scene?.paste),
      reason: !available() ? "Wait for the current document scene" : action.topology && input.scene?.paste ? "Edit the topology generator" : undefined }),
    execute: action.execute,
  } })));
  return (id: string) => void invoke(`${parent}:${id}`, {}).catch(() => {});
}
