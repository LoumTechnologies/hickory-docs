<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/uiGitHistoryActions.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `6f718fed7ac76afaa4a1f5ddc402126704a36874fcbc10599adfb150ce2bec79`.

<book:entry path="apps/web/src/views/uiGitHistoryActions.test.tsx">
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { GitLog } from "../api/types";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useGitHistoryActions } from "./uiGitHistoryActions";
afterEach(cleanup);
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses expanded commit evidence, shares navigation, and refuses replaced history handles", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let log: GitLog = { repository: true, floor: null, commits: [{ sha: "abc", short: "abc", parents: [], author: "A", email: "private@example.test", time: 1, subject: "Summary", body: "PRIVATE BODY", refs: [], files: [{ path: "secret.py", status: "M", added: 1, removed: 0 }], added: 1, removed: 0 }] };
  let open = new Set<string>();
  const toggle = vi.fn((sha: string) => { open = new Set(open.has(sha) ? [] : [sha]); });
  const openFile = vi.fn(), readCommit = vi.fn();
  let human!: ReturnType<typeof useGitHistoryActions>;
  function Harness() { human = useGitHistoryActions({ log, error: null, open, limit: 120, toggle, readCommit, openFile, more: vi.fn(), refresh: vi.fn() }); return null; }
  const ui = render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const group = registry.groups().groups[0];
  const actions = () => registry.observe(group).actions;
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result as { outcome: { value: { bytes: number[] } } }; };
  const initial = await run("Read Git history bytes", { offset: 0, limit: 8192 });
  expect(new TextDecoder().decode(new Uint8Array(initial.outcome.value.bytes))).not.toContain("PRIVATE BODY");
  expect(actions().some(a => a.label === "Read commit abc")).toBe(false);
  await run("Expand commit abc");
  ui.rerender(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const expanded = await run("Read Git history bytes", { offset: 0, limit: 8192 });
  expect(new TextDecoder().decode(new Uint8Array(expanded.outcome.value.bytes))).toContain("PRIVATE BODY");
  await run("Open current file secret.py from commit abc");
  expect(openFile).toHaveBeenCalledWith("secret.py");
  await act(async () => { await human("commit:abc"); });
  expect(readCommit).toHaveBeenCalledWith("abc");
  const old = actions().find(a => a.label === "Read commit abc")!;
  log = { ...log, commits: [{ ...log.commits[0], body: "REPLACED BODY" }] };
  ui.rerender(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "changed" })).toMatchObject({ status: "stale" });
  log = { repository: true, floor: null, commits: [] };
  ui.rerender(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old" })).toMatchObject({ status: "unavailable" });
  expect(readCommit).toHaveBeenCalledTimes(1);
});
</book:entry>

</book:doc>
