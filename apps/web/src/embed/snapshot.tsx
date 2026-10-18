import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { decodeSnapshot, PublishedSnapshot, type ViewerSnapshot } from "./PublishedSnapshot";
import "../styles.css";
import "./snapshot.css";
function SnapshotPage() {
  const [snapshot, setSnapshot] = useState<ViewerSnapshot | null>(null);
  const [error, setError] = useState("");
  const read = async (text: string) => { try { setSnapshot(await decodeSnapshot(JSON.parse(text))); setError(""); } catch (e) { setError(String(e)); } };
  useEffect(() => {
    const src = new URL(location.href).searchParams.get("src");
    if (!src) return;
    const url = new URL(src, location.href);
    if (url.origin !== location.origin) { setError("Host the snapshot on the same origin as this viewer."); return; }
    void fetch(url).then(response => { if (!response.ok) throw new Error(`Snapshot download failed: ${response.status}`); return response.text(); }).then(read).catch(e => setError(String(e)));
  }, []);
  return <><header className="snapshot-header"><h1>Hickory Docs snapshot viewer</h1><label>Open exported snapshot <input type="file" accept=".json,application/json" onChange={event => { const file = event.target.files?.[0]; if (file) void file.text().then(read).catch(e => setError(String(e))); }} /></label></header>
    {error && <p role="alert">{error}</p>}{snapshot && <PublishedSnapshot snapshot={snapshot} />}</>;
}
createRoot(document.getElementById("root")!).render(<SnapshotPage />);
