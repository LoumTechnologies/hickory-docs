<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/elements/patch/Editor.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-dc5641c063e3"></a>
## `apps/web/src/elements/patch/Editor.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Finish an agent turn against its captured project boundary](../../../../../../guarantees/agent/changeset-completion-is-negotiated-and-checked/contract.md), [Captured reading surfaces share owned native actions](../../../../../../guarantees/agent/scoped-reading-surfaces-share-native-actions/contract.md), [Debug the latest atomic project that still contains the displayed bytes](../../../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md), [Keep a patch edit as a draft until the whole construction still works](../../../../../../guarantees/literate-projects/patch-edits-revalidate-downstream-construction/contract.md).

<a id="f-dc5641c063e3-patchPaths"></a>
### `patchPaths`

<storycode:copy id="f-dc5641c063e3-patchPaths">
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

</storycode:copy>
<a id="f-dc5641c063e3-Props"></a>
### `Props`

<storycode:copy id="f-dc5641c063e3-Props">
type Props = { slot: RenderedSlot; cx: SlotContext };
</storycode:copy>
<a id="f-dc5641c063e3-PatchEditor"></a>
### `PatchEditor`

<storycode:copy id="f-dc5641c063e3-PatchEditor">
export function PatchEditor(props: Props) {
  const { cx } = props;
  const owner = useMemo(() => crypto.randomUUID(), [cx.view, cx.docId, cx.path, cx.capturedProject?.entry, cx.capturedProject?.document]);
  return <OwnedPatchEditor key={owner} {...props} />;
}
</storycode:copy>
<a id="f-dc5641c063e3-OwnedPatchEditor"></a>
### `OwnedPatchEditor`

<storycode:copy id="f-dc5641c063e3-OwnedPatchEditor">
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

</storycode:copy>
<a id="f-dc5641c063e3-PatchOptions"></a>
### `PatchOptions`

Mounted inside the comparison options disclosure, so hidden choices retire.

<storycode:copy id="f-dc5641c063e3-PatchOptions">
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
</storycode:copy>

### Assemble `apps/web/src/elements/patch/Editor.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/patch/Editor.tsx"><storycode:paste select="#f-dc5641c063e3-patchPaths"/><storycode:paste select="#f-dc5641c063e3-Props"/><storycode:paste select="#f-dc5641c063e3-PatchEditor"/><storycode:paste select="#f-dc5641c063e3-OwnedPatchEditor"/><storycode:paste select="#f-dc5641c063e3-PatchOptions"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-dc4ea70ab787"></a>
## `apps/web/src/elements/patch/Editor.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Captured reading surfaces share owned native actions](../../../../../../guarantees/agent/scoped-reading-surfaces-share-native-actions/contract.md), [Keep a patch edit as a draft until the whole construction still works](../../../../../../guarantees/literate-projects/patch-edits-revalidate-downstream-construction/contract.md).

<a id="f-dc4ea70ab787-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-dc4ea70ab787-expression_statement">
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EditorState, StateEffect } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterEach, expect, it, vi } from "vitest";
import { request } from "../../api/client";
import { parseHickDoc } from "../../editor/hickDoc";
import { materializeProject } from "../../editor/hickLang";
import type { RenderedSlot } from "../../editor/rendered";
import type { SlotContext } from "../types";
import { UiActions } from "../../lib/uiActions";
import { UiActionContext } from "../../lib/uiActionBridge";
import { PatchEditor } from "./Editor";

vi.mock("../../api/client", () => ({ request: vi.fn() }));
</storycode:copy>
<a id="f-dc4ea70ab787-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-dc4ea70ab787-expression_statement-2">
vi.mock("../../components/ComparisonTools", () => ({ ComparisonTools: () => null }));
</storycode:copy>
<a id="f-dc4ea70ab787-views"></a>
### `views`

<storycode:copy id="f-dc4ea70ab787-views">
const views: EditorView[] = [];
</storycode:copy>
<a id="f-dc4ea70ab787-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-dc4ea70ab787-expression_statement-3">
afterEach(() => { cleanup(); views.splice(0).forEach(view => view.destroy()); vi.clearAllMocks(); });
</storycode:copy>
<a id="f-dc4ea70ab787-source"></a>
### `source`

