<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/graph/SceneNodeView.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-daf7e784fe4c"></a>
## `apps/web/src/components/graph/SceneNodeView.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-daf7e784fe4c-SceneNodeData"></a>
### `SceneNodeData`

<storycode:copy id="f-daf7e784fe4c-SceneNodeData">
import type { UiActor } from "../../lib/uiActions";
// One node on the graph canvas: the shape, the label, and — on double click,
// when the topology is the author's to edit — the label as an input.
//
// A derived node's label is NOT editable: the topology came from a generator
// through a paste, and an edit here would be silently thrown away by the next
// re-run. The panel's banner says where to change it instead. Resizing is
// allowed on BOTH kinds — a size is layout, the person's half.

import { useEffect, useRef } from "react";
import { Handle, NodeResizer, Position, useConnection } from "@xyflow/react";
import type { NodeProps, ResizeParams } from "@xyflow/react";

import { useGraphLabelDraft } from "./useGraphLabelDraft";
import { sideHandles } from "./scene";
import type { SceneNode, Side } from "./scene";

export interface SceneNodeData extends Record<string, unknown> {
  node: SceneNode;
  derived: boolean;
  onRename: (id: string, label: string, actor?: UiActor) => boolean | void;
  /** A resize ended: the node's new place and size, to snap and commit. */
  onResize: (id: string, layout: { x: number; y: number; w: number; h: number }) => void;
  /** Which slots each side has lines attached to — decides how many bubbles
   * the side offers (every occupied one, plus one free). */
  slots: Partial<Record<Side, number[]>>;
}

</storycode:copy>
<a id="f-daf7e784fe4c-SceneNodeView"></a>
### `SceneNodeView`

<storycode:copy id="f-daf7e784fe4c-SceneNodeView">
export function SceneNodeView({ data, selected }: NodeProps) {
  const { node, derived, onRename, onResize, slots } = data as SceneNodeData;
  // Whether ANY connection drag is live on the canvas: the moment the extra
  // bubbles earn their ink (see below).
  const connecting = useConnection((c) => c.inProgress);
  const label = useGraphLabelDraft({ identity: `node:${node.id}`, value: node.label ?? node.id,
    disabled: derived, nonempty: true, commit: (value, actor) => onRename(node.id, value, actor) });
  const inputRef = useRef<HTMLInputElement>(null);
  useEffect(() => { if (label.editing) inputRef.current?.select(); }, [label.editing]);

  const shape = node.shape ?? "rect";
  return (
    <div
      className={`graph-node graph-node--${shape}${selected ? " graph-node--selected" : ""}${connecting ? " graph-node--connecting" : ""}`}
      // Colours travel as custom properties rather than direct styles: the
      // clip-path shapes (diamond, hexagon) cannot use a CSS border — the
      // clip cuts it off the slanted edges — so their outline is a colour
      // layer behind an inset fill layer, and both layers need to read the
      // same two values the bordered shapes read.
      style={
        {
          ...(node.fill ? { "--gn-fill": node.fill } : {}),
          ...(node.stroke ? { "--gn-stroke": node.stroke } : {}),
          ...(node.text ? { color: node.text } : {}),
        } as React.CSSProperties
      }
      onDoubleClick={label.open}
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
        // The lines in use stay centred; one EXTRA bubble flanks them on
        // each end, symmetric, so a vacancy never pushes a line off the
        // side's midline. The extras are drop targets first and ink second:
        // always there to catch a connector, visible only while one is
        // being dragged (or the node is hovered, which is how a second
        // line STARTS from the side). At rest a side with one line shows
        // one bubble, dead centre.
        sideHandles(side, slots?.[side] ?? []).map(({ id, offset, extra }) => {
          const className = extra ? "graph-handle--extra" : undefined;
          const style =
            side === "top" || side === "bottom"
              ? { left: `calc(50% + ${offset}px)` }
              : { top: `calc(50% + ${offset}px)` };
          return (
            <span key={id}>
              <Handle type="target" position={position} id={id} style={style} className={className} />
              <Handle type="source" position={position} id={id} style={style} className={className} />
            </span>
          );
        }),
      )}
      {label.error && <span role="alert">{label.error}</span>}
      {label.editing ? (
        <input
          ref={inputRef}
          className="graph-node__rename"
          value={label.value}
          onChange={(e) => label.change(e.target.value)}
          onBlur={() => label.finish(true)}
          onKeyDown={(e) => {
            if (e.key === "Enter") label.finish(true);
            if (e.key === "Escape") label.finish(false);
          }}
        />
      ) : (
        <span className="graph-node__label">{node.label ?? node.id}</span>
      )}
    </div>
  );
}

