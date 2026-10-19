import { useState, useEffect } from "react";
import { request } from "../api/client";
import type { AgentEditorContext } from "../api/agentTypes";
import { unifiedDiff } from "../lib/diff";
import { DiffView } from "./DiffView";

type Item = { reference: string; path: string; before: string; after: string | null; status: string; revision: string; eligible: boolean; role: string; scope: string; request: string };
type View = { items: Item[]; updates: { id: string; old: string; state: string; request: string | null; source: { revision: string } }[]; coverage: string };
export function ContextUpdates({ doc, branch, getContext }: { doc: string; branch: string | null; getContext?: () => Promise<AgentEditorContext> }) {
  const [view, setView] = useState<View | null>(null);
  const [selected, setSelected] = useState<string[]>([]);
  const [selectedRevisions, setSelectedRevisions] = useState<Record<string, string>>({});
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  async function inspect(queue = false) {
    if (!branch) return;
    setBusy(true); setError("");
    try {
      const context = getContext ? await getContext() : { buffers: [] };
      setView(await request<View>("POST", `/api/docs/${encodeURIComponent(doc)}/agent/python/context`, {
        branch, context, selected: queue ? selected : [],
        revisions: queue ? selectedRevisions : {},
      }));
      if (queue) setSelected([]);
    } catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  useEffect(() => {
    let active = true;
    async function refresh() {
      if (!branch) return;
      try {
        const context = getContext ? await getContext() : { buffers: [] };
        const result = await request<View>("POST", `/api/docs/${encodeURIComponent(doc)}/agent/python/context`, { branch, context });
        if (active) setView(result);
      } catch (e) { if (active) setError(String(e)); }
    }
    void refresh();
    const timer = setInterval(() => void refresh(), 5000);
    return () => { active = false; clearInterval(timer); };
  }, [doc, branch, getContext]);
  const changed = view?.items.filter(i => i.status !== "unchanged") ?? [];
  return <details onToggle={e => { if (e.currentTarget.open) void inspect(); }}>
    <summary>Changed agent context ({changed.length})</summary>
    <p>Compare with confirmed delivery on this branch. Updates enter a subsequent model boundary. Unsaved text is never saved by this action.</p>
    <button className="btn" disabled={busy || !branch} onClick={() => void inspect()}>Refresh patches and delivery status</button>
    {error && <p role="alert">{error}</p>}
    {view?.items.filter(i => i.status !== "unchanged").map(item => <details key={item.reference}>
      <summary>{item.path} · {item.status}</summary>
      <p>{item.role} · {item.scope} · last supplied in {item.request}</p>
      <label><input type="checkbox" disabled={!item.eligible} checked={selected.includes(item.reference)} onChange={e => { setSelected(s => e.target.checked ? [...s, item.reference] : s.filter(id => id !== item.reference)); setSelectedRevisions(s => ({ ...s, [item.reference]: item.revision })); }} /> Select update</label>
      <DiffView path={item.path} diff={unifiedDiff(item.path, item.before, item.after ?? "")} binary={false} staged={false} label="last supplied → candidate" />
      {view.updates.filter(u => u.old === item.reference).map(u => <p key={u.id}>{u.state}{u.request ? ` in request ${u.request}` : ""}{u.source.revision !== item.revision ? " · changed again" : ""}</p>)}
    </details>)}
    <button className="btn" disabled={busy || !selected.length} onClick={() => void inspect(true)}>Queue selected updates</button>
    <button className="btn" disabled={busy || !view?.items.some(i => i.eligible)} onClick={() => { setSelected(view?.items.filter(i => i.eligible).map(i => i.reference) ?? []); setSelectedRevisions(Object.fromEntries((view?.items.filter(i => i.eligible) ?? []).map(i => [i.reference, i.revision]))); }}>Select all eligible</button>
    {view?.updates.map(u => <p key={u.id}>{u.state === "provider_confirmed" ? "Included" : u.state}{u.request ? ` in request ${u.request}` : ""}</p>)}
    <p className="muted">{view?.coverage}</p>
  </details>;
}