<storycode:copy id="f-dc4ea70ab787-source">
const source = `<hick:project/><hick:file id="base" path="app.py">old\n</hick:file>
<hick:patch id="change" after="base">
--- a/app.py
+++ b/app.py
@@ -1 +1 @@
-old
+new
</hick:patch>`;
</storycode:copy>
<a id="f-dc4ea70ab787-mount"></a>
### `mount`

<storycode:copy id="f-dc4ea70ab787-mount">
function mount(text = source, readOnly = false, registry?: UiActions) {
  const view = new EditorView({ state: EditorState.create({ doc: text, extensions: [EditorState.readOnly.of(readOnly)] }) }); views.push(view);
  const block = parseHickDoc(text).blocks.find(block => block.name === "patch")!;
  const slot: RenderedSlot = { key: "patch-0", el: document.createElement("div"), kind: "patch", index: 0,
    at: block.from, span: [block.from, block.to], text: text.slice(block.contentFrom, block.contentTo), renderer: "", asserts: [] };
  const cx: SlotContext = { view, path: "project.md", docId: "untitled", execBlocks: [], diagramBlocks: [], runningCells: new Set(), replaying: [], replaceBlockContent: vi.fn() };
  const ui = render(registry ? <UiActionContext.Provider value={registry}><PatchEditor slot={slot} cx={cx} /></UiActionContext.Provider> : <PatchEditor slot={slot} cx={cx} />);
  const inner = () => EditorView.findFromDOM(ui.container.querySelector<HTMLElement>(".comparison-side:last-child .cm-editor")!)!;
  return { ui, view, inner, slot, cx };
}

</storycode:copy>
<a id="f-dc4ea70ab787-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-dc4ea70ab787-expression_statement-4">
// Guarantee: docs/guarantees/literate-projects/patch-edits-revalidate-downstream-construction.md
it("opens as a normal code editor and automatically regenerates the patch without losing selection", async () => {
  const { ui, view, inner } = mount();
  await waitFor(() => expect(ui.container.querySelector(".comparison-side .cm-editor")).not.toBeNull());
  expect(screen.queryByText("Inspect and edit patch")).toBeNull();
  const editor = inner();
  act(() => editor.dispatch({ changes: { from: 0, to: 3, insert: "héllo 😀" }, selection: { anchor: 8 } }));
  await waitFor(() => expect(view.state.doc.toString()).toContain("+héllo 😀"));
  expect(inner()).toBe(editor); expect(editor.state.selection.main.anchor).toBe(8);
  expect(new TextDecoder().decode(new Uint8Array(materializeProject("project.md", { "project.md": view.state.doc.toString() }).new[0].bytes))).toBe("héllo 😀\n");
  expect(screen.getByText("app.py · Up to date")).toBeTruthy(); expect(request).not.toHaveBeenCalled();
});

</storycode:copy>
<a id="f-dc4ea70ab787-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-dc4ea70ab787-expression_statement-5">
it("keeps a downstream-breaking edit as an unsaved draft and saves a corrected draft", async () => {
  const text = `${source}\n<hick:patch id="later" after="change">
--- a/app.py
+++ b/app.py
@@ -1 +1 @@
-new
+final
</hick:patch>`;
  const { ui, view, inner } = mount(text);
  await waitFor(() => expect(ui.container.querySelector(".comparison-side .cm-editor")).not.toBeNull());
  act(() => inner().dispatch({ changes: { from: 0, to: 3, insert: "broken" } }));
  await waitFor(() => expect(screen.getByRole("alert").textContent).toMatch(/invalidates/));
  expect(view.state.doc.toString()).toBe(text); expect(inner().state.doc.toString()).toBe("broken\n");
  expect(screen.getByText("app.py · Unsaved draft")).toBeTruthy();
  act(() => inner().dispatch({ changes: { from: 0, to: 6, insert: "new" } }));
  await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
  await waitFor(() => expect(screen.getByText("app.py · Up to date")).toBeTruthy());
});

</storycode:copy>
<a id="f-dc4ea70ab787-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-dc4ea70ab787-expression_statement-6">
it("makes historical code read-only while the New code is editable", async () => {
  const { ui } = mount();
  await waitFor(() => expect(ui.container.querySelector(".comparison-side .cm-editor")).not.toBeNull());
  expect((screen.getByLabelText("Comparison layout") as HTMLSelectElement).value).toBe("side-by-side");
  const editors = ui.container.querySelectorAll(".comparison-side .cm-content");
  expect(editors[0].getAttribute("contenteditable")).toBe("false");
  expect(editors[1].getAttribute("contenteditable")).toBe("true");
});

