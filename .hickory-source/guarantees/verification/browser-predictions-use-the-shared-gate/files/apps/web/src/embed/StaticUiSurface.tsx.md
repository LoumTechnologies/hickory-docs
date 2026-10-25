<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/StaticUiSurface.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Browser-local prediction practice uses the shared gate](../../../../../contract.md)

Captured SHA-256: `ad17f0f8466c792dafe7883d1670c960fb954255bf6f99e73d48ec97e20d3a31`.

<book:entry path="apps/web/src/embed/StaticUiSurface.tsx">
import { useEffect, useState, type ReactNode } from "react";
import { UiActionContext } from "../lib/uiActionBridge";
import { createEmbedUiSession } from "./uiHostSession";

export const STATIC_UI_CHANNEL = "hickory-static-ui";

/** Explicit parent attachment; static pages never probe a local engine. */
export function connectStaticUiParent(target: Window, parentOrigin: string, session: ReturnType<typeof createEmbedUiSession>) {
  if (new URL(parentOrigin).origin !== parentOrigin || parentOrigin === "null") throw Error("A concrete parent origin is required");
  let disposed = false;
  const send = (event: string, data: unknown, id?: string) => {
    if (!disposed) target.parent.postMessage({ channel: STATIC_UI_CHANNEL, version: 1, event, data, id }, parentOrigin);
  };
  const listener = (event: MessageEvent) => {
    if (disposed || event.source !== target.parent || event.origin !== parentOrigin) return;
    const message = event.data;
    if (!message || message.channel !== STATIC_UI_CHANNEL || message.version !== 1 || message.op !== "ui") return;
    if (typeof message.id !== "string" || !message.id.length || message.id.length > 128) return;
    void session.invoke(message.data).then(result => send("ui-result", { scope: session.scope, result }, message.id),
      error => send("error", { scope: session.scope, message: String(error) }, message.id));
  };
  target.addEventListener("message", listener);
  send("ready", { scope: session.scope, actor: "embed-parent agent", nativeExecution: false, receiptStorage: "parent must persist ui-receipt events" });
  return { receipt: (value: unknown) => send("ui-receipt", value), dispose() { disposed = true; target.removeEventListener("message", listener); } };
}

export function StaticUiSurface({ children }: { children: ReactNode }) {
  const [session, setSession] = useState<ReturnType<typeof createEmbedUiSession> | null>(null);
  useEffect(() => {
    let bridge: ReturnType<typeof connectStaticUiParent> | undefined;
    const current = createEmbedUiSession(receipt => bridge?.receipt(receipt));
    const origin = new URL(location.href).searchParams.get("parentOrigin");
    try { if (origin && window.parent !== window) bridge = connectStaticUiParent(window, origin, current); }
    catch { /* Invalid attachment cannot grant authority; ordinary viewing remains available. */ }
    setSession(current);
    return () => { bridge?.dispose(); current.dispose(); };
  }, []);
  return <UiActionContext.Provider value={session?.registry ?? null}>{children}</UiActionContext.Provider>;
}
</book:entry>

</book:doc>
