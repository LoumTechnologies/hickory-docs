<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/uiGitHistoryActions.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a3c9920d0b05"></a>
## `apps/web/src/views/uiGitHistoryActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-a3c9920d0b05-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-a3c9920d0b05-expression_statement">
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { GitLog } from "../api/types";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useGitHistoryActions } from "./uiGitHistoryActions";
afterEach(cleanup);
</storycode:copy>
<a id="f-a3c9920d0b05-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-a3c9920d0b05-expression_statement-2">
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
</storycode:copy>

### Assemble `apps/web/src/views/uiGitHistoryActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/uiGitHistoryActions.test.tsx"><storycode:paste select="#f-a3c9920d0b05-expression_statement"/><storycode:paste select="#f-a3c9920d0b05-expression_statement-2"/></storycode:file>

</storycode:doc>
