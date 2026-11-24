<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Compose only files, documents and agent panes

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Workspace debugger hosts remain mounted above tab content, so stepping into another file or moving the launch tab cannot destroy its session. Cross-file stop requests open the existing tree entry and queue the adapter line for the mounted editor.

The shell owns layout and commands. Documents remain in the buffer store when
hidden or moved. A new file request or explicit tab selection retires an older
request for the same pane. A completed read may warm the buffer store. Opening
the tab, recording a visit, and revealing a line require the same current
request. A provider result also carries its originating source and caret check
while the destination loads. A successful tab change can detach its origin,
so the destination uses the admitted pane request to reveal the result.
Other panes can finish their own reads independently. Once focus
moves away, their late replies can only update their own tab layout.

Coverage and Git Review keep their own component identities within the
workspace. Scope each model to the workspace path and ID so a refresh of the
same workspace preserves live panel state, while a different checkout with
the same display name gets fresh state. Hold that scope through navigation.

Pane content classifies internal tab drags before embedded text controls can consume their payload. An edge drop invokes docking; the center appends the existing tab to this group without adding a split. The half-pane preview is pointer-transparent and never reflows the content. Recompute the edge at drop time rather than trusting an old hover position.

Each tab accepts a drop before itself; the surrounding strip remains the append target. Stop tab-drop bubbling so its parent cannot append the same tab afterward. An absolutely positioned insertion mark does not change tab widths or move editor content. Only layout identities change.

File commands act on the connected workspace: path entry opens a file or directory, New Tab (Ctrl+N or Cmd+N) keeps an unnamed buffer, and Save As exclusively creates a new destination before a hash-checked write. Existing destinations are never overwritten by Save As. Save All visits each named buffer; unnamed drafts ask for a path and failed saves retain their original edits. Refresh updates the tree and checks the active source. Returning to the app checks the active source as well. These reads keep the same editor and report disk divergence when the active editor holds a draft.

The browser may enter this same workspace at `simple.html?path=<encoded-path>`.
Request raw text and the file tree independently. While the tree and guarded
reader resolve the exact file, put the read-only observation in its normal
tab. Show the containing folder in Files and keep the workspace root available
to search and Open Directory. Refresh updates the scoped tree through the same
folder identity. If the initial authority read failed, Refresh completes it
against the same observed buffer, preserving its selection and viewport. A
reply to an older tree request cannot replace the latest tree or authority.
A startup file read applies only while the initial layout is still current, so
a slow request cannot replace a tab the user opened in the meantime.

An ordinary source link can be requested while the startup document is
readable and the file listing is still pending. Keep the latest request and
admit it after the listing arrives only if the same workspace, buffer text
and editor interaction still own it.
When Files and the guarded reader deliver readable text first, the same
startup lease owns that buffer until a matching reply confirms the save guard.
Record the provisional owner before activating its tab so a later Refresh can
confirm it without reopening the editor.

A document component belongs to one buffer path. Switching tabs mounts the next component around its already cached editor; it does not reuse React state that still points to the previous file's editor. Without that identity boundary, a queued cross-file reveal can be consumed by the detached old editor before the destination mounts. The cache retains selection and undo when the original tab returns.

Browser source navigation and provider requests translate retained display selection offsets into authored UTF-16 positions. Construction reveals translate the returned authored offset back into the same retained editor; neither operation rewrites source bytes.

Hover explanations use the same workspace-command availability reason that
guards activation. The connection form explains whether a request is already
in flight or the address is still blank. The tooltip layer draws these messages
over the workspace without changing any control’s dimensions.

The browser lets its own origin open an already configured workspace. The
welcome screen appears only when that initial lookup fails on a blank launch.
It can retry that origin or navigate to an editor served by another Hickory
host. Keep the normal editor mounted for an explicit file or after a workspace
has loaded, so a failed listing cannot hide the user's source.

