<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/nativeWindowActions.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [Keep a prediction separate from the answer it is tested against](../../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md)

Captured SHA-256: `d6e41286eb25d2013435967ae489774e769ea68621317c04a00d55de79c6468c`.

<book:entry path="apps/web/src/lib/nativeWindowActions.test.ts">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { afterEach, expect, it, vi } from "vitest";
import { ApiError, request } from "../api/client";
import { UiActions } from "./uiActions";
import { registerNativeWindowActions } from "./nativeWindowActions";
vi.mock("../api/client", () => ({ request: vi.fn(), ApiError: class extends Error { constructor(public status: number) { super("host error"); } } }));
afterEach(() => vi.clearAllMocks());
function fixture() {
  const registry = new UiActions({ workspace: "local", window: "one" }, () => true, async () => {});
  registerNativeWindowActions(registry);
  let sequence = 0;
  return async (label: string, args = {}) => {
    const action = registry.observe("native-window").actions.find(action => action.label === label)!;
    return registry.act(registry.scope, { kind: "agent", id: "test" }, { handle: action.handle, arguments: args, request_id: String(++sequence) });
  };
}
it("uses finite current-window callbacks and reports actual minimized state", async () => {
  const invoke = fixture();
  vi.mocked(request).mockResolvedValue({ native: true, platform: "linux", minimized: true });
  expect(await invoke("Minimize current native window")).toMatchObject({ status: "result", outcome: { status: "completed", value: { minimized: true } } });
  expect(request).toHaveBeenLastCalledWith("POST", "/api/window/minimize", {});
  vi.mocked(request).mockResolvedValue({ native: true, platform: "linux", minimized: false });
  expect(await invoke("Restore current native window")).toMatchObject({ status: "result", outcome: { status: "completed", value: { minimized: false } } });
  expect(request).toHaveBeenLastCalledWith("POST", "/api/window/restore", {});
});
it("refuses extra response fields without disclosing their values", async () => {
  const invoke = fixture();
  vi.mocked(request).mockResolvedValue({ native: true, platform: "linux", minimized: false, target_actual: "held-out-secret" });
  const result = await invoke("Read current native window state");
  expect(result).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  expect(JSON.stringify(result)).not.toContain("held-out-secret");
});
it("distinguishes missing host input from an uncertain callback failure", async () => {
  const invoke = fixture();
  vi.mocked(request).mockRejectedValue(new ApiError(501, "missing"));
  expect(await invoke("Minimize current native window")).toMatchObject({ outcome: { status: "requires_input" } });
  vi.mocked(request).mockRejectedValue(new ApiError(503, "failed"));
  expect(await invoke("Minimize current native window")).toMatchObject({ outcome: { status: "unavailable" } });
});
it("retains an OS input request without claiming human or OS completion", async () => {
  const invoke = fixture();
  vi.mocked(request).mockResolvedValue({ native: true, platform: "macos", minimized: false });
  expect(await invoke("Request OS-owned application menu operation", { operation: "services" })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(await invoke("Read native OS menu request status")).toMatchObject({ outcome: { value: { completion_observed: false, request: { operation: "services", state: "requires_human_OS_input" } } } });
  expect(await invoke("Request OS-owned application menu operation", { operation: "arbitrary-command" })).toMatchObject({ status: "invalid" });
});
</book:entry>

</book:doc>
