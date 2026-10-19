import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { decodeSnapshot, PublishedSnapshot, type ViewerSnapshot } from "./PublishedSnapshot";
import "../styles.css";
import "./snapshot.css";
interface PublicationIndex { version: number; revision: string; documents: { path: string; status: string; snapshot: string; revision: string; source_hash: string }[] }
function SnapshotPage() {
  const [index, setIndex] = useState<PublicationIndex | null>(null);
  const [artifactUrl, setArtifactUrl] = useState<string | undefined>();
  const [indexUrl, setIndexUrl] = useState("");
  const [snapshot, setSnapshot] = useState<ViewerSnapshot | null>(null);
  const [error, setError] = useState("");
  const read = async (text: string) => { try { setSnapshot(await decodeSnapshot(JSON.parse(text))); setError(""); } catch (e) { setError(String(e)); } };
  useEffect(() => {
    const src = new URL(location.href).searchParams.get("src");
    if (!src) {
      const selected = new URL(location.href).searchParams.get("index");
      if (!selected) return;
      const indexLocation = new URL(selected, location.href);
      if (indexLocation.origin !== location.origin) { setError("Host publication indexes on the same origin."); return; }
      setIndexUrl(indexLocation.href);
      void fetch(indexLocation).then(async r => { if (!r.ok) throw Error("Publication index unavailable"); const value = await r.json() as PublicationIndex; if (value.version !== 1 || !Array.isArray(value.documents) || value.documents.some(d => typeof d.path !== "string" || typeof d.snapshot !== "string" || !["passed", "failed", "not_run", "unavailable", "stale"].includes(d.status))) throw Error("Invalid publication index"); setIndex(value); }).catch(e => setError(String(e)));
      return;
    }
    const url = new URL(src, location.href);
    if (url.origin !== location.origin) { setError("Host the snapshot on the same origin as this viewer."); return; }
    setArtifactUrl(new URL("./", url).href);
    void fetch(url).then(response => { if (!response.ok) throw new Error(`Snapshot download failed: ${response.status}`); return response.text(); }).then(read).catch(e => setError(String(e)));
  }, []);
  return <><header className="snapshot-header"><h1>Hickory Docs snapshot viewer</h1><label>Open exported snapshot <input type="file" accept=".json,application/json" onChange={event => { const file = event.target.files?.[0]; if (file) { setArtifactUrl(undefined); void file.text().then(read).catch(e => setError(String(e))); } }} /></label></header>
    {index && <main className="published-snapshot"><h2>Published executable specifications</h2><p>Selected public evidence at {index.revision}; each result is revision-bound.</p><ul>{index.documents.map((d, i) => { const src = new URL(d.snapshot, indexUrl); return <li key={i}>{src.origin === location.origin ? <a href={`?src=${encodeURIComponent(src.href)}`}>{d.path} · {d.status}</a> : <span>External artifact refused</span>} · {d.source_hash}</li>; })}</ul></main>}
    {error && <p role="alert">{error}</p>}{snapshot && <PublishedSnapshot snapshot={snapshot} artifactUrl={artifactUrl} />}</>;
}
createRoot(document.getElementById("root")!).render(<SnapshotPage />);
