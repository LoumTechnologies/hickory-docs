<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/CommandPathSettings.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `16760eb2c36352d847bf56f35e7f28566ef215d1e8843b756bc068c449ba0065`.

<book:entry path="apps/web/src/views/CommandPathSettings.tsx">
import { useEffect, useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiOutcome } from "../lib/uiActions";
import { request } from "../api/client";

export interface CommandPathStatus {
  available: boolean;
  installed: boolean;
  can_remove: boolean;
  command: string;
  source: string;
  conflict: string | null;
  message: string;
}
const endpoint = "/api/settings/command-path";
export const readCommandPath = () => request<CommandPathStatus>("GET", endpoint);
// Whether the desktop supplies this action cannot change during a session.
// Cache that capability so typing in the command palette does not repeatedly
// start Windows PATH discovery. Settings still reads fresh installation state.
let capability: Promise<boolean> | null = null;
export const hasCommandInstaller = () => capability ??= readCommandPath()
  .then((status) => status.available === true).catch(() => false);

export function CommandPathSettings() {
  const [status, setStatus] = useState<CommandPathStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const pending = useRef(false);
  const epoch = useRef(0);
  const parent = `command-path:${useId()}`;
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let live = true; const stamp = ++epoch.current;
    readCommandPath().then(
      (answer) => { if (live && stamp === epoch.current && typeof answer.available === "boolean") setStatus(answer); },
      reason => { if (live && stamp === epoch.current) setError(String(reason)); }, // A headless engine may have no desktop installer.
    );
    return () => { live = false; };
  }, []);
  const apply = async (action: "install" | "remove"): Promise<UiOutcome> => {
    if (pending.current) return { status: "unavailable", reason: "Command installer operation already running" };
    pending.current = true; const stamp = ++epoch.current; setBusy(true); setError(null);
    try {
      const actual = await request<CommandPathStatus>("POST", endpoint, { action }); if (stamp === epoch.current) setStatus(actual);
      return { status: "completed", value: { action, available: actual.available, installed: actual.installed, can_remove: actual.can_remove, conflict: actual.conflict, message: actual.message, scope: "Returned desktop installer status; not a new CLI execution" } };
    } catch (error) { const reason = error instanceof Error ? error.message : String(error); setError(reason); return { status: "unavailable", reason }; }
    finally { pending.current = false; setBusy(false); }
  };
  const revision = useUiRevision(() => [status, busy, error]);
  const invoke = useUiCommands([
    { key: `${parent}:install`, command: { label: "Install hick command", parent, subject: parent, parameters: {}, prerequisites: ["Desktop installer available; no conflicting command"], read: () => ({ revision: revision(), enabled: !busy && !!status?.available && !status.installed && !status.conflict, reason: busy ? "Installer busy" : status?.message ?? error ?? "Desktop installer status unavailable" }), execute: async () => apply("install") } },
    { key: `${parent}:remove`, command: { label: "Remove hick command", parent, subject: parent, parameters: {}, prerequisites: ["The host recognizes a removable installation"], read: () => ({ revision: revision(), enabled: !busy && !!status?.can_remove, reason: busy ? "Installer busy" : status?.message ?? error ?? "Desktop installer status unavailable" }), execute: async () => apply("remove") } },
    { key: `${parent}:refresh`, command: { label: "Refresh hick command status", parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !busy }), execute: async () => {
      if (pending.current) return { status: "unavailable", reason: "Command installer operation already running" };
      pending.current = true; const stamp = ++epoch.current; setBusy(true);
      try { const actual = await readCommandPath(); if (stamp === epoch.current) { setStatus(actual); setError(null); } return { status: "completed", value: { available: actual.available, installed: actual.installed, message: actual.message } }; }
      catch (error) { const reason = String(error); setError(reason); return { status: "unavailable", reason }; }
      finally { pending.current = false; setBusy(false); }
    } } },
    { key: `${parent}:read`, command: { label: "Read hick command status bytes", parent, subject: parent, parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute: async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ status, busy, error, scope: "Desktop command installer; unsupported/unavailable host status is explicit" }));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside command status" };
      return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
    } } },
  ]);
  const click = (key: string) => { void invoke(`${parent}:${key}`).catch(reason => setError(String(reason))); };
  if (!status) return null;
  return (
    <section className="settings__appearance" aria-label="Command line" id="command-line">
      <h2 className="settings__section-title">Command line</h2>
      <p className="muted"><code>hick</code> opens Hickory Docs. <code>hick .</code> opens a folder;
        <code> hick run</code>, <code>hick test</code>, and <code>hick up</code> use the CLI.</p>
      <div className="settings-row">
        <div className="settings-row__who">
          <span className="settings-row__label">hick command</span>
          <span className="settings-row__state">{status.installed ? "Installed" : "Not installed"}</span>
        </div>
        <div className="settings-row__actions">
          {status.available && !status.installed && !status.conflict && (
            <button className="btn" disabled={busy} onClick={() => click("install")}>Install hick command…</button>
          )}
          {status.can_remove && (
            <button className="btn btn-quiet" disabled={busy} onClick={() => click("remove")}>Remove hick command</button>
          )}
          <button className="btn btn-quiet" disabled={busy} onClick={() => click("refresh")}>Refresh command status</button>
        </div>
      </div>
      <p className="muted settings__note" role="status">{status.message}</p>
      <p className="mono">{status.command}</p>
      {status.conflict && <p className="error">Another hick command is installed at {status.conflict}.
        Remove that installation or adjust PATH before installing this one.</p>}
      {error && <p className="error" role="alert">{error}</p>}
    </section>
  );
}
</book:entry>

</book:doc>
