import { useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import { DebuggableDocument } from "./DebuggableDocument";
import type { DocumentEmbedProps } from "./DocumentEmbed";

const CHANNEL = "hickory-embed";
interface LoadedDocument { path: string; source: string; revision: string; edit: number; readOnly: boolean; assets: Record<string, string>; }
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
            setDocument({ path: data.path, source: data.source, revision: data.revision, edit: 0, readOnly: data.readOnly === true, assets });
            setError(null); send("loaded", { revision: data.revision }, message.id); break;
          }
          case "save":
            setDocument((current) => { if (current) send("save", { path: current.path, source: current.source, revision: current.revision }, message.id); return current; }); break;
          case "dispose": ended = true; window.removeEventListener("message", listener); setDisposed(true); setDocument(null); send("disposed", null, message.id); break;
          default: throw Error(`Unsupported embedding operation: ${message.op}`);
        }
      } catch (failure) { setError(String(failure)); send("error", { message: String(failure) }, message.id); }
    };
    window.addEventListener("message", listener);
    send("ready", { capabilities: { editing: true, browserDebug: true, nativeExecution: false, storage: "host" } });
    return () => window.removeEventListener("message", listener);
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
    ? <DebuggableDocument path={document.path} source={document.source} revision={`${document.revision}:${document.edit}`} readOnly={document.readOnly} host={host}
        onChange={change} onSelection={(selection) => send("selection", selection)} />
    : <p role="status">Waiting for the host's document…</p>}</>;
}
createRoot(document.getElementById("root")!).render(<Frame />);
