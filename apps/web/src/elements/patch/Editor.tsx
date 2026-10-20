import { useEffect, useId, useMemo, useRef, useState, type ReactNode } from "react";
import { useUiCommands, useUiRevision } from "../../lib/uiActionBridge";
import { useUiReading } from "../../lib/uiReading";
import { request } from "../../api/client";
import { ComparisonEditor } from "../../components/ComparisonEditor";
import { DiffView } from "../../components/DiffView";
import { parseHickDoc } from "../../editor/hickDoc";
import { materializeProject, regenerateProjectPatch, reverseProjectEdit, type EvaluatedProject } from "../../editor/hickLang";
import { showBlockSource, type RenderedSlot } from "../../editor/rendered";
import type { ComparisonModel } from "../../lib/projectComparison";
import type { SlotContext } from "../types";

function patchPaths(state: EvaluatedProject["local_states"][string]) {
  const all = [...new Set([...state.old, ...state.new].map(entry => entry.path))];
  const changed = all.filter(path => {
    const before = state.old.find(entry => entry.path === path), after = state.new.find(entry => entry.path === path);
    return !before || !after || before.kind !== after.kind || before.executable !== after.executable || before.bytes.join(",") !== after.bytes.join(",");
  });
  return changed.length ? changed : all;
}

type Props = { slot: RenderedSlot; cx: SlotContext };
export function PatchEditor(props: Props) {
  const { cx } = props;
  const owner = useMemo(() => crypto.randomUUID(), [cx.view, cx.docId, cx.path, cx.capturedProject?.entry, cx.capturedProject?.document]);
  return <OwnedPatchEditor key={owner} {...props} />;
}
function OwnedPatchEditor({ slot, cx }: Props) {
  const callbacks = useMemo(() => crypto.randomUUID(), [cx.capturedProject?.editPatch, cx.capturedProject?.draftPending]);
  const currentCallbacks = useRef(callbacks); currentCallbacks.current = callbacks;
  const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const contextRevision = JSON.stringify([cx.capturedProject?.entry, cx.capturedProject?.sources, cx.capturedProject?.input]);
  const [capture, setCapture] = useState<{ contextRevision: string; entry: string; sources: Record<string, string>; source: string; project: EvaluatedProject; id: string } | null>(null);
  const [path, setPath] = useState("");
  const [side, setSide] = useState<"old" | "new">("new");
  const [draft, setDraft] = useState<ComparisonModel | null>(null);
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(true);
  const draftAssisted = useRef(false);
  const latestDraft = useRef(draft); latestDraft.current = draft;
  const loadGeneration = useRef(0);
  const inspect = async () => {
    const generation = ++loadGeneration.current;
    setError(""); setLoading(true);
    try {
      if (!cx.view) throw new Error("Open the source editor to inspect this patch.");
      const source = cx.view.state.doc.toString();
      const context = cx.capturedProject ? { entry: cx.capturedProject.entry, sources: { ...cx.capturedProject.sources, [cx.capturedProject.document ?? cx.capturedProject.entry]: source } } : cx.docId && cx.docId !== "untitled"
        ? await request<{ entry: string; sources: Record<string, string> }>("POST", `/api/docs/${encodeURIComponent(cx.docId)}/project`, { source })
        : { entry: "project.md", sources: { "project.md": source } };
      const project = materializeProject(context.entry, context.sources, cx.capturedProject?.input);
      const id = parseHickDoc(source).blocks.find(b => b.name === "patch" && b.from === slot.at)?.attrs.id;
      if (!id || !project.local_states[id]) throw new Error("Give this patch an explicit id before editing its constructed code.");
      const state = project.local_states[id];
      const paths = patchPaths(state);
      if (generation !== loadGeneration.current) return;
      setCapture({ ...context, source, project, id, contextRevision }); setPath(previous => paths.includes(previous) ? previous : paths[0] ?? ""); setDraft(null); draftAssisted.current = false;
    } catch (e) { if (generation === loadGeneration.current) setError(e instanceof Error ? e.message : String(e)); }
    finally { if (generation === loadGeneration.current) setLoading(false); }
  };
  useEffect(() => { void inspect(); return () => { loadGeneration.current++; }; }, [cx.view, cx.docId, cx.capturedProject?.sources, cx.capturedProject?.input]);
  useEffect(() => {
    if (capture && !draft && cx.view?.state.doc.toString() !== capture.source) void inspect();
  }, [slot.text]);
  const save = async () => {
    setError("");
    setSaving(true);
    try {
      if (!capture || !draft || !cx.view) return;
      if (contextRevision !== capture.contextRevision) throw Error("The captured project changed. Reload this patch before saving its draft.");
      if (cx.view.state.readOnly && !cx.capturedProject?.editPatch) throw new Error("This source is read-only. Fork it before editing.");
      if (cx.view.state.doc.toString() !== capture.source) throw new Error("The document changed elsewhere. This draft is unsaved; reload the patch before editing further.");
      const state = capture.project.local_states[capture.id];
      const text = draft.sources.find(s => s.id === draft[side])!.text;
      const newEntries = state.new.map(e => e.path === path ? { ...e, bytes: [...new TextEncoder().encode(text)], origins: [] } : e);
      if (side === "new" && !newEntries.some(e => e.path === path)) throw new Error("This path is removed by the patch. Edit the patch source to restore it.");
      const input = cx.capturedProject?.input ?? [];
      let sources: Record<string, string>;
      if (side === "old") {
        const file = state.old.find(e => e.path === path);
        if (!file) throw new Error("There is no authored Old construction for this path.");
        const oldText = new TextDecoder("utf-8", { fatal: true }).decode(new Uint8Array(file.bytes));
        // Minimize in Unicode codepoints so an edit never splits a surrogate.
        const a = [...oldText], b = [...text]; let first = 0, suffix = 0;
        while (first < a.length && first < b.length && a[first] === b[first]) first++;
        while (suffix < a.length-first && suffix < b.length-first && a[a.length-1-suffix] === b[b.length-1-suffix]) suffix++;
        const bytes = (s: string) => new TextEncoder().encode(s).length;
        sources = reverseProjectEdit(capture.entry, capture.sources, input, state.old, { path,
          range: [bytes(a.slice(0, first).join("")), bytes(a.slice(0, a.length-suffix).join(""))], replacement: b.slice(first,b.length-suffix).join("") });
      } else sources = regenerateProjectPatch(capture.entry, capture.sources, input, capture.id, newEntries);
      const replacement = sources[cx.capturedProject?.document ?? capture.entry];
      const project = materializeProject(capture.entry, sources, input);
      if (cx.capturedProject?.editPatch) {
        cx.capturedProject.editPatch(sources);
        cx.capturedProject.draftPending?.(capture.id, false);
      }
      if (!cx.capturedProject?.editPatch && Object.keys(sources).some(p => p !== capture.entry && sources[p] !== capture.sources[p])) {
        if (!cx.docId) throw new Error("Open the project in its native workspace to publish contributing source edits.");
        const completed = materializeProject(capture.entry,sources).new;
        const entries=Object.fromEntries(completed.map(e=>[e.path,{bytes:e.bytes,kind:e.kind === "file" ? "File" : "Symlink",executable:e.executable,origins:e.origins ?? []}]));
        await request("POST",`/api/docs/${encodeURIComponent(cx.docId)}/project/apply`,{baseline:capture.sources,documents:sources,expected:{entries}});
      }
      if (!mounted.current || currentCallbacks.current !== callbacks) return;
      if (cx.view.state.doc.toString() !== capture.source) throw new Error("The document changed during saving. This draft is unsaved; reload the patch.");
      // Change only the differing source span so surrounding selections and
      // rendered editors retain their positions, focus and undo history.
      let from = 0, suffix = 0;
      while (from < capture.source.length && from < replacement.length && capture.source[from] === replacement[from]) from++;
      if (from > 0 && /[\uD800-\uDBFF]/.test(capture.source[from - 1])) from--;
      while (suffix < capture.source.length - from && suffix < replacement.length - from && capture.source[capture.source.length - suffix - 1] === replacement[replacement.length - suffix - 1]) suffix++;
      if (suffix > 0 && /[\uDC00-\uDFFF]/.test(capture.source[capture.source.length - suffix])) suffix--;
      setCapture({ ...capture, sources, source: replacement, project });
      cx.view.dispatch({ changes: { from, to: capture.source.length - suffix, insert: replacement.slice(from, replacement.length - suffix) }, ...(!cx.capturedProject?.editPatch ? {userEvent: draftAssisted.current ? "input.uiAgentProjectPatch" : "input.project-patch"} : {}) });
      if (latestDraft.current === draft) { setDraft(null); draftAssisted.current = false; }
    } catch (e) { if (mounted.current) setError(e instanceof Error ? e.message : String(e)); }
    finally { if (mounted.current) setSaving(false); }
  };
  useEffect(() => {
    if (!draft || saving || error || !cx.view || (cx.view.state.readOnly && !cx.capturedProject?.editPatch)) return;
    const timer = window.setTimeout(() => void save(), 400);
    return () => window.clearTimeout(timer);
  }, [draft, saving, capture, error]);
  let model: ComparisonModel | null = null;
  if (capture && path) {
    const state = capture.project.local_states[capture.id];
    const decode = (entries: typeof state.old) => {
      const entry = entries.find(e => e.path === path);
      if (!entry) return "";
      if (entry.kind !== "file") throw new Error("Symlink entries have no editable code pane.");
      return new TextDecoder("utf-8", { fatal: true }).decode(new Uint8Array(entry.bytes));
    };
    try {
      const filesystemOnly = cx.capturedProject?.filesystemOnly;
      const editorAddress = (path: string) => path.startsWith("__hickory_editor_buffers__/");
      const tools = cx.capturedProject?.tools !== false && (!filesystemOnly || !editorAddress(path));
      // Editor addresses stay in replay. Native tools receive only the complete
      // captured filesystem; a buffer is never disguised as a disk file.
      const nativeTree = (entries: typeof state.old) => filesystemOnly ? entries.filter(e => !editorAddress(e.path)) : entries;
      model = draft ?? { sources: [
        { id: "old", label: "Before this patch", kind: "authored", target: capture.entry, text: decode(state.old), project: tools ? nativeTree(capture.project.old) : undefined, versions: tools ? capture.project.versions.map(nativeTree) : undefined, local: nativeTree(state.old), debugSurface: cx.view && tools && !filesystemOnly ? { view: cx.view, entry: cx.capturedProject?.document ?? capture.entry, source: capture.source, path: cx.path ?? undefined } : undefined },
        { id: "new", label: "After this patch", kind: "authored", target: capture.entry, text: decode(state.new), project: tools ? nativeTree(capture.project.new) : undefined, versions: tools ? capture.project.versions.map(nativeTree) : undefined, local: nativeTree(state.new), debugSurface: cx.view && tools && !filesystemOnly ? { view: cx.view, entry: cx.capturedProject?.document ?? capture.entry, source: capture.source, path: cx.path ?? undefined } : undefined },
      ], old: "old", new: "new", policy: cx.capturedProject?.editPatch ? cx.capturedProject.activePatch && cx.capturedProject.activePatch !== capture.id ? "neither" : "new" : side, evidence: "unverified" };
    } catch { /* The explicit diagnostic below leaves binary source intact. */ }
  }
  const parent = `patch-editor:${useId()}`;
  const revision = useUiRevision(() => [callbacks, contextRevision, cx.docId, cx.path, slot.at, slot.text, capture?.source, path, side, draft, saving, loading, error, cx.view?.state.doc.toString(), cx.view?.state.readOnly, cx.capturedProject?.activePatch]);
  const selectSide = (next: "old" | "new") => { setSide(next); setDraft(null); };
  const selectPath = (next: string) => { setPath(next); setDraft(null); };
  const editSource = () => { if (cx.view && !cx.view.state.readOnly) cx.view.dispatch({ effects: showBlockSource.of(slot.at) }); };
  const reload = async () => { cx.capturedProject?.draftPending?.(capture?.id ?? "", false); await inspect(); };
  useUiReading("project patch editor status", { document: cx.path ?? null, patch: capture?.id ?? null, path, side, loading, saving, error: error || null,
    unsaved_draft: !!draft, readonly: !!cx.view?.state.readOnly && !cx.capturedProject?.editPatch,
    scope: "Current patch editor controls. Comparison panes own source reading/editing. Edits automatically check downstream patches; up-to-date does not attest execution." });
  useUiCommands([
    ...(!loading && !capture && cx.view && !cx.view.state.readOnly ? [{ key: `${parent}:source`, command: { label: "Edit patch source", parent, subject: parent, parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true }), execute: async () => { editSource(); return { status: "completed" as const, value: { source_view_requested: true, source_changed: false } }; } } }] : []),
    ...(error ? [{ key: `${parent}:reload`, command: { label: `Reload patch${draft ? " and discard draft" : ""}`, parent, subject: parent, parameters: {}, prerequisites: ["Discards this patch editor's unsaved comparison draft"],
      read: () => ({ revision: revision(), enabled: !saving && !loading }), execute: async () => { await reload(); return { status: "completed" as const, value: { inspection_requested: true, source_written: false, scope: "Inspect current patch status for the actual read outcome" } }; } } }] : []),
  ]);
  return <section aria-label="Project patch editor">
    {loading && <p className="muted">Loading patch code…</p>}
    {!loading && !capture && <>
      <DiffView path="Project patch" diff={slot.text.replace(/^\r?\n/, "")} binary={false} staged={false} label="New · completed project; Old · scope input" />
      {cx.view && !cx.view.state.readOnly && <button className="btn" onClick={editSource}>Edit patch source</button>}
    </>}
    {error && <p role="alert">{error}</p>}
    {capture && <>
      {model ? <ComparisonEditor path={path} model={model} compact onChange={loading || contextRevision !== capture.contextRevision || (cx.view?.state.readOnly && !cx.capturedProject?.editPatch) ? undefined : (value, assisted) => { draftAssisted.current ||= !!assisted; setError(""); cx.capturedProject?.draftPending?.(capture.id, true); setDraft(value); }} options={<PatchOptions paths={patchPaths(capture.project.local_states[capture.id])} path={path} side={side} disabled={!!draft || saving || loading || contextRevision !== capture.contextRevision} allowSide={!cx.capturedProject?.editPatch} onPath={selectPath} onSide={selectSide} revision={revision}>
      <p>{path} · {cx.view?.state.readOnly && !cx.capturedProject?.editPatch ? "read-only" : saving ? "Checking…" : draft ? "Unsaved draft" : "Up to date"}</p>
      {!cx.capturedProject?.editPatch && <label>Editable side <select disabled={!!draft || saving || loading || contextRevision !== capture.contextRevision} value={side} onChange={e => selectSide(e.target.value as "old" | "new")}><option value="new">New</option><option value="old">Old · authored predecessor</option></select></label>}
      <label>File <select disabled={!!draft || saving || loading || contextRevision !== capture.contextRevision} value={path} onChange={e => selectPath(e.target.value)}>
        {patchPaths(capture.project.local_states[capture.id]).map(p => <option key={p}>{p}</option>)}
      </select></label>
      <p className="muted">Edits save automatically after checking later patches. Debugging uses the latest project containing exactly the code shown.</p>
      </PatchOptions>} /> : <p role="alert">This entry is binary or a symlink. Edit its explicit entry construction.</p>}
    </>}
    {error && <button className="btn" onClick={() => void reload()} disabled={saving || loading}>Reload patch{draft ? " and discard draft" : ""}</button>}
  </section>;
}

