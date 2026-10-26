<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/ContextUpdates.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-83abe8d9f9e9"></a>
## `apps/web/src/components/ContextUpdates.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit context updates preserve confirmed history](../../../../../guarantees/agent/explicit-context-updates/contract.md).

<a id="f-83abe8d9f9e9-Item"></a>
### `Item`

<storycode:copy id="f-83abe8d9f9e9-Item">
import { useState, useEffect, useId, useRef } from "react";
import { UiActionsVisibility, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiField } from "../lib/uiActions";
import { request } from "../api/client";
import type { AgentEditorContext } from "../api/agentTypes";
import { unifiedDiff } from "../lib/diff";
import { DiffView } from "./DiffView";

type Item = { reference: string; path: string; before: string; after: string | null; status: string; revision: string; eligible: boolean; role: string; scope: string; request: string };
</storycode:copy>
<a id="f-83abe8d9f9e9-View"></a>
### `View`

<storycode:copy id="f-83abe8d9f9e9-View">
type View = { items: Item[]; updates: { id: string; old: string; state: string; request: string | null; source: { revision: string } }[]; coverage: string };
</storycode:copy>
<a id="f-83abe8d9f9e9-ContextUpdateProps"></a>
### `ContextUpdateProps`

<storycode:copy id="f-83abe8d9f9e9-ContextUpdateProps">
type ContextUpdateProps = { doc: string; branch: string | null; getContext?: () => Promise<AgentEditorContext> };
</storycode:copy>
<a id="f-83abe8d9f9e9-ContextUpdates"></a>
### `ContextUpdates`

<storycode:copy id="f-83abe8d9f9e9-ContextUpdates">
export function ContextUpdates(props: ContextUpdateProps) {
  return <OwnedContextUpdates key={JSON.stringify([props.doc, props.branch])} {...props} />;
}
</storycode:copy>
<a id="f-83abe8d9f9e9-OwnedContextUpdates"></a>
### `OwnedContextUpdates`

