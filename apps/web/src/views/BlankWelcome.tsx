import { useState } from "react";
import { request } from "../api/client";
import { requestMenuAction } from "../lib/menuBridge";
import { useUiCommands } from "../lib/uiActionBridge";

export function BlankWelcome() {
  const [error, setError] = useState("");
  const [pending, setPending] = useState(false);
  const open = async (kind: "file" | "folder") => {
    setPending(true); setError("");
    try { await request("POST", `/api/window/open-${kind}`, {}); return { status: "completed" as const, value: { picker_requested: true, path_opened: "Not yet observed" } }; }
    catch { const reason = `Could not open the ${kind} picker. In the desktop app, use File → Open ${kind === "file" ? "File" : "Folder"}. In a browser, open a workspace with hick open PATH.`; setError(reason); return { status: "unavailable" as const, reason }; }
    finally { setPending(false); }
  };
  const invoke = useUiCommands(([
    ["file", "Open File…"], ["folder", "Open Folder…"], ["new", "New File"],
  ] as const).map(([id, label]) => ({ key: `blank-welcome:${id}`, command: {
    label, parent: "blank-welcome", subject: "blank-welcome", parameters: {}, prerequisites: [],
    read: () => ({ revision: String(pending), enabled: !pending }),
    execute: async () => { if (id === "new") { requestMenuAction("new"); return { status: "completed", value: { untitled_requested: true } }; } return open(id); },
  } })));
  return <section className="welcome" aria-label="Welcome"><div className="welcome__inner">
    <header className="welcome__head"><h1 className="welcome__title">Hickory Docs</h1></header>
    <div className="welcome__columns"><section className="welcome__col" aria-label="Start">
      {([ ["file", "Open File…"], ["folder", "Open Folder…"], ["new", "New File"] ] as const).map(([id, label]) => <button key={id} type="button" className="welcome__action" disabled={pending} onClick={() => { void invoke(`blank-welcome:${id}`); }}>{label}</button>)}
      {error && <p role="alert">{error}</p>}
    </section></div>
  </div></section>;
}