<ui:copy id="simple-e8a50a27b2-0">
import {authoredSource,authoredOffset,editorOffset} from './SourceText';
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
import { matchesWorkspaceSymbol } from "../lib/workspaceSearch";
import { documentUri } from "../lsp/documentUri";
import { utf16ToPosition } from "../lsp/positions";
import { editorForBuffer } from "./Document";
import { TooltipLayer } from "../components/TooltipLayer";
import { useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
import buildStamp from "virtual:hickory-dev-build";
import {EditorRecoveryModel} from "./EditorRecovery";
import {captureEditorRecovery,editorRecoveryLease,restoreEditorRecovery} from "./EditorRecoveryAdapter";
import {EditorRecoveryView,editorRecoveryTransport} from "./EditorRecoveryView";
import { api,request } from "../api/client";
import { PlainDebugHosts,hasActivePlainDebugSession,retirePlainDebugIdentity } from "../debug/plainDebugHosts";
import { onOpenLocation, onShowTerminal, openLocation, revealLine } from "../lib/revealLine";
import type { OpenLocation } from "../lib/revealLine";
import type { FileNode } from "../api/types";
import { Buffer, Buffers, createSourceCopy,adoptCreatedDraft } from "./buffers";
import { RichOwners } from "./RichInline";
import { enterMode, type EnterMode } from "./SmartEnter";
import { Document, semanticQueries, openSemanticQuery, openFind, wordNavigationMode, type WordNavigationMode } from "./Document";
import { Terminal, TerminalSession } from "./Terminal";
import { Agent, AgentHistory, AgentSession, AgentStatus } from "./Agent";
import {AgentProposalReview} from "./AgentProposalView";
import {prepareRecordedPatch,proposalEditorContext,type ProposalSelection,type ReviewRecordedPatch} from "./AgentProposalAdapter";
import { sourceTab, tabIdentity, activate, groups, initialLayout, isGroup, move, remove, split, update, dock, dropEdge, type Edge, type Axis, type Layout } from "./layout";
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
type OpenFileOptions = {
  startup?: { observe: (buffer: Buffer) => void; refuse: (message: string) => void };
  current?: () => boolean;
  originCurrent?: () => boolean;
  observed?: Buffer;
  verify?: (buffer: Buffer) => Promise<void>;
};
type OpenedFile = {buffer: Buffer; current: () => boolean};
export function App() {
  const [layout, setLayout] = useState<Layout>(initialLayout);
  const [hiddenFilesPane, setHiddenFilesPane] = useState<string | null>(null);
  const liveLayout=useRef(layout);liveLayout.current=layout;
  const [focused, updateFocused] = useState("document-group");
  const [terminal] = useState(() => new TerminalSession());
  const [agent] = useState(() => new AgentSession());
  const [filesFolder, setFilesFolder] = useState("");
  const [workspaceFiles, setWorkspaceFiles] = useState<FileNode[]>([]);
  const [notice, setNotice] = useState("");
  const [analysis] = useState(() => new WorkspaceAnalysisModel());
  const [showCoverage,setShowCoverage]=useState(false);
  const [showTests,setShowTests]=useState(false);
  const [showTemplates,setShowTemplates]=useState(false);
  const [showRecovery,setShowRecovery]=useState(false);
  const [showEditorRecovery,setShowEditorRecovery]=useState(false);
  const [showConstruction,setShowConstruction]=useState(false);
  const [showAgentProposals,setShowAgentProposals]=useState(false);
  const [proposalSelection, setProposalSelection] = useState<ProposalSelection | null>(null);
  const proposalRequest = useRef(0);
  const [showReview,setShowReview]=useState(false);
  const [attributionPath,setAttributionPath]=useState<string|null>(null);
  const [showSourceHistory,setShowSourceHistory]=useState(false);
  const [showHistory,setShowHistory]=useState(false),[historyRequest,setHistoryRequest]=useState<GitHistoryRequest>({kind:'log',path:null});
  const [testInspection,setTestInspection]=useState<SourceTestInspection|null>(null);
  const [workspaceRoot,setWorkspaceRoot]=useState("");
  const [workspaceLookup, setWorkspaceLookup] = useState<"opening" | "welcome" | "opened">("opening");
  const [requestedPath] = useState(() => new URLSearchParams(window.location.search).get("path"));
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
  const overlayWorkspace = () => JSON.stringify([buffers.workspace || workspaceIdentity.current, buffers.workspaceId]);
  const overlayScope = overlayWorkspace();
  const overlayCurrent = () => overlayWorkspace() === overlayScope;
  const requested = useRef(new Map<string, {path: string}>());
  const filesGeneration = useRef(0);
  const listingFailure = useRef<string | null>(null);
  const acceptStartupFiles = useRef<((response: Awaited<ReturnType<typeof api.files>>) => void) | null>(null);
  const pendingStartupLocation = useRef<{
    location: OpenLocation;
    origin: Buffer;
    text: string;
    interaction: number;
    workspace: string | null;
  } | null>(null);
  const [retainedTabs]=useState(()=>new RetainedResourceTabs());
  const recoveryFocus=useRef(focused), focusEpoch=useRef(0);
  function setFocused(group: string) {
    if (recoveryFocus.current !== group) {
      recoveryFocus.current = group;
      focusEpoch.current++;
    }
    updateFocused(group);
  }
  const editorRecovery=useMemo(()=>new EditorRecoveryModel(editorRecoveryTransport),[buffers.workspaceId,buffers.workspace]);
  useEffect(()=>{let living=true;const workspace=buffers.workspaceId,namespace=buffers.workspace;const current=()=>living&&buffers.workspaceId===workspace&&buffers.workspace===namespace;const timer=window.setInterval(()=>{if(!current()||!workspace||!namespace)return;try{const snapshot=captureEditorRecovery(buffers,liveLayout.current,recoveryFocus.current,retainedTabs.buffers.values(),tab=>{const id=tabIdentity(tab);return id.kind==='source'?(retainedTabs.buffers.get(id.key)??buffers.values.get(id.key)):undefined;});void editorRecovery.checkpoint(workspace,namespace,snapshot,current);}catch(e){editorRecovery.notice=String(e);}},1500);return()=>{living=false;window.clearInterval(timer);};},[buffers,editorRecovery,retainedTabs]);

  const bufferForTab=(tab:string)=>{const identity=tabIdentity(tab);return identity.kind==='source'?(retainedTabs.buffers.get(identity.key)??buffers.values.get(identity.key)):undefined;};
  const tabRoutes=(routes:ResourceEditorRoute<Buffer>[])=>retainedTabs.routes(routes,hasActivePlainDebugSession).map(route=>({...route,from:route.from===null?null:sourceTab(route.from),to:route.to===null?null:sourceTab(route.to)}));
  const resourcePlacements=useRef(new Map<Buffer,{group:string;index:number;active:boolean}>());
  function newTab() {
    const path = `Untitled-${crypto.randomUUID()}`;
    const draft=new Buffer({ name: `Untitled ${[...buffers.values.values()].filter(value=>value.isUntitledDraft).length + 1}`, path, dir: false }, "", "", () => refresh(value => value + 1));draft.isUntitledDraft=true;buffers.values.set(path,draft);
    openTab(sourceTab(path));
  }
  function findPath(path: string, nodes = workspaceFiles): FileNode | undefined {
    for (const node of nodes) { if (node.path === path) return node; const found = findPath(path, node.children ?? []); if (found) return found; }
  }
  function showContainingFolder(path: string) {
    setFilesFolder(path.slice(0, Math.max(0, path.lastIndexOf("/"))));
  }
  async function openPath(directory = false) {
    const path = window.prompt(directory ? "Open Directory — path in this workspace (empty for root)" : "Open File — path in this workspace");
    if (path === null) return;
    if (directory && !path) { setFilesFolder(""); await loadFiles(); openTab("files"); return; }
    const node = findPath(path);
    if (!node || node.dir !== directory) { setNotice("Path not found. Refresh and choose a path in this workspace."); return; }
    if (directory) { setFilesFolder(node.path); openTab("files"); }
    else { showContainingFolder(node.path); await openFileVisit(node); }
  }
  async function saveAs(source = buffer) {
    if (!source || source.saving) return;
    const path = window.prompt("Save As — new path in this workspace", source.isUntitledDraft ? "" : source.file.path);
    if (!path) return;
    if (path === source.file.path) { if(source.resourceDeleted)setNotice("This source was deleted. Choose a new unused path to save its retained text.");else await source.save(); return; }
    const text = source.text,editor=editorForBuffer(source),interaction=editor?.interaction,placement=liveLayout.current;
    const current=()=>liveLayout.current===placement&&!!editor&&editorForBuffer(source)===editor&&editor.view.dom.isConnected&&!editor.view.composing&&editor.interaction===interaction&&authoredSource(editor.view.state)===text;
    source.saving = true; source.publish();
    try {
      const copy=await createSourceCopy(buffers,source,path,request,current);
      if((source.resourceDeleted||source.isUntitledDraft)&&!hasActivePlainDebugSession(source)){
        const tabs=groups(liveLayout.current).flatMap(group=>group.tabs).filter(tab=>bufferForTab(tab)===source);
        adoptCreatedDraft(buffers,source,copy,current);retirePlainDebugIdentity(source);retainedTabs.retire(source);
        const routes=tabs.map(from=>({identity:source.sourceId,from,to:sourceTab(source.file.path),before:text,after:text,editor:source}));
        requested.current.clear();const next=resourceLayout(liveLayout.current,routes,resourcePlacements.current);liveLayout.current=next;setLayout(next);await open(source.file);
      }else{
        await open(copy.file);
        if(source.resourceDeleted||source.isUntitledDraft){retainedTabs.noteCopy(source,text);setNotice("Copy saved. The original launch source remains readable until its debugger closes.");}
      }
      void loadFiles();

    } catch (error) { setNotice(`${String(error)} Original edits and the exact copy request are retained. Inspect local history before changing the destination; retry only while the captured editors remain current.`); }
    finally { source.saving = false; source.publish(); refresh(value => value + 1); }
  }
  async function saveCurrent() { if (buffer?.isUntitledDraft||buffer?.resourceDeleted) await saveAs(); else await buffer?.save(); }
  async function saveAll() {
    for (const item of [...buffers.values.values()]) {
      if (item.isUntitledDraft) { if (item.dirty) await saveAs(item); }
      else await item.save();
    }
  }
  async function loadFiles() {
    const ticket = ++filesGeneration.current;
    try {
      const response = await api.files();
      if (ticket !== filesGeneration.current) return;
      if (typeof response.root !== "string" || !Array.isArray(response.tree)) {
        throw new Error("The server did not return a Hickory workspace.");
      }
      setWorkspaceLookup("opened");
      const previousFailure = listingFailure.current;
      if (previousFailure) setNotice(current => current === previousFailure ? "" : current);
      listingFailure.current = null;
      buffers.workspace = response.root_path ?? "";
      buffers.workspaceId = response.workspace_id ?? "";
      setWorkspaceFiles(response.tree);
      setWorkspaceRoot(response.root);
      setStatus(response.root);
      acceptStartupFiles.current?.(response);
    } catch (error) {
      if (ticket !== filesGeneration.current) return;
      listingFailure.current = `${String(error)} Check the Hickory backend and choose File → Refresh.`;
      setNotice(listingFailure.current);
      setWorkspaceLookup(current => current === "opened" ? current : "welcome");
    }
  }
  useEffect(() => {
    const path = requestedPath;
    const initial = liveLayout.current;
    let attached = true;
    let resolved = false;
    let authorityFailure: string | null = null;
    let observed: Buffer | undefined;
    const current = () => {
      const held = observed;
      return attached && !resolved && (liveLayout.current === initial || !!held
        && buffers.values.get(held.file.path) === held
        && groups(liveLayout.current).some(group => group.id === "document-group" && group.active === sourceTab(held.file.path)));
    };
    const reportAuthorityFailure = (message: string) => {
      authorityFailure = `${message} Choose File → Refresh to check ownership again.`;
      setNotice(authorityFailure);
    };
    if (path) void api.startupSource(path).then(snapshot => {
      if (!current() || snapshot?.version !== 1 || snapshot.path !== path || typeof snapshot.content !== "string" || !snapshot.root_path) return;
      const preview = buffers.observeStartup(snapshot);
      if (preview) {
        observed = preview;
        showContainingFolder(path);
        void open(preview.file, {current, observed: preview});
      }
    }).catch(() => { /* The authoritative tree reader can still open this file. */ });
    // Refresh uses the same lease after an unavailable listing or guarded
    // read. Promote the mounted owner without activating its tab again.
    acceptStartupFiles.current = response => {
      if (path === null || !current()) return;
      const file = findPath(path, response.tree);
      if (!file || file.dir) { setNotice(`Requested file ${path || "(empty path)"} is unavailable. Choose a file from the workspace tree.`); return; }
      if (observed) {
        void buffers.open(file).then(() => {
          if (!current()) return;
          resolved = true;
          if (authorityFailure) setNotice(notice => notice === authorityFailure ? "" : notice);
        }, error => {
          if (!current()) return;
          reportAuthorityFailure(String(error));
        });
      } else {
        showContainingFolder(file.path);
        void open(file, {current, startup: {
          observe: owner => {
            if (owner.startupWorkspace === undefined) return;
            observed = owner;
            if (owner.saveGuardReason) reportAuthorityFailure(owner.saveGuardReason);
          },
          refuse: reportAuthorityFailure,
        }});
      }
    };
    void loadFiles();
    return () => { attached = false; filesGeneration.current++; acceptStartupFiles.current = null; };
  }, []);
  useEffect(() => {
    const warn = (event: BeforeUnloadEvent) => { if (agent.prompt.length > 0 || agent.busy || buffers.hasUnsavedChanges) event.preventDefault(); };
    window.addEventListener("beforeunload", warn); return () => window.removeEventListener("beforeunload", warn);
  }, [buffers, agent, terminal]);
  const currentGroup = groups(layout).find(group => group.id === focused) ?? groups(layout)[0];
  const active = currentGroup.active;
  const buffer = active ? bufferForTab(active) : undefined;
  async function refreshCurrentSource() {
    if (!buffer) return;
    const current = () => {
      const group = groups(liveLayout.current).find(item => item.id === recoveryFocus.current);
      return !!group?.active && bufferForTab(group.active) === buffer && !editorForBuffer(buffer)?.view.composing;
    };
    if (!current()) return;
    const message = await buffers.refreshSavedSource(buffer, current);
    if (message && current()) setStatus(message);
  }
  useEffect(() => {
    const refresh = () => { void refreshCurrentSource(); };
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  }, [buffer, buffers]);
  agent.captureContext=()=>proposalEditorContext(buffers,buffer);
  function openAgentProposals(selection: ProposalSelection | null = null) {
    proposalRequest.current++;
    setProposalSelection(selection);
    setShowAgentProposals(true);
  }
  const reviewRecordedPatch: ReviewRecordedPatch = async (turn, fence, alive) => {
    const ticket = ++proposalRequest.current;
    const current = () => ticket === proposalRequest.current && alive();
    const selection = await prepareRecordedPatch(buffers, turn, fence, current, buffer);
    if (current()) openAgentProposals(selection);
  };
  async function openTerminal() { openTab("terminal"); await terminal.open(); }
  function activateTab(group: string, tab: string) {
    requested.current.delete(group);
    setLayout(current => activate(current, group, tab));
    setFocused(group);
  }
  function openTab(tab: string) {
    if (tab === "files") setHiddenFilesPane(null);
    const owner = groups(liveLayout.current).find(group => group.tabs.includes(tab));
    activateTab(owner?.id ?? currentGroup.id, tab);
  }
  useEffect(() => onShowTerminal(({ id }) => {
    if (!/^term-[0-9a-f]{32}$/.test(id)) return;
    terminal.id = id;
    terminal.error = "";
    terminal.publish();
    openTab("terminal");
  }), [terminal, currentGroup.id]);
  async function open(file: FileNode, {current: isCurrent = () => true, originCurrent = () => true, observed, verify, startup}: OpenFileOptions = {}): Promise<OpenedFile | null> {
    if (!isCurrent() || !originCurrent()) return null;
    const path = file.path, tab = sourceTab(path), panes = groups(liveLayout.current);
    const target = panes.find(group => group.tabs.includes(tab))?.id
      ?? panes.find(group => group.id === focused && !group.tabs.includes("files"))?.id
      ?? panes.find(group => !group.tabs.includes("files"))?.id ?? currentGroup.id;
    const ticket = {path};
    const namespace = buffers.workspace, workspace = buffers.workspaceId;
    const pending = () => {
      if (requested.current.get(target) !== ticket || !isCurrent()
        || !observed && (buffers.workspace !== namespace || buffers.workspaceId !== workspace)) return false;
      const live = groups(liveLayout.current);
      const owner = live.find(group => group.tabs.includes(tab));
      return owner ? owner.id === target : live.some(group => group.id === target);
    };
    requested.current.set(target, ticket);
    setFocused(target);
    const focus = focusEpoch.current;
    const foreground = () => recoveryFocus.current === target && focusEpoch.current === focus;
    try {
      const owner = observed ?? await buffers.open(file, startup !== undefined);
      if (!pending() || !originCurrent()) return null;
      const revision = owner.sourceRevision;
      if (verify) await verify(owner);
      const admitted = () => pending() && buffers.values.get(path) === owner
        && owner.file.path === path && (!verify || owner.sourceRevision === revision);
      // Admit the origin before handing the reveal to its destination.
      if (!admitted() || !originCurrent()) return null;
      startup?.observe(owner);
      setLayout(layout => admitted() ? activate(layout, target, tab) : layout);
      const current = () => admitted() && foreground();
      if (current()) setStatus(status => current() ? owner.file.generated_by ? "Generated file · read-only" : path : status);
      return {buffer: owner, current};
    } catch (error) {
      if (!pending() || !foreground() || !originCurrent()) return null;
      if (verify) throw error;
      if (startup) startup.refuse(String(error));
      else setNotice(notice => pending() && foreground() && originCurrent() ? `${String(error)} Refresh files and try opening ${path} again.` : notice);
      return null;
    }
  }
  function currentLocation(): SemanticLocation | null {
    const view = buffer && editorForBuffer(buffer)?.view; if (!buffer || !view) return null;
    const start = utf16ToPosition(buffer.text, authoredOffset(view.state,view.state.selection.main.head));
    return { uri: documentUri(buffer.file.path), range: { start, end: start } };
  }
  function recordVisit(departed: SemanticLocation | null, path: string, start: {line: number; character: number}) {
    trail.record(departed, {uri: documentUri(path), range: {start, end: start}});
    refreshTrail(value => value + 1);
  }
  function revealVisit(opened: OpenedFile, departed: SemanticLocation | null, line: number, character?: number, record = true) {
    if (!opened.current()) return;
    const {text: source, file: {path}} = opened.buffer;
    if (record) recordVisit(departed, path, {line: line - 1, character: character ?? 0});
    revealLine(path, line, character, {source, current: opened.current});
  }
  async function openLine(file: FileNode, line: number, character: number, options: OpenFileOptions) {
    const departed = currentLocation(), opened = await open(file, options);
    if (!opened?.current()) return false;
    revealVisit(opened, departed, line, character);
    return true;
  }
  function navigateHistory(direction: -1 | 1) {
    const location = trail.step(direction, currentLocation()); refreshTrail(value => value + 1);
    const path = location && semanticPath(location.uri);
    if (path && location) openLocation(path, location.range.start.line + 1, location.range.start.character, false);
  }
  useEffect(() => {
    function navigate(location: OpenLocation) {
      if (workspaceLookup === "opening" && buffer && buffer.startupWorkspace !== undefined &&
          !location.capture && !location.providerName) {
        pendingStartupLocation.current = {
          location, origin: buffer, text: buffer.text,
          interaction: editorForBuffer(buffer)?.interaction ?? 0,
          workspace: buffer.startupWorkspace,
        };
        return;
      }
      const departed = currentLocation(),workspace=workspaceIdentity.current,origin=buffer&&editorForBuffer(buffer),originInteraction=origin?.interaction??0;
      const current=()=>workspaceIdentity.current===workspace&&(!location.capture||location.capture.current()&&(!origin||origin.interaction===originInteraction));
      if (!current() || location.originCurrent && !location.originCurrent()) return;
      const find = (nodes: FileNode[]): FileNode | undefined => {
        for (const node of nodes) { if (node.path === location.path) return node; const child = find(node.children ?? []); if (child) return child; }
      };
      const file = buffers.values.get(location.path)?.file ?? find(workspaceFiles);
      let accepted: OpenedFile | null = null;
      const name = location.providerName;
      const verify = name && file?.path !== buffer?.file.path ? async (target: Buffer) => {
        const row = target.text.split("\n")[location.line - 1];
        if (!matchesWorkspaceSymbol(row, location.character ?? -1, name)) {
          throw new Error("The reported symbol has moved or changed. Query the location again.");
        }
      } : undefined;
      if (file) { void open(file, {current, originCurrent: location.originCurrent, verify}).then(async opened => {
        if (!opened?.current()) return;
        accepted = opened;
        const target = opened.buffer;
        if(location.capture){
          if(!location.capture.hash){setNotice(location.capture.sourceReference&&location.capture.sourceReference>0?"Adapter-owned source bytes have not been read; opened workspace text without a precise position.":location.capture.positionKnown===false?"Frame position is unavailable; opened readable source text only.":"Opened source text only; the debugger has no captured mapping identity for this frame.");return;}
          const source=target.text,interaction=editorForBuffer(target)?.interaction??0;
          for(const character of source)if(character.length===1&&character.charCodeAt(0)>=0xD800&&character.charCodeAt(0)<=0xDFFF){setNotice("Source text has an incomplete Unicode scalar; opened without a debugger position.");return;}
          const digest=await crypto.subtle.digest("SHA-256",new TextEncoder().encode(source)),hash=[...new Uint8Array(digest)].map(byte=>byte.toString(16).padStart(2,"0")).join("");
          if (!opened.current()) return;
          const exact=()=>opened.current()&&target.text===source&&(editorForBuffer(target)?.interaction??0)===interaction;
          if(!exact()){setNotice("Debugger position or editor interaction changed while checking source. Select the frame again.");return;}
          if(hash!==location.capture.hash){setNotice("Source differs from the captured debugger mapping; opened without a precise position.");return;}
          if(!Number.isInteger(location.line)||location.line<1||location.line>source.split("\n").length){setNotice("Captured debugger line does not fit this source; opened text only.");return;}
          revealLine(file.path,location.line,0,{source,current:exact});return;
        }
        if (location.expectedSource !== undefined && target.text !== location.expectedSource) { setNotice("Source changed. Refresh the query before navigating."); return; }
        if (location.expectedSource === undefined && location.recordHistory !== false && location.character !== undefined && target.dirty && file.path !== buffer?.file.path) { setNotice(`Save ${file.path} and query again before using a cross-file source position.`); return; }
        revealVisit(opened, departed, location.line, location.character, location.recordHistory !== false);
      }).catch(error => {
        if (accepted?.current()) {
          setNotice(String(error) + " Source text remains available; no precise location was applied.");
        } else if (verify && current() && location.originCurrent?.()) {
          setNotice(error instanceof Error ? error.message : String(error));
        }
      }); } else setNotice(location.capture?.sourceReference&&location.capture.sourceReference>0?"This frame uses adapter-owned source; no matching workspace text is available.":`Location not found: ${location.path}. Refresh files and query again.`);
    }
    const stop = onOpenLocation(navigate, true);
    const pending = pendingStartupLocation.current;
    if (pending && workspaceLookup !== "opening") {
      pendingStartupLocation.current = null;
      if (workspaceLookup === "opened" && buffer === pending.origin && buffer.text === pending.text &&
          (editorForBuffer(buffer)?.interaction ?? 0) === pending.interaction && buffer.namespace === buffers.workspace &&
          (!pending.workspace || pending.workspace === buffers.workspaceId)) {
        navigate(pending.location);
      }
    }
    return stop;
  }, [workspaceFiles, layout, focused, workspaceLookup]);
  useEffect(()=>{const inspect=(event:Event)=>{const path=(event as CustomEvent<{path:string}>).detail.path,file=buffers.values.get(path)?.file??findPath(path);if(!file||!overlayCurrent())return;void open(file,{current:overlayCurrent}).then(opened=>{if(opened?.current())setShowCoverage(true);});};window.addEventListener(SOURCE_COVERAGE_INSPECTION,inspect);return()=>window.removeEventListener(SOURCE_COVERAGE_INSPECTION,inspect);},[workspaceFiles,layout,focused,overlayScope]);
  useEffect(()=>{const inspect=(event:Event)=>{const request=(event as CustomEvent<SourceTestInspection>).detail,owner=request.owner??buffers.values.get(request.path),namespace=buffers.workspace;const current=()=>!!owner&&!owner.resourceDeleted&&buffers.values.get(request.path)===owner&&owner.file.path===request.path&&owner.namespace===namespace&&buffers.workspace===namespace&&(request.revision===undefined||owner.sourceRevision===request.revision);const file=owner?.file;if(!file||!current())return;void open(file,{current}).then(opened=>{if(opened?.current()){setTestInspection(request);setShowTests(true);}});};window.addEventListener(SOURCE_TEST_INSPECTION,inspect);return()=>window.removeEventListener(SOURCE_TEST_INSPECTION,inspect);},[workspaceFiles,layout,focused]);
  async function openFileVisit(file: FileNode, options?: OpenFileOptions) {
    const departed = currentLocation(), opened = await open(file, options);
    if (!opened?.current()) return false;
    const target = opened.buffer, view = editorForBuffer(target)?.view;
    const start = view ? utf16ToPosition(target.text, authoredOffset(view.state,view.state.selection.main.head)) : { line: 0, character: 0 };
    recordVisit(departed, file.path, start);
    return true;
  }
  function close(tab: string) {
    const candidate = bufferForTab(tab);
    if(candidate?.resourceDeleted&&hasActivePlainDebugSession(candidate)){setNotice("Stop the debugger before closing its retained deleted source. Its existing controls remain available here.");return;}
    if (candidate?.saving || candidate?.dirty&&!retainedTabs.copied(candidate)) { setNotice("Unsaved edits retained. Save this document before closing its tab."); return; }
    if (candidate?.startupWorkspace !== undefined && buffers.values.get(candidate.file.path) === candidate) buffers.values.delete(candidate.file.path);
    if(candidate?.resourceDeleted&&retainedTabs.copied(candidate)&&!hasActivePlainDebugSession(candidate)){retireCopiedBuffer(buffers,candidate,candidate.text);retirePlainDebugIdentity(candidate);retainedTabs.retire(candidate);}
    if (candidate) buffers.closeSavedPlainSource(candidate);
    const activePane = groups(liveLayout.current).find(pane => pane.active === tab);
    for (const [group, ticket] of requested.current) {
      if (sourceTab(ticket.path) === tab || activePane?.id === group) requested.current.delete(group);
    }
    setLayout(current => remove(current, tab));
  }
  function retireTabMove(tab: string, target: string) {
    requested.current.delete(target);
    const previous = groups(liveLayout.current).find(group => group.tabs.includes(tab));
    if (previous) requested.current.delete(previous.id);
  }
  function divide(axis: Axis) { setLayout(current => split(current, currentGroup.id, axis, crypto.randomUUID())); }
  function nextGroup() {
    if (!active) return;
    const leaves = groups(layout), index = leaves.findIndex(group => group.id === focused), target = leaves[(index + 1) % leaves.length].id;
    retireTabMove(active, target);
    setLayout(current => move(current, active, target)); setFocused(target);
  }
  useEffect(() => {
    const command = (event: KeyboardEvent) => {
      if (requestedPath === null && workspaceLookup !== "opened") return;
      // Terminal control keys belong to the PTY, including Ctrl+N and Ctrl+S.
      if (event.defaultPrevented || (event.target instanceof Element && event.target.closest(".terminal-screen"))) return;
      if (!(event.metaKey || event.ctrlKey)) return;
      if (!event.shiftKey && !event.altKey && event.key.toLowerCase() === "n") {
        event.preventDefault(); newTab();
      } else if (event.key === "s") { event.preventDefault(); void saveCurrent(); }
    };
    window.addEventListener("keydown", command); return () => window.removeEventListener("keydown", command);
  }, [buffer, layout, focused, requestedPath, workspaceLookup]);
</ui:copy>

The window keyboard handler consumes Ctrl/Cmd+N before the browser default and uses the current layout and focused group so the draft opens beside the active document. Shift and Alt combinations remain available for other commands.

Menu and tab-strip commands expose the same operations as native menus. Empty leaves accept drops. Recursive splits retain arbitrary arrangements; resizing updates only their proportions.

<ui:copy id="simple-e8a50a27b2-1">
  const label = (tab: string) => tab === "files" ? "Files" : tab === "agent" ? "Agent" : tab === "terminal" ? "Terminal" : `${bufferForTab(tab)?.file.name ?? tab.split("/").at(-1)}${bufferForTab(tab)?.resourceDeleted?" (deleted source)":""}${bufferForTab(tab)?.dirty ? " •" : ""}`;
  function content(tab: string) {
    if (tab === "files") return <div className="file-tree" aria-label="Files"><Tree nodes={filesFolder ? findPath(filesFolder)?.children ?? [] : workspaceFiles} open={openFileVisit} /></div>;
    if (tab === "terminal") return <Terminal session={terminal} />;
    if (tab === "agent") return <Agent session={agent} reviewProposals={() => openAgentProposals()} reviewRecordedPatch={reviewRecordedPatch} />;
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
  const visibleDocumentPane = groups(layout).find(pane => pane.active && bufferForTab(pane.active));
  const separateFilesPane = visibleDocumentPane
    ? groups(layout).find(pane => pane.tabs.length === 1 && pane.tabs[0] === "files")?.id
    : undefined;
  const collapsedFilesPane = hiddenFilesPane === separateFilesPane ? hiddenFilesPane : null;
  function toggleFilesPane() {
    if (!separateFilesPane) return;
    setHiddenFilesPane(collapsedFilesPane ? null : separateFilesPane);
    if (!collapsedFilesPane && focused === separateFilesPane && visibleDocumentPane) setFocused(visibleDocumentPane.id);
  }
  const context:CommandContext={separateFiles:!!separateFilesPane,...sourceSnippetContext(buffer),editor:!!buffer,writable:!!buffer&&!buffer.readonly&&!buffer.resourceDeleted,dirty:buffer?.dirty??false,saving:buffer?.saving??false,back:trail.canBack,forward:trail.canForward,tab:!!active,groups:groups(layout).length};
  const contextRef=useRef(context);contextRef.current=context;
  const handlers:Record<string,()=>void>={
    "file.open":()=>{void openPath();},"file.directory":()=>{void openPath(true);},"file.new":newTab,"file.window":()=>{window.open(window.location.href,"_blank","noopener");},
    "file.save":()=>{void saveCurrent();},"file.saveAs":()=>{void saveAs();},"file.saveAll":()=>{void saveAll();},"file.refresh":()=>{const recheck = buffer?.startupWorkspace === undefined; void loadFiles().then(() => { if (recheck) return refreshCurrentSource(); });},"file.close":()=>{if(active)close(active);},
    "shell.search":()=>setShowEverywhere(true),"edit.find":()=>{if(buffer)openFind(buffer);},"workspace.find":()=>setShowWorkspaceSearch(true),
    "symbol.rename":()=>requestWorkspaceChange("rename"),"symbol.fixAll":()=>requestWorkspaceChange("fixAll"),"symbol.sourceActions":()=>requestWorkspaceChange("sourceActions"),"symbol.actions":()=>requestWorkspaceChange("actions"),"document.format":()=>requestWorkspaceChange("format"),"document.cleanup":()=>requestWorkspaceChange("cleanup"),"workspace.moveEntry":()=>requestWorkspaceChange("entryMove"),"workspace.undo":()=>requestWorkspaceChange("undo"),"workspace.redo":()=>requestWorkspaceChange("redo"),
    "document.attribution":()=>{if(buffer){setShowReview(false);setShowHistory(false);setAttributionPath(buffer.file.path);}},"workspace.review":()=>{setAttributionPath(null);setShowHistory(false);setShowReview(true);},"workspace.history":()=>{setAttributionPath(null);setHistoryRequest({kind:'log',path:null});setShowReview(false);setShowHistory(true);},"document.history":()=>{setAttributionPath(null);setHistoryRequest({kind:'log',path:buffer?.file.path??null});setShowReview(false);setShowHistory(true);},"workspace.coverage":()=>setShowCoverage(true),"workspace.recovery":()=>setShowRecovery(true),"workspace.construction":()=>setShowConstruction(true),"agent.proposals":()=>setShowAgentProposals(true),"workspace.tests":()=>setShowTests(true),"source.templates":()=>setShowTemplates(true),"source.history":()=>setShowSourceHistory(true),"workspace.problems":()=>setShowProblems(true),"workspace.analysis":requestWorkspaceAnalysis,"problem.next":()=>requestProblemNavigation(1),"problem.previous":()=>requestProblemNavigation(-1),"navigation.back":()=>navigateHistory(-1),"navigation.forward":()=>navigateHistory(1),
    "view.files":()=>openTab("files"),"view.toggleFiles":toggleFilesPane,"view.agent":()=>openTab("agent"),"view.terminal":()=>{void openTerminal();},"view.splitRight":()=>divide("horizontal"),"view.splitBelow":()=>divide("vertical"),"view.move":nextGroup,
  };
  for(const query of semanticQueries)handlers[`semantic.${query.kind}`]=()=>{if(buffer){const text=query.kind==="symbols"?window.prompt("Search workspace symbols",""):"";if(text!==null)openSemanticQuery(buffer,query.kind,text);}};
  for(const command of ["tools","undo","redo","rectangle","cursorBelow","cursorAbove","snippetNext","snippetPrevious","snippetChoices","snippetFinish"] as const)handlers[`editor.${command}`]=()=>{if(buffer&&!runEditorCommand(buffer,command))setNotice("Editor command unavailable. Wait for the editor or language service, then retry.");};
  const handlersRef=useRef(handlers);handlersRef.current=handlers;
  const commands=workspaceCommands(()=>contextRef.current,Object.fromEntries(Object.keys(handlers).map(id=>[id,()=>handlersRef.current[id]?.()])));
  const commandMenu=(group:string)=>commands.filter(command=>command.group===group&&command.id in handlers).map(command=><button key={command.id} disabled={command.reason!==null} data-tip={command.reason??command.label} aria-description={command.reason??undefined} onClick={command.run}>{command.label}</button>);
  const statusNotices = [notice, buffer?.completionNotice, buffer?.inlineRenderNotice].filter(Boolean).join(" · ");
  if (requestedPath === null && workspaceLookup !== "opened") {
    if (workspaceLookup === "opening") {
      return <main className="workspace-welcome"><p role="status">Opening workspace…</p></main>;
    }
    return <WorkspaceWelcome retry={async () => {
      await loadFiles();
      return listingFailure.current;
    }} />;
  }
  return <SimpleLanguages buffers={buffers}><SourceTestGutterProvider workspace={workspaceRoot}><SourceCoverageProvider workspace={overlayScope}><main>
    {/* Shared gutter controls declare data-tip; mount their delegated reader here. */}
    <TooltipLayer />
    <AgentHistory session={agent} visible={groups(layout).some(group => group.active === "agent")} />
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
    <div className="workspace"><LayoutView node={layout} collapsedFilesPane={collapsedFilesPane} focused={focused} focus={setFocused} label={label} content={content} close={close}
      drop={(tab, target, before) => { retireTabMove(tab, target); setLayout(current => move(current, tab, target, before)); setFocused(target); }}
      activate={activateTab}
      dock={(tab, target, edge) => {
        const id = crypto.randomUUID(), next = dock(layout, tab, target, edge, id);
        if (next !== layout) { retireTabMove(tab, target); setLayout(next); setFocused(id); }
      }}
      resize={(id, ratio) => setLayout(current => update(current, id, node => isGroup(node) ? node : { ...node, ratio }))} /></div>
    <SimpleCoverageView key={`coverage:${overlayScope}`} visible={showCoverage} workspace={overlayScope} path={buffer?.file.path??null} language={buffer?.file.language??"unsupported"} close={navigated=>{setShowCoverage(false);if(!navigated&&buffer)editorForBuffer(buffer)?.view.focus();}} navigate={async(location:CoverageSourceNavigation,current)=>{
      const originCurrent = () => overlayCurrent() && current();
      if (!originCurrent()) return false;
      const file=buffers.values.get(location.path)?.file??findPath(location.path);if(!file)throw new Error("Source no longer listed. Refresh the workspace.");
      const line=location.line;
      if (line===null) return openFileVisit(file,{originCurrent});
      return openLine(file,line,0,{originCurrent,verify:async target=>{
        const source=target.text,digest=await crypto.subtle.digest("SHA-256",new TextEncoder().encode(source)),hash=[...new Uint8Array(digest)].map(byte=>byte.toString(16).padStart(2,"0")).join("");
        if(target.text!==source||hash!==location.sourceHash||line<1||line>source.split("\n").length)throw new Error("Source changed or report position is unavailable. Refresh coverage before navigating.");
      }});
    }}/>
    <SourceTemplatesView visible={showTemplates} close={()=>{setShowTemplates(false);if(buffer)editorForBuffer(buffer)?.view.focus();}} editor={()=>{const owner=buffer,held=owner?editorForBuffer(owner):undefined;if(!owner||!held)return null;const mount=held.mount;return {buffer:owner,buffers,view:held.view,mounted:()=>editorForBuffer(owner)===held&&held.mount===mount,interaction:()=>held.interaction};}} />
    <TestSourcesView inspectDebugger={owner=>{if(owner.namespace!==buffers.workspace)throw Error('The original debugger belongs to a different workspace namespace. Its record remains readable.');const tab=[...buffers.values,...retainedTabs.buffers].find(([,held])=>held===owner)?.[0];if(!tab)throw Error('The original debugger editor is no longer retained. No replacement path was opened.');openTab(sourceTab(tab));setShowTests(false);}} inspection={testInspection} buffers={buffers} visible={showTests} close={()=>{setShowTests(false);if(buffer)editorForBuffer(buffer)?.view.focus();}} current={entry=>buffer?{workspace:workspaceRoot,owner:buffer,namespace:buffers.workspace,revision:buffer.sourceRevision,executionCurrent:!buffer.dirty&&!buffer.saving&&!buffer.resourceDeleted&&buffer.namespace===buffers.workspace&&buffers.values.get(buffer.file.path)===buffer,request:{path:buffer.file.path,source:buffer.text,...(entry?{project_entry:entry}:{}),sources:Object.fromEntries([...buffers.values].filter(([,item])=>!item.readonly).map(([path,item])=>[path,item.text]))}}:null} navigate={async(point:TestSourcePoint)=>{
      const file=buffers.values.get(point.path)?.file??findPath(point.path);if(!file)throw new Error("Source no longer listed. Refresh files and candidates.");
      if (await openLine(file,point.line,point.character,{verify:async target=>{
        const source=target.text,bytes=await crypto.subtle.digest("SHA-256",new TextEncoder().encode(source));
        const hash=[...new Uint8Array(bytes)].map(byte=>byte.toString(16).padStart(2,"0")).join("");
        if(target.text!==source||hash!==point.hash.toLowerCase())throw new Error("Source changed. Refresh candidates before navigating.");
      }})) setShowTests(false);
    }} terminal={id=>{terminal.id=id;terminal.error="";terminal.publish();openTab("terminal");setShowTests(false);}} />
    <SearchEverywhereView visible={showEverywhere} setVisible={setShowEverywhere} buffers={buffers} files={workspaceFiles} analysis={analysis} commands={commands} openFile={path=>{const file=buffers.values.get(path)?.file??findPath(path);if(file)void openFileVisit(file);else setNotice("File no longer listed. Refresh the workspace.");}} focusCurrent={()=>{if(buffer)editorForBuffer(buffer)?.view.focus();}} />
    <WorkspaceChanges history={changeHistory} adoptResources={routes=>{requested.current.clear();const next=resourceLayout(liveLayout.current,tabRoutes(routes),resourcePlacements.current);liveLayout.current=next;setLayout(next);void loadFiles();}} files={workspaceFiles} buffers={buffers} current={() => buffer} focus={() => { if (buffer) editorForBuffer(buffer)?.view.focus(); }} />
    <WorkspaceSearch visible={showWorkspaceSearch} close={restoreFocus => { setShowWorkspaceSearch(false); if (restoreFocus && buffer) editorForBuffer(buffer)?.view.focus(); }} buffers={buffers} files={workspaceFiles}
      navigate={(file,line,character,current,verify)=>openLine(file,line,character,{current,verify})} />
    <AgentProposalReview key={"agent-proposals:"+buffers.workspaceId} open={showAgentProposals} initialSelection={proposalSelection} close={()=>{proposalRequest.current++;setShowAgentProposals(false);setProposalSelection(null);if(buffer)editorForBuffer(buffer)?.view.focus();}} buffers={buffers} history={changeHistory} acquire={async path=>{const namespace=buffers.workspace,workspace=buffers.workspaceId,response=await api.files();if(response.root_path!==namespace||response.workspace_id!==workspace||buffers.workspace!==namespace||buffers.workspaceId!==workspace)throw Error('Proposal workspace changed while acquiring a target.');const file=findPath(path,response.tree);if(!file||file.dir)throw Error('Reviewed source is not available in this workspace: '+path);const target=await buffers.open(file);if(buffers.workspace!==namespace||buffers.workspaceId!==workspace)throw Error('Proposal workspace changed.');return target;}} adopt={routes=>{requested.current.clear();const next=resourceLayout(liveLayout.current,tabRoutes(routes),resourcePlacements.current);liveLayout.current=next;setLayout(next);void loadFiles();}} />
    {showSourceHistory&&buffer&&<SourceHistoryView key={buffers.workspace+":"+buffer.file.path} buffer={buffer} buffers={buffers} history={changeHistory} alive={()=>workspaceIdentity.current===workspaceRoot} close={()=>{setShowSourceHistory(false);editorForBuffer(buffer)?.view.focus();}} />}
    {showEditorRecovery&&<EditorRecoveryView model={editorRecovery} workspace={buffers.workspaceId} namespace={buffers.workspace} lease={()=>editorRecoveryLease(buffers,()=>liveLayout.current,()=>recoveryFocus.current,()=>retainedTabs.buffers.values())} restore={snapshot=>{const next=restoreEditorRecovery(buffers,snapshot,liveLayout.current);requested.current.clear();liveLayout.current=next.tree;setLayout(next.tree);setFocused(next.focused);}} close={()=>setShowEditorRecovery(false)}/>}
    {showRecovery&&<SourceRecoveryView editorSessions={()=>{setShowRecovery(false);setShowEditorRecovery(true);}} key={"recovery:"+workspaceRoot} buffers={buffers} epoch={()=>changeHistory.stamp()} alive={()=>workspaceIdentity.current===workspaceRoot} acquire={async path=>{
      const identity=workspaceIdentity.current,response=await api.files();if(workspaceIdentity.current!==identity||response.root!==identity)throw new Error("Workspace changed. Read recovery again.");
      const find=(nodes:FileNode[]):FileNode|undefined=>{for(const file of nodes){if(file.path===path&&!file.dir)return file;const nested=find(file.children??[]);if(nested)return nested;}};
      const file=buffers.values.get(path)?.file??find(response.tree);if(!file)throw new Error("Recovery source is not available in the current workspace: "+path);return await buffers.open(file);
    }} adopt={(routes:ResourceEditorRoute<Buffer>[],inverse:string|null)=>{if(inverse)changeHistory.record({kind:"resource",recovery:inverse});requested.current.clear();const next=resourceLayout(liveLayout.current,tabRoutes(routes),resourcePlacements.current);liveLayout.current=next;setLayout(next);void loadFiles();setNotice("Saved baselines and drafts are retained separately.");}} close={()=>{setShowRecovery(false);if(buffer)editorForBuffer(buffer)?.view.focus();}}/>}
    {showConstruction&&<ConstructionReadingView key={"construction:"+workspaceRoot} workspace={buffers.workspace} engineWorkspace={buffers.workspaceId} capture={()=>{const namespace=buffers.workspace,engine=buffers.workspaceId;try{const roster=captureResourceRoster(buffers);return ()=>!!namespace&&buffers.workspace===namespace&&buffers.workspaceId===engine&&workspaceIdentity.current===workspaceRoot&&roster.current();}catch{return ()=>false;}}} terminal={id=>{terminal.id=id;terminal.error="";terminal.publish();openTab("terminal");setShowConstruction(false);}} retainedSources={()=>{const roster=captureResourceRoster(buffers);return {sources:roster.ownership_sources,unaddressed_sources:roster.unaddressed_sources,saved_guards:Object.fromEntries(roster.captures.filter(row=>Object.hasOwn(roster.ownership_sources,row.path)&&!!row.guard).map(row=>[row.path,row.guard!])),missing_sources:roster.captures.filter(row=>row.deleted&&!row.readonly&&!row.transient&&!row.path.startsWith('/')&&!row.buffer.isUntitledDraft).map(row=>row.path)};}} sources={()=>captureResourceRoster(buffers).ownership_sources} navigate={async(point,current)=>{
      const file=buffers.values.get(point.path)?.file??findPath(point.path);if(!file)throw new Error("Source no longer listed. Refresh the construction reading.");
      const departed=currentLocation();
      const location: {offset: number | null} = {offset: null};
      const opened=await open(file,{current,verify:async target=>{
        const source=target.text;
        const hash=[...new Uint8Array(await crypto.subtle.digest("SHA-256",new TextEncoder().encode(source)))].map(byte=>byte.toString(16).padStart(2,"0")).join("");
        location.offset=constructionPointOffset(source,point,hash);
        if(target.text!==source||location.offset===null)throw new Error("Source changed or position is unavailable. Refresh the construction reading.");
      }});
      if(!opened?.current()||location.offset===null)return;
      const offset=location.offset,target=opened.buffer,source=target.text;
      for(let attempt=0;attempt<20;attempt++){if(!opened.current()||target.text!==source)return;const view=editorForBuffer(target)?.view;if(view?.dom.isConnected){if(authoredSource(view.state)!==source)throw new Error("Editor source changed. Refresh the construction reading.");view.dispatch({selection:{anchor:editorOffset(view.state,offset)},scrollIntoView:true});view.focus();recordVisit(departed,point.path,{line:point.line-1,character:point.character});setShowConstruction(false);return;}await new Promise<void>(resolve=>requestAnimationFrame(()=>resolve()));}
      throw new Error("Source opened but its editor is not ready. Try the source link again.");
    }} close={()=>{setShowConstruction(false);if(buffer)editorForBuffer(buffer)?.view.focus();}} />}
    {attributionPath!==null&&<GitAttributionView key={"attribution:"+workspaceRoot} workspace={workspaceRoot} path={attributionPath} close={()=>{setAttributionPath(null);if(buffer)editorForBuffer(buffer)?.view.focus();}} readCommit={(sha,title)=>{setAttributionPath(null);setShowReview(false);setHistoryRequest({kind:'commit',sha,title});setShowHistory(true);}} />}
    <GitHistoryView key={"history:"+workspaceRoot} workspace={workspaceRoot} visible={showHistory} request={historyRequest} openComparison={async (comparison,current)=>{
      const namespace=workspaceIdentity.current,file=buffers.values.get(comparison.path)?.file??findPath(comparison.path);
      if(!file||file.dir)return false;
      const opened=await open(file,{current:()=>current()&&workspaceIdentity.current===namespace});
      if(!opened?.current()||!current()||opened.buffer.file.path!==comparison.path)return false;
      opened.buffer.setGitComparison(comparison);return true;
    }} close={()=>{setShowHistory(false);if(buffer)editorForBuffer(buffer)?.view.focus();}} />
    <GitReviewView key={`git-review:${overlayScope}`} workspace={overlayScope} buffers={buffers} history={changeHistory} alive={overlayCurrent} adopt={routes=>{requested.current.clear();const next=resourceLayout(liveLayout.current,tabRoutes(routes),resourcePlacements.current);liveLayout.current=next;setLayout(next);void loadFiles();}} acquire={async path=>{if(!overlayCurrent())throw new Error("Workspace changed. Prepare the conflict again.");const file=buffers.values.get(path)?.file??findPath(path);if(!file||file.dir)throw new Error("Conflict source is no longer listed. Refresh files and try again.");const target=await buffers.open(file);if(!overlayCurrent())throw new Error("Workspace changed. Prepare the conflict again.");return target;}} visible={showReview} close={()=>{setShowReview(false);if(buffer)editorForBuffer(buffer)?.view.focus();}} />
    <footer role="status" title={statusNotices}>{buildStamp && <DevBuildStatus ui={buildStamp} />}<WorkspaceAnalysisView buffers={buffers} files={workspaceFiles} model={analysis} focusCurrent={() => { if (buffer) editorForBuffer(buffer)?.view.focus(); }} /><WorkspaceProblems buffers={buffers} source={buffer ?? undefined} visible={showProblems} setVisible={setShowProblems} current={currentLocation} focusCurrent={() => { if (buffer) editorForBuffer(buffer)?.view.focus(); }} />{active === "agent" ? <AgentStatus session={agent} /> : <>{statusNotices ? `${statusNotices} · ` : ""}{buffer && <EditorPosition buffer={buffer} />}{buffer ? `${buffer.file.name} · ${buffer.resourceDeleted?"Deleted source · retained snapshot":buffer.readonly ? "Read-only" : buffer.saving ? "Saving…" : buffer.dirty ? "Modified" : "Saved"}` : status}</>}</footer>
  </main></SourceCoverageProvider></SourceTestGutterProvider></SimpleLanguages>;
}
function WorkspaceWelcome({ retry }: { retry: () => Promise<string | null> }) {
  const [connecting, setConnecting] = useState(false);
  const [address, setAddress] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  async function retryCurrent() {
    setBusy(true);
    setError("");
    const failure = await retry();
    if (failure) {
      setError("Could not open the workspace on this server. Check its address and try again.");
      setBusy(false);
    }
  }
  function connect() {
    let url: URL;
    try {
      url = new URL(address.trim());
      if (!["http:", "https:"].includes(url.protocol) || url.username || url.password ||
          url.pathname !== "/" || url.href.includes("?") || url.href.includes("#") || url.port === "0") {
        throw new Error();
      }
    } catch {
      setError("Enter the server’s HTTP or HTTPS address, without a path or credentials.");
      return;
    }
    if (url.origin === window.location.origin) void retryCurrent();
    else window.location.assign(new URL("/simple.html", url));
  }
  const pendingReason = busy ? "A connection attempt is in progress." : undefined;
  const connectReason = pendingReason ?? (!address.trim() ? "Enter the workspace server address." : undefined);
  return <main className="workspace-welcome"><TooltipLayer /><section aria-labelledby="workspace-welcome-title">
    <h1 id="workspace-welcome-title">Welcome to Hickory Docs</h1>
    {connecting ? <form onSubmit={event => { event.preventDefault(); connect(); }}>
      <label htmlFor="workspace-server-address">Workspace server address</label>
      <input id="workspace-server-address" autoFocus required disabled={busy} data-tip={pendingReason} aria-description={pendingReason} value={address}
        placeholder="https://server:port" onChange={event => setAddress(event.target.value)} />
      <div className="workspace-welcome-actions">
        <button type="submit" disabled={!!connectReason} data-tip={connectReason} aria-description={connectReason}>{busy ? "Connecting…" : "Connect"}</button>
        <button type="button" disabled={busy} data-tip={pendingReason} aria-description={pendingReason} onClick={() => { setConnecting(false); setError(""); }}>Back</button>
      </div>
    </form> : <div className="workspace-welcome-actions">
      <button type="button" disabled={busy} data-tip={pendingReason} aria-description={pendingReason} onClick={() => { setConnecting(true); setError(""); }}>Connect to a Workspace Server…</button>
    </div>}
    {error && <p role="alert">{error}</p>}
  </section></main>;
}

function Tree({ nodes, open }: { nodes: FileNode[]; open: (file: FileNode) => void }) {
  return <ul>{nodes.map(node => <li key={node.path}>{node.dir
    ? <details><summary>{node.name}</summary><Tree nodes={node.children ?? []} open={open} /></details>
    : <button title={node.path} onClick={() => open(node)}>{node.name}</button>}</li>)}</ul>;
}
interface LayoutProps {
  node: Layout; collapsedFilesPane: string | null; focused: string; focus: (id: string) => void; label: (tab: string) => string;
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
  if (isGroup(node)) return <section className={`group ${props.focused === node.id ? "focused" : ""}`}
    style={props.collapsedFilesPane === node.id ? {visibility: "hidden"} : undefined} onFocusCapture={() => props.focus(node.id)}>
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
  const firstHidden = props.collapsedFilesPane === node.first.id;
  const secondHidden = props.collapsedFilesPane === node.second.id;
  const collapsed = firstHidden || secondHidden;
  const tracks = firstHidden ? "0px 0px minmax(0,1fr)" : secondHidden ? "minmax(0,1fr) 0px 0px"
    : `${node.ratio}fr 5px ${1 - node.ratio}fr`;
  return <div ref={host} className={`split ${node.axis}`} style={{ gridTemplateColumns: horizontal ? tracks : undefined, gridTemplateRows: horizontal ? undefined : tracks }}>
    <LayoutView {...props} node={node.first} />
    <div className="resize" role="separator" style={collapsed ? {visibility: "hidden"} : undefined} tabIndex={collapsed ? -1 : 0} aria-label="Resize panes" aria-orientation={horizontal ? "vertical" : "horizontal"} aria-valuenow={Math.round(node.ratio * 100)}
      onPointerDown={resize} onKeyDown={event => { const delta = ["ArrowRight", "ArrowDown"].includes(event.key) ? .05 : ["ArrowLeft", "ArrowUp"].includes(event.key) ? -.05 : 0; if (delta) { event.preventDefault(); props.resize(node.id, Math.max(.1, Math.min(.9, node.ratio + delta))); } }} />
    <LayoutView {...props} node={node.second} />
  </div>;
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

The browser shows its own stamp before the first engine reply. Poll outside
the editor lifecycle and abort on unmount. A failed read replaces a previous
verdict so a disconnected browser cannot keep claiming Latest. Details float
above the existing status bar, including in the Linux and Windows webviews.

<ui:copy id="dev-browser-status-view">

type DevBuildReading = { backend: string; checkout: string; verdict: string };

function EditorPosition({ buffer }: { buffer: Buffer }) {
  useSyncExternalStore(buffer.subscribe, buffer.snapshot);
  const { line, character } = utf16ToPosition(buffer.text, Math.min(buffer.caretOffset, buffer.text.length));
  return <span aria-label="Editor position">Ln {line + 1}, Col {character + 1} · </span>;
}

function DevBuildStatus({ ui }: { ui: string }) {
  const [reading, setReading] = useState<DevBuildReading | null>();
  const [open, setOpen] = useState(false);
  useEffect(() => {
    const lifetime = new AbortController();
    const url = "/api/dev-build?" + new URLSearchParams({ ui: ui.replace("+", "~") });
    let timer: ReturnType<typeof setTimeout>;
    async function observe() {
      try {
        const response = await fetch(url, {
          cache: "no-store",
          signal: AbortSignal.any([lifetime.signal, AbortSignal.timeout(4000)]),
        });
        if (!response.ok) throw new Error("Build status unavailable");
        const value = await response.json() as DevBuildReading;
        if (typeof value.backend !== "string" || typeof value.checkout !== "string" || typeof value.verdict !== "string") {
          throw new Error("Invalid build status");
        }
        if (lifetime.signal.aborted) return;
        setReading(value);
      } catch {
        if (lifetime.signal.aborted) return;
        setReading(null);
      }
      timer = setTimeout(observe, 5000);
    }
    void observe();
    return () => { lifetime.abort(); clearTimeout(timer); };
  }, [ui]);
  const short = ui === "unknown" ? "?" : ui.slice(0, 8) + (ui.endsWith("+") ? "+" : "");
  const describe = (stamp = "unknown") => stamp === "unknown" ? stamp
    : stamp.endsWith("+") ? stamp.slice(0, -1) + " · uncommitted changes" : stamp + " · clean";
  const label = `${reading?.verdict ?? (reading === null ? "Backend unknown" : "Checking")} · ${short}`;
  return <span onKeyDown={event => { if (event.key === "Escape") setOpen(false); }}>
    <button className="simple-build-status" aria-expanded={open} aria-label={`Development build ${label}`}
      onClick={() => setOpen(value => !value)}>{label}</button>
    {open && <span className="simple-build-details">
      <span>UI · {describe(ui)}</span>
      <span>Backend · {describe(reading?.backend)}</span>
      <span>Engine checkout · {describe(reading?.checkout)}</span>
    </span>}
  </span>;
}
</ui:copy>

<ui:file path="apps/web/src/simple/App.tsx"><ui:paste select="#simple-e8a50a27b2-0"/><ui:paste select="#simple-e8a50a27b2-1"/><ui:paste select="#dev-browser-status-view"/></ui:file>

</ui:doc>
