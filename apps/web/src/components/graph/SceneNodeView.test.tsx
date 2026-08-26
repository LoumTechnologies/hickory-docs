// @vitest-environment jsdom
// What a node OFFERS: bubbles on sides only, one per attached line plus one
// free — and the free one beside an attached line carries no ink until a
// connector is being dragged (or the node is hovered). The canvas library is
// mocked; these tests are about which handles exist and how they are marked.
import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const connection = { inProgress: false };
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

import { SceneNodeView } from "./SceneNodeView";
import type { SceneNodeData } from "./SceneNodeView";

afterEach(() => {
  cleanup();
  connection.inProgress = false;
});

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
