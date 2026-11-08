<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Compose only files, documents and agent panes

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The shell owns layout and commands. Documents remain in the buffer store when hidden or moved. File reads publish into their requested tab group without changing the currently focused group when a late reply arrives.

Pane content classifies internal tab drags before embedded text controls can consume their payload. An edge drop invokes docking; the center appends the existing tab to this group without adding a split. The half-pane preview is pointer-transparent and never reflows the content. Recompute the edge at drop time rather than trusting an old hover position.

Each tab accepts a drop before itself; the surrounding strip remains the append target. Stop tab-drop bubbling so its parent cannot append the same tab afterward. An absolutely positioned insertion mark does not change tab widths or move editor content. Only layout identities change.

<ui:copy id="simple-e8a50a27b2-0">
import { useEffect, useRef, useState } from "react";
import { api } from "../api/client";
import type { FileNode } from "../api/types";
import { Buffers } from "./buffers";
import { Document } from "./Document";
import { Terminal, TerminalSession } from "./Terminal";
import { Agent, AgentSession } from "./Agent";
import { activate, groups, initialLayout, isGroup, move, remove, split, update, dock, dropEdge, type Edge, type Axis, type Layout } from "./layout";
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
  async function loadFiles() {
    try { const response = await api.files(); setTree(response.tree); setStatus(response.root); }
    catch (error) { setNotice(`${String(error)} Start the Hickory backend, then choose Refresh files.`); }
  }
  useEffect(() => { void loadFiles(); }, []);
  useEffect(() => {
    const warn = (event: BeforeUnloadEvent) => { if (agent.prompt.length > 0 || terminal.input.length > 0 || agent.busy || terminal.sending || [...buffers.values.values()].some(buffer => buffer.dirty || buffer.saving)) event.preventDefault(); };
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
    const save = (event: KeyboardEvent) => { if ((event.metaKey || event.ctrlKey) && event.key === "s") { event.preventDefault(); void buffer?.save(); } };
    window.addEventListener("keydown", save); return () => window.removeEventListener("keydown", save);
  }, [buffer]);
</ui:copy>

Menu and tab-strip commands expose the same operations as native menus. Empty leaves accept drops. Recursive splits retain arbitrary arrangements; resizing updates only their proportions.

<ui:copy id="simple-e8a50a27b2-1">
  const label = (tab: string) => tab === "files" ? "Files" : tab === "agent" ? "Agent" : tab === "terminal" ? "Terminal" : `${tab.split("/").at(-1)}${buffers.values.get(tab)?.dirty ? " •" : ""}`;
  function content(tab: string) {
    if (tab === "files") return <div className="file-tree" aria-label="Files"><Tree nodes={tree} open={open} /></div>;
    if (tab === "terminal") return <Terminal session={terminal} />;
    if (tab === "agent") return <Agent session={agent} />;
    const document = buffers.values.get(tab);
    return document ? <Document buffer={document} /> : null;
  }
  return <main>
    <nav aria-label="Menu" onClick={event => { if ((event.target as HTMLElement).closest("button")) event.currentTarget.querySelectorAll("details[open]").forEach(menu => menu.removeAttribute("open")); }}><details><summary>File</summary><div className="menu">
      <button onClick={() => { void loadFiles(); }}>Refresh files</button>
      <button disabled={!buffer || buffer.readonly || !buffer.dirty || buffer.saving} onClick={() => { void buffer?.save(); }}>Save</button>
      <button disabled={!active} onClick={() => active && close(active)}>Close tab</button>
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
    <footer role="status">{notice ? `${notice} · ` : ""}{buffer ? `${buffer.file.path} · ${buffer.readonly ? "Read-only" : buffer.saving ? "Saving…" : buffer.dirty ? "Modified" : "Saved"}` : status}</footer>
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
