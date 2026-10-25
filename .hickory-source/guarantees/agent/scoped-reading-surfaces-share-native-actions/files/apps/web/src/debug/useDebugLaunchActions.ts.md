<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/useDebugLaunchActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Captured reading surfaces share owned native actions](../../../../../contract.md)

Captured SHA-256: `a4e2f3af054e2de713bdb64ec48816d5dc40f81dc22cebf0250e86217f57ae5d`.

<book:entry path="apps/web/src/debug/useDebugLaunchActions.ts">
import { useId, useMemo, useRef, useState } from "react";
import type { EditorView } from "@codemirror/view";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";

type Target = { key: string; label: string; disabled?: boolean; run: () => unknown };
/** The existing launch callback owns resolution/startup; this reports its invocation. */
export function useDebugLaunchActions(view: EditorView | null, targets: Target[]) {
  const parent = `debug-launch:${useId()}`;
  const capture = view?.state.doc;
  const callbacks = useMemo(() => crypto.randomUUID(), [view, targets]);
  const pending = useRef(false);
  const [failure, setFailure] = useState<{ capture: object | undefined; message: string } | null>(null);
  const error = failure?.capture === capture ? failure?.message : null;
  const revision = useUiRevision(() => [callbacks, capture?.toString(), view?.state.doc.toString(), targets.map(target => [target.key, target.disabled]), pending.current]);
  const current = () => !!view?.dom.isConnected && view.state.doc === capture;
  const launch = async (key: string): Promise<UiOutcome> => {
    const target = targets.find(target => target.key === key);
    if (!target || target.disabled || !current()) return { status: "unavailable", reason: "Launch target changed, closed or is disabled. Observe the current source controls again." };
    if (pending.current) return { status: "unavailable", reason: "A launch callback is already pending" };
    pending.current = true; setFailure(null);
    try {
      await target.run();
      return { status: "completed", value: { launch_callback_completed: true, launch_completed: false, scope: "Inspect the profile picker, debugger or terminal for the actual launch outcome" } };
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      setFailure({ capture, message }); return { status: "unavailable", reason: message };
    } finally { pending.current = false; }
  };
  useUiReading("source debug launch status", { targets: targets.map(target => ({ label: target.label, disabled: !!target.disabled })), error: error ?? null,
    scope: "Displayed source launch controls. Invoking a callback does not establish a running process or a successful debugger." });
  useUiCommands(targets.map(target => ({ key: `${parent}:${target.key}`, command: {
    label: target.label, parent, subject: parent, parameters: {}, prerequisites: ["The displayed source and launch callback must remain current"],
    read: () => ({ revision: revision(), enabled: current() && !target.disabled && !pending.current }),
    execute: async () => launch(target.key),
  } })));
  return { launch, error };
}
</book:entry>

</book:doc>
