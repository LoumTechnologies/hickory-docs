import { describe, expect, it } from "vitest";
import { branchOf, childrenOf, deepestFrom } from "./ChatDock";
import type { AgentTurn } from "../api/types";

function turn(id: string, parent: string | null): AgentTurn {
  return {
    id,
    parent_id: parent,
    prompt: `p:${id}`,
    answer: `a:${id}`,
    status: "ok",
    error: null,
    created_at: "2026-01-01T00:00:00Z",
  };
}

//        root
//       /    \
//      a      b        <- a rewind at `root` forked `b`
//      |      |
//      a2     b2
const TREE = [turn("root", null), turn("a", "root"), turn("a2", "a"), turn("b", "root"), turn("b2", "b")];

describe("conversation tree", () => {
  it("shows the path from the root to the selected tip, oldest first", () => {
    expect(branchOf(TREE, "a2").map((t) => t.id)).toEqual(["root", "a", "a2"]);
    expect(branchOf(TREE, "b2").map((t) => t.id)).toEqual(["root", "b", "b2"]);
  });

  it("returns nothing for an empty or unknown tip rather than throwing", () => {
    expect(branchOf(TREE, null)).toEqual([]);
    expect(branchOf(TREE, "nope")).toEqual([]);
  });

  it("terminates on a cycle instead of hanging the dock", () => {
    const cyclic = [turn("x", "y"), turn("y", "x")];
    expect(branchOf(cyclic, "x").length).toBeLessThanOrEqual(2);
  });

  it("lists the alternatives at a fork, so the switcher can page through them", () => {
    expect(childrenOf(TREE, "root").map((t) => t.id)).toEqual(["a", "b"]);
    expect(childrenOf(TREE, null).map((t) => t.id)).toEqual(["root"]);
    expect(childrenOf(TREE, "a2")).toEqual([]);
  });

  it("switching branches lands on that branch's newest tip, not its fork point", () => {
    expect(deepestFrom(TREE, "b")).toBe("b2");
    expect(deepestFrom(TREE, "a")).toBe("a2");
    expect(deepestFrom(TREE, "a2")).toBe("a2");
  });
});
