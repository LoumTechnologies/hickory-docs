<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/terminal/useTerminalChromeActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `df0793720d64e9c596c61c9b0cacf6cbb5d2c80157951e8f104ac1c2a9411679`.

<book:entry path="apps/web/src/terminal/useTerminalChromeActions.ts">
import { useEffect, useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";
type Control = { id: string; label: string; humanRequest?: string; run: () => boolean | void | Promise<boolean | void> };
/** Explicit terminal chrome controls; never derives actions from terminal output. */
export function useTerminalChromeActions(label: string, reading: unknown, controls: Control[]) {
  const parent = `terminal-chrome:${useId()}`;
  const [pending, setPending] = useState<string | null>(null);
  const [errorState, setErrorState] = useState<{ owner: string | undefined; message: string } | null>(null);
  const owner = JSON.stringify(reading);
  const error = errorState?.owner === owner ? errorState?.message ?? null : null;
  const setError = (message: string | null) => setErrorState(message === null ? null : { owner, message });
  const latestOwner = useRef(owner); latestOwner.current = owner;
  const lock = useRef(false), live = useRef(true);
  const previous = useRef<{ callbacks: Control["run"][]; token: string } | null>(null);
  const callbacks = controls.map(control => control.run);
  if (!previous.current || callbacks.length !== previous.current.callbacks.length || callbacks.some((callback, index) => callback !== previous.current!.callbacks[index]))
    previous.current = { callbacks, token: crypto.randomUUID() };
  const revision = useUiRevision(() => [reading, controls.map(control => [control.id, control.label]), previous.current!.token, pending]);
  useEffect(() => { live.current = true; return () => { live.current = false; }; }, []);
  const invoke = useUiCommands(controls.map(control => ({ key: `${parent}:${control.id}`, command: {
    parent, subject: parent, label: control.label, parameters: {}, prerequisites: ["Current terminal owner and actual host state apply"],
    read: () => ({ revision: revision(), enabled: !lock.current }), execute: async (_args, actor): Promise<UiOutcome> => {
      if (actor.kind === "agent" && control.humanRequest) return { status: "requires_input", request: control.humanRequest, reason: "The existing terminal policy reserves prompt decisions for the human attention card. Read its current question and observe actual supervision afterward; no answer or approval was sent." };
      if (!live.current || lock.current) return { status: "unavailable", reason: "Terminal control is closed or busy" };
      lock.current = true; setPending(control.id); setError(null);
      try {
        const response = await control.run();
        if (response === false) throw Error("Host terminal request was refused; inspect its current state and supervision error");
        return { status: "completed", value: { callback_completed: true, host_request_completed: response === true,
          decision_request: control.humanRequest ?? null, actor_kind: actor.kind, approval_attested: false,
          scope: "Inspect current terminal supervision/output for effects; completion does not certify process exit or recording state" } };
      } catch (cause) {
        if (live.current && latestOwner.current === owner) setError(String(cause));
        return { status: "unavailable", reason: String(cause), ...(control.humanRequest ? { request: control.humanRequest } : {}) };
      } finally { lock.current = false; if (live.current) setPending(null); }
    },
  } })));
  useUiReading(label, { view: reading, pending, error });
  return { pending, error, human: (id: string) => void invoke(`${parent}:${id}`).catch(cause => { if (live.current && latestOwner.current === owner) setError(String(cause)); }) };
}
</book:entry>

</book:doc>
