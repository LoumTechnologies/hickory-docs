<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/nativeWindowActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-14de373a5fd7"></a>
## `apps/web/src/lib/nativeWindowActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-14de373a5fd7-Status"></a>
### `Status`

<storycode:copy id="f-14de373a5fd7-Status">
import { ApiError, request } from "../api/client";
import { UiActions, type UiOutcome } from "./uiActions";

type Status = { native: true; platform: string; minimized: boolean };
</storycode:copy>
<a id="f-14de373a5fd7-status"></a>
### `status`

<storycode:copy id="f-14de373a5fd7-status">
function status(value: unknown): value is Status {
  if (!value || typeof value !== "object") return false;
  const record = value as Record<string, unknown>;
  return Object.keys(record).every(key => ["native", "platform", "minimized"].includes(key)) && record.native === true && typeof record.platform === "string" && record.platform.length > 0 && record.platform.length <= 32 && typeof record.minimized === "boolean";
}
</storycode:copy>
<a id="f-14de373a5fd7-registerNativeWindowActions"></a>
### `registerNativeWindowActions`

Current host window only; no arbitrary IPC, target window or project route.

<storycode:copy id="f-14de373a5fd7-registerNativeWindowActions">
/** Current host window only; no arbitrary IPC, target window or project route. */
export function registerNativeWindowActions(registry: UiActions) {
  let revision = crypto.randomUUID();
  let osRequest: { request: string; operation: string; state: string } | null = null;
  const parent = "native-window", subject = `native-window:${registry.scope.window}`;
  const read = () => ({ revision, enabled: true });
  const invoke = async (operation: "status" | "minimize" | "restore"): Promise<UiOutcome> => {
    try {
      const actual = await request<unknown>(operation === "status" ? "GET" : "POST", `/api/window/${operation}`, operation === "status" ? undefined : {});
      if (!status(actual)) return { status: "unavailable", reason: "Invalid native window response; update this host before retrying" };
      revision = crypto.randomUUID();
      return { status: "completed", value: { native: actual.native, platform: actual.platform, minimized: actual.minimized, operation, host_callback_completed: true, scope: "Current host-reported window state; an OS request is not a document save or execution verdict" } };
    } catch (cause) {
      if (cause instanceof ApiError && [404, 501].includes(cause.status)) return { status: "requires_input", request: `native-window:${operation}`, reason: "This client has no native window callback. Use the browser/OS window controls and report the result; no window effect is claimed" };
      return { status: "unavailable", reason: "Native window request could not be confirmed; inspect its current state before retrying" };
    }
  };
  return [
    ...([ ["status", "Read current native window state"], ["minimize", "Minimize current native window"], ["restore", "Restore current native window"] ] as const).map(([operation, label]) => registry.register(`${parent}:${operation}`, { parent, subject, label, parameters: {}, prerequisites: ["Current native host window; browser-only clients return an OS input request"], read, execute: () => invoke(operation) })),
    registry.register(`${parent}:os-menu`, { parent, subject, label: "Request OS-owned application menu operation", parameters: { operation: { type: "string", maxLength: 32, choices: ["about", "services", "hide", "hide-others", "show-all"] } },
      prerequisites: ["Predefined macOS application/Services menus remain OS-owned; no arbitrary service or other application can be invoked by this bridge"], read,
      execute: async args => {
        const observed = await invoke("status");
        if (observed.status !== "completed") return observed;
        const value = observed.value as unknown as Status;
        if (value.platform !== "macos") return { status: "unavailable", reason: "This native application menu is not present on this platform" };
        const operation = String(args.operation), id = `native-os-menu:${crypto.randomUUID()}`;
        osRequest = { request: id, operation, state: "requires_human_OS_input" }; revision = crypto.randomUUID();
        return { status: "requires_input", request: id, reason: `Use the macOS application menu for ${operation}, then report the result in the conversation. This build cannot inspect OS Services/other applications or attest their completion` };
      } }),
    registry.register(`${parent}:os-status`, { parent, subject: `${subject}:reading`, label: "Read native OS menu request status", parameters: {}, prerequisites: [], read,
      execute: async () => ({ status: "completed", value: { request: osRequest, completion_observed: false, scope: "Retained required-input result. A conversation reply is human-reported, not host-observed OS completion" } }) }),
  ];
}
</storycode:copy>

