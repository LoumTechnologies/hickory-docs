<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Compose only files, documents and agent panes

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Workspace debugger hosts remain mounted above tab content, so stepping into another file or moving the launch tab cannot destroy its session. Cross-file stop requests open the existing tree entry and queue the adapter line for the mounted editor.

The shell owns layout and commands. Documents remain in the buffer store when hidden or moved. File reads publish into their requested tab group without changing the currently focused group when a late reply arrives.

Pane content classifies internal tab drags before embedded text controls can consume their payload. An edge drop invokes docking; the center appends the existing tab to this group without adding a split. The half-pane preview is pointer-transparent and never reflows the content. Recompute the edge at drop time rather than trusting an old hover position.

Each tab accepts a drop before itself; the surrounding strip remains the append target. Stop tab-drop bubbling so its parent cannot append the same tab afterward. An absolutely positioned insertion mark does not change tab widths or move editor content. Only layout identities change.

File commands act on the connected workspace: path entry opens a file or directory, New Tab (Ctrl+N or Cmd+N) keeps an unnamed buffer, and Save As exclusively creates a new destination before a hash-checked write. Existing destinations are never overwritten by Save As. Save All visits each named buffer; unnamed drafts ask for a path and failed saves retain their original edits. Refresh observes the tree without replacing open buffers.

A document component belongs to one buffer path. Switching tabs mounts the next component around its already cached editor; it does not reuse React state that still points to the previous file's editor. Without that identity boundary, a queued cross-file reveal can be consumed by the detached old editor before the destination mounts. The cache retains selection and undo when the original tab returns.

