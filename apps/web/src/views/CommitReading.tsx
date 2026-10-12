import { SurfaceInfo } from "../components/SurfaceInfo";
import { useEffect, useState } from "react";
import type { ProjectEntry } from "../editor/hickLang";
import { api, request } from "../api/client";
import type { CommitReadingData } from "../api/git";
import { DocumentComparison } from "./DocumentReading";

export function CommitReading({ sha }: { sha: string }) {
  const [data, setData] = useState<CommitReadingData | null>(null);
  const [projects,setProjects] = useState<{old:ProjectEntry[];new:ProjectEntry[]}|undefined>();
  const [error, setError] = useState("");
  useEffect(() => {
    setProjects(undefined);
    let live = true;
    api.gitReading(sha).then(data => { if (live) setData(data); }, e => { if (live) setError(String(e)); });
    return () => { live = false; };
  }, [sha]);
  if (error) return <p role="alert">{error}</p>;
  if (!data) return <p className="muted">Reading commit…</p>;
  return <article className="document-reading" aria-label="Commit reading">
    <SurfaceInfo name={`Commit · ${data.sha.slice(0, 12)}`} editable={false} destination="Historical bytes; this view writes nothing. Open a worktree to edit or run this revision." />
    <header className="doc-tab-toolbar">{data.sha.slice(0, 12)} · read-only · {data.parent ? "compared with its first parent" : "initial commit"}</header>
    <DocumentComparison id={`${sha}:message`} path="message.md" before={data.message} after={data.message} />
    <button className="btn" onClick={()=>void Promise.all([
      request<{files:ProjectEntry[]}>("POST","/api/revision-projects",{kind:"git",revision:data.parent ?? ":empty"}),
      request<{files:ProjectEntry[]}>("POST","/api/revision-projects",{kind:"git",revision:data.sha}),
    ]).then(([old,next])=>setProjects({old:old.files,new:next.files}),e=>setError(String(e)))}>Load complete revision contexts</button>
    {data.files.length === 0 && <p className="muted">No files changed.</p>}
    {data.files.map(file => <section key={file.path} className="commit-reading__file">
      <h2>{file.from !== file.path ? `${file.from} → ` : ""}{file.path} · {file.status}</h2>
      {file.binary ? <p className="muted">Binary file or submodule changed.</p>
        : <DocumentComparison id={`${sha}:${file.path}`} path={file.path} oldPath={file.from} before={file.before} after={file.after} projects={projects} />}
    </section>)}
  </article>;
}
