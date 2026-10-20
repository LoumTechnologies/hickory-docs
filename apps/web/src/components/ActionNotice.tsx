import { useId, useRef, useState, type ReactNode } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { ownedText } from "../lib/ownedText";
import type { UiCommand } from "../lib/uiActions";

/** Action feedback belongs beside the work that owns it, including recovery. */
export function ActionNotice({ kind, children, retry, onDismiss }: {
  kind: "pending" | "pass" | "fail";
  children: ReactNode;
  retry?: { label: string; run: () => void };
  onDismiss?: () => void;
}) {
  const parent = `action-notice:${useId()}`;
  const identity = useRef({ retry: retry?.run, dismiss: onDismiss, version: crypto.randomUUID() });
  if (identity.current.retry !== retry?.run || identity.current.dismiss !== onDismiss)
    identity.current = { retry: retry?.run, dismiss: onDismiss, version: crypto.randomUUID() };
  const [error, setError] = useState<string | null>(null);
  const message = ownedText(children);
  const revision = useUiRevision(() => [kind, message, retry?.label, identity.current.version, error]);
  const commands: { key: string; command: UiCommand }[] = [];
  for (const action of [
    ...(retry ? [{ id: "retry", label: retry.label, run: retry.run }] : []),
    ...(onDismiss ? [{ id: "dismiss", label: "Dismiss action notice", run: onDismiss }] : []),
  ]) commands.push({ key: `${parent}:${action.id}`, command: {
    parent, subject: parent, label: action.label, parameters: {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }), execute: async () => {
      await action.run(); return { status: "completed", value: { callback_requested: true,
        scope: "Inspect the owning operation status; requesting recovery does not establish save or verification" } };
    },
  } });
  const invoke = useUiCommands(commands);
  const human = (id: string) => void invoke(`${parent}:${id}`, {}).catch(cause => setError(String(cause)));
  useUiReading("action notice", { kind, message, error, retry_label: retry?.label ?? null });
  return <div className={`banner banner-${kind} action-notice`} role={kind === "fail" ? "alert" : "status"}>
    <span>{children}</span>
    {retry && <button className="btn btn-small" onClick={() => human("retry")}>{retry.label}</button>}
    {onDismiss && <button className="btn btn-small" onClick={() => human("dismiss")}>Dismiss</button>}
    {error && <span role="alert">{error}</span>}
  </div>;
}
