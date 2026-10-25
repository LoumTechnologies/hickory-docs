<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/graph/useGraphActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `09e65751559e50ff78998bea6df7931edb3ba1bbf160e2403e65a099ce81f093`.

<book:entry path="apps/web/src/components/graph/useGraphActions.ts">
import { useId, useMemo } from "react";
import { useUiCommands, useUiRevision } from "../../lib/uiActionBridge";
import { useUiReading } from "../../lib/uiReading";
import type { UiField, UiCommand, UiOutcome, UiActor } from "../../lib/uiActions";
import { occupiedSlots, sideHandles, type Scene, type SceneNode, type SceneEdge } from "./scene";
import type { Connection } from "@xyflow/react";
import { graphInspectorActions } from "./graphInspectorActions";
const sourceRevisions = new WeakMap<object, string>();
function sourceRevisionId(value: object | null | undefined) {
  if (!value) return null;
  let id = sourceRevisions.get(value);
  if (!id) { id = crypto.randomUUID(); sourceRevisions.set(value, id); }
  return id;
}
export type GraphAction = { id: string; label: string; topology?: boolean; viewOnly?: boolean; parameters?: UiCommand["parameters"]; execute: UiCommand["execute"] };
export type GraphControls = {
  patchNodes: (ids: string[], patch: Partial<SceneNode>, actor?: UiActor) => boolean | undefined;
  patchEdges: (ids: string[], patch: Partial<SceneEdge>, actor?: UiActor) => boolean | undefined;
  deleteNodes: (ids: string[], actor?: UiActor) => boolean | undefined;
  deleteEdges: (ids: string[], actor?: UiActor) => boolean | undefined;
  select: (selection: { nodes: string[]; edges: string[] }) => void;
  connect: (connection: Connection, actor?: UiActor) => boolean | undefined;
  reconnect: (id: string, connection: Connection, actor?: UiActor) => boolean | undefined;
  openEdgeLabel: (id: string) => void;
};

/** Controller-owned scene data and the same toolbar callbacks as native input. */
export function useGraphActions(input: GraphControls & {
  sourceOwner?: object; sourceRevision?: () => object | null; scene: Scene | null; current: () => boolean; writable: () => boolean; source: string; resolved?: string | null;
  selection: { nodes: string[]; edges: string[] }; assertions: unknown; error: string | null;
  viewport: { x: number; y: number; zoom: number };
  setViewport: (value: { x: number; y: number; zoom: number }, options?: { duration: number }) => Promise<boolean>;
  active: boolean; commit: (scene: Scene, actor?: UiActor) => boolean;
  rename: (id: string, label: string, actor?: UiActor) => boolean | undefined;
  resize: (id: string, at: { x: number; y: number; w: number; h: number }, actor?: UiActor) => boolean | undefined;
  move: (id: string, at: { x: number; y: number }, actor?: UiActor) => boolean | undefined;
  add: (actor?: UiActor) => boolean | undefined; layout: (actor?: UiActor) => boolean | undefined; fit: () => Promise<boolean>;
}) {
  const parent = `graph:${useId()}`;
  const ownerRevision = useMemo(() => crypto.randomUUID(), [input.sourceOwner]);
  const revision = useUiRevision(() => [ownerRevision, sourceRevisionId(input.sourceRevision?.()), input.source, input.resolved, input.scene, input.selection, input.error, input.active, input.viewport]);
  const slots = occupiedSlots(input.scene?.edges ?? []);
  useUiReading("graph scene", {
    connectors: (input.scene?.nodes ?? []).map(node => ({ id: node.id,
      handles: (["top", "right", "bottom", "left"] as const).flatMap(side => sideHandles(side, slots[node.id]?.[side] ?? [])),
    })),
    scene: input.scene, selection: input.selection, assertions: input.assertions ?? [],
    viewport: input.viewport, error: input.error, canvas_owns_wheel: input.active,
    topology_read_only: input.scene?.paste != null,
    evidence: "Controller scene and reported assertions; no execution or save acknowledgement",
  }, [input.source, input.resolved]);
  const available = () => !!input.scene && input.current();
  const coordinate: UiField = { type: "number", minimum: -1000000, maximum: 1000000 };
  const size: UiField = { type: "number", minimum: 1, maximum: 1000000 };
  const receipt = (committed: boolean | undefined): UiOutcome => committed
    ? { status: "completed", value: { commit_callback_requested: true, saved: false } }
    : { status: "unavailable", reason: "Scene refused the edit; read current graph status" };
  const actions: GraphAction[] = [
    { id: "add", label: "Add graph node", topology: true, execute: async (_args, actor) => {
      return input.add(actor) ? { status: "completed", value: { commit_callback_requested: true, saved: false } } : { status: "unavailable", reason: "Scene refused the edit; read current graph status" };
    } },
    { id: "layout", label: "Auto-layout graph", execute: async (_args, actor) => {
      return input.layout(actor) ? { status: "completed", value: { commit_callback_requested: true, saved: false } } : { status: "unavailable", reason: "Scene refused the layout; read current graph status" };
    } },
    { id: "viewport", viewOnly: true, label: "Set graph viewport", parameters: {
      x: coordinate, y: coordinate, zoom: { type: "number", minimum: 0.5, maximum: 2 },
    }, execute: async args => ({ status: "completed", value: { viewport_changed: await input.setViewport({
      x: args.x as number, y: args.y as number, zoom: args.zoom as number,
    }, { duration: 0 }) } }) },
    { id: "fit", viewOnly: true, label: "Zoom graph to fit", execute: async () => ({ status: "completed", value: { viewport_changed: await input.fit() } }) },
  ];
  for (const node of input.scene?.nodes ?? []) {
    actions.push(
      { id: `rename:${node.id}`, label: `Rename graph node ${node.id}`, topology: true,
        parameters: { label: { type: "string", maxLength: 8192 } },
        execute: async (args, actor) => {
          const label = (args.label as string).trim();
          if (!label) return { status: "unavailable", reason: "Node labels must contain text" };
          return receipt(input.rename(node.id, label, actor));
        } },
      { id: `move:${node.id}`, label: `Move graph node ${node.id}`,
        parameters: { x: coordinate, y: coordinate },
        execute: async (args, actor) => receipt(input.move(node.id, { x: args.x as number, y: args.y as number }, actor)) },
      { id: `resize:${node.id}`, label: `Resize graph node ${node.id}`,
        parameters: { x: coordinate, y: coordinate, w: size, h: size },
        execute: async (args, actor) => receipt(input.resize(node.id, {
          x: args.x as number, y: args.y as number, w: args.w as number, h: args.h as number,
        }, actor)) },
    );
  }
  actions.push(...graphInspectorActions(input.scene, input, receipt));
  const invoke = useUiCommands(actions.map(action => ({ key: `${parent}:${action.id}`, command: {
    parent, subject: parent, label: action.label, parameters: action.parameters ?? {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: available() && (action.viewOnly || input.writable()) && !(action.topology && input.scene?.paste),
      reason: !available() ? "Wait for the current document scene" : !action.viewOnly && !input.writable() ? "The source is read-only" : action.topology && input.scene?.paste ? "Edit the topology generator" : undefined }),
    execute: action.execute,
  } })));
  return (id: string) => void invoke(`${parent}:${id}`, {}).catch(() => {});
}
</book:entry>

</book:doc>