</storycode:copy>
<a id="f-dc4ea70ab787-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-dc4ea70ab787-expression_statement-7">
it("respects an immutable owning document", async () => {
  const { ui } = mount(source, true);
  await waitFor(() => expect(ui.container.querySelector(".comparison-side .cm-editor")).not.toBeNull());
  expect(ui.container.querySelector(".comparison-side .cm-content")?.getAttribute("contenteditable")).toBe("false");
});

</storycode:copy>
<a id="f-dc4ea70ab787-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-dc4ea70ab787-expression_statement-8">
it("discloses patch choices and preserves agent edit markers through regeneration", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const { ui, view } = mount(source, false, registry);
  await waitFor(() => expect(ui.container.querySelector(".comparison-side .cm-editor")).not.toBeNull());
  const markers: string[] = [];
  view.dispatch({ effects: StateEffect.appendConfig.of(EditorView.updateListener.of(update => {
    for (const transaction of update.transactions) if (transaction.isUserEvent("input.uiAgentProjectPatch")) markers.push("agent");
  })) });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const run = async (label: string, args: Record<string, string | number | boolean>) => {
    const action = actions().find(action => action.label === label && action.enabled)!;
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle: action.handle, arguments: args, request_id: crypto.randomUUID() }); });
    return result;
  };
  expect(actions().some(action => action.label === "Select constructed patch file")).toBe(false);
  await run("Set Patch options visibility", { open: true });
  expect(await run("Select constructed patch editable side", { side: "old" })).toMatchObject({ status: "result" });
  expect((screen.getByLabelText("Editable side") as HTMLSelectElement).value).toBe("old");
  fireEvent.change(screen.getByLabelText("Editable side"), { target: { value: "new" } });
  expect(await run("Replace comparison draft range", { from: 0, to: 3, text: "agent code" })).toMatchObject({ status: "result", outcome: { value: { draft_changed: true, saved: false } } });
  await waitFor(() => expect(view.state.doc.toString()).toContain("+agent code"));
  expect(markers).toEqual(["agent"]);
  await run("Set Patch options visibility", { open: false });
  expect(actions().some(action => action.label === "Select constructed patch file")).toBe(false);
});
</storycode:copy>
<a id="f-dc4ea70ab787-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-dc4ea70ab787-expression_statement-9">
it("does not adopt an inspection error from a retired document owner", async () => {
  const { ui, slot, cx } = mount();
  let reject!: (error: Error) => void;
  vi.mocked(request).mockImplementation(async (_method, path) => path.includes("first") ? new Promise((_resolve, fail) => { reject = fail; }) : { entry: "project.md", sources: { "project.md": source } });
  ui.rerender(<PatchEditor slot={slot} cx={{ ...cx, docId: "first" }} />);
  await waitFor(() => expect(reject).toBeTruthy());
  ui.rerender(<PatchEditor slot={slot} cx={{ ...cx, docId: "second" }} />);
  await screen.findByText("app.py · Up to date");
  await act(async () => { reject(Error("retired inspection failed")); });
  expect(screen.queryByText("retired inspection failed")).toBeNull();
});
</storycode:copy>

### Assemble `apps/web/src/elements/patch/Editor.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/patch/Editor.test.tsx"><storycode:paste select="#f-dc4ea70ab787-expression_statement"/><storycode:paste select="#f-dc4ea70ab787-expression_statement-2"/><storycode:paste select="#f-dc4ea70ab787-views"/><storycode:paste select="#f-dc4ea70ab787-expression_statement-3"/><storycode:paste select="#f-dc4ea70ab787-source"/><storycode:paste select="#f-dc4ea70ab787-mount"/><storycode:paste select="#f-dc4ea70ab787-expression_statement-4"/><storycode:paste select="#f-dc4ea70ab787-expression_statement-5"/><storycode:paste select="#f-dc4ea70ab787-expression_statement-6"/><storycode:paste select="#f-dc4ea70ab787-expression_statement-7"/><storycode:paste select="#f-dc4ea70ab787-expression_statement-8"/><storycode:paste select="#f-dc4ea70ab787-expression_statement-9"/></storycode:file>


</storycode:doc>
