<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Compose only files, documents and agent panes

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Workspace debugger hosts remain mounted above tab content, so stepping into another file or moving the launch tab cannot destroy its session. Cross-file stop requests open the existing tree entry and queue the adapter line for the mounted editor.

The shell owns layout and commands. Documents remain in the buffer store when hidden or moved. File reads publish into their requested tab group without changing the currently focused group when a late reply arrives.

Pane content classifies internal tab drags before embedded text controls can consume their payload. An edge drop invokes docking; the center appends the existing tab to this group without adding a split. The half-pane preview is pointer-transparent and never reflows the content. Recompute the edge at drop time rather than trusting an old hover position.

Each tab accepts a drop before itself; the surrounding strip remains the append target. Stop tab-drop bubbling so its parent cannot append the same tab afterward. An absolutely positioned insertion mark does not change tab widths or move editor content. Only layout identities change.

File commands act on the connected workspace: path entry opens a file or directory, New Tab (Ctrl+N or Cmd+N) keeps an unnamed buffer, and Save As exclusively creates a new destination before a hash-checked write. Existing destinations are never overwritten by Save As. Save All visits each named buffer; unnamed drafts ask for a path and failed saves retain their original edits. Refresh observes the tree without replacing open buffers.

<ui:copy id="simple-e8a50a27b2-0">
import { TooltipLayer } from "../components/TooltipLayer";
import { useEffect, useRef, useState } from "react";
import { api } from "../api/client";
import { PlainDebugHosts } from "../debug/plainDebugHosts";
import { onOpenLocation, revealLine } from "../lib/revealLine";
import type { FileNode } from "../api/types";
import { Buffer, Buffers } from "./buffers";
import { RichOwners } from "./RichInline";
import { enterMode, type EnterMode } from "./SmartEnter";
import { Document, openFind, wordNavigationMode, type WordNavigationMode } from "./Document";
import { Terminal, TerminalSession } from "./Terminal";
import { Agent, AgentSession } from "./Agent";
import { activate, groups, initialLayout, isGroup, move, remove, split, update, dock, dropEdge, type Edge, type Axis, type Layout } from "./layout";
function EditorSettings() {
  const [error, setError] = useState("");
  const [enter, setEnter] = useState(enterMode);
  const [modes, setModes] = useState(() => ({ control: wordNavigationMode("control"), alt: wordNavigationMode("alt") }));
  return <fieldset><legend>Editor settings</legend>
    <label>Enter indentation<select aria-label="Enter indentation" value={enter} onChange={event => {
      const value = event.target.value as EnterMode;
      try { localStorage.setItem("editor.enterIndent", value); setEnter(value); setError(""); }
      catch { setError("Cannot save editor settings. Allow local storage in this browser, then try again."); }
    }}><option value="smart">Smart (language rules)</option><option value="copy">Match current line</option><option value="none">None</option></select></label>{(["control", "alt"] as const).map(modifier => <label key={modifier}>
    {modifier === "control" ? "Ctrl" : "Alt/Option"} + Left/Right
    <select aria-label={`${modifier === "control" ? "Ctrl" : "Alt/Option"} + Left/Right`} value={modes[modifier]} onChange={event => {
      const value = event.target.value as WordNavigationMode;
      try { localStorage.setItem(`editor.${modifier}Arrow`, value); setModes(current => ({ ...current, [modifier]: value })); setError(""); }
      catch { setError("Cannot save editor settings. Allow local storage in this browser, then try again."); }
    }}>
      <option value="native">Platform default</option><option value="word">Whole identifier</option><option value="pieces">Word pieces</option>
    </select>
  </label>)}{error && <p role="status">{error}</p>}</fieldset>;
}
export function App() {
  const [layout, setLayout] = useState<Layout>(initialLayout);
  const [focused, setFocused] = useState("document-group");
  const [terminal] = useState(() => new TerminalSession());
  const [agent] = useState(() => new AgentSession());
  const [tree, setTree] = useState<FileNode[]>([]);
  const [notice, setNotice] = useState("");
  const [status, setStatus] = useState("Choose a file");
  const [, refresh] = useState(0);
  const [buffers] = useState(() => new Buffers(() => refresh(value => value + 1)));
  const requested = useRef(new Map<string, string>());
  function newTab() {
    const path = `Untitled-${crypto.randomUUID()}`;
    buffers.values.set(path, new Buffer({ name: `Untitled ${[...buffers.values.keys()].filter(key => key.startsWith("Untitled-")).length + 1}`, path, dir: false }, "", "", () => refresh(value => value + 1)));
    openTab(path);
  }
  function findPath(path: string, nodes = tree): FileNode | undefined {
    for (const node of nodes) { if (node.path === path) return node; const found = findPath(path, node.children ?? []); if (found) return found; }
  }
  async function openPath(directory = false) {
    const path = window.prompt(directory ? "Open Directory — path in this workspace (empty for root)" : "Open File — path in this workspace");
    if (path === null) return;
    if (directory && !path) { await loadFiles(); openTab("files"); return; }
    const node = findPath(path);
    if (!node || node.dir !== directory) { setNotice("Path not found. Refresh and choose a path in this workspace."); return; }
    if (directory) { setTree(node.children ?? []); openTab("files"); } else await open(node);
  }
  async function saveAs(source = buffer) {
    if (!source || source.saving) return;
    const path = window.prompt("Save As — new path in this workspace", source.file.path.startsWith("Untitled-") ? "" : source.file.path);
    if (!path) return;
    if (path === source.file.path) { await source.save(); return; }
    const text = source.text; source.saving = true; source.publish();
    try {
      if (/\.md$/i.test(path)) await api.createDoc("local", path, text);
      else {
        await api.fileOp({ op: "create", path });
        const empty = await api.file(path);
        if (empty.content !== "") throw new Error("Destination changed after creation");
        await api.saveFile(path, text, empty.hash);
      }
      await loadFiles();
      const opened = await api.openFile(path);
      await open({ name: path.split("/").at(-1)!, path, dir: false, doc_id: opened.doc_id });
      if (source.text === text) { setLayout(current => remove(current, source.file.path)); buffers.values.delete(source.file.path); }

    } catch (error) { setNotice(`${String(error)} Original edits are retained. Choose a new unused path and retry Save As.`); }
    finally { source.saving = false; source.publish(); refresh(value => value + 1); }
  }
  async function saveCurrent() { if (buffer?.file.path.startsWith("Untitled-")) await saveAs(); else await buffer?.save(); }
  async function saveAll() {
    for (const item of [...buffers.values.values()]) {
      if (item.file.path.startsWith("Untitled-")) { if (item.dirty) await saveAs(item); }
      else await item.save();
    }
  }
  async function loadFiles() {
    try { const response = await api.files(); setTree(response.tree); setStatus(response.root); }
    catch (error) { setNotice(`${String(error)} Start the Hickory backend, then choose Refresh files.`); }
  }
  useEffect(() => { void loadFiles(); }, []);
  useEffect(() => {
    const warn = (event: BeforeUnloadEvent) => { if (agent.prompt.length > 0 || agent.busy || [...buffers.values.values()].some(buffer => buffer.dirty || buffer.saving)) event.preventDefault(); };
    window.addEventListener("beforeunload", warn); return () => window.removeEventListener("beforeunload", warn);
  }, [buffers, agent, terminal]);
  const currentGroup = groups(layout).find(group => group.id === focused) ?? groups(layout)[0];
  const active = currentGroup.active;
  const buffer = active ? buffers.values.get(active) : undefined;
  async function openTerminal() { openTab("terminal"); await terminal.open(); }
  function openTab(tab: string) {
    const owner = groups(layout).find(group => group.tabs.includes(tab));
    const target = owner?.id ?? currentGroup.id;
    setLayout(current => activate(current, target, tab)); setFocused(target);
  }
  async function open(file: FileNode) {
    const target = groups(layout).find(group => group.id === focused && !group.tabs.includes("files"))?.id
      ?? groups(layout).find(group => !group.tabs.includes("files"))?.id ?? currentGroup.id;
    requested.current.set(target, file.path);
    setFocused(target);
    try {
      await buffers.open(file);
      if (requested.current.get(target) !== file.path) return;
      setLayout(current => {
        const owner = groups(current).find(group => group.tabs.includes(file.path));
        const live = owner?.id ?? groups(current).find(group => group.id === target)?.id ?? groups(current)[0].id;
        return activate(current, live, file.path);
      });
      setStatus(file.generated_by ? "Generated file · read-only" : file.path);
    } catch (error) { setNotice(`${String(error)} Refresh files and try opening ${file.path} again.`); }
  }
  useEffect(() => onOpenLocation(location => {
    const find = (nodes: FileNode[]): FileNode | undefined => {
      for (const node of nodes) { if (node.path === location.path) return node; const child = find(node.children ?? []); if (child) return child; }
    };
    const file = find(tree);
    if (file) { void open(file).then(() => revealLine(file.path, location.line)); }
  }), [tree, layout, focused]);
  function close(tab: string) {
    const candidate = buffers.values.get(tab);
    if (candidate?.dirty || candidate?.saving) { setNotice("Unsaved edits retained. Save this document before closing its tab."); return; }
    setLayout(current => remove(current, tab));
  }
  function divide(axis: Axis) { setLayout(current => split(current, currentGroup.id, axis, crypto.randomUUID())); }
  function nextGroup() {
    if (!active) return;
    const leaves = groups(layout), index = leaves.findIndex(group => group.id === focused), target = leaves[(index + 1) % leaves.length].id;
    setLayout(current => move(current, active, target)); setFocused(target);
  }
  useEffect(() => {
    const command = (event: KeyboardEvent) => {
      // Terminal control keys belong to the PTY, including Ctrl+N and Ctrl+S.
      if (event.defaultPrevented || (event.target instanceof Element && event.target.closest(".terminal-screen"))) return;
      if (!(event.metaKey || event.ctrlKey)) return;
      if (!event.shiftKey && !event.altKey && event.key.toLowerCase() === "n") {
        event.preventDefault(); newTab();
      } else if (event.key === "s") { event.preventDefault(); void saveCurrent(); }
    };
    window.addEventListener("keydown", command); return () => window.removeEventListener("keydown", command);
  }, [buffer, layout, focused]);
</ui:copy>

The window keyboard handler consumes Ctrl/Cmd+N before the browser default and uses the current layout and focused group so the draft opens beside the active document. Shift and Alt combinations remain available for other commands.

Menu and tab-strip commands expose the same operations as native menus. Empty leaves accept drops. Recursive splits retain arbitrary arrangements; resizing updates only their proportions.

<ui:copy id="simple-e8a50a27b2-1">
  const label = (tab: string) => tab === "files" ? "Files" : tab === "agent" ? "Agent" : tab === "terminal" ? "Terminal" : `${buffers.values.get(tab)?.file.name ?? tab.split("/").at(-1)}${buffers.values.get(tab)?.dirty ? " •" : ""}`;
  function content(tab: string) {
    if (tab === "files") return <div className="file-tree" aria-label="Files"><Tree nodes={tree} open={open} /></div>;
    if (tab === "terminal") return <Terminal session={terminal} />;
    if (tab === "agent") return <Agent session={agent} />;
    const document = buffers.values.get(tab);
    return document ? <Document buffer={document} /> : null;
  }
  // Keep the scroll recipient while the pointer rests. Scrolling content under
  // a stationary cursor must not silently transfer the next wheel to a grid.
  useEffect(() => {
    let owner: HTMLElement | null = null;
    let pointer: { x: number; y: number } | null = null;
    const moved = (event: MouseEvent) => {
      if (pointer && (event.clientX !== pointer.x || event.clientY !== pointer.y)) owner = null;
      pointer = { x: event.clientX, y: event.clientY };
    };
    const wheel = (event: WheelEvent) => {
      if (event.ctrlKey || event.defaultPrevented) return;
      moved(event);
      const path = event.composedPath();
      const target = path.find(node => node instanceof HTMLElement &&
        /(auto|scroll)/.test(getComputedStyle(node).overflow + getComputedStyle(node).overflowY + getComputedStyle(node).overflowX) &&
        (node.scrollHeight > node.clientHeight || node.scrollWidth > node.clientWidth)) as HTMLElement | undefined;
      if (!owner?.isConnected) owner = target ?? null;
      if (owner && target && owner !== target && path.includes(owner)) {
        event.preventDefault();
        const scale = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? owner.clientHeight : 1;
        owner.scrollBy(event.deltaX * scale, event.deltaY * scale);
      }
    };
    const release = () => { owner = null; };
    document.addEventListener("pointermove", moved, true);
    document.addEventListener("pointerdown", release, true);
    document.addEventListener("wheel", wheel, { capture: true, passive: false });
    return () => {
      document.removeEventListener("pointermove", moved, true);
      document.removeEventListener("pointerdown", release, true);
      document.removeEventListener("wheel", wheel, true);
    };
  }, []);
  return <main>
    {/* Shared gutter controls declare data-tip; mount their delegated reader here. */}
    <TooltipLayer />
    <PlainDebugHosts />
    <RichOwners buffers={buffers} />
    <nav aria-label="Menu" onClick={event => { if ((event.target as HTMLElement).closest("button")) event.currentTarget.querySelectorAll("details[open]").forEach(menu => menu.removeAttribute("open")); }}><details><summary>File</summary><div className="menu">
      <button onClick={() => { void openPath(); }}>Open File</button>
      <button onClick={() => { void openPath(true); }}>Open Directory</button>
      <button onClick={newTab}>New Tab</button>
      <button onClick={() => window.open(window.location.href, "_blank", "noopener")}>New Window</button>
      <button disabled={!buffer || buffer.readonly || !buffer.dirty || buffer.saving} onClick={() => { void saveCurrent(); }}>Save</button>
      <button disabled={!buffer || buffer.saving} onClick={() => { void saveAs(); }}>Save As</button>
      <button onClick={() => { void saveAll(); }}>Save All</button>
      <button onClick={() => { void loadFiles(); }}>Refresh</button>
      <button disabled={!active} onClick={() => active && close(active)}>Close tab</button>
    </div></details><details><summary>Edit</summary><div className="menu">
      <button disabled={!buffer} onClick={() => buffer && openFind(buffer)}>Find and replace</button>
      <EditorSettings />
    </div></details><details><summary>View</summary><div className="menu">
      <button onClick={() => openTab("files")}>Files</button><button onClick={() => openTab("agent")}>Agent</button>
      <button onClick={() => { void openTerminal(); }}>Terminal</button>
      <button onClick={() => divide("horizontal")}>Split right</button><button onClick={() => divide("vertical")}>Split below</button>
      <button disabled={!active || groups(layout).length < 2} onClick={nextGroup}>Move tab to next group</button>
    </div></details><span className="app-name">Hickory Simple</span></nav>
    <div className="workspace"><LayoutView node={layout} focused={focused} focus={setFocused} label={label} content={content} close={close}
      drop={(tab, target, before) => { setLayout(current => move(current, tab, target, before)); setFocused(target); }}
      activate={(group, tab) => { setLayout(current => activate(current, group, tab)); setFocused(group); }}
      dock={(tab, target, edge) => {
        const id = crypto.randomUUID(), next = dock(layout, tab, target, edge, id);
        if (next !== layout) { setLayout(next); setFocused(id); }
      }}
      resize={(id, ratio) => setLayout(current => update(current, id, node => isGroup(node) ? node : { ...node, ratio }))} /></div>
    <footer role="status">{notice ? `${notice} · ` : ""}{buffer ? `${buffer.file.name} · ${buffer.readonly ? "Read-only" : buffer.saving ? "Saving…" : buffer.dirty ? "Modified" : "Saved"}` : status}</footer>
  </main>;
}
function Tree({ nodes, open }: { nodes: FileNode[]; open: (file: FileNode) => void }) {
  return <ul>{nodes.map(node => <li key={node.path}>{node.dir
    ? <details><summary>{node.name}</summary><Tree nodes={node.children ?? []} open={open} /></details>
    : <button title={node.path} onClick={() => open(node)}>{node.name}</button>}</li>)}</ul>;
}
interface LayoutProps {
  node: Layout; focused: string; focus: (id: string) => void; label: (tab: string) => string;
  content: (tab: string) => React.ReactNode; close: (tab: string) => void;
  drop: (tab: string, target: string, before?: string) => void; activate: (group: string, tab: string) => void;
  dock: (tab: string, target: string, edge: Edge) => void;
  resize: (id: string, ratio: number) => void;
}
function LayoutView(props: LayoutProps) {
  const { node } = props;
  const host = useRef<HTMLDivElement>(null);
  const [edge, setEdge] = useState<Edge | null>(null);
  const [overPane, setOverPane] = useState(false);
  const [dropBefore, setDropBefore] = useState<string | null>(null);
  useEffect(() => {
    const clear = () => { setEdge(null); setOverPane(false); setDropBefore(null); };
    window.addEventListener("dragend", clear);
    return () => window.removeEventListener("dragend", clear);
  }, []);
  if (isGroup(node)) return <section className={`group ${props.focused === node.id ? "focused" : ""}`} onFocusCapture={() => props.focus(node.id)}>
    <div className="tabs" role="tablist" aria-label="Pane tabs"
      onDragOver={event => { if (event.dataTransfer.types.includes("application/x-hickory-tab")) { event.preventDefault(); event.dataTransfer.dropEffect = "move"; } }}
      onDrop={event => { event.preventDefault(); setDropBefore(null); props.drop(event.dataTransfer.getData("application/x-hickory-tab"), node.id); }}>

      {node.tabs.map(tab => <div className="tab" data-active={tab === node.active} data-drop-before={dropBefore === tab} key={tab} draggable
        onDragStart={event => { event.dataTransfer.effectAllowed = "move"; event.dataTransfer.setData("application/x-hickory-tab", tab); }}
        onDragEnd={() => setDropBefore(null)}
        onDragLeave={() => setDropBefore(current => current === tab ? null : current)}
        onDragOver={event => { if (event.dataTransfer.types.includes("application/x-hickory-tab")) { event.preventDefault(); event.stopPropagation(); event.dataTransfer.dropEffect = "move"; setDropBefore(tab); } }}
        onDrop={event => { event.preventDefault(); event.stopPropagation(); setDropBefore(null); props.drop(event.dataTransfer.getData("application/x-hickory-tab"), node.id, tab); }}>
        <button role="tab" aria-selected={tab === node.active} onClick={() => props.activate(node.id, tab)}>{props.label(tab)}</button>
        <button aria-label={`Close ${props.label(tab)}`} onClick={() => props.close(tab)}>×</button>
      </div>)}
    </div>
    <div className="pane" role="tabpanel" data-drop-edge={overPane ? edge ?? "center" : undefined} onPointerDown={() => props.focus(node.id)}
      onDragOverCapture={event => {
        if (!event.dataTransfer.types.includes("application/x-hickory-tab")) return;
        event.preventDefault(); event.stopPropagation(); setOverPane(true);
        const rect = event.currentTarget.getBoundingClientRect();
        setEdge(dropEdge((event.clientX - rect.left) / rect.width, (event.clientY - rect.top) / rect.height));
        event.dataTransfer.dropEffect = "move";
      }}
      onDragLeave={event => { if (!(event.relatedTarget instanceof Node) || !event.currentTarget.contains(event.relatedTarget)) { setEdge(null); setOverPane(false); } }}
      onDropCapture={event => {
        if (!event.dataTransfer.types.includes("application/x-hickory-tab")) return;
        event.preventDefault(); event.stopPropagation(); setEdge(null); setOverPane(false);
        const rect = event.currentTarget.getBoundingClientRect();
        const selected = dropEdge((event.clientX - rect.left) / rect.width, (event.clientY - rect.top) / rect.height);
        const tab = event.dataTransfer.getData("application/x-hickory-tab");
        if (selected) props.dock(tab, node.id, selected); else props.drop(tab, node.id);
      }}>
      {node.active ? props.content(node.active) : <p className="empty">Open a file or move a tab here</p>}
      {overPane && <div className={`pane-drop-preview ${edge ?? "center"}`} aria-hidden="true" />}
    </div>
  </section>;
  const horizontal = node.axis === "horizontal";
  function resize(event: React.PointerEvent<HTMLDivElement>) {
    event.currentTarget.setPointerCapture(event.pointerId);
    const rect = host.current!.getBoundingClientRect();
    const track = (move: PointerEvent) => props.resize(node.id, Math.max(.1, Math.min(.9, horizontal ? (move.clientX - rect.left) / rect.width : (move.clientY - rect.top) / rect.height)));
    const end = () => { window.removeEventListener("pointermove", track); window.removeEventListener("pointerup", end); window.removeEventListener("pointercancel", end); };
    window.addEventListener("pointermove", track); window.addEventListener("pointerup", end); window.addEventListener("pointercancel", end);
  }
  return <div ref={host} className={`split ${node.axis}`} style={{ gridTemplateColumns: horizontal ? `${node.ratio}fr 5px ${1 - node.ratio}fr` : undefined, gridTemplateRows: horizontal ? undefined : `${node.ratio}fr 5px ${1 - node.ratio}fr` }}>
    <LayoutView {...props} node={node.first} />
    <div className="resize" role="separator" tabIndex={0} aria-label="Resize panes" aria-orientation={horizontal ? "vertical" : "horizontal"} aria-valuenow={Math.round(node.ratio * 100)}
      onPointerDown={resize} onKeyDown={event => { const delta = ["ArrowRight", "ArrowDown"].includes(event.key) ? .05 : ["ArrowLeft", "ArrowUp"].includes(event.key) ? -.05 : 0; if (delta) { event.preventDefault(); props.resize(node.id, Math.max(.1, Math.min(.9, node.ratio + delta))); } }} />
    <LayoutView {...props} node={node.second} />
  </div>;
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/App.tsx"><ui:paste select="#simple-e8a50a27b2-0"/><ui:paste select="#simple-e8a50a27b2-1"/></ui:file>

</ui:doc>
