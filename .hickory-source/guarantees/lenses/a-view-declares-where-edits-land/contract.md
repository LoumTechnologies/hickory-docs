<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A view declares where edits land

For: Authors reviewing history and choosing where an edit should land.

Given a document, plain or generated file, literate view, change review,
commit reading, conversation history, worktree comparison, or history story,
when the working surface is shown, then its identity, editability, and write
destination appear together. Historical comparisons disable writes. Generated
regions retain their existing protection; declaring an editable surface does
not make every byte editable or grant a new engine capability.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lenses/a-view-declares-where-edits-land.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-6227c4f8a79f"></a>
## `apps/web/src/views/CommitReading.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A view declares where edits land](contract.md).

<a id="f-6227c4f8a79f-message"></a>
### `message`

<storycode:copy id="f-6227c4f8a79f-message">
import { SurfaceInfo } from "../components/SurfaceInfo";
import { useEffect, useId, useRef, useState } from "react";
import type { ProjectEntry } from "../editor/hickLang";
import { api, request } from "../api/client";
import type { CommitReadingData } from "../api/git";
import { DocumentComparison } from "./DocumentReading";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
const message = (cause: unknown) => cause instanceof Error ? cause.message : String(cause);

</storycode:copy>
<a id="f-6227c4f8a79f-CommitReading"></a>
### `CommitReading`

<storycode:copy id="f-6227c4f8a79f-CommitReading">
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
</storycode:copy>

### Assemble `apps/web/src/views/CommitReading.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/CommitReading.tsx"><storycode:paste select="#f-6227c4f8a79f-message"/><storycode:paste select="#f-6227c4f8a79f-CommitReading"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/SurfaceInfo.tsx](../../../mechanisms/apps/web/src/components/SurfaceInfo.tsx.md#output-6c34dd749b33)
- [apps/web/src/views/CommitReading.tsx](contract.md#output-6227c4f8a79f)
- [apps/web/src/views/DocumentReading.tsx](../../../mechanisms/apps/web/src/views/DocumentReading.tsx.md#output-d10820ba127c)
- [apps/web/src/views/HistoryLens.tsx](../../../mechanisms/apps/web/src/views/HistoryLens.tsx.md#output-b01a7a57f285)
- [apps/web/src/views/MergedView.tsx](../../../mechanisms/apps/web/src/views/MergedView.tsx.md#output-0e4f75bd1f77)
- [apps/web/src/views/RepresentationPane.tsx](../../../mechanisms/apps/web/src/views/RepresentationPane.tsx.md#output-cd66c1c2250e)
- [apps/web/src/views/SessionLens.tsx](../../../mechanisms/apps/web/src/views/SessionLens.tsx.md#output-11b8d4e005a0)
- [apps/web/src/views/workspaceTabs.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-b5a48fd337ae)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [apps/web/src/views/RepresentationPane.test.tsx](../../../mechanisms/apps/web/src/views/RepresentationPane.tsx.md#output-17926fbc73d7)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-5370130a675d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
