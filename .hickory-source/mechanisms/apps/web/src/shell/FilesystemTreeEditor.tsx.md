<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/FilesystemTreeEditor.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d30e9ea77e15"></a>
## `apps/web/src/shell/FilesystemTreeEditor.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A write exposes its proposed effects](../../../../../guarantees/authoring/a-write-exposes-its-proposed-effects/contract.md), [Filesystem Acts Keep Dired's Safety](../../../../../guarantees/authoring/the-tree-is-a-dired/contract.md), [The Workspace Tree Is An Editor Buffer](../../../../../guarantees/authoring/the-workspace-tree-edits-like-an-editor/contract.md).

<a id="f-d30e9ea77e15-said"></a>
### `said`

<storycode:copy id="f-d30e9ea77e15-said">
import { useId } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiOutcome, UiValue } from "../lib/uiActions";
import { useEditorActions } from "../editor/uiEditorActions";
import { useEditorEditing } from "../editor/useEditorEditing";
import { prepareEditorChanges } from "../editor/textInput";
import { ChangePreview } from "../components/ChangePreview";
import { useEffect, useMemo, useRef, useState } from "react";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { codeFolding, foldGutter, foldService } from "@codemirror/language";
import { EditorState } from "@codemirror/state";
import { Decoration, EditorView, highlightActiveLine, keymap, lineNumbers } from "@codemirror/view";

import { api } from "../api/client";
import type { FileNode } from "../api/types";
import { editorChrome } from "../editor/chrome";
import { multipleCursors } from "../editor/multiCursor";
import {
  filesystemTextEntries,
  parseFilesystemTreeText,
  reconcileFilesystemTreeText,
  type FilesystemTreeReconciliation,
} from "../lib/filesystemTreeText";

const said = (error: unknown) => error instanceof Error ? error.message : String(error);

</storycode:copy>
<a id="f-d30e9ea77e15-WorkspaceTextExtra"></a>
### `WorkspaceTextExtra`

<storycode:copy id="f-d30e9ea77e15-WorkspaceTextExtra">
export interface WorkspaceTextExtra {
  key: string;
  parentPath: string;
  label: string;
  activate?: () => void;
  edit?: {
    prefix: string;
    value: string;
    suffix: string;
    apply: (value: string) => Promise<void>;
  };
}

</storycode:copy>
<a id="f-d30e9ea77e15-ProjectedLine"></a>
### `ProjectedLine`

<storycode:copy id="f-d30e9ea77e15-ProjectedLine">
type ProjectedLine =
  | { text: string; filesystem: ReturnType<typeof filesystemTextEntries>[number] }
  | { text: string; extra: WorkspaceTextExtra };

</storycode:copy>
<a id="f-d30e9ea77e15-PreparedPlan"></a>
### `PreparedPlan`

<storycode:copy id="f-d30e9ea77e15-PreparedPlan">
interface PreparedPlan {
  filesystem: FilesystemTreeReconciliation;
  remote: { apply: (value: string) => Promise<void>; value: string; summary: string }[];
}

</storycode:copy>
<a id="f-d30e9ea77e15-planLines"></a>
### `planLines`

<storycode:copy id="f-d30e9ea77e15-planLines">
function planLines(plan: PreparedPlan): string[] {
  return [
    ...plan.filesystem.creates.filter((creation) => creation.dir).map((creation) => `Create folder ${creation.path}/`),
    ...plan.filesystem.renames.map((operation) => `Rename or move ${operation.path} → ${operation.to}`),
    ...plan.filesystem.creates.filter((creation) => !creation.dir).map((creation) => `Create file ${creation.path}`),
    ...plan.remote.map((edit) => edit.summary),
    ...plan.filesystem.deletes.map((path) => `Delete ${path} — confirmation required; there is no trash`),
  ];
}

</storycode:copy>
<a id="f-d30e9ea77e15-projectedLines"></a>
### `projectedLines`

