<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/graph/SceneNodeView.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Drawn Diagram Is Document Text](../../../../../../../guarantees/authoring/a-drawn-diagram-is-document-text/contract.md)

Captured SHA-256: `f9d4b17a5b98cdaad28b11fe892198b7343c490c11b5ba7d37c674641e4c2c78`.

<book:entry path="apps/web/src/components/graph/SceneNodeView.test.tsx">
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
</book:entry>

</book:doc>
