<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/CapturedReferences.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-81bc4ab2895e"></a>
## `apps/web/src/components/CapturedReferences.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-81bc4ab2895e-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-81bc4ab2895e-expression_statement">
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { openCapturedCode } from "../lib/capturedCode";
import { CapturedReferences } from "./CapturedReferences";
vi.mock("../lib/capturedCode", () => ({ openCapturedCode: vi.fn() }));
</storycode:copy>
<a id="f-81bc4ab2895e-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-81bc4ab2895e-expression_statement-2">
afterEach(() => { cleanup(); vi.clearAllMocks(); });
</storycode:copy>
<a id="f-81bc4ab2895e-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-81bc4ab2895e-expression_statement-3">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("awaits the exact captured navigation, retains failures, and refuses a live changed source", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const target = { uri: "hick-project:///capture/a.py", range: { start: { line: 4, character: 2 }, end: { line: 4, character: 3 } } };
  const close = vi.fn(), error = vi.fn(); let current = true;
  vi.mocked(openCapturedCode).mockRejectedValueOnce(new Error("captured host refused")).mockResolvedValue(undefined);
  render(<UiActionContext.Provider value={registry}><CapturedReferences targets={[target]} scope="PRIVATE SOURCE SCOPE" owner="debug-owner" current={() => current} onClose={close} onError={error} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(g => registry.observe(g).actions);
  const action = actions().find(a => a.label === "Open captured reference")!;
  expect(JSON.stringify(actions())).not.toContain("PRIVATE SOURCE SCOPE");
  let id = 0;
  const run = async (handle: string) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle, arguments: { index: 0 }, request_id: String(++id) }); }); return result; };
  expect(await run(action.handle)).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: "Error: captured host refused" } });
  expect(close).not.toHaveBeenCalled(); expect(error).toHaveBeenCalledWith("Error: captured host refused");
  current = false;
  expect(await run(action.handle)).toMatchObject({ status: "stale" });
  expect(openCapturedCode).toHaveBeenCalledTimes(1);
  current = true;
  const next = actions().find(a => a.label === "Open captured reference")!;
  expect(await run(next.handle)).toMatchObject({ status: "result", outcome: { status: "completed", value: { captured_navigation_requested: true, uri: target.uri, line: 4 } } });
  expect(openCapturedCode).toHaveBeenLastCalledWith(target, "debug-owner"); expect(close).toHaveBeenCalledOnce();
});
</storycode:copy>

### Assemble `apps/web/src/components/CapturedReferences.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/CapturedReferences.test.tsx"><storycode:paste select="#f-81bc4ab2895e-expression_statement"/><storycode:paste select="#f-81bc4ab2895e-expression_statement-2"/><storycode:paste select="#f-81bc4ab2895e-expression_statement-3"/></storycode:file>

</storycode:doc>