<storycode:copy id="f-83abe8d9f9e9-OwnedContextUpdates">
function OwnedContextUpdates({ doc, branch, getContext }: ContextUpdateProps) {
  const captureContext = useRef(getContext); captureContext.current = getContext;
  const instance = useId(); const parent = `context-updates:${instance}`;
  const disclosure = useRef<HTMLDetailsElement>(null);
  const [open, setOpen] = useState(false);
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const scope = `${doc}:${branch}`; const scopeRef = useRef(scope); scopeRef.current = scope;
  const [view, setView] = useState<View | null>(null);
  const [selected, setSelected] = useState<string[]>([]);
  const [selectedRevisions, setSelectedRevisions] = useState<Record<string, string>>({});
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const activeOwner = useRef(true), admission = useRef(false), requestVersion = useRef(0);
  useEffect(() => { activeOwner.current = true; return () => { activeOwner.current = false; requestVersion.current++; }; }, []);
  useEffect(() => { setView(null); setSelected([]); setSelectedRevisions({}); setExpanded({}); setError(""); setBusy(false); }, [doc, branch]);
  async function inspect(queue = false) {
    if (!branch || admission.current) return { error: "Select a branch and wait for the current context operation" };
    const requestedScope = scope, ticket = ++requestVersion.current;
    admission.current = true; setBusy(true); setError("");
    try {
      const context = getContext ? await getContext() : { buffers: [] };
      if (!activeOwner.current || scopeRef.current !== requestedScope || requestVersion.current !== ticket) return { error: "The originating context branch was retired before dispatch; no update was submitted" };
      const result = await request<View>("POST", `/api/docs/${encodeURIComponent(doc)}/agent/python/context`, {
        branch, context, selected: queue ? selected : [],
        revisions: queue ? selectedRevisions : {},
      });
      if (!activeOwner.current || scopeRef.current !== requestedScope || requestVersion.current !== ticket) return { error: "Branch changed while the context operation was in flight; inspect that originating branch before retrying a queued update" };
      setView(result);
      if (queue) setSelected([]);
      return { result };
    } catch (e) { if (activeOwner.current && scopeRef.current === requestedScope && requestVersion.current === ticket) setError(String(e)); return { error: String(e) }; } finally { if (requestVersion.current === ticket) { admission.current = false; if (activeOwner.current && scopeRef.current === requestedScope) setBusy(false); } }
  }
  useEffect(() => {
    let active = true;
    let pending = false;
    async function refresh() {
      if (!branch || pending || admission.current) return;
      const ticket = ++requestVersion.current; pending = true;
      try {
        const context = captureContext.current ? await captureContext.current() : { buffers: [] };
        if (!active || !activeOwner.current || ticket !== requestVersion.current) return;
        const result = await request<View>("POST", `/api/docs/${encodeURIComponent(doc)}/agent/python/context`, { branch, context });
        if (active && activeOwner.current && ticket === requestVersion.current) setView(result);
      } catch (e) { if (active && activeOwner.current && ticket === requestVersion.current) setError(String(e)); }
      finally { pending = false; }
    }
    void refresh();
    const timer = setInterval(() => void refresh(), 5000);
    return () => { active = false; clearInterval(timer); };
  }, [doc, branch]);
  const changed = view?.items.filter(i => i.status !== "unchanged") ?? [];
  const select = (item: Item, checked: boolean) => {
    if (!item.eligible || admission.current) return;
    setSelected(s => checked ? [...new Set([...s, item.reference])] : s.filter(id => id !== item.reference));
    setSelectedRevisions(s => ({ ...s, [item.reference]: item.revision }));
  };
  const selectAll = () => { if (admission.current) return; setSelected(view?.items.filter(i => i.eligible).map(i => i.reference) ?? []); setSelectedRevisions(Object.fromEntries((view?.items.filter(i => i.eligible) ?? []).map(i => [i.reference, i.revision]))); };
  const revision = useUiRevision(() => [scope, open, expanded, view, selected, selectedRevisions, busy, admission.current, error]);
  const commands: { key: string; command: UiCommand }[] = [];
  function add(id: string, label: string, parameters: Record<string, UiField>, execute: UiCommand["execute"], enabled = !!branch && !busy, group = parent) {
    commands.push({ key: `${parent}/${id}`, command: { label, parent: group, subject: parent, parameters, prerequisites: ["Updates retain confirmed branch/reference/revision lineage; queuing is not provider delivery"], read: () => ({ revision: revision(), enabled: enabled && (!admission.current || id.startsWith("read") || id === "disclosure"), reason: enabled && (!admission.current || id.startsWith("read") || id === "disclosure") ? undefined : "Select a branch and wait for the current context operation" }), execute } });
  }
  add("disclosure", "Set changed agent context visibility", { open: { type: "boolean" } }, async args => { if (!disclosure.current) return { status: "unavailable", reason: "Context disclosure is not mounted" }; disclosure.current.open = Boolean(args.open); setOpen(Boolean(args.open)); return { status: "completed", value: { open: Boolean(args.open) } }; }, true, "agent-context-disclosures");
  if (open) {
    const page: Record<string, UiField> = { offset: { type: "number", minimum: 0, maximum: 100000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } };
    add("read", "Read context update status bytes", page, async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ doc, branch, selected, error, busy, coverage: view?.coverage, items: changed.map(({ before: _before, after: _after, ...item }) => item), updates: view?.updates }));
      const offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds current context status" };
      return { status: "completed", value: { encoding: "JSON encoded as UTF-8 bytes", bytes: [...bytes.slice(offset, end)], span: [offset, end], total_bytes: bytes.length, next: end < bytes.length ? end : null } };
    }, true);
    for (const queue of [false, true]) add(queue ? "queue" : "refresh", queue ? "Queue selected context updates" : "Refresh patches and delivery status", {}, async () => {
      const outcome = await inspect(queue);
      return outcome.result ? { status: "completed", value: { branch, updates: JSON.parse(JSON.stringify(outcome.result.updates)), coverage: outcome.result.coverage, scope: queue ? "Queue response; inspect actual subsequent delivery receipts" : "Current comparison response; no update queued" } } : { status: "unavailable", reason: outcome.error ?? "Context operation unavailable" };
    }, !!branch && !busy && (!queue || !!selected.length));
    add("select-all", "Select all eligible context updates", {}, async () => { selectAll(); return { status: "completed", value: { queued: false } }; }, !busy && !!view?.items.some(item => item.eligible));
    for (const item of changed) {
      add(`open/${item.reference}`, `Set context patch visibility: ${item.path}`, { open: { type: "boolean" } }, async args => { setExpanded(current => ({ ...current, [item.reference]: Boolean(args.open) })); return { status: "completed", value: { open: Boolean(args.open) } }; });
      if (expanded[item.reference]) {
        const group = `${parent}/patch/${item.reference}`;
        add(`select/${item.reference}`, `Select context update: ${item.path}`, { checked: { type: "boolean" } }, async args => { select(item, Boolean(args.checked)); return { status: "completed", value: { selected: Boolean(args.checked), queued: false, revision: item.revision } }; }, item.eligible && !busy, group);
        add(`read/${item.reference}`, `Read context patch bytes: ${item.path}`, page, async args => {
          const bytes = new TextEncoder().encode(JSON.stringify(item)); const offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
          if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds this retained comparison" };
          return { status: "completed", value: { reference: item.reference, revision: item.revision, encoding: "JSON encoded as UTF-8 bytes", bytes: [...bytes.slice(offset, end)], span: [offset, end], total_bytes: bytes.length, next: end < bytes.length ? end : null, edit_baseline: false } };
        }, true, group);
      }
    }
  }
  useUiCommands(commands);
  return <details ref={disclosure} onToggle={e => { setOpen(e.currentTarget.open); if (e.currentTarget.open) void inspect(); }}>
    <summary>Changed agent context ({changed.length})</summary>
    <p>Compare with confirmed delivery on this branch. Updates enter a subsequent model boundary. Unsaved text is never saved by this action.</p>
    <button className="btn" disabled={busy || !branch} onClick={() => void inspect()}>Refresh patches and delivery status</button>
    {error && <p role="alert">{error}</p>}
    {view?.items.filter(i => i.status !== "unchanged").map(item => <details key={item.reference} open={!!expanded[item.reference]} onToggle={event => { const open = event.currentTarget.open; setExpanded(current => current[item.reference] === open ? current : { ...current, [item.reference]: open }); }}>
      <summary>{item.path} · {item.status}</summary>
      <p>{item.role} · {item.scope} · last supplied in {item.request}</p>
      <label><input type="checkbox" disabled={busy || !item.eligible} checked={selected.includes(item.reference)} onChange={e => select(item, e.target.checked)} /> Select update</label>
      <UiActionsVisibility visible={open && !!expanded[item.reference]}><DiffView path={item.path} diff={unifiedDiff(item.path, item.before, item.after ?? "")} binary={false} staged={false} label="last supplied → candidate" /></UiActionsVisibility>
      {view.updates.filter(u => u.old === item.reference).map(u => <p key={u.id}>{u.state}{u.request ? ` in request ${u.request}` : ""}{u.source.revision !== item.revision ? " · changed again" : ""}</p>)}
    </details>)}
    <button className="btn" disabled={busy || !selected.length} onClick={() => void inspect(true)}>Queue selected updates</button>
    <button className="btn" disabled={busy || !view?.items.some(i => i.eligible)} onClick={selectAll}>Select all eligible</button>
    {view?.updates.map(u => <p key={u.id}>{u.state === "provider_confirmed" ? "Included" : u.state}{u.request ? ` in request ${u.request}` : ""}</p>)}
    <p className="muted">{view?.coverage}</p>
  </details>;
}
</storycode:copy>

### Assemble `apps/web/src/components/ContextUpdates.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ContextUpdates.tsx"><storycode:paste select="#f-83abe8d9f9e9-Item"/><storycode:paste select="#f-83abe8d9f9e9-View"/><storycode:paste select="#f-83abe8d9f9e9-ContextUpdateProps"/><storycode:paste select="#f-83abe8d9f9e9-ContextUpdates"/><storycode:paste select="#f-83abe8d9f9e9-OwnedContextUpdates"/></storycode:file>

</storycode:doc>
