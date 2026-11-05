<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/BlankWelcome.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-302f315b6ad0"></a>
## `apps/web/src/views/BlankWelcome.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-302f315b6ad0-BlankWelcome"></a>
### `BlankWelcome`

File → New Window opens this intentionally small surface. It has no API
behind it: until a person chooses a file or folder, there is no workspace to
list, modify, lock, or accidentally remember.

A blank window should let someone act immediately. File and folder buttons request the same native pickers as File → Open, with no arbitrary path capability. Cancel remains normal and a browser without native pickers gets an actionable explanation. New File enters the existing unsaved-editor route and needs no folder or provider configuration.

<storycode:copy id="f-302f315b6ad0-BlankWelcome">
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
</storycode:copy>

### Assemble `apps/web/src/views/BlankWelcome.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/BlankWelcome.tsx"><storycode:paste select="#f-302f315b6ad0-BlankWelcome"/></storycode:file>

</storycode:doc>
