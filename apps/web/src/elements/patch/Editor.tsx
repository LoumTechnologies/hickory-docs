import { useEffect, useRef, useState } from "react";
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

export function PatchEditor({ slot, cx }: { slot: RenderedSlot; cx: SlotContext }) {
  const [capture, setCapture] = useState<{ entry: string; sources: Record<string, string>; source: string; project: EvaluatedProject; id: string } | null>(null);
  const [path, setPath] = useState("");
  const [side, setSide] = useState<"old" | "new">("new");
  const [draft, setDraft] = useState<ComparisonModel | null>(null);
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(true);
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
      setCapture({ ...context, source, project, id }); setPath(paths[0] ?? ""); setDraft(null);
    } catch (e) { if (generation === loadGeneration.current) setError(e instanceof Error ? e.message : String(e)); }
    finally { if (generation === loadGeneration.current) setLoading(false); }
  };
  useEffect(() => { void inspect(); return () => { loadGeneration.current++; }; }, [cx.view, cx.docId, cx.capturedProject]);
  useEffect(() => {
    if (capture && !draft && cx.view?.state.doc.toString() !== capture.source) void inspect();
  }, [slot.text]);
  const save = async () => {
    setError("");
    setSaving(true);
    try {
      if (!capture || !draft || !cx.view) return;
      if (cx.view.state.readOnly) throw new Error("This source is read-only. Fork it before editing.");
      if (cx.view.state.doc.toString() !== capture.source) throw new Error("The document changed elsewhere. This draft is unsaved; reload the patch before editing further.");
      const state = capture.project.local_states[capture.id];
      const text = draft.sources.find(s => s.id === draft[side])!.text;
      const newEntries = state.new.map(e => e.path === path ? { ...e, bytes: [...new TextEncoder().encode(text)], origins: [] } : e);
      if (side === "new" && !newEntries.some(e => e.path === path)) throw new Error("This path is removed by the patch. Edit the patch source to restore it.");
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
        sources = reverseProjectEdit(capture.entry, capture.sources, [], state.old, { path,
          range: [bytes(a.slice(0, first).join("")), bytes(a.slice(0, a.length-suffix).join(""))], replacement: b.slice(first,b.length-suffix).join("") });
      } else sources = regenerateProjectPatch(capture.entry, capture.sources, [], capture.id, newEntries);
      const replacement = sources[capture.entry];
      const project = materializeProject(capture.entry, sources);
      if (Object.keys(sources).some(p => p !== capture.entry && sources[p] !== capture.sources[p])) {
        if (!cx.docId) throw new Error("Open the project in its native workspace to publish contributing source edits.");
        const completed = materializeProject(capture.entry,sources).new;
        const entries=Object.fromEntries(completed.map(e=>[e.path,{bytes:e.bytes,kind:e.kind === "file" ? "File" : "Symlink",executable:e.executable,origins:e.origins ?? []}]));
        await request("POST",`/api/docs/${encodeURIComponent(cx.docId)}/project/apply`,{baseline:capture.sources,documents:sources,expected:{entries}});
      }
      if (cx.view.state.doc.toString() !== capture.source) throw new Error("The document changed during saving. This draft is unsaved; reload the patch.");
      // Change only the differing source span so surrounding selections and
      // rendered editors retain their positions, focus and undo history.
      let from = 0, suffix = 0;
      while (from < capture.source.length && from < replacement.length && capture.source[from] === replacement[from]) from++;
      if (from > 0 && /[\uD800-\uDBFF]/.test(capture.source[from - 1])) from--;
      while (suffix < capture.source.length - from && suffix < replacement.length - from && capture.source[capture.source.length - suffix - 1] === replacement[replacement.length - suffix - 1]) suffix++;
      if (suffix > 0 && /[\uDC00-\uDFFF]/.test(capture.source[capture.source.length - suffix])) suffix--;
      setCapture({ ...capture, sources, source: replacement, project });
      cx.view.dispatch({ changes: { from, to: capture.source.length - suffix, insert: replacement.slice(from, replacement.length - suffix) }, userEvent: "input.project-patch" });
      if (latestDraft.current === draft) setDraft(null);
    } catch (e) { setError(e instanceof Error ? e.message : String(e)); }
    finally { setSaving(false); }
  };
  useEffect(() => {
    if (!draft || saving || error || !cx.view || cx.view.state.readOnly) return;
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
      ], old: "old", new: "new", policy: side, evidence: "unverified" };
    } catch { /* The explicit diagnostic below leaves binary source intact. */ }
  }
  return <section aria-label="Project patch editor">
    {loading && <p className="muted">Loading patch code…</p>}
    {!loading && !capture && <>
      <DiffView path="Project patch" diff={slot.text.replace(/^\r?\n/, "")} binary={false} staged={false} label="New · completed project; Old · scope input" />
      {cx.view && !cx.view.state.readOnly && <button className="btn" onClick={() => cx.view?.dispatch({ effects: showBlockSource.of(slot.at) })}>Edit patch source</button>}
    </>}
    {error && <p role="alert">{error}</p>}
    {capture && <>
      {model ? <ComparisonEditor path={path} model={model} compact onChange={cx.view?.state.readOnly ? undefined : value => { setError(""); setDraft(value); }} options={<>
      <p>{path} · {cx.view?.state.readOnly ? "read-only" : saving ? "Checking…" : draft ? "Unsaved draft" : "Up to date"}</p>
      <label>Editable side <select disabled={!!draft || saving} value={side} onChange={e => { setSide(e.target.value as "old" | "new"); setDraft(null); }}><option value="new">New</option><option value="old">Old · authored predecessor</option></select></label>
      <label>File <select disabled={!!draft || saving} value={path} onChange={e => { setPath(e.target.value); setDraft(null); }}>
        {patchPaths(capture.project.local_states[capture.id]).map(p => <option key={p}>{p}</option>)}
      </select></label>
      <p className="muted">Edits save automatically after checking later patches. Debugging uses the latest project containing exactly the code shown.</p>
      </>} /> : <p role="alert">This entry is binary or a symlink. Edit its explicit entry construction.</p>}
    </>}
    {error && <button className="btn" onClick={() => void inspect()} disabled={saving}>Reload patch{draft ? " and discard draft" : ""}</button>}
  </section>;
}