/** Mounted inside the comparison options disclosure, so hidden choices retire. */
function PatchOptions({ paths, path, side, disabled, allowSide, onPath, onSide, revision, children }: {
  paths: string[]; path: string; side: "old" | "new"; disabled: boolean; allowSide: boolean;
  onPath: (path: string) => void; onSide: (side: "old" | "new") => void; revision: () => string; children: ReactNode;
}) {
  const parent = `patch-options:${useId()}`;
  useUiReading("constructed patch choices", { paths, path, side, disabled, allow_side_selection: allowSide });
  useUiCommands([
    { key: `${parent}:path`, command: { label: "Select constructed patch file", parent, subject: parent,
      parameters: { path: { type: "string", maxLength: 8192, choices: paths } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !disabled }), execute: async args => { onPath(args.path as string); return { status: "completed", value: { path_selected: args.path } }; } } },
    ...(allowSide ? [{ key: `${parent}:side`, command: { label: "Select constructed patch editable side", parent, subject: parent,
      parameters: { side: { type: "string" as const, maxLength: 3, choices: ["old", "new"] } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !disabled }), execute: async (args: Record<string, import("../../lib/uiActions").UiValue>) => { onSide(args.side as "old" | "new"); return { status: "completed" as const, value: { side_selected: args.side } }; } } }] : []),
  ]);
  return <>{children}</>;
}
