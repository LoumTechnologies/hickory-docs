<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/nativeWindowActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [Keep a prediction separate from the answer it is tested against](../../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md)

Captured SHA-256: `8d673546ea02e309bb03bcb68bb6e3c0a62cfe8a16e23e72913c4d156872c9b5`.

<book:entry path="apps/web/src/lib/nativeWindowActions.ts">
import { ApiError, request } from "../api/client";
import { UiActions, type UiOutcome } from "./uiActions";

type Status = { native: true; platform: string; minimized: boolean };
function status(value: unknown): value is Status {
  if (!value || typeof value !== "object") return false;
  const record = value as Record<string, unknown>;
  return Object.keys(record).every(key => ["native", "platform", "minimized"].includes(key)) && record.native === true && typeof record.platform === "string" && record.platform.length > 0 && record.platform.length <= 32 && typeof record.minimized === "boolean";
}
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
</book:entry>

</book:doc>
