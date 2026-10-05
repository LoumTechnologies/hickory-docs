// An external host: imports only the supported embedding entry, no App/site entry.
import { createRoot } from "react-dom/client";
import { useEffect, useState } from "react";
import { BrowserWorkspace, openLocalStorage, openS3Storage, openSyncedStorage, DocumentEmbed, DebuggableDocument } from "./index";
import type { WorkspaceStorage } from "./index";
import { measurePortableCore } from "./measure";
import { browserExample } from "../landing/demos/BrowserDebugDemo";

function Fixture() {
  const [metrics, setMetrics] = useState("");
  useEffect(() => { void measurePortableCore().then((value) => setMetrics(JSON.stringify(value))); }, []);
  const [left, setLeft] = useState("# First note\n\nAn ordinary note.\n\n![asset](images/mark.svg)\n<hick:unknown untouched=\"yes\">🦀 raw</hick:unknown>\n");
  const [right, setRight] = useState("# Second note\n\nIndependent text.\n");
  const [source, setSource] = useState(browserExample("ts"));
  const [revision, setRevision] = useState(0);
  const [readOnly, setReadOnly] = useState(false);
  const [mounted, setMounted] = useState(true);
  const [secondDebug, setSecondDebug] = useState(false);
  const [saved, setSaved] = useState("");
  return <main style={{ maxWidth: 1000, margin: "30px auto", fontFamily: "system-ui" }}>
    <h1>A host that owns its documents</h1>
    <output aria-label="Browser measurements">{metrics}</output>
    <button onClick={() => setReadOnly((value) => !value)}>Toggle read-only</button>
    <button onClick={() => setMounted((value) => !value)}>Mount/unmount</button>
    <button onClick={() => { setLeft("# Host replacement\n"); setRevision((r) => r + 1); }}>Replace source</button>
    <button onClick={() => setSaved(left)}>Save explicitly</button>
    <output aria-label="Saved document">{saved}</output>
    {mounted && <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
      <DocumentEmbed path="first.md" source={left} revision={String(revision)} readOnly={readOnly} onChange={(change) => { setLeft(change.source); setRevision((r) => r + 1); }}
        host={{ resolveAsset: async () => "data:image/svg+xml," + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="40" height="40"><circle cx="20" cy="20" r="18" fill="orange"/></svg>') }} />
      <DocumentEmbed path="second.md" source={right} revision="0" onChange={(change) => setRight(change.source)} />
    </div>}
    <h2>Local debugger</h2>
    <button onClick={() => setSecondDebug((value) => !value)}>Add/remove second debugger</button>
    <textarea aria-label="Debug source" style={{ width: "100%", height: 100 }} value={source} onChange={(event) => { setSource(event.target.value); setRevision((r) => r + 1); }} />
    {secondDebug && <DebuggableDocument path="second-debug.md" revision="1" source={'<hick:file path="second.js">console.log(7);</hick:file>'} />}
    <DebuggableDocument source={source} path="fixture.md" revision={String(revision)} onChange={(change) => { setSource(change.source); setRevision((r) => r + 1); }} />
  </main>;
}
function LocalFixture() {
  const [storage, setStorage] = useState<WorkspaceStorage | null>(null);
  const [error, setError] = useState("");
  useEffect(() => {
    let disposed = false;
    const resources: WorkspaceStorage[] = [];
    void (async () => {
      const params = new URLSearchParams(location.search);
      const local = await openLocalStorage(params.has("s3") ? `fixture-s3-${params.get("local") ?? "one"}` : "fixture"); resources.push(local);
      let current: WorkspaceStorage = local;
      if (params.has("s3")) {
        // Test host callback, intercepted by acceptance tests; no signing or credentials.
        const remote = await openS3Storage({ prefix: "fixture", request: ({ method, key, body, headers, signal }) =>
          fetch(`http://localhost:4178/bucket/${key}`, { method, headers, body: body?.slice().buffer, signal }) });
        resources.push(remote);
        current = await openSyncedStorage(local, remote, "fixture"); resources.push(current);
      }
      if (disposed) for (const resource of resources) resource.close(); else setStorage(current);
    })().catch((e) => { for (const resource of resources) resource.close(); if (!disposed) setError(String(e)); });
    return () => { disposed = true; for (const resource of resources) resource.close(); };
  }, []);
  return storage ? <BrowserWorkspace storage={storage} /> : <p>{error || "Loading local storage…"}</p>;
}
createRoot(document.getElementById("root")!).render(location.search.includes("workspace") ? <LocalFixture /> : <Fixture />);
