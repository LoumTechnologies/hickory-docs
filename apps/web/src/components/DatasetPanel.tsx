import { useEffect, useState } from "react";
import { ReadOnlyTable, type NotebookTable } from "./ReadOnlyTable";

export function DatasetPanel({ src, format, path }: { src: string; format: string; path: string | null }) {
  const [page, setPage] = useState(0);
  const [revision, setRevision] = useState(0);
  const [table, setTable] = useState<NotebookTable | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [remote, setRemote] = useState<string | null>(null);
  useEffect(() => { setPage(0); }, [src, format, path]);
  useEffect(() => {
    setTable(null); setError(null);
    if (src.startsWith("s3://") && remote !== src) return;
    const controller = new AbortController();
    const query = new URLSearchParams({ src, format, offset: String(page * 100), limit: "100", ...(path ? { doc_path: path } : {}) });
    fetch(`/api/dataset?${query}`, { signal: controller.signal }).then(async response => {
      if (!response.ok) {
        const raw = await response.text();
        let message = raw;
        try { message = JSON.parse(raw).error ?? raw; } catch { /* Plain-text response. */ }
        throw new Error(message);
      }
      return response.json() as Promise<NotebookTable>;
    }).then(value => { if (!controller.signal.aborted) setTable(value); }).catch(e => { if (!controller.signal.aborted) setError(String(e.message)); });
    return () => controller.abort();
  }, [src, format, path, page, revision, remote]);
  return <section className="dataset-panel" aria-label="Dataset preview">
    <div><strong>{src}</strong> <span className="muted">{format} · read-only</span> <button onClick={() => setRevision(v => v + 1)}>Refresh</button></div>
    {src.startsWith("s3://") && remote !== src ? <button onClick={() => setRemote(src)}>Load from S3</button> : error ? <p role="alert">{error}</p> : !table ? <p>Loading dataset…</p> : <>
      <ReadOnlyTable key={`${src}:${page}:${revision}`} table={table} />
      <nav aria-label="Dataset pages"><button disabled={page === 0} onClick={() => setPage(v => v - 1)}>Previous</button> <span>Page {page + 1}</span> <button disabled={!table.has_more} onClick={() => setPage(v => v + 1)}>Next</button></nav>
    </>}
  </section>;
}