<storycode:copy id="f-d30e9ea77e15-projectedLines">
function projectedLines(nodes: readonly FileNode[], extras: readonly WorkspaceTextExtra[]): ProjectedLine[] {
  const entries = filesystemTextEntries(nodes);
  const under = new Map<string, WorkspaceTextExtra[]>();
  for (const extra of extras) under.set(extra.parentPath, [...(under.get(extra.parentPath) ?? []), extra]);
  return entries.flatMap((entry) => [
    { text: `${"  ".repeat(entry.depth)}${entry.name}${entry.dir ? "/" : ""}`, filesystem: entry } as ProjectedLine,
    ...(entry.dir ? (under.get(entry.path) ?? []).map((extra): ProjectedLine => ({
      text: `${"  ".repeat(entry.depth + 1)}${extra.label}`,
      extra,
    })) : []),
  ]);
}

</storycode:copy>
<a id="f-d30e9ea77e15-FilesystemTreeEditor"></a>
### `FilesystemTreeEditor`

<storycode:copy id="f-d30e9ea77e15-FilesystemTreeEditor">
export function FilesystemTreeEditor({
  nodes,
  onChanged,
  onDeleted,
  onOpenPath,
  onContextPath,
  kindOfPath = (_path, dir) => dir ? "directory" : "file",
  extras = [],
  dirtyPaths = new Set(),
}: {
  nodes: readonly FileNode[];
  onChanged: (renames?: readonly { path: string; to: string }[]) => void;
  onDeleted?: (paths: readonly string[]) => void;
  onOpenPath: (path: string) => void;
  onContextPath: (anchor: { clientX: number; clientY: number }, path: string, dir: boolean) => void;
  kindOfPath?: (path: string, dir: boolean) => string;
  extras?: readonly WorkspaceTextExtra[];
  dirtyPaths?: ReadonlySet<string>;
}) {
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView>();
  const [actionView, setActionView] = useState<EditorView | null>(null);
  const pending = useRef(false);
  const owner = useRef<string | null>(null);
  const entries = useMemo(() => filesystemTextEntries(nodes), [nodes]);
  const projection = useMemo(() => projectedLines(nodes, extras), [nodes, extras]);
  const version = projection.map((line) => "filesystem" in line ? line.filesystem.path : `${line.extra.key}:${line.text}`).join("\n");
  const liveVersion = useRef(version); liveVersion.current = version;
  const base = useRef(projection);
  const [dirty, setDirty] = useState(false);
  const [busy, setBusy] = useState(false);
  const [pendingDeletes, setPendingDeletes] = useState<{ paths: string[]; source: string; version: string; plan: PreparedPlan } | null>(null);
  const [receipt, setReceipt] = useState<UiValue>(null);
  const [dryRun, setDryRun] = useState<{ lines?: string[]; error?: string } | null>(null);
  const [message, setMessage] = useState("Edit names or indentation. Changes stay here until Apply.");
  const save = useRef<() => void>(() => undefined);
  const activate = useRef<(index: number, editor: EditorView) => boolean>(() => false);
  const context = useRef<(index: number, editor: EditorView, anchor: { clientX: number; clientY: number }) => boolean>(() => false);
  const target = (index: number, editor: EditorView): ProjectedLine | null => {
    if (pending.current || editor !== view.current || version !== liveVersion.current || index < 0 || index >= editor.state.doc.lines) return null;
    const reordered = extras.length === 0 ? parseFilesystemTreeText(editor.state.doc.toString(), entries).edits?.[index] : undefined;
    if (reordered) return { text: editor.state.doc.line(index + 1).text, filesystem: reordered };
    return base.current[index] ?? null;
  };
  activate.current = (index, editor) => {
    const line = target(index, editor);
    if (!line) return false;
    const run = "extra" in line ? line.extra.activate : line.filesystem.dir ? undefined : () => onOpenPath(line.filesystem.path);
    if (!run) return false;
    const at = editor.state.doc.line(index + 1);
    editor.dispatch({ selection: { anchor: at.from, head: at.to } }); run(); return true;
  };
  context.current = (index, editor, anchor) => {
    const line = target(index, editor);
    if (!line || !("filesystem" in line)) return false;
    onContextPath(anchor, line.filesystem.path, line.filesystem.dir); return true;
  };

  useEffect(() => {
    if (!host.current) return;
    base.current = projection;
    const generation = crypto.randomUUID(); owner.current = generation;
    const editor = new EditorView({
      parent: host.current,
      state: EditorState.create({
        doc: projection.map((line) => line.text).join("\n"),
        extensions: [
          editorChrome("code"),
          EditorState.transactionFilter.of(transaction => {
            if (!transaction.docChanged) return transaction;
            if (pending.current) return [];
            const changes: { from: number; to: number; insert: string }[] = [];
            transaction.changes.iterChanges((from, to, _fromB, _toB, text) => changes.push({ from, to, insert: text.toString() }));
            return "reason" in prepareEditorChanges(transaction.startState.doc.toString(), changes) ? [] : transaction;
          }),
          lineNumbers(),
          history(),
          multipleCursors(),
          highlightActiveLine(),
          EditorView.decorations.compute(["doc"], (state) => {
            const reordered = extras.length === 0
              ? parseFilesystemTreeText(state.doc.toString(), entries).edits
              : undefined;
            return Decoration.set(projection.flatMap((projected, index) => {
              const filesystem = reordered?.[index] ?? ("filesystem" in projected ? projected.filesystem : undefined);
              return index < state.doc.lines && filesystem ? [Decoration.line({
                class: `filesystem-editor__line${dirtyPaths.has(filesystem.path) ? " filesystem-editor__line--dirty" : ""}`,
                attributes: {
                  "data-tree-path": filesystem.path,
                  "data-tree-kind": kindOfPath(filesystem.path, filesystem.dir),
                },
              }).range(state.doc.line(index + 1).from)] : [];
            }));
          }),
          codeFolding(),
          foldGutter(),
          foldService.of((state, from) => {
            const line = state.doc.lineAt(from);
            const spaces = line.text.length - line.text.trimStart().length;
            if (!line.text.trimEnd().endsWith("/")) return null;
            let end = line.to;
            for (let number = line.number + 1; number <= state.doc.lines; number += 1) {
              const child = state.doc.line(number);
              const childSpaces = child.text.length - child.text.trimStart().length;
              if (child.text.trim() && childSpaces <= spaces) break;
              end = child.to;
            }
            return end > line.to ? { from: line.to, to: end } : null;
          }),
          keymap.of([
            { key: "Mod-s", run: () => { save.current(); return true; } },
            ...defaultKeymap,
            ...historyKeymap,
            indentWithTab,
          ]),
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              setDryRun(null);
              setPendingDeletes(null);
              const reorderedOnly = extras.length === 0 && (() => {
                const reconciliation = reconcileFilesystemTreeText(
                  update.state.doc.toString(),
                  entries,
                ).reconciliation;
                return Boolean(reconciliation
                  && reconciliation.renames.length === 0
                  && reconciliation.creates.length === 0
                  && reconciliation.deletes.length === 0);
              })();
              setDirty(!reorderedOnly);
              setMessage(reorderedOnly
                ? "No filesystem changes — row order is presentation only."
                : "Unsaved filesystem edits — use Dry Run or Apply.");
            }
          }),
          EditorView.domEventHandlers({
            dblclick: (event, current) => {
              let position: number | null = null;
              try { position = current.posAtDOM(event.target as Node); } catch { position = current.posAtCoords({ x: event.clientX, y: event.clientY }); }
              if (position === null) return false;
              const handled = activate.current(current.state.doc.lineAt(position).number - 1, current);
              if (handled) event.preventDefault();
              return handled;
            },
            contextmenu: (event, current) => {
              let position: number | null = null;
              try { position = current.posAtDOM(event.target as Node); } catch { position = current.posAtCoords({ x: event.clientX, y: event.clientY }); }
              if (position === null) return false;
              const handled = context.current(current.state.doc.lineAt(position).number - 1, current, event);
              if (handled) event.preventDefault();
              return handled;
            },
          }),
        ],
      }),
    });
    view.current = editor;
    setActionView(editor);
    setDirty(false);
    setPendingDeletes(null);
    setDryRun(null);
    return () => { if (owner.current === generation) owner.current = null; view.current = undefined; setActionView(null); editor.destroy(); };
  }, [version]);

  const prepare = (): { plan?: PreparedPlan; error?: string } => {
    if (!view.current) return { error: "The Files buffer is not ready." };
    const lines = view.current.state.doc.toString().split("\n");
    if (extras.length > 0 && lines.length !== base.current.length) {
      return { error: "Create or delete filesystem lines after folding live terminal and issue lines out of this buffer." };
    }
    const filesystemLines: string[] = [];
    const remote: PreparedPlan["remote"] = [];
    if (extras.length === 0) {
      filesystemLines.push(...lines);
    } else {
      for (let index = 0; index < lines.length; index += 1) {
        const original = base.current[index];
        if ("extra" in original) {
          if (lines[index] !== original.text) {
            const edit = original.extra.edit;
            const indent = original.text.slice(0, original.text.length - original.text.trimStart().length);
            const prefix = `${indent}${edit?.prefix ?? ""}`;
            if (!edit || !lines[index].startsWith(prefix) || !lines[index].endsWith(edit.suffix)) {
              return { error: `Line ${index + 1} has no semantic edit for that text.` };
            }
            const value = lines[index].slice(prefix.length, lines[index].length - edit.suffix.length).trim();
            if (!value) return { error: `Line ${index + 1}: the editable title cannot be empty.` };
            remote.push({
              apply: edit.apply,
              value,
              summary: `Edit ${original.extra.key}: ${JSON.stringify(edit.value)} → ${JSON.stringify(value)}`,
            });
          }
        } else filesystemLines.push(lines[index]);
      }
    }
    const result = reconcileFilesystemTreeText(filesystemLines.join("\n"), entries);
    const reconciliation = result.reconciliation;
    if (!reconciliation) return { error: result.error ?? "The Files buffer is invalid." };
    if (reconciliation.deletes.length > 0) {
      if (remote.length > 0) return { error: "Save remote title edits separately from filesystem deletions." };
    }
    return { plan: { filesystem: reconciliation, remote } };
  };

  type Operation = { summary: string; run: () => Promise<unknown>; deletedPath?: string };
  const execute = async (operations: Operation[], renames: readonly { path: string; to: string }[] | undefined,
    signal: AbortSignal): Promise<UiOutcome> => {
    if (pending.current) return { status: "unavailable", reason: "A Files operation is already pending" };
    const editor = view.current, capturedOwner = owner.current, capturedVersion = liveVersion.current;
    if (!editor || !capturedOwner) return { status: "unavailable", reason: "Files editor closed" };
    pending.current = true; setBusy(true);
    const results: { operation: string; status: string; result?: UiValue; reason?: string }[] = [];
    let failed = false;
    try {
      for (const operation of operations) {
        if (signal.aborted || owner.current !== capturedOwner || liveVersion.current !== capturedVersion || view.current !== editor) {
          failed = true; results.push({ operation: operation.summary, status: "not_started", reason: "Cancelled or Files owner changed; preceding effects are not rolled back" }); break;
        }
        try {
          const result = await operation.run();
          results.push({ operation: operation.summary, status: "completed", result: result === undefined ? null : result as UiValue });
          if (operation.deletedPath) onDeleted?.([operation.deletedPath]);
        } catch (error) {
          failed = true; results.push({ operation: operation.summary, status: "failed", reason: said(error) }); break;
        }
      }
      const value = { results, requested: operations.length, completed: results.filter(result => result.status === "completed").length,
        all_completed: !failed && results.length === operations.length, atomic: false, rollback: false, failed_operation_effects: "Not asserted; a failed request may have applied effects" };
      if (owner.current === capturedOwner) {
        setReceipt(value); setDirty(failed); setMessage(failed
          ? "Filesystem edits stopped. Inspect the operation results before preparing another plan; completed effects remain."
          : `Applied ${operations.length} filesystem ${operations.length === 1 ? "edit" : "edits"}.`);
        onChanged(failed ? undefined : renames);
      }
      return { status: "completed", value };
    } finally { pending.current = false; setBusy(false); }
  };

  const operationsFor = ({ filesystem, remote }: PreparedPlan): Operation[] => {
    return [
      ...filesystem.creates.filter(creation => creation.dir).map(creation => ({ summary: `Create folder ${creation.path}`, run: () => api.fileOp({ op: "mkdir", path: creation.path }) })),
      ...filesystem.renames.map(operation => ({ summary: `Rename ${operation.path} → ${operation.to}`, run: () => api.fileOp({ op: "rename", path: operation.path, to: operation.to }) })),
      ...filesystem.creates.filter(creation => !creation.dir).map(creation => ({ summary: `Create file ${creation.path}`, run: () => api.fileOp({ op: "create", path: creation.path }) })),
      ...remote.map(edit => ({ summary: edit.summary, run: () => edit.apply(edit.value) })),
      ...filesystem.deletes.map(path => ({ summary: `Delete ${path}`, deletedPath: path, run: () => api.fileOp({ op: "delete", path }) })),
    ];
  };

  const apply = async (signal: AbortSignal): Promise<UiOutcome> => {
    if (pending.current) return { status: "unavailable", reason: "Wait for the pending Files operation" };
    setPendingDeletes(null); setDryRun(null);
    const prepared = prepare();
    if (!prepared.plan) { const reason = prepared.error ?? "The Files buffer is invalid."; setMessage(reason); return { status: "unavailable", reason }; }
    const { filesystem } = prepared.plan;
    if (filesystem.deletes.length > 0) {
      setPendingDeletes({ paths: filesystem.deletes, source: view.current!.state.doc.toString(), version, plan: prepared.plan });
      setMessage(`Delete ${filesystem.deletes.length === 1 ? filesystem.deletes[0] : `${filesystem.deletes.length} entries`}? There is no trash.`);
      return { status: "requires_input", reason: "Read and explicitly confirm the current deletion plan; there is no trash", request: `${parent}:deletion-confirmation` };
    }
    const operations = operationsFor(prepared.plan);
    if (!operations.length) { setDirty(false); setMessage("No filesystem changes."); return { status: "completed", value: { completed: 0 } }; }
    return execute(operations, filesystem.renames, signal);
  };

  const preview = () => {
    const prepared = prepare();
    if (!prepared.plan) {
      setDryRun({ error: prepared.error ?? "The Files buffer is invalid." });
      return;
    }
    const lines = planLines(prepared.plan);
    setDryRun({ lines: lines.length > 0 ? lines : ["No changes."] });
  };

  const confirmDeletes = async (signal: AbortSignal): Promise<UiOutcome> => {
    const plan = pendingDeletes;
    if (!plan || pending.current || plan.version !== liveVersion.current || plan.source !== view.current?.state.doc.toString())
      return { status: "unavailable", reason: "Deletion plan changed. Apply the current Files draft and review its deletion request again." };
    setPendingDeletes(null);
    const result = await execute(operationsFor(plan.plan), plan.plan.filesystem.renames, signal);
    return result;
  };
  const cancelDeletes = () => { setPendingDeletes(null); setMessage("Deletion cancelled; the buffer still differs from disk."); };
  const parent = `filesystem-editor:${useId()}`;
  const revision = useUiRevision(() => [version, owner.current, view.current?.state.doc.toString(), dirty, pending.current, pendingDeletes, dryRun, receipt]);
  useEditorActions(actionView, parent);
  useEditorEditing(actionView, { source: parent, label: "Files draft", localOnly: true,
    current: () => !pending.current && !!owner.current && version === liveVersion.current,
    identity: () => [version, owner.current, pending.current] });
  useUiReading("Files reconciliation status", { dirty, busy, message, dry_run: dryRun, pending_deletion: pendingDeletes?.paths ?? null,
    deletion_request: pendingDeletes ? `${parent}:deletion-confirmation` : null, receipt, atomic: false });
  const commands: { key: string; command: UiCommand }[] = [
    { key: "preview", label: "Dry run Files draft", enabled: dirty, execute: async () => { preview(); return { status: "completed", value: { preview_requested: true, writes: false } }; } },
    { key: "apply", label: "Apply Files draft", enabled: dirty, execute: async (_args: unknown, _actor: unknown, signal: AbortSignal) => apply(signal) },
    { key: "delete", label: "Confirm Files deletions", enabled: !!pendingDeletes, execute: async (_args: unknown, _actor: unknown, signal: AbortSignal) => confirmDeletes(signal) },
    { key: "cancel", label: "Cancel Files deletions", enabled: !!pendingDeletes, execute: async () => { cancelDeletes(); return { status: "completed", value: { cancelled: true, writes: false } }; } },
  ].map(action => ({ key: `${parent}:${action.key}`, command: { parent, subject: parent, label: action.label, parameters: {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: action.enabled && !pending.current && !!owner.current && version === liveVersion.current }), execute: action.execute as UiCommand["execute"] } }));
  const row = { type: "number" as const, minimum: 1, maximum: 1000000, integer: true };
  for (const menu of [false, true]) commands.push({ key: `${parent}:${menu ? "menu" : "activate"}`, command: {
    parent, subject: parent, label: menu ? "Open Files row menu" : "Activate Files row", parameters: { line: row }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: !!view.current && !!owner.current && !pending.current && version === liveVersion.current }),
    execute: async args => {
      const editor = view.current;
      if (!editor || pending.current || !owner.current || version !== liveVersion.current) return { status: "unavailable", reason: "Files owner changed or is busy" };
      const index = (args.line as number) - 1;
      const rect = host.current?.getBoundingClientRect();
      const handled = menu ? context.current(index, editor, { clientX: rect?.left ?? 0, clientY: rect?.top ?? 0 }) : activate.current(index, editor);
      return handled ? { status: "completed", value: { dispatched: true, line: index + 1, effect: menu ? "Read the opened menu" : "Observe the actual opened pane" } }
        : { status: "unavailable", reason: "This current row has no requested semantic capability" };
    },
  } });
  const invoke = useUiCommands(commands);
  const invokeHuman = (id: string) => void invoke(`${parent}:${id}`, {}).catch(error => setMessage(said(error)));
  save.current = () => invokeHuman("apply");

  return <section className={`filesystem-editor${dirty ? " dirty" : ""}`} aria-label="Filesystem text buffer">
    {dirty && <div className="filesystem-editor__toolbar" role="toolbar" aria-label="Unsaved Files changes">
      <button type="button" onClick={() => invokeHuman("preview")} disabled={busy}>Dry Run</button>
      <button type="button" className="primary" onClick={() => save.current()} disabled={busy}>Apply</button>
    </div>}
    {dryRun && <aside className={`filesystem-editor__dry-run${dryRun.error ? " error" : ""}`} aria-label="Files dry run" aria-live="polite">
      <ChangePreview heading="Dry run — nothing has been applied" effects="Apply performs the reviewed filesystem edits; the result stays in this pane.">
      {dryRun.error
        ? <p>{dryRun.error}</p>
        : <ol>{dryRun.lines?.map((line, index) => <li key={`${index}:${line}`}>{line}</li>)}</ol>}
      </ChangePreview>
    </aside>}
    <div ref={host} className="filesystem-editor__code" aria-label="Files editor" />
    <footer className="filesystem-editor__status" aria-live="polite">
      <span>{busy ? "Working — " : ""}{message}</span>
      {pendingDeletes && <span className="filesystem-editor__confirm">
        <button type="button" onClick={() => invokeHuman("delete")}>Delete</button>
        <button type="button" onClick={() => invokeHuman("cancel")}>Cancel</button>
      </span>}
    </footer>
  </section>;
}
</storycode:copy>

### Assemble `apps/web/src/shell/FilesystemTreeEditor.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/FilesystemTreeEditor.tsx"><storycode:paste select="#f-d30e9ea77e15-said"/><storycode:paste select="#f-d30e9ea77e15-WorkspaceTextExtra"/><storycode:paste select="#f-d30e9ea77e15-ProjectedLine"/><storycode:paste select="#f-d30e9ea77e15-PreparedPlan"/><storycode:paste select="#f-d30e9ea77e15-planLines"/><storycode:paste select="#f-d30e9ea77e15-projectedLines"/><storycode:paste select="#f-d30e9ea77e15-FilesystemTreeEditor"/></storycode:file>

</storycode:doc>
