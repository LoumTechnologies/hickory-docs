import { describe, expect, it } from "vitest";

import {
  projectUnread,
  workspaceNodeKey,
  type WorkspaceNode,
} from "./workspaceTree";

const node = (
  key: string,
  parent: string | null,
  unread = 0,
  provider = "filesystem",
): WorkspaceNode => ({
  key,
  parent,
  unread,
  kind: parent === null || provider === "filesystem" ? "folder" : "work-item",
  label: key,
  capabilities: [],
  source: { provider, id: key },
  freshness: { kind: "live" },
});

// Guarantee: docs/guarantees/integrations/an-unread-item-notifies-at-its-nearest-visible-ancestor.md
describe("projecting unread events onto the visible tree", () => {
  const nodes = [
    node("root", null),
    node("folder", "root"),
    node("ticket", "folder", 2, "jira"),
    node("comment", "ticket", 1, "github"),
  ];

  it("puts hidden descendants on their nearest visible ancestor", () => {
    const badges = projectUnread(nodes, new Set());
    expect(badges.get("root")?.count).toBe(3);
    expect([...badges.get("root")!.sources]).toEqual([
      ["jira", 2],
      ["github", 1],
    ]);
  });

  it("moves a badge down as expansion makes nodes visible", () => {
    const folderVisible = projectUnread(nodes, new Set(["root"]));
    expect(folderVisible.has("root")).toBe(false);
    expect(folderVisible.get("folder")?.count).toBe(3);

    const ticketVisible = projectUnread(nodes, new Set(["root", "folder"]));
    expect(ticketVisible.get("ticket")?.count).toBe(3);

    const commentVisible = projectUnread(nodes, new Set(["root", "folder", "ticket"]));
    expect(commentVisible.get("ticket")?.count).toBe(2);
    expect(commentVisible.get("comment")?.count).toBe(1);
  });

  it("does not change the unread events when expansion changes", () => {
    const before = nodes.map((item) => item.unread);
    projectUnread(nodes, new Set(["root", "folder", "ticket"]));
    expect(nodes.map((item) => item.unread)).toEqual(before);
  });

  it("omits orphaned and cyclic nodes rather than misplacing their badges", () => {
    const broken = [
      node("root", null),
      node("orphan", "missing", 2, "jira"),
      node("a", "b", 1, "github"),
      node("b", "a"),
    ];
    expect(projectUnread(broken, new Set(["root", "a", "b"])).size).toBe(0);
  });
});

describe("workspace node identity", () => {
  it("includes the provider and escapes delimiters", () => {
    expect(workspaceNodeKey({ provider: "github", id: "org/repo#12" })).toBe(
      "github:org%2Frepo%2312",
    );
  });
});
