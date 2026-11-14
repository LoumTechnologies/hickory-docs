import { SourceTestGutterProvider, SOURCE_TEST_INSPECTION, type SourceTestInspection } from "./SourceTestGutters";
import {SourceCoverageProvider,SOURCE_COVERAGE_INSPECTION} from "./SourceCoverage";
import {SimpleCoverageView} from "./SimpleCoverageView";
import type {CoverageSourceNavigation} from "../coverage/CoverageSurface";
import { TestSourcesView } from "./TestSourcesView";
import type { TestSourcePoint } from "../api/testSources";
import { workspaceCommands, type CommandContext } from "./WorkspaceCommands";
import { SearchEverywhereView } from "./SearchEverywhereView";
import { WorkspaceAnalysis as WorkspaceAnalysisModel } from "./WorkspaceAnalysis";
import { runEditorCommand } from "./Document";
import { WorkspaceChanges, requestWorkspaceChange } from "./WorkspaceChanges";
import { WorkspaceAnalysisView, requestWorkspaceAnalysis } from "./WorkspaceAnalysisView";
import { WorkspaceSearch } from "./WorkspaceSearch";
import { SimpleLanguages } from "./WorkspaceLanguages";
import { WorkspaceProblems, requestProblemNavigation } from "./Problems";
import { NavigationTrail, semanticPath, type SemanticLocation } from "./Intelligence";
import { documentUri } from "../lsp/documentUri";
import { utf16ToPosition } from "../lsp/positions";
import { editorForBuffer } from "./Document";
import { TooltipLayer } from "../components/TooltipLayer";
import { useEffect, useRef, useState } from "react";
import { api } from "../api/client";
import { PlainDebugHosts } from "../debug/plainDebugHosts";
import { onOpenLocation, openLocation, revealLine } from "../lib/revealLine";
import type { FileNode } from "../api/types";
import { Buffer, Buffers } from "./buffers";
import { RichOwners } from "./RichInline";
import { enterMode, type EnterMode } from "./SmartEnter";
import { Document, semanticQueries, openSemanticQuery, openFind, wordNavigationMode, type WordNavigationMode } from "./Document";
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
  const [workspaceFiles, setWorkspaceFiles] = useState<FileNode[]>([]);
  const [notice, setNotice] = useState("");
  const [analysis] = useState(() => new WorkspaceAnalysisModel());
  const [showCoverage,setShowCoverage]=useState(false);
  const [showTests,setShowTests]=useState(false);
  const [testInspection,setTestInspection]=useState<SourceTestInspection|null>(null);
  const [workspaceRoot,setWorkspaceRoot]=useState("");
  const workspaceIdentity=useRef(workspaceRoot);workspaceIdentity.current=workspaceRoot;
  const [showEverywhere, setShowEverywhere] = useState(false);
  const [showProblems, setShowProblems] = useState(false);
  const [showWorkspaceSearch, setShowWorkspaceSearch] = useState(false);
  const [trail] = useState(() => new NavigationTrail());
  const [, refreshTrail] = useState(0);
  const [status, setStatus] = useState("Choose a file");
  const [, refresh] = useState(0);
  const [buffers] = useState(() => new Buffers(() => refresh(value => value + 1)));
  const requested = useRef(new Map<string, string>());
  function newTab() {
    const path = `Untitled-${crypto.randomUUID()}`;
    buffers.values.set(path, new Buffer({ name: `Untitled ${[...buffers.values.keys()].filter(key => key.startsWith("Untitled-")).length + 1}`, path, dir: false }, "", "", () => refresh(value => value + 1)));
    openTab(path);
  }
  function findPath(path: string, nodes = workspaceFiles): FileNode | undefined {
    for (const node of nodes) { if (node.path === path) return node; const found = findPath(path, node.children ?? []); if (found) return found; }
  }
  async function openPath(directory = false) {
    const path = window.prompt(directory ? "Open Directory — path in this workspace (empty for root)" : "Open File — path in this workspace");
    if (path === null) return;
    if (directory && !path) { await loadFiles(); openTab("files"); return; }
    const node = findPath(path);
    if (!node || node.dir !== directory) { setNotice("Path not found. Refresh and choose a path in this workspace."); return; }
    if (directory) { setTree(node.children ?? []); openTab("files"); } else await openFileVisit(node);
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
    try { const response = await api.files(); setTree(response.tree); setWorkspaceFiles(response.tree); setWorkspaceRoot(response.root); setStatus(response.root); }
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
  function currentLocation(): SemanticLocation | null {
    const view = buffer && editorForBuffer(buffer)?.view; if (!buffer || !view) return null;
    const start = utf16ToPosition(buffer.text, view.state.selection.main.head);
    return { uri: documentUri(buffer.file.path), range: { start, end: start } };
  }
  function navigateHistory(direction: -1 | 1) {
    const location = trail.step(direction, currentLocation()); refreshTrail(value => value + 1);
    const path = location && semanticPath(location.uri);
    if (path && location) openLocation(path, location.range.start.line + 1, location.range.start.character, false);
  }
  useEffect(() => onOpenLocation(location => {
    const departed = currentLocation();
    const find = (nodes: FileNode[]): FileNode | undefined => {
      for (const node of nodes) { if (node.path === location.path) return node; const child = find(node.children ?? []); if (child) return child; }
    };
    const file = buffers.values.get(location.path)?.file ?? find(workspaceFiles);
    if (file) { void open(file).then(() => {
      const target = buffers.values.get(file.path); if (!target) return;
      if (location.expectedSource !== undefined && target.text !== location.expectedSource) { setNotice("Source changed. Refresh the query before navigating."); return; }
      if (location.expectedSource === undefined && location.recordHistory !== false && location.character !== undefined && target.dirty && file.path !== buffer?.file.path) { setNotice(`Save ${file.path} and query again before using a cross-file source position.`); return; }
      if (location.recordHistory !== false) {
        const start = { line: location.line - 1, character: location.character ?? 0 };
        trail.record(departed, { uri: documentUri(file.path), range: { start, end: start } }); refreshTrail(value => value + 1);
      }
      revealLine(file.path, location.line, location.character);
    }); } else setNotice(`Location not found: ${location.path}. Refresh files and query again.`);
  }), [workspaceFiles, layout, focused]);
  useEffect(()=>{const inspect=(event:Event)=>{const path=(event as CustomEvent<{path:string}>).detail.path,file=buffers.values.get(path)?.file??findPath(path),identity=workspaceIdentity.current;if(!file)return;void open(file).then(()=>{if(workspaceIdentity.current===identity)setShowCoverage(true);});};window.addEventListener(SOURCE_COVERAGE_INSPECTION,inspect);return()=>window.removeEventListener(SOURCE_COVERAGE_INSPECTION,inspect);},[workspaceFiles,layout,focused]);
  useEffect(()=>{const inspect=(event:Event)=>{const request=(event as CustomEvent<SourceTestInspection>).detail;const file=buffers.values.get(request.path)?.file??findPath(request.path);if(!file)return;void open(file).then(()=>{setTestInspection(request);setShowTests(true);});};window.addEventListener(SOURCE_TEST_INSPECTION,inspect);return()=>window.removeEventListener(SOURCE_TEST_INSPECTION,inspect);},[workspaceFiles,layout,focused]);
  async function openFileVisit(file: FileNode) {
    const departed = currentLocation(); await open(file);
    const target = buffers.values.get(file.path); if (!target) return;
    const view = editorForBuffer(target)?.view;
    const start = view ? utf16ToPosition(target.text, view.state.selection.main.head) : { line: 0, character: 0 };
    trail.record(departed, { uri: documentUri(file.path), range: { start, end: start } }); refreshTrail(value => value + 1);
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
  const label = (tab: string) => tab === "files" ? "Files" : tab === "agent" ? "Agent" : tab === "terminal" ? "Terminal" : `${buffers.values.get(tab)?.file.name ?? tab.split("/").at(-1)}${buffers.values.get(tab)?.dirty ? " •" : ""}`;
  function content(tab: string) {
    if (tab === "files") return <div className="file-tree" aria-label="Files"><Tree nodes={tree} open={openFileVisit} /></div>;
    if (tab === "terminal") return <Terminal session={terminal} />;
    if (tab === "agent") return <Agent session={agent} />;
    const document = buffers.values.get(tab);
    return document ? <Document key={document.file.path} buffer={document} /> : null;
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
      if (event.ctrlKey) return;
      moved(event);
      const path = event.composedPath();
      const target = path.find(node => node instanceof HTMLElement &&
        (node.dataset.wheelOwner === "true" || /(auto|scroll)/.test(getComputedStyle(node).overflow + getComputedStyle(node).overflowY + getComputedStyle(node).overflowX) &&
        (node.scrollHeight > node.clientHeight || node.scrollWidth > node.clientWidth))) as HTMLElement | undefined;
      if (!owner?.isConnected) owner = target ?? null;
      if (owner && target && owner !== target && path.includes(owner)) {
        event.preventDefault();
        event.stopImmediatePropagation();
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
  const context:CommandContext={editor:!!buffer,writable:!!buffer&&!buffer.readonly,dirty:buffer?.dirty??false,saving:buffer?.saving??false,back:trail.canBack,forward:trail.canForward,tab:!!active,groups:groups(layout).length};
  const contextRef=useRef(context);contextRef.current=context;
  const handlers:Record<string,()=>void>={
    "file.open":()=>{void openPath();},"file.directory":()=>{void openPath(true);},"file.new":newTab,"file.window":()=>{window.open(window.location.href,"_blank","noopener");},
    "file.save":()=>{void saveCurrent();},"file.saveAs":()=>{void saveAs();},"file.saveAll":()=>{void saveAll();},"file.refresh":()=>{void loadFiles();},"file.close":()=>{if(active)close(active);},
    "shell.search":()=>setShowEverywhere(true),"edit.find":()=>{if(buffer)openFind(buffer);},"workspace.find":()=>setShowWorkspaceSearch(true),
    "symbol.rename":()=>requestWorkspaceChange("rename"),"symbol.actions":()=>requestWorkspaceChange("actions"),"document.format":()=>requestWorkspaceChange("format"),"workspace.undo":()=>requestWorkspaceChange("undo"),"workspace.redo":()=>requestWorkspaceChange("redo"),
    "workspace.coverage":()=>setShowCoverage(true),"workspace.tests":()=>setShowTests(true),"workspace.problems":()=>setShowProblems(true),"workspace.analysis":requestWorkspaceAnalysis,"problem.next":()=>requestProblemNavigation(1),"problem.previous":()=>requestProblemNavigation(-1),"navigation.back":()=>navigateHistory(-1),"navigation.forward":()=>navigateHistory(1),
    "view.files":()=>openTab("files"),"view.agent":()=>openTab("agent"),"view.terminal":()=>{void openTerminal();},"view.splitRight":()=>divide("horizontal"),"view.splitBelow":()=>divide("vertical"),"view.move":nextGroup,
  };
  for(const query of semanticQueries)handlers[`semantic.${query.kind}`]=()=>{if(buffer){const text=query.kind==="symbols"?window.prompt("Search workspace symbols",""):"";if(text!==null)openSemanticQuery(buffer,query.kind,text);}};
  for(const command of ["tools","rectangle","cursorBelow","cursorAbove"] as const)handlers[`editor.${command}`]=()=>{if(buffer&&!runEditorCommand(buffer,command))setNotice("Editor command unavailable. Wait for the editor or language service, then retry.");};
  const handlersRef=useRef(handlers);handlersRef.current=handlers;
  const commands=workspaceCommands(()=>contextRef.current,Object.fromEntries(Object.keys(handlers).map(id=>[id,()=>handlersRef.current[id]?.()])));
  const commandMenu=(group:string)=>commands.filter(command=>command.group===group&&command.id in handlers).map(command=><button key={command.id} disabled={command.reason!==null} title={command.reason??command.label} onClick={command.run}>{command.label}</button>);
  return <SimpleLanguages buffers={buffers}><SourceTestGutterProvider workspace={workspaceRoot}><SourceCoverageProvider workspace={workspaceRoot}><main>
    {/* Shared gutter controls declare data-tip; mount their delegated reader here. */}
    <TooltipLayer />
    <PlainDebugHosts />
    <RichOwners buffers={buffers} />
    <nav aria-label="Menu" onClick={event => { if ((event.target as HTMLElement).closest("button")) event.currentTarget.querySelectorAll("details[open]").forEach(menu => menu.removeAttribute("open")); }}><details><summary>File</summary><div className="menu">
      {commandMenu("File")}
    </div></details><details><summary>Edit</summary><div className="menu">
      {commandMenu("Edit")}
      <EditorSettings />
    </div></details><details><summary>View</summary><div className="menu">
      {commandMenu("View")}
    </div></details><span className="app-name">Hickory Simple</span></nav>
    <div className="workspace"><LayoutView node={layout} focused={focused} focus={setFocused} label={label} content={content} close={close}
      drop={(tab, target, before) => { setLayout(current => move(current, tab, target, before)); setFocused(target); }}
      activate={(group, tab) => { setLayout(current => activate(current, group, tab)); setFocused(group); }}
      dock={(tab, target, edge) => {
        const id = crypto.randomUUID(), next = dock(layout, tab, target, edge, id);
        if (next !== layout) { setLayout(next); setFocused(id); }
      }}
      resize={(id, ratio) => setLayout(current => update(current, id, node => isGroup(node) ? node : { ...node, ratio }))} /></div>
    <SimpleCoverageView key={workspaceRoot} visible={showCoverage} workspace={workspaceRoot} path={buffer?.file.path??null} language={buffer?.file.language??"unsupported"} close={()=>{setShowCoverage(false);if(buffer)editorForBuffer(buffer)?.view.focus();}} navigate={async(location:CoverageSourceNavigation)=>{
      const before=workspaceIdentity.current,file=buffers.values.get(location.path)?.file??findPath(location.path);if(!file)throw new Error("Source no longer listed. Refresh the workspace.");
      const target=await buffers.open(file),source=target.text;
      if(before!==workspaceIdentity.current)throw new Error("Workspace changed. Reopen coverage.");
      if(location.line===null){await openFileVisit(file);setShowCoverage(false);return;}
      const digest=await crypto.subtle.digest("SHA-256",new TextEncoder().encode(source)),hash=[...new Uint8Array(digest)].map(byte=>byte.toString(16).padStart(2,"0")).join("");
      if(before!==workspaceIdentity.current||target.text!==source||hash!==location.sourceHash||location.line<1||location.line>source.split("\n").length)throw new Error("Source changed or report position is unavailable. Refresh coverage before navigating.");
      openLocation(location.path,location.line,0,true,source);setShowCoverage(false);
    }}/>
    <TestSourcesView inspection={testInspection} buffers={buffers} visible={showTests} close={()=>{setShowTests(false);if(buffer)editorForBuffer(buffer)?.view.focus();}} current={entry=>buffer?{workspace:workspaceRoot,request:{path:buffer.file.path,source:buffer.text,...(entry?{project_entry:entry}:{}),sources:Object.fromEntries([...buffers.values].filter(([,item])=>!item.readonly).map(([path,item])=>[path,item.text]))}}:null} navigate={async(point:TestSourcePoint)=>{
      const file=buffers.values.get(point.path)?.file??findPath(point.path);if(!file)throw new Error("Source no longer listed. Refresh files and candidates.");
      const target=await buffers.open(file),source=target.text;
      const bytes=await crypto.subtle.digest("SHA-256",new TextEncoder().encode(source));const hash=[...new Uint8Array(bytes)].map(byte=>byte.toString(16).padStart(2,"0")).join("");
      if(target.text!==source||hash!==point.hash.toLowerCase())throw new Error("Source changed. Refresh candidates before navigating.");
      openLocation(point.path,point.line,point.character,true,source);setShowTests(false);
    }} terminal={id=>{terminal.id=id;terminal.error="";terminal.publish();openTab("terminal");setShowTests(false);}} />
    <SearchEverywhereView visible={showEverywhere} setVisible={setShowEverywhere} buffers={buffers} files={workspaceFiles} analysis={analysis} commands={commands} openFile={path=>{const file=buffers.values.get(path)?.file??findPath(path);if(file)void openFileVisit(file);else setNotice("File no longer listed. Refresh the workspace.");}} focusCurrent={()=>{if(buffer)editorForBuffer(buffer)?.view.focus();}} />
    <WorkspaceChanges buffers={buffers} current={() => buffer} focus={() => { if (buffer) editorForBuffer(buffer)?.view.focus(); }} />
    <WorkspaceSearch visible={showWorkspaceSearch} close={() => { setShowWorkspaceSearch(false); if (buffer) editorForBuffer(buffer)?.view.focus(); }} buffers={buffers} files={workspaceFiles} />
    <footer role="status"><WorkspaceAnalysisView buffers={buffers} files={workspaceFiles} model={analysis} focusCurrent={() => { if (buffer) editorForBuffer(buffer)?.view.focus(); }} /><WorkspaceProblems buffers={buffers} visible={showProblems} setVisible={setShowProblems} current={currentLocation} focusCurrent={() => { if (buffer) editorForBuffer(buffer)?.view.focus(); }} />{notice ? `${notice} · ` : ""}{buffer ? `${buffer.file.name} · ${buffer.readonly ? "Read-only" : buffer.saving ? "Saving…" : buffer.dirty ? "Modified" : "Saved"}` : status}</footer>
  </main></SourceCoverageProvider></SourceTestGutterProvider></SimpleLanguages>;
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
