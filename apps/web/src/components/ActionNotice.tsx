import type { ReactNode } from "react";

/** Action feedback belongs beside the work that owns it, including recovery. */
export function ActionNotice({ kind, children, retry, onDismiss }: {
  kind: "pending" | "pass" | "fail";
  children: ReactNode;
  retry?: { label: string; run: () => void };
  onDismiss?: () => void;
}) {
  return <div className={`banner banner-${kind} action-notice`} role={kind === "fail" ? "alert" : "status"}>
    <span>{children}</span>
    {retry && <button className="btn btn-small" onClick={retry.run}>{retry.label}</button>}
    {onDismiss && <button className="btn btn-small" onClick={onDismiss}>Dismiss</button>}
  </div>;
}
