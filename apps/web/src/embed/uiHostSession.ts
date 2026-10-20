import { UiActions, type UiReceipt } from "../lib/uiActions";
import { dispatchUiFrame } from "../lib/uiActionBridge";

/** One parent-owned embedded document, with no native engine or provider authority. */
export function createEmbedUiSession(receipt: (value: UiReceipt) => void) {
  const window = `embed-${crypto.randomUUID()}`;
  const registry = new UiActions({ workspace: window, window }, () => true, async value => { receipt(value); });
  let disposed = false;
  return {
    registry,
    scope: registry.scope,
    async invoke(value: unknown) {
      if (disposed) throw Error("Embedded document scope is disposed");
      if (!value || typeof value !== "object" || Array.isArray(value)) throw Error("Expected structured embedded UI request");
      const data = value as Record<string, unknown>;
      if (Object.keys(data).some(key => !["window", "method", "arguments"].includes(key)) || data.window !== window) throw Error("Embedded UI request belongs to another document scope; use the latest loaded scope");
      if (typeof data.method !== "string" || !["ui_observe", "ui_act", "ui_cancel"].includes(data.method) || !data.arguments || typeof data.arguments !== "object" || Array.isArray(data.arguments)) throw Error("Invalid embedded UI method or arguments");
      return dispatchUiFrame(registry, { id: "embed-parent", actor: { id: "embed-parent", kind: "agent" }, method: data.method as string, arguments: data.arguments as Record<string, import("../lib/uiActions").UiValue> });
    },
    dispose() { disposed = true; registry.dispose(); },
  };
}