</storycode:copy>
<a id="f-daf7e784fe4c-SIDES"></a>
### `SIDES`

<storycode:copy id="f-daf7e784fe4c-SIDES">
const SIDES = [
  ["top", Position.Top],
  ["right", Position.Right],
  ["bottom", Position.Bottom],
  ["left", Position.Left],
] as const;
</storycode:copy>

### Assemble `apps/web/src/components/graph/SceneNodeView.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/graph/SceneNodeView.tsx"><storycode:paste select="#f-daf7e784fe4c-SceneNodeData"/><storycode:paste select="#f-daf7e784fe4c-SceneNodeView"/><storycode:paste select="#f-daf7e784fe4c-SIDES"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-d7bdcc63ee38"></a>
## `apps/web/src/components/graph/SceneNodeView.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Drawn Diagram Is Document Text](../../../../../../guarantees/authoring/a-drawn-diagram-is-document-text/contract.md).

<a id="f-d7bdcc63ee38-connection"></a>
### `connection`

@vitest-environment jsdom
What a node OFFERS: bubbles on sides only, one per attached line plus one
free — and the free one beside an attached line carries no ink until a
connector is being dragged (or the node is hovered). The canvas library is
mocked; these tests are about which handles exist and how they are marked.

<storycode:copy id="f-d7bdcc63ee38-connection">
// @vitest-environment jsdom
// What a node OFFERS: bubbles on sides only, one per attached line plus one
// free — and the free one beside an attached line carries no ink until a
// connector is being dragged (or the node is hovered). The canvas library is
// mocked; these tests are about which handles exist and how they are marked.
import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const connection = { inProgress: false };
</storycode:copy>
<a id="f-d7bdcc63ee38-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d7bdcc63ee38-expression_statement">
vi.mock("@xyflow/react", () => ({
  Handle: (props: { id?: string; type: string; className?: string }) => (
    <i
      data-testid="handle"
      data-id={props.id}
      data-type={props.type}
      className={props.className ?? ""}
    />
  ),
  NodeResizer: () => null,
  Position: { Top: "top", Left: "left", Right: "right", Bottom: "bottom" },
  useConnection: (selector?: (c: { inProgress: boolean }) => unknown) =>
    selector ? selector(connection) : connection,
}));

</storycode:copy>
<a id="f-d7bdcc63ee38-import_statement"></a>
### `import_statement`

<storycode:copy id="f-d7bdcc63ee38-import_statement">
import { SceneNodeView } from "./SceneNodeView";
</storycode:copy>
<a id="f-d7bdcc63ee38-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-d7bdcc63ee38-import_statement-2">
import type { SceneNodeData } from "./SceneNodeView";

</storycode:copy>
<a id="f-d7bdcc63ee38-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d7bdcc63ee38-expression_statement-2">
afterEach(() => {
  cleanup();
  connection.inProgress = false;
});

</storycode:copy>
<a id="f-d7bdcc63ee38-view"></a>
### `view`

<storycode:copy id="f-d7bdcc63ee38-view">
function view(slots: SceneNodeData["slots"]) {
  const data: SceneNodeData = {
    node: { id: "api" },
    derived: false,
    onRename: () => undefined,
    onResize: () => undefined,
    slots,
  };
  const props = { data, selected: false } as unknown as Parameters<typeof SceneNodeView>[0];
  return render(<SceneNodeView {...props} />);
}