<ui:copy id="simple-e8a50a27b2-0">
import { SourceTestGutterProvider, SOURCE_TEST_INSPECTION, type SourceTestInspection } from "./SourceTestGutters";
import {SourceCoverageProvider,SOURCE_COVERAGE_INSPECTION} from "./SourceCoverage";
import {SimpleCoverageView} from "./SimpleCoverageView";
import type {CoverageSourceNavigation} from "../coverage/CoverageSurface";
import { TestSourcesView } from "./TestSourcesView";
import { SourceTemplatesView } from "./SourceTemplatesView";
import type { TestSourcePoint } from "../api/testSources";
import { workspaceCommands, type CommandContext } from "./WorkspaceCommands";
import { SearchEverywhereView } from "./SearchEverywhereView";
import { WorkspaceAnalysis as WorkspaceAnalysisModel } from "./WorkspaceAnalysis";
import { runEditorCommand,sourceSnippetContext } from "./Document";
import { WorkspaceChanges, requestWorkspaceChange } from "./WorkspaceChanges";
import { WorkspaceAnalysisView, requestWorkspaceAnalysis } from "./WorkspaceAnalysisView";
import { WorkspaceSearch } from "./WorkspaceSearch";
import {SourceRecoveryView} from "./SourceRecovery";
import {SourceHistoryView} from "./SourceHistoryView";
import {WorkspaceChangeHistory} from "./ChangePlan";
import {captureResourceRoster,resourceLayout,RetainedResourceTabs,retireCopiedBuffer,type ResourceEditorRoute} from "./ResourceEditors";
import {ConstructionReadingView,constructionPointOffset} from "./ConstructionReading";
import {GitAttributionView} from "./GitAttribution";
import {GitHistoryView,type GitHistoryRequest} from "./GitHistory";
import {GitReviewView} from "./GitReview";
import { SimpleLanguages } from "./WorkspaceLanguages";
import { WorkspaceProblems, requestProblemNavigation } from "./Problems";
import { NavigationTrail, semanticPath, type SemanticLocation } from "./Intelligence";
import { documentUri } from "../lsp/documentUri";
import { utf16ToPosition } from "../lsp/positions";
import { editorForBuffer } from "./Document";
import { TooltipLayer } from "../components/TooltipLayer";
import { useEffect, useRef, useState } from "react";
import { api,request } from "../api/client";
import { PlainDebugHosts,hasActivePlainDebugSession,retirePlainDebugIdentity } from "../debug/plainDebugHosts";
import { onOpenLocation, openLocation, revealLine } from "../lib/revealLine";
import type { FileNode } from "../api/types";
import { Buffer, Buffers, createSourceCopy } from "./buffers";
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
  const liveLayout=useRef(layout);liveLayout.current=layout;
  const [focused, setFocused] = useState("document-group");
  const [terminal] = useState(() => new TerminalSession());
  const [agent] = useState(() => new AgentSession());
  const [tree, setTree] = useState<FileNode[]>([]);
  const [workspaceFiles, setWorkspaceFiles] = useState<FileNode[]>([]);
  const [notice, setNotice] = useState("");
  const [analysis] = useState(() => new WorkspaceAnalysisModel());
  const [showCoverage,setShowCoverage]=useState(false);
  const [showTests,setShowTests]=useState(false);
  const [showTemplates,setShowTemplates]=useState(false);
  const [showRecovery,setShowRecovery]=useState(false);
  const [showConstruction,setShowConstruction]=useState(false);
  const [showReview,setShowReview]=useState(false);
  const [attributionPath,setAttributionPath]=useState<string|null>(null);
  const [showSourceHistory,setShowSourceHistory]=useState(false);
  const [showHistory,setShowHistory]=useState(false),[historyRequest,setHistoryRequest]=useState<GitHistoryRequest>({kind:'log',path:null});
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
  const [changeHistory]=useState(()=>new WorkspaceChangeHistory());
  const [buffers] = useState(() => new Buffers(() => refresh(value => value + 1)));
  const requested = useRef(new Map<string, string>());
  const [retainedTabs]=useState(()=>new RetainedResourceTabs());
  const bufferForTab=(tab:string)=>retainedTabs.buffers.get(tab)??buffers.values.get(tab);
  const resourcePlacements=useRef(new Map<Buffer,{group:string;index:number;active:boolean}>());
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
    if (path === source.file.path) { if(source.resourceDeleted)setNotice("This source was deleted. Choose a new unused path to save its retained text.");else await source.save(); return; }
    const text = source.text,editor=editorForBuffer(source),interaction=editor?.interaction,placement=liveLayout.current;
    const current=()=>liveLayout.current===placement&&!!editor&&editorForBuffer(source)===editor&&editor.view.dom.isConnected&&!editor.view.composing&&editor.interaction===interaction&&editor.view.state.doc.toString()===text;
    source.saving = true; source.publish();
    try {
      const copy=await createSourceCopy(buffers,source,path,request,current);
      await open(copy.file);
      // Existing sources keep their editor, selection and undo. A deleted or
      // unnamed tombstone can retire only after its exact text was copied.
      if(source.resourceDeleted||source.file.path.startsWith("Untitled-")){
        if(hasActivePlainDebugSession(source)){retainedTabs.noteCopy(source,text);setNotice("Copy saved. The original launch source remains readable until its debugger closes.");}
        else {const tabs=groups(liveLayout.current).flatMap(group=>group.tabs).filter(tab=>bufferForTab(tab)===source);if(retireCopiedBuffer(buffers,source,text)){retirePlainDebugIdentity(source);retainedTabs.retire(source);setLayout(current=>tabs.reduce((next,tab)=>remove(next,tab),current));}}
      }
      void loadFiles();

    } catch (error) { setNotice(`${String(error)} Original edits and the exact copy request are retained. Inspect local history before changing the destination; retry only while the captured editors remain current.`); }
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
    try { const response = await api.files(); buffers.workspace=response.root_path??"";buffers.workspaceId=response.workspace_id??"";setTree(response.tree); setWorkspaceFiles(response.tree); setWorkspaceRoot(response.root); setStatus(response.root); }
    catch (error) { setNotice(`${String(error)} Start the Hickory backend, then choose Refresh files.`); }
  }
  useEffect(() => { void loadFiles(); }, []);
  useEffect(() => {
    const warn = (event: BeforeUnloadEvent) => { if (agent.prompt.length > 0 || agent.busy || buffers.hasUnsavedChanges) event.preventDefault(); };
    window.addEventListener("beforeunload", warn); return () => window.removeEventListener("beforeunload", warn);
  }, [buffers, agent, terminal]);
  const currentGroup = groups(layout).find(group => group.id === focused) ?? groups(layout)[0];
  const active = currentGroup.active;
  const buffer = active ? bufferForTab(active) : undefined;
  async function openTerminal() { openTab("terminal"); await terminal.open(); }
  function openTab(tab: string) {
    const owner = groups(layout).find(group => group.tabs.includes(tab));
    const target = owner?.id ?? currentGroup.id;
    setLayout(current => activate(current, target, tab)); setFocused(target);
  }
  async function open(file: FileNode,isCurrent:()=>boolean=()=>true) {
    if(!isCurrent())return;
    const target = groups(layout).find(group => group.id === focused && !group.tabs.includes("files"))?.id
      ?? groups(layout).find(group => !group.tabs.includes("files"))?.id ?? currentGroup.id;
    requested.current.set(target, file.path);
    setFocused(target);
    try {
      await buffers.open(file);
      if (!isCurrent()||requested.current.get(target) !== file.path) return;
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
    const departed = currentLocation(),workspace=workspaceIdentity.current,origin=buffer&&editorForBuffer(buffer),originInteraction=origin?.interaction??0;
    const current=()=>workspaceIdentity.current===workspace&&(!location.capture||location.capture.current()&&(!origin||origin.interaction===originInteraction));
    if(!current())return;
    const find = (nodes: FileNode[]): FileNode | undefined => {
      for (const node of nodes) { if (node.path === location.path) return node; const child = find(node.children ?? []); if (child) return child; }
    };
    const file = buffers.values.get(location.path)?.file ?? find(workspaceFiles);
    if (file) { void open(file,current).then(async() => {
      const target = buffers.values.get(file.path); if (!target||!current()) return;
      if(location.capture){
        if(!location.capture.hash){setNotice(location.capture.sourceReference&&location.capture.sourceReference>0?"Adapter-owned source bytes have not been read; opened workspace text without a precise position.":location.capture.positionKnown===false?"Frame position is unavailable; opened readable source text only.":"Opened source text only; the debugger has no captured mapping identity for this frame.");return;}
        const source=target.text,interaction=editorForBuffer(target)?.interaction??0;
        for(const character of source)if(character.length===1&&character.charCodeAt(0)>=0xD800&&character.charCodeAt(0)<=0xDFFF){setNotice("Source text has an incomplete Unicode scalar; opened without a debugger position.");return;}
        const digest=await crypto.subtle.digest("SHA-256",new TextEncoder().encode(source)),hash=[...new Uint8Array(digest)].map(byte=>byte.toString(16).padStart(2,"0")).join("");
        const exact=()=>current()&&[...requested.current.values()].includes(file.path)&&buffers.values.get(file.path)===target&&target.text===source&&(editorForBuffer(target)?.interaction??0)===interaction;
        if(!exact()){setNotice("Debugger position or editor interaction changed while checking source. Select the frame again.");return;}
        if(hash!==location.capture.hash){setNotice("Source differs from the captured debugger mapping; opened without a precise position.");return;}
        if(!Number.isInteger(location.line)||location.line<1||location.line>source.split("\n").length){setNotice("Captured debugger line does not fit this source; opened text only.");return;}
        revealLine(file.path,location.line,0,{source,current:exact});return;
      }
      if (location.expectedSource !== undefined && target.text !== location.expectedSource) { setNotice("Source changed. Refresh the query before navigating."); return; }
      if (location.expectedSource === undefined && location.recordHistory !== false && location.character !== undefined && target.dirty && file.path !== buffer?.file.path) { setNotice(`Save ${file.path} and query again before using a cross-file source position.`); return; }
      if (location.recordHistory !== false) {
        const start = { line: location.line - 1, character: location.character ?? 0 };
        trail.record(departed, { uri: documentUri(file.path), range: { start, end: start } }); refreshTrail(value => value + 1);
      }
      revealLine(file.path, location.line, location.character,location.expectedSource===undefined?undefined:{source:location.expectedSource,current:()=>workspaceIdentity.current===workspace&&buffers.values.get(file.path)===target&&target.text===location.expectedSource});
    }).catch(error=>{if(current())setNotice(String(error)+" Source text remains available; no precise location was applied.");}); } else setNotice(location.capture?.sourceReference&&location.capture.sourceReference>0?"This frame uses adapter-owned source; no matching workspace text is available.":`Location not found: ${location.path}. Refresh files and query again.`);
  },true), [workspaceFiles, layout, focused]);
  useEffect(()=>{const inspect=(event:Event)=>{const path=(event as CustomEvent<{path:string}>).detail.path,file=buffers.values.get(path)?.file??findPath(path),identity=workspaceIdentity.current;if(!file)return;void open(file).then(()=>{if(workspaceIdentity.current===identity)setShowCoverage(true);});};window.addEventListener(SOURCE_COVERAGE_INSPECTION,inspect);return()=>window.removeEventListener(SOURCE_COVERAGE_INSPECTION,inspect);},[workspaceFiles,layout,focused]);
  useEffect(()=>{const inspect=(event:Event)=>{const request=(event as CustomEvent<SourceTestInspection>).detail,owner=request.owner??buffers.values.get(request.path),namespace=buffers.workspace;const current=()=>!!owner&&!owner.resourceDeleted&&buffers.values.get(request.path)===owner&&owner.file.path===request.path&&owner.namespace===namespace&&buffers.workspace===namespace&&(request.revision===undefined||owner.sourceRevision===request.revision);const file=owner?.file;if(!file||!current())return;void open(file,current).then(()=>{if(current()){setTestInspection(request);setShowTests(true);}});};window.addEventListener(SOURCE_TEST_INSPECTION,inspect);return()=>window.removeEventListener(SOURCE_TEST_INSPECTION,inspect);},[workspaceFiles,layout,focused]);
  async function openFileVisit(file: FileNode) {
    const departed = currentLocation(); await open(file);
    const target = buffers.values.get(file.path); if (!target) return;
    const view = editorForBuffer(target)?.view;
    const start = view ? utf16ToPosition(target.text, view.state.selection.main.head) : { line: 0, character: 0 };
    trail.record(departed, { uri: documentUri(file.path), range: { start, end: start } }); refreshTrail(value => value + 1);
  }
  function close(tab: string) {
    for(const [group,path] of requested.current)if(path===tab)requested.current.delete(group);
    const candidate = bufferForTab(tab);
    if(candidate?.resourceDeleted&&hasActivePlainDebugSession(candidate)){setNotice("Stop the debugger before closing its retained deleted source. Its existing controls remain available here.");return;}
    if (candidate?.saving || candidate?.dirty&&!retainedTabs.copied(candidate)) { setNotice("Unsaved edits retained. Save this document before closing its tab."); return; }
    if(candidate?.resourceDeleted&&retainedTabs.copied(candidate)&&!hasActivePlainDebugSession(candidate)){retireCopiedBuffer(buffers,candidate,candidate.text);retirePlainDebugIdentity(candidate);retainedTabs.retire(candidate);}
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
  const label = (tab: string) => tab === "files" ? "Files" : tab === "agent" ? "Agent" : tab === "terminal" ? "Terminal" : `${bufferForTab(tab)?.file.name ?? tab.split("/").at(-1)}${bufferForTab(tab)?.resourceDeleted?" (deleted source)":""}${bufferForTab(tab)?.dirty ? " •" : ""}`;
  function content(tab: string) {
    if (tab === "files") return <div className="file-tree" aria-label="Files"><Tree nodes={tree} open={openFileVisit} /></div>;
    if (tab === "terminal") return <Terminal session={terminal} />;
    if (tab === "agent") return <Agent session={agent} />;
    const document = bufferForTab(tab);
    return document ? <Document key={document.file.path} buffer={document} current={()=>!document.resourceDeleted&&document.namespace===buffers.workspace&&buffers.values.get(document.file.path)===document} /> : null;
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
  const context:CommandContext={...sourceSnippetContext(buffer),editor:!!buffer,writable:!!buffer&&!buffer.readonly&&!buffer.resourceDeleted,dirty:buffer?.dirty??false,saving:buffer?.saving??false,back:trail.canBack,forward:trail.canForward,tab:!!active,groups:groups(layout).length};
  const contextRef=useRef(context);contextRef.current=context;
  const handlers:Record<string,()=>void>={
    "file.open":()=>{void openPath();},"file.directory":()=>{void openPath(true);},"file.new":newTab,"file.window":()=>{window.open(window.location.href,"_blank","noopener");},
    "file.save":()=>{void saveCurrent();},"file.saveAs":()=>{void saveAs();},"file.saveAll":()=>{void saveAll();},"file.refresh":()=>{void loadFiles();},"file.close":()=>{if(active)close(active);},
    "shell.search":()=>setShowEverywhere(true),"edit.find":()=>{if(buffer)openFind(buffer);},"workspace.find":()=>setShowWorkspaceSearch(true),
    "symbol.rename":()=>requestWorkspaceChange("rename"),"symbol.actions":()=>requestWorkspaceChange("actions"),"document.format":()=>requestWorkspaceChange("format"),"document.cleanup":()=>requestWorkspaceChange("cleanup"),"workspace.undo":()=>requestWorkspaceChange("undo"),"workspace.redo":()=>requestWorkspaceChange("redo"),
    "document.attribution":()=>{if(buffer){setShowReview(false);setShowHistory(false);setAttributionPath(buffer.file.path);}},"workspace.review":()=>{setAttributionPath(null);setShowHistory(false);setShowReview(true);},"workspace.history":()=>{setAttributionPath(null);setHistoryRequest({kind:'log',path:null});setShowReview(false);setShowHistory(true);},"document.history":()=>{setAttributionPath(null);setHistoryRequest({kind:'log',path:buffer?.file.path??null});setShowReview(false);setShowHistory(true);},"workspace.coverage":()=>setShowCoverage(true),"workspace.recovery":()=>setShowRecovery(true),"workspace.construction":()=>setShowConstruction(true),"workspace.tests":()=>setShowTests(true),"source.templates":()=>setShowTemplates(true),"source.history":()=>setShowSourceHistory(true),"workspace.problems":()=>setShowProblems(true),"workspace.analysis":requestWorkspaceAnalysis,"problem.next":()=>requestProblemNavigation(1),"problem.previous":()=>requestProblemNavigation(-1),"navigation.back":()=>navigateHistory(-1),"navigation.forward":()=>navigateHistory(1),
    "view.files":()=>openTab("files"),"view.agent":()=>openTab("agent"),"view.terminal":()=>{void openTerminal();},"view.splitRight":()=>divide("horizontal"),"view.splitBelow":()=>divide("vertical"),"view.move":nextGroup,
  };
  for(const query of semanticQueries)handlers[`semantic.${query.kind}`]=()=>{if(buffer){const text=query.kind==="symbols"?window.prompt("Search workspace symbols",""):"";if(text!==null)openSemanticQuery(buffer,query.kind,text);}};
  for(const command of ["tools","undo","redo","rectangle","cursorBelow","cursorAbove","snippetNext","snippetPrevious","snippetChoices","snippetFinish"] as const)handlers[`editor.${command}`]=()=>{if(buffer&&!runEditorCommand(buffer,command))setNotice("Editor command unavailable. Wait for the editor or language service, then retry.");};
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
    <SourceTemplatesView visible={showTemplates} close={()=>{setShowTemplates(false);if(buffer)editorForBuffer(buffer)?.view.focus();}} editor={()=>{const owner=buffer,held=owner?editorForBuffer(owner):undefined;if(!owner||!held)return null;const mount=held.mount;return {buffer:owner,buffers,view:held.view,mounted:()=>editorForBuffer(owner)===held&&held.mount===mount,interaction:()=>held.interaction};}} />
    <TestSourcesView inspectDebugger={owner=>{if(owner.namespace!==buffers.workspace)throw Error('The original debugger belongs to a different workspace namespace. Its record remains readable.');const tab=[...buffers.values,...retainedTabs.buffers].find(([,held])=>held===owner)?.[0];if(!tab)throw Error('The original debugger editor is no longer retained. No replacement path was opened.');openTab(tab);setShowTests(false);}} inspection={testInspection} buffers={buffers} visible={showTests} close={()=>{setShowTests(false);if(buffer)editorForBuffer(buffer)?.view.focus();}} current={entry=>buffer?{workspace:workspaceRoot,owner:buffer,namespace:buffers.workspace,revision:buffer.sourceRevision,executionCurrent:!buffer.dirty&&!buffer.saving&&!buffer.resourceDeleted&&buffer.namespace===buffers.workspace&&buffers.values.get(buffer.file.path)===buffer,request:{path:buffer.file.path,source:buffer.text,...(entry?{project_entry:entry}:{}),sources:Object.fromEntries([...buffers.values].filter(([,item])=>!item.readonly).map(([path,item])=>[path,item.text]))}}:null} navigate={async(point:TestSourcePoint)=>{
      const file=buffers.values.get(point.path)?.file??findPath(point.path);if(!file)throw new Error("Source no longer listed. Refresh files and candidates.");
      const target=await buffers.open(file),source=target.text;
      const bytes=await crypto.subtle.digest("SHA-256",new TextEncoder().encode(source));const hash=[...new Uint8Array(bytes)].map(byte=>byte.toString(16).padStart(2,"0")).join("");
      if(target.text!==source||hash!==point.hash.toLowerCase())throw new Error("Source changed. Refresh candidates before navigating.");
      openLocation(point.path,point.line,point.character,true,source);setShowTests(false);
    }} terminal={id=>{terminal.id=id;terminal.error="";terminal.publish();openTab("terminal");setShowTests(false);}} />
    <SearchEverywhereView visible={showEverywhere} setVisible={setShowEverywhere} buffers={buffers} files={workspaceFiles} analysis={analysis} commands={commands} openFile={path=>{const file=buffers.values.get(path)?.file??findPath(path);if(file)void openFileVisit(file);else setNotice("File no longer listed. Refresh the workspace.");}} focusCurrent={()=>{if(buffer)editorForBuffer(buffer)?.view.focus();}} />
    <WorkspaceChanges history={changeHistory} adoptResources={routes=>{requested.current.clear();const next=resourceLayout(liveLayout.current,retainedTabs.routes(routes,hasActivePlainDebugSession),resourcePlacements.current);liveLayout.current=next;setLayout(next);void loadFiles();}} files={workspaceFiles} buffers={buffers} current={() => buffer} focus={() => { if (buffer) editorForBuffer(buffer)?.view.focus(); }} />
    <WorkspaceSearch visible={showWorkspaceSearch} close={() => { setShowWorkspaceSearch(false); if (buffer) editorForBuffer(buffer)?.view.focus(); }} buffers={buffers} files={workspaceFiles} />
    {showSourceHistory&&buffer&&<SourceHistoryView key={buffers.workspace+":"+buffer.file.path} buffer={buffer} buffers={buffers} history={changeHistory} alive={()=>workspaceIdentity.current===workspaceRoot} close={()=>{setShowSourceHistory(false);editorForBuffer(buffer)?.view.focus();}} />}
    {showRecovery&&<SourceRecoveryView key={"recovery:"+workspaceRoot} buffers={buffers} epoch={()=>changeHistory.stamp()} alive={()=>workspaceIdentity.current===workspaceRoot} acquire={async path=>{
      const identity=workspaceIdentity.current,response=await api.files();if(workspaceIdentity.current!==identity||response.root!==identity)throw new Error("Workspace changed. Read recovery again.");
      const find=(nodes:FileNode[]):FileNode|undefined=>{for(const file of nodes){if(file.path===path&&!file.dir)return file;const nested=find(file.children??[]);if(nested)return nested;}};
      const file=buffers.values.get(path)?.file??find(response.tree);if(!file)throw new Error("Recovery source is not available in the current workspace: "+path);return await buffers.open(file);
    }} adopt={(routes:ResourceEditorRoute<Buffer>[],recovery:string)=>{changeHistory.record({kind:"resource",recovery});requested.current.clear();const next=resourceLayout(liveLayout.current,retainedTabs.routes(routes,hasActivePlainDebugSession),resourcePlacements.current);liveLayout.current=next;setLayout(next);void loadFiles();setNotice("Source restoration confirmed; saved baselines and drafts are retained separately.");}} close={()=>{setShowRecovery(false);if(buffer)editorForBuffer(buffer)?.view.focus();}}/>}
    {showConstruction&&<ConstructionReadingView key={"construction:"+workspaceRoot} workspace={buffers.workspace} engineWorkspace={buffers.workspaceId} capture={()=>{const namespace=buffers.workspace,engine=buffers.workspaceId;try{const roster=captureResourceRoster(buffers);return ()=>!!namespace&&buffers.workspace===namespace&&buffers.workspaceId===engine&&workspaceIdentity.current===workspaceRoot&&roster.current();}catch{return ()=>false;}}} terminal={id=>{terminal.id=id;terminal.error="";terminal.publish();openTab("terminal");setShowConstruction(false);}} retainedSources={()=>{const roster=captureResourceRoster(buffers);return {sources:roster.ownership_sources,unaddressed_sources:roster.unaddressed_sources,saved_guards:Object.fromEntries(roster.captures.filter(row=>Object.hasOwn(roster.ownership_sources,row.path)&&!!row.guard).map(row=>[row.path,row.guard!])),missing_sources:roster.captures.filter(row=>row.deleted&&!row.readonly&&!row.transient&&!row.path.startsWith('/')&&!row.path.startsWith('Untitled-')).map(row=>row.path)};}} sources={()=>captureResourceRoster(buffers).ownership_sources} navigate={async(point,current)=>{
      const identity=workspaceIdentity.current,file=buffers.values.get(point.path)?.file??findPath(point.path);if(!file)throw new Error("Source no longer listed. Refresh the construction reading.");
      const target=await buffers.open(file),source=target.text;
      const hash=[...new Uint8Array(await crypto.subtle.digest("SHA-256",new TextEncoder().encode(source)))].map(byte=>byte.toString(16).padStart(2,"0")).join("");
      const offset=constructionPointOffset(source,point,hash),valid=()=>current()&&workspaceIdentity.current===identity&&target.text===source;
      if(!valid())return;if(offset===null)throw new Error("Source changed or position is unavailable. Refresh the construction reading.");
      const departed=currentLocation();await open(file);if(!valid())return;
      for(let attempt=0;attempt<20;attempt++){if(!valid())return;const view=editorForBuffer(target)?.view;if(view?.dom.isConnected){if(view.state.doc.toString()!==source)throw new Error("Editor source changed. Refresh the construction reading.");view.dispatch({selection:{anchor:offset},scrollIntoView:true});view.focus();const start={line:point.line-1,character:point.character};trail.record(departed,{uri:documentUri(point.path),range:{start,end:start}});refreshTrail(value=>value+1);setShowConstruction(false);return;}await new Promise<void>(resolve=>requestAnimationFrame(()=>resolve()));}
      throw new Error("Source opened but its editor is not ready. Try the source link again.");
    }} close={()=>{setShowConstruction(false);if(buffer)editorForBuffer(buffer)?.view.focus();}} />}
    {attributionPath!==null&&<GitAttributionView key={"attribution:"+workspaceRoot} workspace={workspaceRoot} path={attributionPath} close={()=>{setAttributionPath(null);if(buffer)editorForBuffer(buffer)?.view.focus();}} readCommit={(sha,title)=>{setAttributionPath(null);setShowReview(false);setHistoryRequest({kind:'commit',sha,title});setShowHistory(true);}} />}
    <GitHistoryView key={"history:"+workspaceRoot} workspace={workspaceRoot} visible={showHistory} request={historyRequest} close={()=>{setShowHistory(false);if(buffer)editorForBuffer(buffer)?.view.focus();}} />
    <GitReviewView key={workspaceRoot} workspace={workspaceRoot} buffers={buffers} history={changeHistory} alive={()=>workspaceIdentity.current===workspaceRoot} adopt={routes=>{requested.current.clear();const next=resourceLayout(liveLayout.current,retainedTabs.routes(routes,hasActivePlainDebugSession),resourcePlacements.current);liveLayout.current=next;setLayout(next);void loadFiles();}} acquire={async path=>{const scope=workspaceRoot;if(workspaceIdentity.current!==scope)throw new Error("Workspace changed. Prepare the conflict again.");const file=buffers.values.get(path)?.file??findPath(path);if(!file||file.dir)throw new Error("Conflict source is no longer listed. Refresh files and try again.");const target=await buffers.open(file);if(workspaceIdentity.current!==scope)throw new Error("Workspace changed. Prepare the conflict again.");return target;}} visible={showReview} close={()=>{setShowReview(false);if(buffer)editorForBuffer(buffer)?.view.focus();}} />
    <footer role="status" title={[notice,buffer?.completionNotice].filter(Boolean).join(" · ")}><WorkspaceAnalysisView buffers={buffers} files={workspaceFiles} model={analysis} focusCurrent={() => { if (buffer) editorForBuffer(buffer)?.view.focus(); }} /><WorkspaceProblems buffers={buffers} visible={showProblems} setVisible={setShowProblems} current={currentLocation} focusCurrent={() => { if (buffer) editorForBuffer(buffer)?.view.focus(); }} />{notice ? `${notice} · ` : ""}{buffer?.completionNotice?`${buffer.completionNotice} · `:""}{buffer ? `${buffer.file.name} · ${buffer.resourceDeleted?"Deleted source · retained snapshot":buffer.readonly ? "Read-only" : buffer.saving ? "Saving…" : buffer.dirty ? "Modified" : "Saved"}` : status}</footer>
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
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/App.tsx"><ui:paste select="#simple-e8a50a27b2-0"/><ui:paste select="#simple-e8a50a27b2-1"/></ui:file>

</ui:doc>
