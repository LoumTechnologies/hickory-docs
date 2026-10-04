import { useEffect, useState } from "react";
import { api } from "../api/client";
import type { CommitReadingData } from "../api/git";
import { DocumentComparison } from "./DocumentReading";

export function CommitReading({ sha }: { sha: string }) {
  const [data, setData] = useState<CommitReadingData | null>(null);
  const [error, setError] = useState("");
  useEffect(() => {
    let live = true;
    api.gitReading(sha).then(data => { if (live) setData(data); }, e => { if (live) setError(String(e)); });
    return () => { live = false; };
  }, [sha]);
  if (error) return <p role="alert">{error}</p>;
  if (!data) return <p className="muted">Reading commit…</p>;
  return <article className="document-reading" aria-label="Commit reading">
    <header className="doc-tab-toolbar">{data.sha.slice(0, 12)} · read-only · {data.parent ? "compared with its first parent" : "initial commit"}</header>
    <DocumentComparison id={`${sha}:message`} path="message.md" before={data.message} after={data.message} />
    {data.files.length === 0 && <p className="muted">No files changed.</p>}
    {data.files.map(file => <section key={file.path} className="commit-reading__file">
      <h2>{file.from !== file.path ? `${file.from} → ` : ""}{file.path} · {file.status}</h2>
      {file.binary ? <p className="muted">Binary file or submodule changed.</p>
        : <DocumentComparison id={`${sha}:${file.path}`} path={file.path} before={file.before} after={file.after} />}
    </section>)}
  </article>;
}
