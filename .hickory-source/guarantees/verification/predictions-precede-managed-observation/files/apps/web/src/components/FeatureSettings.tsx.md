<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/FeatureSettings.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `c2851465b5e59b692d135a3c522ccaeaf57c3e74d138434cb6c63db9f321173f`.

<book:entry path="apps/web/src/components/FeatureSettings.tsx">
import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { useUiCommands, useUiPendingHumanActions } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import "./FeatureSettings.css";

/** Settings stay beside the feature they control, outside pane clipping. */
export function FeatureSettings({ label, children, owner }: { label: string; children: ReactNode; owner?: string }) {
  const [position, setPosition] = useState<{ right: number; top?: number; bottom?: number } | null>(null);
  const id = useId();
  const root = useRef<HTMLSpanElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  const subject = `feature-settings:${id}:${owner ?? "local"}`;
  const pending = useUiPendingHumanActions(subject);
  const epoch = useRef(0);
  const [error, setError] = useState("");
  useEffect(() => { epoch.current++; return () => { epoch.current++; }; }, [owner]);
  useUiReading(`feature settings ${label}`, { open: !!position, owner: owner ?? null, error });
  const close = () => {
    const captured = epoch.current;
    const finish = () => {
      if (epoch.current !== captured || !root.current) return { closed: false, reason: "Settings owner changed before dismissal" };
      epoch.current++; setPosition(null); return { closed: true };
    };
    const requests = pending();
    if (!requests.length) return Promise.resolve(finish());
    // Retain mounted form controllers through intent AND result persistence.
    // The visibility command's own subject is excluded, so it cannot await itself.
    return Promise.all(requests.map(request => request.result)).then(results => {
      const failed = results.find(result => result.status !== "result" || result.outcome.status === "unavailable");
      if (failed) {
        const reason = failed.status !== "result" ? failed.reason : failed.outcome.status === "unavailable" ? failed.outcome.reason : "Settings change unavailable";
        if (epoch.current === captured && root.current) setError(reason);
        return { closed: false, reason };
      }
      return finish();
    });
  };
  const open = () => {
    if (!root.current) return;
    epoch.current++; setError("");
    const rect = root.current.getBoundingClientRect();
    const right = Math.max(8, Math.min(window.innerWidth - rect.right, window.innerWidth - Math.min(352, window.innerWidth * .8) - 8));
    setPosition(root.current.closest(".status-bar")
      ? { right, bottom: window.innerHeight - rect.top }
      : { right, top: rect.bottom });
  };
  useUiCommands([{ key: `feature-settings:${id}`, command: {
    label: `Set ${label} visibility`, parent: "feature-settings", subject, parameters: { open: { type: "boolean" } }, prerequisites: [],
    read: () => ({ revision: `${owner ?? "local"}:${!!position}`, enabled: !!root.current, value: { open: !!position, owner: owner ?? null, disclosure: "Children mount only while this settings panel is open" } }),
    execute: async args => { if (args.open) open(); else { const result = await close(); if (!result.closed) return { status: "unavailable", reason: result.reason ?? "Settings dismissal unavailable" }; } return { status: "completed", value: { open: Boolean(args.open), result: "Observe disclosed setting controls" } }; },
  } }]);
  useEffect(() => {
    if (!position) return;
    const dismiss = (event: PointerEvent) => {
      const target = event.target as Node;
      if (!root.current?.contains(target) && !panel.current?.contains(target)) void close();
    };
    document.addEventListener("pointerdown", dismiss);
    const resize = () => { void close(); };
    window.addEventListener("resize", resize);
    return () => { document.removeEventListener("pointerdown", dismiss); window.removeEventListener("resize", resize); };
  }, [position]);
  const escape = (event: React.KeyboardEvent) => { if (event.key === "Escape") { event.stopPropagation(); void close(); } };
  return <span ref={root} className="feature-settings" onKeyDown={escape}>
    <button type="button" className="feature-settings__gear" aria-label={label} aria-expanded={!!position} aria-controls={id}
      data-tip={label} onClick={() => {
        if (position) { void close(); return; }
        open();
      }}><span aria-hidden="true">⚙</span></button>
    {position && createPortal(<div ref={panel} id={id} style={position} className="feature-settings__panel" role="region" aria-label={label} onKeyDown={escape}>
      <strong>{label}</strong>{error && <p role="alert">{error}</p>}{children}
    </div>, document.body)}
  </span>;
}
</book:entry>

</book:doc>
