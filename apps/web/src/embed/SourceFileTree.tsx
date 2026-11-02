import { useEffect, useMemo, useRef } from "react";
import { EditorState, StateEffect } from "@codemirror/state";
import { Decoration, EditorView, keymap } from "@codemirror/view";
import { defaultKeymap } from "@codemirror/commands";
import { foldEffect, foldable, foldedRanges, toggleFold, unfoldEffect } from "@codemirror/language";
import { editorChrome, paneSearch } from "../editor/chrome";
import { filesystemTreeFolding } from "../editor/filesystemTreeFolding";
import { filesystemTextEntries, filesystemTreeText } from "../lib/filesystemTreeText";
import { buildTree, type DirNode } from "../lib/tree";
import type { FileNode } from "../api/types";

function fileNodes(tree: DirNode, prefix = ""): FileNode[] {
  return [
    ...[...tree.dirs.values()].map(dir => {
      const path = prefix + dir.name;
      return { name: dir.name, path, dir: true, children: fileNodes(dir, path + "/") };
    }),
    ...tree.files.map(file => ({ ...file, dir: false })),
  ];
}

export function SourceFileTree({ paths, current, identity, onOpen }: {
  paths: readonly string[]; current: string; identity: string; onOpen: (path: string) => void;
}) {
  const host = useRef<HTMLDivElement>(null);
  const open = useRef(onOpen); open.current = onOpen;
  const entries = useMemo(() => filesystemTextEntries(fileNodes(buildTree([...paths].sort()))), [paths]);
  useEffect(() => {
    if (!host.current) return;
    const key = `hickory.source.files:${identity}`;
    let saved: { expanded?: string[]; top?: number; left?: number } = {};
    try { saved = JSON.parse(sessionStorage.getItem(key) ?? "{}"); if (!saved || typeof saved !== "object") saved = {}; } catch { /* A blocked store cannot prevent reading. */ }
    const expanded = new Set(Array.isArray(saved.expanded) ? saved.expanded.filter(path => typeof path === "string") : []);
    let offset = 0;
    const marks = entries.flatMap(entry => {
      const from = offset;
      offset += entry.depth * 2 + entry.name.length + (entry.dir ? 1 : 0) + 1;
      const attributes = { "data-source-path": entry.path, title: entry.path };
      return [
        Decoration.line({ class: `${entry.dir ? "source-tree-directory" : "source-tree-file"}${entry.path === current ? " source-tree-current" : ""}`, attributes }).range(from),
        ...(!entry.dir ? [Decoration.mark({ tagName: "a", attributes: { href: `?path=${encodeURIComponent(entry.path)}`, title: entry.path, "data-source-path": entry.path, ...(entry.path === current ? { "aria-current": "page" } : {}) } }).range(from + entry.depth * 2, from + entry.depth * 2 + entry.name.length)] : []),
      ];
    });
    const persist = (view: EditorView) => {
      const collapsed = new Set<number>(); foldedRanges(view.state).between(0, view.state.doc.length, from => { collapsed.add(from); });
      try { sessionStorage.setItem(key, JSON.stringify({
        expanded: entries.filter((entry, index) => entry.dir && !collapsed.has(view.state.doc.line(index + 1).to)).map(entry => entry.path),
        top: view.scrollDOM.scrollTop, left: view.scrollDOM.scrollLeft,
      })); } catch { /* Session state is optional. */ }
    };
    const activate = (view: EditorView) => {
      const entry = entries[view.state.doc.lineAt(view.state.selection.main.head).number - 1];
      if (!entry) return false;
      if (entry.dir) return toggleFold(view);
      persist(view); open.current(entry.path); return true;
    };
    const editor = new EditorView({ parent: host.current, state: EditorState.create({
      doc: filesystemTreeText(entries), extensions: [
        editorChrome("code"), paneSearch("Source files"), filesystemTreeFolding(),
        EditorState.readOnly.of(true), EditorView.editable.of(false),
        EditorView.contentAttributes.of({ tabindex: "0", "aria-label": "Published files" }),
        EditorView.decorations.of(Decoration.set(marks, true)),
        keymap.of([{ key: "Enter", run: activate }, ...defaultKeymap]),
        EditorView.domEventHandlers({ click(event, view) {
          if (!(event.target instanceof Element) || event.target.closest(".cm-gutterElement, .cm-foldPlaceholder")) return false;
          const row = event.target.closest<HTMLElement>("[data-source-path]");
          const path = row?.dataset.sourcePath;
          const index = entries.findIndex(entry => entry.path === path);
          if (index < 0) return false;
          if (!entries[index].dir && (event.ctrlKey || event.metaKey || event.shiftKey || event.altKey || event.button !== 0)) return false;
          view.dispatch({ selection: { anchor: view.state.doc.line(index + 1).from } });
          event.preventDefault(); return activate(view);
        } }),
      ],
    }) });
    const selected = entries.findIndex(entry => entry.path === current);
    editor.dispatch({ effects: entries.flatMap((entry, index) => {
      if (!entry.dir || expanded.has(entry.path) || current.startsWith(entry.path + "/")) return [];
      const line = editor.state.doc.line(index + 1);
      const range = foldable(editor.state, line.from, line.to);
      return range ? [foldEffect.of(range)] : [];
    }), ...(selected >= 0 ? { selection: { anchor: editor.state.doc.line(selected + 1).from } } : {}) });
    if (Number.isFinite(saved.top)) editor.scrollDOM.scrollTop = Math.max(0, saved.top!);
    if (Number.isFinite(saved.left)) editor.scrollDOM.scrollLeft = Math.max(0, saved.left!);
    if (selected >= 0) requestAnimationFrame(() => {
      if (!editor.dom.isConnected) return;
      const at = editor.state.doc.line(selected + 1).from;
      const bounds = editor.coordsAtPos(at), viewport = editor.scrollDOM.getBoundingClientRect();
      if (!bounds || bounds.top < viewport.top || bounds.bottom > viewport.bottom) editor.dispatch({ effects: EditorView.scrollIntoView(at, { y: "nearest" }) });
    });
    let timer: ReturnType<typeof setTimeout> | undefined;
    const remember = () => { clearTimeout(timer); timer = setTimeout(() => persist(editor), 100); };
    editor.scrollDOM.addEventListener("scroll", remember);
    const foldListener = EditorView.updateListener.of(update => { if (update.transactions.some(transaction => transaction.effects.some(effect => effect.is(foldEffect) || effect.is(unfoldEffect)))) remember(); });
    // Install the listener after restoring folds so the restoration does not overwrite saved scrolling.
    editor.dispatch({ effects: StateEffect.appendConfig.of(foldListener) });
    return () => { clearTimeout(timer); persist(editor); editor.scrollDOM.removeEventListener("scroll", remember); editor.destroy(); };
  }, [entries, current, identity]);
  return <div ref={host} className="source-tree" />;
}
