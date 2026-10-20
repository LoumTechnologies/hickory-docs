import { useEffect, useId, useRef, useState } from "react";
import { representations, type Representation } from "../api/representations";
import { showRepresentation } from "../lib/openRepresentation";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
const message = (e: unknown) => e instanceof Error ? e.message : String(e);

export function RepresentationLibrary() {
  const [paths, setPaths] = useState("");
  const [views, setViews] = useState<Representation[]>([]);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const pending = useRef(false), mounted = useRef(true);
  const input = useRef(paths); input.current = paths;
  const parent = `literate-library:${useId()}`;
  const revision = useUiRevision(() => [input.current, views.map(v => [v.id, v.revision]), busy, error]);
  const perform = async (action: () => Promise<UiOutcome>): Promise<UiOutcome> => {
    if (pending.current) return { status: "unavailable", reason: "Literate view library request already pending" };
    pending.current = true; setBusy(true); setError("");
    try { return await action(); }
    catch (e) { const reason = message(e); if (mounted.current) setError(reason); return { status: "unavailable", reason }; }
    finally { pending.current = false; if (mounted.current) setBusy(false); }
  };
  const refresh = () => perform(async () => {
    const reply = await representations.list();
    if (mounted.current) setViews(reply.views);
    return { status: "completed", value: { count: reply.views.length, scope: "Loaded view catalogue; source contents require opening a view" } };
  });
  useEffect(() => { mounted.current = true; void refresh(); return () => { mounted.current = false; }; }, []); // initial catalogue only
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], enabled: boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { parent, subject: parent, label, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled, reason: enabled ? undefined : "Enter source paths and wait for pending library requests" }), execute } });
  const setInput = (value: string) => { input.current = value; setPaths(value); };
  add("paths", "Set literate library source paths", { value: { type: "string", maxLength: 65536 } }, !busy, async args => { setInput(args.value as string); return { status: "completed", value: { paths_requested: args.value } }; });
  add("refresh", "Refresh literate view catalogue", {}, !busy, refresh);
  add("create", "Create and open literate view from source paths", {}, !!paths.trim() && !busy, () => perform(async () => {
    const reply = await representations.create({ kind: "files", paths: input.current.split("\n").map(p => p.trim()).filter(Boolean) });
    if (mounted.current) { setViews(current => [...current.filter(v => v.id !== reply.id), reply]); showRepresentation(reply.id); }
    return { status: "completed", value: { created_view: reply.id, revision: reply.revision, navigation_requested: mounted.current, verified: false } };
  }));
  const invoke = useUiCommands(commands);
  const human = (key: string) => void invoke(`${parent}:${key}`).catch(e => setError(message(e)));
  useUiReading("literate view catalogue", { source_paths: paths, pending: busy, error, views: views.map(v => ({ id: v.id, revision: v.revision, backing: v.backing, files: v.files.map(f => f.path), explanation_stale: v.explanation_stale })), scope: "Loaded catalogue and input paths only; source and file contents omitted" });
  return <section className="representation-library"><h2>Literate views</h2>
    <p>Keep ordinary source files in git. Use a personal literate arrangement to read and edit them.</p>
    <form onSubmit={e => { e.preventDefault(); human("create"); }}>
      <label>Source files, one relative path per line<textarea aria-label="Literate source files" rows={4} value={paths} disabled={busy} onChange={e => setInput(e.target.value)} /></label>
      <button className="btn btn-primary" disabled={!paths.trim() || busy}>Open literate view</button>
    </form><button className="btn" disabled={busy} onClick={() => human("refresh")}>Refresh views</button>
    {error && <p role="alert">{error}</p>}
    <ul>{views.map(view => <ViewEntry key={view.id} view={view} busy={busy} discard={() => perform(async () => {
      const reply = await representations.discard(view.id);
      if (mounted.current) setViews(current => current.filter(v => v.id !== view.id));
      return { status: "completed", value: { discarded_view: view.id, host_reply: JSON.stringify(reply), scope: "Local arrangement discarded; source files are not deleted" } };
    })} onError={setError} />)}</ul>
  </section>;
}
function ViewEntry({ view, busy, discard, onError }: { view: Representation; busy: boolean; discard: () => Promise<UiOutcome>; onError: (reason: string) => void }) {
  const parent = `literate-entry:${useId()}`;
  const revision = useUiRevision(() => [view.id, view.revision, busy]);
  const make = (key: string, label: string, execute: UiCommand["execute"]) => ({ key: `${parent}:${key}`, command: { parent, subject: parent, label, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !busy }), execute } });
  const invoke = useUiCommands([
    make("open", `Open literate view ${view.id}`, async () => { showRepresentation(view.id); return { status: "completed", value: { navigation_requested: view.id } }; }),
    make("discard", `Discard local literate view ${view.id}`, discard),
  ]);
  const human = (key: string) => void invoke(`${parent}:${key}`).catch(e => onError(message(e)));
  return <li><button className="btn-link" disabled={busy} onClick={() => human("open")}>{view.files.map(f => f.path).join(", ")}</button> <button className="btn" disabled={busy} onClick={() => human("discard")}>Discard view</button></li>;
}
