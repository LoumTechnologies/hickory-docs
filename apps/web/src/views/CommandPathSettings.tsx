import { useEffect, useState } from "react";
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
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    readCommandPath().then(
      (answer) => { if (live && typeof answer.available === "boolean") setStatus(answer); },
      () => {}, // Headless engines have no desktop command installer.
    );
    return () => { live = false; };
  }, []);
  const apply = async (action: "install" | "remove") => {
    setBusy(true);
    setError(null);
    try { setStatus(await request<CommandPathStatus>("POST", endpoint, { action })); }
    catch (error) { setError(error instanceof Error ? error.message : String(error)); }
    finally { setBusy(false); }
  };
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
            <button className="btn" disabled={busy} onClick={() => void apply("install")}>Install hick command…</button>
          )}
          {status.can_remove && (
            <button className="btn btn-quiet" disabled={busy} onClick={() => void apply("remove")}>Remove hick command</button>
          )}
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
