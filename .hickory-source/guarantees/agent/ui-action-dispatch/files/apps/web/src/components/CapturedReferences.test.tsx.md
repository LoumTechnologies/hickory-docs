<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/CapturedReferences.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `3c3536652194634dfac744ff44493349af5bb5e45dd919b9bd3ad2e9ad4b5fd1`.

<book:entry path="apps/web/src/components/CapturedReferences.test.tsx">
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { openCapturedCode } from "../lib/capturedCode";
import { CapturedReferences } from "./CapturedReferences";
vi.mock("../lib/capturedCode", () => ({ openCapturedCode: vi.fn() }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
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
</book:entry>

</book:doc>
