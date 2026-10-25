<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/iframe.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [An iframe validates its host](../../../../../contract.md)

Captured SHA-256: `ad6e46f073d0ba50299fbc939940233367290084271ea4d59899790d02f7cc99`.

<book:entry path="apps/web/src/embed/iframe.tsx">
import { useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import { UiActionContext } from "../lib/uiActionBridge";
import { createEmbedUiSession } from "./uiHostSession";
import { DebuggableDocument } from "./DebuggableDocument";
import type { DocumentEmbedProps } from "./DocumentEmbed";

const CHANNEL = "hickory-embed";
type UiSession = ReturnType<typeof createEmbedUiSession>;
interface LoadedDocument { ui: UiSession; path: string; source: string; revision: string; edit: number; readOnly: boolean; assets: Record<string, string>; }
const configured = new URL(location.href).searchParams.get("parentOrigin");
let parentOrigin: string | null = null;
try { if (configured && new URL(configured).origin === configured && configured !== "null") parentOrigin = configured; } catch { /* rejected below */ }

function Frame() {
  const [document, setDocument] = useState<LoadedDocument | null>(null);
  const [disposed, setDisposed] = useState(false);
  const [error, setError] = useState<string | null>(null);
  function send(event: string, data: unknown = null, id?: string) {
    if (parentOrigin) window.parent.postMessage({ channel: CHANNEL, version: 1, event, data, id }, parentOrigin);
  }
  useEffect(() => {
    if (!parentOrigin || window.parent === window) return;
    let ended = false;
    let ui: UiSession | null = null;
    const listener = (event: MessageEvent) => {
      if (event.source !== window.parent || event.origin !== parentOrigin || ended) return;
      const message = event.data;
      if (!message || message.channel !== CHANNEL || message.version !== 1) return;
      try {
        switch (message.op) {
          case "load": {
            const data = message.data;
            if (!data || typeof data.source !== "string" || typeof data.path !== "string" || !data.path.endsWith(".md") || typeof data.revision !== "string") throw Error("load requires a .md path, source and revision");
            const assets: Record<string, string> = Object.create(null);
            for (const [path, url] of Object.entries(data.assets ?? {})) {
              if (typeof url !== "string" || !/^(https?:|blob:|data:image\/)/.test(url)) throw Error(`Invalid asset URL: ${path}`);
              assets[path] = url;
            }
            ui?.dispose();
            ui = createEmbedUiSession(value => send("ui-receipt", value));
            setDocument({ ui, path: data.path, source: data.source, revision: data.revision, edit: 0, readOnly: data.readOnly === true, assets });
            setError(null); send("loaded", { revision: data.revision, ui_scope: ui.scope }, message.id); break;
          }
          case "ui": {
            if (!ui) throw Error("Load a document before requesting its UI controls");
            if (typeof message.id !== "string" || !message.id.length || message.id.length > 128) throw Error("Embedded UI requests require a bounded response id");
            const currentUi = ui;
            void currentUi.invoke(message.data).then(result => send("ui-result", { scope: currentUi.scope, result }, message.id), failure => send("error", { message: String(failure), ui_scope: currentUi.scope }, message.id));
            break;
          }
          case "save":
            setDocument((current) => { if (current) send("save", { path: current.path, source: current.source, revision: current.revision }, message.id); return current; }); break;
          case "dispose": ui?.dispose(); ended = true; window.removeEventListener("message", listener); setDisposed(true); setDocument(null); send("disposed", null, message.id); break;
          default: throw Error(`Unsupported embedding operation: ${message.op}`);
        }
      } catch (failure) { setError(String(failure)); send("error", { message: String(failure) }, message.id); }
    };
    window.addEventListener("message", listener);
    send("ready", { capabilities: { editing: true, browserDebug: true, nativeExecution: false, uiActions: "scoped-v1", uiActor: "embed-parent agent; no human attestation", receiptStorage: "frame session results; host must persist receipt events", storage: "host" } });
    return () => { ended = true; ui?.dispose(); window.removeEventListener("message", listener); };
  }, []);
  const host = useMemo(() => ({ resolveAsset: async (path: string) => {
    const url = document?.assets[path]; if (!url) throw Error(`Host did not supply asset ${path}`); return url;
  } }), [document?.assets]);
  const change: DocumentEmbedProps["onChange"] = (next) => {
    setDocument((current) => current ? { ...current, source: next.source, edit: current.edit + 1 } : null);
    send("change", { ...next, baseRevision: document?.revision });
  };
  if (!parentOrigin) return <p role="alert">The host must configure a concrete parentOrigin.</p>;
  if (disposed) return null;
  return <>{error && <p role="alert">{error}</p>}{document
    ? <UiActionContext.Provider key={document.ui.scope.window} value={document.ui.registry}><DebuggableDocument path={document.path} source={document.source} revision={`${document.revision}:${document.edit}`} readOnly={document.readOnly} host={host}
        onChange={change} onSelection={(selection) => send("selection", selection)} /></UiActionContext.Provider>
    : <p role="status">Waiting for the host's document…</p>}</>;
}
createRoot(document.getElementById("root")!).render(<Frame />);
</book:entry>

</book:doc>
