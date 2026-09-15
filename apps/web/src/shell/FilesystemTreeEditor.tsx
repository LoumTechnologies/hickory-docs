import { useEffect, useMemo, useRef, useState } from "react";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { codeFolding, foldGutter, foldService } from "@codemirror/language";
import { EditorState } from "@codemirror/state";
import { Decoration, EditorView, highlightActiveLine, keymap } from "@codemirror/view";

import { api } from "../api/client";
import type { FileNode } from "../api/types";
import { editorChrome } from "../editor/chrome";
import { multipleCursors } from "../editor/multiCursor";
import {
  filesystemTextEntries,
  reconcileFilesystemTreeText,
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
  const [message, setMessage] = useState("Edit names or indentation. Ctrl+S applies filesystem changes.");
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
          EditorView.decorations.compute(["doc"], (state) => Decoration.set(
            projection.flatMap((projected, index) => index < state.doc.lines && "filesystem" in projected ? [Decoration.line({
              class: `filesystem-editor__line${dirtyPaths.has(projected.filesystem.path) ? " filesystem-editor__line--dirty" : ""}`,
              attributes: {
                "data-tree-path": projected.filesystem.path,
                "data-tree-kind": kindOfPath(projected.filesystem.path, projected.filesystem.dir),
              },
            }).range(state.doc.line(index + 1).from)] : []),
          )),
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
          ]),
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              setDirty(true);
              setMessage("Unsaved filesystem edits — Ctrl+S to apply.");
            }
          }),
          EditorView.domEventHandlers({
            dblclick: (event, current) => {
              let position: number | null = null;
              try { position = current.posAtDOM(event.target as Node); } catch { position = current.posAtCoords({ x: event.clientX, y: event.clientY }); }
              if (position === null) return false;
              const line = base.current[current.state.doc.lineAt(position).number - 1];
              if (line && "extra" in line) { line.extra.activate?.(); return Boolean(line.extra.activate); }
              if (line && !line.filesystem.dir) onOpenPath(line.filesystem.path);
              return Boolean(line && !line.filesystem.dir);
            },
            contextmenu: (event, current) => {
              let position: number | null = null;
              try { position = current.posAtDOM(event.target as Node); } catch { position = current.posAtCoords({ x: event.clientX, y: event.clientY }); }
              if (position === null) return false;
              const line = base.current[current.state.doc.lineAt(position).number - 1];
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
    return () => { view.current = undefined; editor.destroy(); };
  }, [version]);

  save.current = () => {
    if (busy || !view.current) return;
    const lines = view.current.state.doc.toString().split("\n");
    setPendingDeletes(null);
    if (extras.length > 0 && lines.length !== base.current.length) {
      setMessage("Create or delete filesystem lines after folding live terminal and issue lines out of this buffer.");
      return;
    }
    const filesystemLines: string[] = [];
    const extraEdits: { apply: (value: string) => Promise<void>; value: string }[] = [];
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
              setMessage(`Line ${index + 1} has no semantic edit for that text.`);
              return;
            }
            const value = lines[index].slice(prefix.length, lines[index].length - edit.suffix.length).trim();
            if (!value) { setMessage(`Line ${index + 1}: the editable title cannot be empty.`); return; }
            extraEdits.push({ apply: edit.apply, value });
          }
        } else filesystemLines.push(lines[index]);
      }
    }
    const result = reconcileFilesystemTreeText(filesystemLines.join("\n"), entries);
    const reconciliation = result.reconciliation;
    if (!reconciliation) { setMessage(result.error ?? "The Files buffer is invalid."); return; }
    if (reconciliation.deletes.length > 0) {
      if (extraEdits.length > 0) { setMessage("Save remote title edits separately from filesystem deletions."); return; }
      setPendingDeletes(reconciliation.deletes);
      setMessage(`Delete ${reconciliation.deletes.length === 1 ? reconciliation.deletes[0] : `${reconciliation.deletes.length} entries`}? There is no trash.`);
      return;
    }
    const count = reconciliation.renames.length + reconciliation.creates.length + extraEdits.length;
    if (count === 0) { setDirty(false); setMessage("No filesystem changes."); return; }
    setBusy(true);
    setMessage(`Applying ${count} filesystem ${count === 1 ? "edit" : "edits"}…`);
    void (async () => {
      for (const operation of reconciliation.renames) {
        await api.fileOp({ op: "rename", path: operation.path, to: operation.to });
      }
      for (const creation of reconciliation.creates) {
        await api.fileOp({ op: creation.dir ? "mkdir" : "create", path: creation.path });
      }
      for (const edit of extraEdits) await edit.apply(edit.value);
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