### Assemble `apps/web/src/lib/nativeWindowActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/nativeWindowActions.ts"><storycode:paste select="#f-14de373a5fd7-Status"/><storycode:paste select="#f-14de373a5fd7-status"/><storycode:paste select="#f-14de373a5fd7-registerNativeWindowActions"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-fa3030dd95b0"></a>
## `apps/web/src/lib/nativeWindowActions.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-fa3030dd95b0-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-fa3030dd95b0-expression_statement">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { afterEach, expect, it, vi } from "vitest";
import { ApiError, request } from "../api/client";
import { UiActions } from "./uiActions";
import { registerNativeWindowActions } from "./nativeWindowActions";
vi.mock("../api/client", () => ({ request: vi.fn(), ApiError: class extends Error { constructor(public status: number) { super("host error"); } } }));
</storycode:copy>
<a id="f-fa3030dd95b0-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-fa3030dd95b0-expression_statement-2">
afterEach(() => vi.clearAllMocks());
</storycode:copy>
<a id="f-fa3030dd95b0-fixture"></a>
### `fixture`

<storycode:copy id="f-fa3030dd95b0-fixture">
function fixture() {
  const registry = new UiActions({ workspace: "local", window: "one" }, () => true, async () => {});
  registerNativeWindowActions(registry);
  let sequence = 0;
  return async (label: string, args = {}) => {
    const action = registry.observe("native-window").actions.find(action => action.label === label)!;
    return registry.act(registry.scope, { kind: "agent", id: "test" }, { handle: action.handle, arguments: args, request_id: String(++sequence) });
  };
}
</storycode:copy>
<a id="f-fa3030dd95b0-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-fa3030dd95b0-expression_statement-3">
it("uses finite current-window callbacks and reports actual minimized state", async () => {
  const invoke = fixture();
  vi.mocked(request).mockResolvedValue({ native: true, platform: "linux", minimized: true });
  expect(await invoke("Minimize current native window")).toMatchObject({ status: "result", outcome: { status: "completed", value: { minimized: true } } });
  expect(request).toHaveBeenLastCalledWith("POST", "/api/window/minimize", {});
  vi.mocked(request).mockResolvedValue({ native: true, platform: "linux", minimized: false });
  expect(await invoke("Restore current native window")).toMatchObject({ status: "result", outcome: { status: "completed", value: { minimized: false } } });
  expect(request).toHaveBeenLastCalledWith("POST", "/api/window/restore", {});
});
</storycode:copy>
<a id="f-fa3030dd95b0-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-fa3030dd95b0-expression_statement-4">
it("refuses extra response fields without disclosing their values", async () => {
  const invoke = fixture();
  vi.mocked(request).mockResolvedValue({ native: true, platform: "linux", minimized: false, target_actual: "held-out-secret" });
  const result = await invoke("Read current native window state");
  expect(result).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  expect(JSON.stringify(result)).not.toContain("held-out-secret");
});
</storycode:copy>
<a id="f-fa3030dd95b0-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-fa3030dd95b0-expression_statement-5">
it("distinguishes missing host input from an uncertain callback failure", async () => {
  const invoke = fixture();
  vi.mocked(request).mockRejectedValue(new ApiError(501, "missing"));
  expect(await invoke("Minimize current native window")).toMatchObject({ outcome: { status: "requires_input" } });
  vi.mocked(request).mockRejectedValue(new ApiError(503, "failed"));
  expect(await invoke("Minimize current native window")).toMatchObject({ outcome: { status: "unavailable" } });
});
</storycode:copy>
<a id="f-fa3030dd95b0-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-fa3030dd95b0-expression_statement-6">
it("retains an OS input request without claiming human or OS completion", async () => {
  const invoke = fixture();
  vi.mocked(request).mockResolvedValue({ native: true, platform: "macos", minimized: false });
  expect(await invoke("Request OS-owned application menu operation", { operation: "services" })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(await invoke("Read native OS menu request status")).toMatchObject({ outcome: { value: { completion_observed: false, request: { operation: "services", state: "requires_human_OS_input" } } } });
  expect(await invoke("Request OS-owned application menu operation", { operation: "arbitrary-command" })).toMatchObject({ status: "invalid" });
});
</storycode:copy>

### Assemble `apps/web/src/lib/nativeWindowActions.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/nativeWindowActions.test.ts"><storycode:paste select="#f-fa3030dd95b0-expression_statement"/><storycode:paste select="#f-fa3030dd95b0-expression_statement-2"/><storycode:paste select="#f-fa3030dd95b0-fixture"/><storycode:paste select="#f-fa3030dd95b0-expression_statement-3"/><storycode:paste select="#f-fa3030dd95b0-expression_statement-4"/><storycode:paste select="#f-fa3030dd95b0-expression_statement-5"/><storycode:paste select="#f-fa3030dd95b0-expression_statement-6"/></storycode:file>


</storycode:doc>
