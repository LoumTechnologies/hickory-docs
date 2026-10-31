import { useState, useEffect, useId, useRef } from "react";
import { createPortal } from "react-dom";
import { UiActionsVisibility, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiField } from "../lib/uiActions";
import { request } from "../api/client";
import type { AgentEditorContext } from "../api/agentTypes";
import { unifiedDiff } from "../lib/diff";
import { DiffView } from "./DiffView";

type Item = { reference: string; path: string; before: string; after: string | null; status: string; revision: string; eligible: boolean; role: string; scope: string; request: string };
type View = { items: Item[]; updates: { id: string; old: string; state: string; request: string | null; source: { revision: string } }[]; coverage: string };
type ContextUpdateProps = { doc: string; branch: string | null; getContext?: () => Promise<AgentEditorContext> };
export function ContextUpdates(props: ContextUpdateProps) {
  return <OwnedContextUpdates key={JSON.stringify([props.doc, props.branch])} {...props} />;
}
function OwnedContextUpdates({ doc, branch, getContext }: ContextUpdateProps) {
  const captureContext = useRef(getContext); captureContext.current = getContext;
  const instance = useId(); const parent = `context-updates:${instance}`;
  const disclosure = useRef<HTMLButtonElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({ right: 8, top: 48 });
  const [open, setOpen] = useState(false);
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  useEffect(() => {
    if (!open) return;
    const dismiss = (event: PointerEvent) => {
      if (!panel.current?.contains(event.target as Node) && !disclosure.current?.contains(event.target as Node)) setOpen(false);
    };
    const close = () => setOpen(false);
    document.addEventListener("pointerdown", dismiss);
    window.addEventListener("resize", close);
    return () => { document.removeEventListener("pointerdown", dismiss); window.removeEventListener("resize", close); };
  }, [open]);
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
  function showContext(visible: boolean) {
    const rect = disclosure.current?.getBoundingClientRect();
    if (rect) setPosition({ right: Math.max(8, Math.min(window.innerWidth - rect.right, window.innerWidth - Math.min(448, window.innerWidth * .9) - 8)), top: Math.max(8, Math.min(rect.bottom, window.innerHeight - 160)) });
    setOpen(visible);
    if (visible) void inspect();
  }
  if (changed.length || error || open) add("disclosure", "Set changed agent context visibility", { open: { type: "boolean" } }, async args => { if (!disclosure.current) return { status: "unavailable", reason: "Context disclosure is not mounted" }; showContext(Boolean(args.open)); return { status: "completed", value: { open: Boolean(args.open) } }; }, true, "agent-context-disclosures");
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
  if (!changed.length && !error && !open) return null;
  return <div className="agent-context-gutter">
    <button ref={disclosure} className="btn-link" aria-label={error ? "Context comparison unavailable" : `Changed agent context (${changed.length})`} aria-expanded={open} aria-controls={`${instance}-context`} title={error || `${changed.length} context references changed`} onClick={() => showContext(!open)}>↻{changed.length || "!"}</button>
    {open && createPortal(<div ref={panel} id={`${instance}-context`} className="agent-context-panel" style={position} role="region" aria-label="Changed agent context" onKeyDown={e => { if (e.key === "Escape") { e.stopPropagation(); setOpen(false); disclosure.current?.focus(); } }}>
    <button className="btn-link" aria-label="Close changed context" onClick={() => { setOpen(false); disclosure.current?.focus(); }}>Close</button>
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
    </div>, document.body)}
  </div>;
}