</storycode:copy>
<a id="f-d7bdcc63ee38-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-d7bdcc63ee38-expression_statement-3">
describe("a node's connection bubbles", () => {
  it("centres the occupied slot and flanks it with extras; a bare side is one plain point", () => {
    const { container } = view({ left: [0] });
    const ids = (id: string) =>
      Array.from(container.querySelectorAll(`[data-id="${id}"]`));
    // The occupied slot draws normally, dead centre…
    expect(ids("left")).toHaveLength(2); // target + source
    expect(ids("left")[0].className).toBe("");
    // …one extra flanks it on EACH end, so a new line keeps the group
    // balanced whichever flank takes it…
    expect(ids("left._before")).toHaveLength(2);
    expect(ids("left._before")[0].className).toBe("graph-handle--extra");
    expect(ids("left._after")).toHaveLength(2);
    expect(ids("left._after")[0].className).toBe("graph-handle--extra");
    // …and an empty side's single bubble is ordinary, not "extra": it is the
    // only way to start a line there at all.
    expect(ids("top")).toHaveLength(2);
    expect(ids("top")[0].className).toBe("");
  });

  it("wears the connecting mark while a drag is live, so CSS can show the offers", () => {
    connection.inProgress = true;
    const { container } = view({ left: [0] });
    expect(container.querySelector(".graph-node")!.className).toContain(
      "graph-node--connecting",
    );
  });
});

</storycode:copy>
<a id="f-d7bdcc63ee38-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-d7bdcc63ee38-expression_statement-4">
it("shares the visible label draft and refuses to overwrite a changed label", async () => {
  const { act, fireEvent, screen } = await import("@testing-library/react");
  const { UiActions } = await import("../../lib/uiActions");
  const { UiActionContext } = await import("../../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const rename = vi.fn();
  const props = (label: string) => ({ data: { node: { id: "api", label }, derived: false, slots: {}, onRename: rename, onResize: () => {} } }) as unknown as Parameters<typeof SceneNodeView>[0];
  const mount = (label: string) => <UiActionContext.Provider value={registry}><SceneNodeView {...props(label)} /></UiActionContext.Provider>;
  const view = render(mount("Before"));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, {
      handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id),
    }); }); return result;
  };
  await run("Open graph label node:api");
  await run("Set graph label draft node:api", { value: "Proposed" });
  expect((screen.getByRole("textbox") as HTMLInputElement).value).toBe("Proposed");
  view.rerender(mount("Changed externally"));
  expect(await run("Commit graph label draft node:api")).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  expect(rename).not.toHaveBeenCalled();
  expect((screen.getByRole("textbox") as HTMLInputElement).value).toBe("Proposed");
  await run("Cancel graph label draft node:api");
  fireEvent.doubleClick(screen.getByTestId("graph-node-api"));
  await act(async () => {});
  expect((screen.getByRole("textbox") as HTMLInputElement).value).toBe("Changed externally");
  fireEvent.change(screen.getByRole("textbox"), { target: { value: "Native draft" } });
  await run("Commit graph label draft node:api");
  expect(rename).toHaveBeenCalledExactlyOnceWith("api", "Native draft", { id: "python:turn", kind: "agent" });
  rename.mockReturnValueOnce(false);
  await run("Open graph label node:api");
  await run("Set graph label draft node:api", { value: "Retain refused draft" });
  expect(await run("Commit graph label draft node:api")).toMatchObject({ outcome: { status: "unavailable" } });
  expect((screen.getByRole("textbox") as HTMLInputElement).value).toBe("Retain refused draft");
});
</storycode:copy>

### Assemble `apps/web/src/components/graph/SceneNodeView.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/graph/SceneNodeView.test.tsx"><storycode:paste select="#f-d7bdcc63ee38-connection"/><storycode:paste select="#f-d7bdcc63ee38-expression_statement"/><storycode:paste select="#f-d7bdcc63ee38-import_statement"/><storycode:paste select="#f-d7bdcc63ee38-import_statement-2"/><storycode:paste select="#f-d7bdcc63ee38-expression_statement-2"/><storycode:paste select="#f-d7bdcc63ee38-view"/><storycode:paste select="#f-d7bdcc63ee38-expression_statement-3"/><storycode:paste select="#f-d7bdcc63ee38-expression_statement-4"/></storycode:file>


</storycode:doc>
