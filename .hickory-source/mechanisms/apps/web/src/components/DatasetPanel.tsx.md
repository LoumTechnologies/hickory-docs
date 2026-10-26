<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/DatasetPanel.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2bced9a625eb"></a>
## `apps/web/src/components/DatasetPanel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-2bced9a625eb-DatasetPanel"></a>
### `DatasetPanel`

<storycode:copy id="f-2bced9a625eb-DatasetPanel">
import { useEffect, useId, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand } from "../lib/uiActions";
import { ReadOnlyTable, type NotebookTable } from "./ReadOnlyTable";

export function DatasetPanel({ src, format, path }: { src: string; format: string; path: string | null }) {
  const [page, setPage] = useState(0);
  const [revision, setRevision] = useState(0);
  const [response, setResponse] = useState<{ key: string; table: NotebookTable | null; error: string | null } | null>(null);
  const [remote, setRemote] = useState<string | null>(null);
  const responseKey = JSON.stringify([src, format, path, page, revision, remote]);
  const table = response?.key === responseKey ? response.table : null;
  const error = response?.key === responseKey ? response.error : null;
  useEffect(() => { setPage(0); }, [src, format, path]);
  useEffect(() => {
    setResponse(null);
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
    }).then(value => { if (!controller.signal.aborted) setResponse({ key: responseKey, table: value, error: null }); }).catch(e => { if (!controller.signal.aborted) setResponse({ key: responseKey, table: null, error: String(e.message) }); });
    return () => controller.abort();
  }, [src, format, path, page, revision, remote]);
  const parent = `dataset:${useId()}`;
  const currentRevision = useUiRevision(() => [responseKey, response, error]);
  const refresh = () => setRevision(v => v + 1);
  const loadRemote = () => setRemote(src);
  const navigate = (next: number) => setPage(next);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, enabled: boolean, execute: UiCommand["execute"], parameters: UiCommand["parameters"] = {}) => commands.push({ key: `${parent}:${key}`, command: { label, parent, subject: parent, parameters, prerequisites: [], read: () => ({ revision: currentRevision(), enabled }), execute } });
  add("refresh", "Refresh dataset preview", true, async () => { refresh(); return { status: "completed", value: { refresh_requested: true, loaded: "Observe current page or error" } }; });
  if (src.startsWith("s3://") && remote !== src) add("remote", "Load dataset from S3", true, async () => { loadRemote(); return { status: "completed", value: { remote_read_requested: true, loaded: "Observe actual page/error; host credentials and availability still apply" } }; });
  if (table) {
    add("previous", "Previous dataset page", page > 0, async () => { navigate(page - 1); return { status: "completed", value: { page_requested: page, loaded: false } }; });
    add("next", "Next dataset page", table.has_more, async () => { navigate(page + 1); return { status: "completed", value: { page_requested: page + 2, loaded: false } }; });
  }
  add("read", "Read dataset preview status bytes", true, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ src, format, doc_path: path, page: page + 1, error, waiting_for_remote: src.startsWith("s3://") && remote !== src, loaded: !!table, rows: table?.rows.length ?? 0, source_offset: table?.offset ?? null, has_more: table?.has_more ?? null, total: table?.total ?? null, columns_truncated: table?.columns_truncated ?? null, scope: "Dataset preview status; read the disclosed table for current values" }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside current dataset status" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  }, { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } });
  const invoke = useUiCommands(commands);
  return <section className="dataset-panel" aria-label="Dataset preview">
    <div><strong>{src}</strong> <span className="muted">{format} · read-only</span> <button onClick={() => void invoke(`${parent}:refresh`)}>Refresh</button></div>
    {src.startsWith("s3://") && remote !== src ? <button onClick={() => void invoke(`${parent}:remote`)}>Load from S3</button> : error ? <p role="alert">{error}</p> : !table ? <p>Loading dataset…</p> : <>
      <ReadOnlyTable key={`${src}:${page}:${revision}`} table={table} />
      <nav aria-label="Dataset pages"><button disabled={page === 0} onClick={() => void invoke(`${parent}:previous`)}>Previous</button> <span>Page {page + 1}</span> <button disabled={!table.has_more} onClick={() => void invoke(`${parent}:next`)}>Next</button></nav>
    </>}
  </section>;
}
</storycode:copy>

### Assemble `apps/web/src/components/DatasetPanel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/DatasetPanel.tsx"><storycode:paste select="#f-2bced9a625eb-DatasetPanel"/></storycode:file>

</storycode:doc>
