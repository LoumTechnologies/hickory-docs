import { useEffect, useMemo, useRef, useState } from "react";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { codeFolding, foldGutter, foldService } from "@codemirror/language";
import { EditorState } from "@codemirror/state";
import { Decoration, EditorView, highlightActiveLine, keymap } from "@codemirror/view";

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

type ProjectedLine =
  | { text: string; filesystem: ReturnType<typeof filesystemTextEntries>[number] }
  | { text: string; extra: WorkspaceTextExtra };

interface PreparedPlan {
  filesystem: FilesystemTreeReconciliation;
  remote: { apply: (value: string) => Promise<void>; value: string; summary: string }[];
}

function planLines(plan: PreparedPlan): string[] {
  return [
    ...plan.filesystem.renames.map((operation) => `Rename or move ${operation.path} → ${operation.to}`),
    ...plan.filesystem.creates.map((creation) => `Create ${creation.dir ? "folder" : "file"} ${creation.path}${creation.dir ? "/" : ""}`),
    ...plan.remote.map((edit) => edit.summary),
    ...plan.filesystem.deletes.map((path) => `Delete ${path} — confirmation required; there is no trash`),
  ];
}

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

export function FilesystemTreeEditor({
  nodes,
  onChanged,
  onOpenPath,
  onContextPath,
  kindOfPath = (_path, dir) => dir ? "directory" : "file",
  extras = [],
  dirtyPaths = new Set(),
}: {
  nodes: readonly FileNode[];
  onChanged: () => void;
  onOpenPath: (path: string) => void;
  onContextPath: (event: MouseEvent, path: string, dir: boolean) => void;
  kindOfPath?: (path: string, dir: boolean) => string;
  extras?: readonly WorkspaceTextExtra[];
  dirtyPaths?: ReadonlySet<string>;
}) {
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView>();
  const entries = useMemo(() => filesystemTextEntries(nodes), [nodes]);
  const projection = useMemo(() => projectedLines(nodes, extras), [nodes, extras]);
  const version = projection.map((line) => "filesystem" in line ? line.filesystem.path : `${line.extra.key}:${line.text}`).join("\n");
  const base = useRef(projection);
  const [dirty, setDirty] = useState(false);
  const [busy, setBusy] = useState(false);
  const [pendingDeletes, setPendingDeletes] = useState<string[] | null>(null);
  const [dryRun, setDryRun] = useState<{ lines?: string[]; error?: string } | null>(null);
  const [message, setMessage] = useState("Edit names or indentation. Changes stay here until Apply.");
  const save = useRef<() => void>(() => undefined);

  useEffect(() => {
    if (!host.current) return;
    base.current = projection;
    const editor = new EditorView({
      parent: host.current,
      state: EditorState.create({
        doc: projection.map((line) => line.text).join("\n"),
        extensions: [
          editorChrome("code"),
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
              const lineIndex = current.state.doc.lineAt(position).number - 1;
              const reordered = extras.length === 0
                ? parseFilesystemTreeText(current.state.doc.toString(), entries).edits?.[lineIndex]
                : undefined;
              if (reordered) {
                if (!reordered.dir) onOpenPath(reordered.path);
                return !reordered.dir;
              }
              const line = base.current[lineIndex];
              if (line && "extra" in line) { line.extra.activate?.(); return Boolean(line.extra.activate); }
              if (line && !line.filesystem.dir) onOpenPath(line.filesystem.path);
              return Boolean(line && !line.filesystem.dir);
            },
            contextmenu: (event, current) => {
              let position: number | null = null;
              try { position = current.posAtDOM(event.target as Node); } catch { position = current.posAtCoords({ x: event.clientX, y: event.clientY }); }
              if (position === null) return false;
              const lineIndex = current.state.doc.lineAt(position).number - 1;
              const reordered = extras.length === 0
                ? parseFilesystemTreeText(current.state.doc.toString(), entries).edits?.[lineIndex]
                : undefined;
              if (reordered) {
                event.preventDefault();
                onContextPath(event, reordered.path, reordered.dir);
                return true;
              }
              const line = base.current[lineIndex];
              if (!line || "extra" in line) return false;
              event.preventDefault();
              onContextPath(event, line.filesystem.path, line.filesystem.dir);
              return true;
            },
          }),
        ],
      }),
    });
    view.current = editor;
    setDirty(false);
    setPendingDeletes(null);
    setDryRun(null);
    return () => { view.current = undefined; editor.destroy(); };
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

  save.current = () => {
    if (busy) return;
    setPendingDeletes(null);
    setDryRun(null);
    const prepared = prepare();
    if (!prepared.plan) { setMessage(prepared.error ?? "The Files buffer is invalid."); return; }
    const { filesystem, remote } = prepared.plan;
    if (filesystem.deletes.length > 0) {
      setPendingDeletes(filesystem.deletes);
      setMessage(`Delete ${filesystem.deletes.length === 1 ? filesystem.deletes[0] : `${filesystem.deletes.length} entries`}? There is no trash.`);
      return;
    }
    const count = filesystem.renames.length + filesystem.creates.length + remote.length;
    if (count === 0) { setDirty(false); setMessage("No filesystem changes."); return; }
    setBusy(true);
    setMessage(`Applying ${count} filesystem ${count === 1 ? "edit" : "edits"}…`);
    void (async () => {
      for (const operation of filesystem.renames) {
        await api.fileOp({ op: "rename", path: operation.path, to: operation.to });
      }
      for (const creation of filesystem.creates) {
        await api.fileOp({ op: creation.dir ? "mkdir" : "create", path: creation.path });
      }
      for (const edit of remote) await edit.apply(edit.value);
    })().then(
      () => {
        setBusy(false);
        setDirty(false);
        setMessage(`Applied ${count} filesystem ${count === 1 ? "edit" : "edits"}.`);
        onChanged();
      },
      (error) => {
        setBusy(false);
        setMessage(`Filesystem edit refused: ${said(error)}`);
        onChanged();
      },
    );
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

  const confirmDeletes = () => {
    if (busy || !pendingDeletes) return;
    const paths = pendingDeletes;
    setBusy(true);
    setPendingDeletes(null);
    setMessage(`Deleting ${paths.length === 1 ? paths[0] : `${paths.length} entries`}…`);
    void (async () => {
      for (const path of paths) await api.fileOp({ op: "delete", path });
    })().then(
      () => {
        setBusy(false);
        setDirty(false);
        setMessage(`Deleted ${paths.length === 1 ? paths[0] : `${paths.length} entries`}.`);
        onChanged();
      },
      (error) => {
        setBusy(false);
        setMessage(`Filesystem delete refused: ${said(error)}`);
        onChanged();
      },
    );
  };

  return <section className={`filesystem-editor${dirty ? " dirty" : ""}`} aria-label="Filesystem text buffer">
    {dirty && <div className="filesystem-editor__toolbar" role="toolbar" aria-label="Unsaved Files changes">
      <button type="button" onClick={preview} disabled={busy}>Dry Run</button>
      <button type="button" className="primary" onClick={() => save.current()} disabled={busy}>Apply</button>
    </div>}
    {dryRun && <aside className={`filesystem-editor__dry-run${dryRun.error ? " error" : ""}`} aria-label="Files dry run" aria-live="polite">
      <strong>Dry run — nothing has been applied.</strong>
      {dryRun.error
        ? <p>{dryRun.error}</p>
        : <ol>{dryRun.lines?.map((line, index) => <li key={`${index}:${line}`}>{line}</li>)}</ol>}
    </aside>}
    <div ref={host} className="filesystem-editor__code" aria-label="Files editor" />
    <footer className="filesystem-editor__status" aria-live="polite">
      <span>{busy ? "Working — " : ""}{message}</span>
      {pendingDeletes && <span className="filesystem-editor__confirm">
        <button type="button" onClick={confirmDeletes}>Delete</button>
        <button type="button" onClick={() => { setPendingDeletes(null); setMessage("Deletion cancelled; the buffer still differs from disk."); }}>Cancel</button>
      </span>}
    </footer>
  </section>;
}
