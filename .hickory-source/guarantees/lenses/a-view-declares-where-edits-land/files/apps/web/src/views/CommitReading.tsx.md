<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/CommitReading.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A view declares where edits land](../../../../../contract.md)

Captured SHA-256: `871e12212c5e37caa85f8818f9883a4eff624594aa751ab4bf1325d2ccd6ae67`.

<book:entry path="apps/web/src/views/CommitReading.tsx">
import { SurfaceInfo } from "../components/SurfaceInfo";
import { useEffect, useId, useRef, useState } from "react";
import type { ProjectEntry } from "../editor/hickLang";
import { api, request } from "../api/client";
import type { CommitReadingData } from "../api/git";
import { DocumentComparison } from "./DocumentReading";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
const message = (cause: unknown) => cause instanceof Error ? cause.message : String(cause);

export function CommitReading({ sha }: { sha: string }) {
  const parent = `commit-reading:${useId()}`;
  const owner = useRef({ sha, generation: 0 });
  if (owner.current.sha !== sha) owner.current = { sha, generation: owner.current.generation + 1 };
  const [reading, setReading] = useState<{ owner: number; data: CommitReadingData } | null>(null);
  const [contexts, setContexts] = useState<{ owner: number; old: ProjectEntry[]; new: ProjectEntry[] } | null>(null);
  const [failure, setFailure] = useState<{ owner: number; text: string } | null>(null);
  const [pending, setPending] = useState(false);
  const job = useRef<object | null>(null);
  const generation = owner.current.generation;
  const data = reading?.owner === generation ? reading.data : null;
  const projects = contexts?.owner === generation ? contexts : undefined;
  const error = failure?.owner === generation ? failure.text : "";
  useEffect(() => {
    const captured = owner.current;
    let live = true;
    job.current = null; setPending(false);
    api.gitReading(sha).then(data => { if (live && owner.current === captured) setReading({ owner: captured.generation, data }); }, cause => { if (live && owner.current === captured) setFailure({ owner: captured.generation, text: message(cause) }); });
    return () => { live = false; job.current = null; };
  }, [sha]);
  const revision = useUiRevision(() => [sha, generation, data, !!projects, pending, error, job.current !== null]);
  useUiReading("commit reading status", { requested_revision: sha, loading: !data && !error, commit: data ? { sha: data.sha, parent: data.parent, files: data.files.map(file => ({ path: file.path, from: file.from, status: file.status, binary: file.binary })) } : null, complete_contexts_loaded: !!projects, pending, error });
  const invoke = useUiCommands(data ? [{ key: parent, command: {
    label: "Load complete commit revision contexts", parent, subject: parent, parameters: {}, prerequisites: ["Current historical commit and its first parent; this loads recorded Git bytes without running them"],
    read: () => ({ revision: revision(), enabled: !!data && !pending && !job.current }),
    execute: async () => {
      if (!data || job.current) return { status: "unavailable", reason: "Wait for the current commit/context load" };
      const captured = owner.current, token = {}; job.current = token; setPending(true); setFailure(null);
      try {
        const [old, next] = await Promise.all([
          request<{ files: ProjectEntry[] }>("POST", "/api/revision-projects", { kind: "git", revision: data.parent ?? ":empty" }),
          request<{ files: ProjectEntry[] }>("POST", "/api/revision-projects", { kind: "git", revision: data.sha }),
        ]);
        if (owner.current !== captured || job.current !== token) return { status: "unavailable", reason: "The commit reading changed while its contexts were loading" };
        setContexts({ owner: captured.generation, old: old.files, new: next.files });
        return { status: "completed", value: { revision: data.sha, contexts_loaded: true, historical: true, executed: false } };
      } catch (cause) {
        if (owner.current === captured && job.current === token) setFailure({ owner: captured.generation, text: message(cause) });
        return { status: "unavailable", reason: message(cause) };
      } finally { if (owner.current === captured && job.current === token) { job.current = null; setPending(false); } }
    },
  } }] : []);
  if (!data) return error ? <p role="alert">{error}</p> : <p className="muted">Reading commit…</p>;
  return <article className="document-reading" aria-label="Commit reading">
    {error && <p role="alert">{error}</p>}
    <SurfaceInfo name={`Commit · ${data.sha.slice(0, 12)}`} editable={false} destination="Historical bytes; this view writes nothing. Open a worktree to edit or run this revision." />
    <header className="doc-tab-toolbar">{data.sha.slice(0, 12)} · read-only · {data.parent ? "compared with its first parent" : "initial commit"}</header>
    <DocumentComparison id={`${sha}:message`} path="message.md" before={data.message} after={data.message} />
    <button className="btn" disabled={pending} onClick={() => void invoke(parent).catch(cause => setFailure({ owner: generation, text: message(cause) }))}>Load complete revision contexts</button>
    {data.files.length === 0 && <p className="muted">No files changed.</p>}
    {data.files.map(file => <section key={file.path} className="commit-reading__file">
      <h2>{file.from !== file.path ? `${file.from} → ` : ""}{file.path} · {file.status}</h2>
      {file.binary ? <p className="muted">Binary file or submodule changed.</p>
        : <DocumentComparison id={`${sha}:${file.path}`} path={file.path} oldPath={file.from} before={file.before} after={file.after} projects={projects} />}
    </section>)}
  </article>;
}
</book:entry>

</book:doc>
