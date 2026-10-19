// Guarantee: docs/guarantees/search/find-and-replace-is-exhaustive.md
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { TreeFindReplace } from "./TreeFindReplace";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
it("discloses search, validates hit identity and replaces with observed hashes", async () => {
  const find = vi.spyOn(api, "find").mockResolvedValue({ files: [{ path: "source.md", source_hash: "a".repeat(64), matches: [{ line: 3, text: "old", at: [{ column: 0, length: 3 }] }] }], truncated: false });
  const replace = vi.spyOn(api, "replaceAll").mockResolvedValue({ changed: [], skipped: [{ path: "source.md", matches: 1, reason: "source_changed" }], replacements: 0 });
  const open = vi.fn(), replaced = vi.fn();
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><TreeFindReplace onOpenHit={open} onReplaced={replaced} /></UiActionContext.Provider>);
  expect(registry.groups().groups).toEqual(["folder-find-disclosures"]);
  let n = 0;
  const invoke = async (parent: string, label: string, arguments_ = {}) => {
    const action = registry.observe(parent).actions.find(action => action.label === label)!;
    let result; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: arguments_, request_id: String(++n) }); }); return result;
  };
  await invoke("folder-find-disclosures", "Set folder find visibility", { open: true });
  const parent = registry.groups().groups.find(group => group.startsWith("folder-find:"))!;
  await invoke(parent, "Set folder search query and options", { query: "old", case: false, whole_word: false, regex: false });
  await waitFor(() => expect(registry.observe(parent).actions.find(action => action.label === "Open observed folder search hit")?.enabled).toBe(true));
  expect(find).toHaveBeenCalledWith("old", { case: false, whole_word: false, regex: false });
  expect(await invoke(parent, "Open observed folder search hit", { path: "source.md", line: 4 })).toMatchObject({ outcome: { status: "unavailable" } });
  await invoke(parent, "Open observed folder search hit", { path: "source.md", line: 3 }); expect(open).toHaveBeenCalledWith("source.md", 3);
  await invoke(parent, "Set folder replacement draft", { text: "new" });
  expect(await invoke(parent, "Replace folder matches")).toMatchObject({ outcome: { value: { changed_files: 0, skipped_files: 1, replacements: 0 } } });
  expect(replace).toHaveBeenCalledWith("old", "new", { case: false, whole_word: false, regex: false }, ["source.md"], { "source.md": "a".repeat(64) });
  expect(replaced).toHaveBeenCalledWith(expect.objectContaining({ skipped: [expect.objectContaining({ reason: "source_changed" })] }));
});
