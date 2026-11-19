import { workspaceTreeFiles } from "./workspaceTreeFiles";
import { useUiReading } from "../lib/uiReading";
import { runWorkspaceMenu } from "./workspaceMenuActions";
import { useWorkspaceMenuActions } from "./uiWorkspaceMenuActions";
import { useFileTabMenu } from "./useFileTabMenu";
import { documentCommands } from "./documentCommands";
import { resolveDocumentPath, openIndexedDocument } from "./indexedDocumentPath";
import { viewBuffers } from "../lib/viewBuffers";
import { hasCommandInstaller } from "./CommandPathSettings";
// One workspace owns the layout across navigation. Documents and tools open
// on demand; the startup buffer is an editable introduction without a file.
// Per-document machinery lives in documentSession.tsx.
import {
  Suspense,
  lazy,
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { useRecentFiles } from "./useRecentFiles";
import { STARTUP_INTRODUCTION } from "../lib/newDoc";
import { api, request as callApi } from "../api/client";
import { MergeDriverNotice } from "../components/MergeDriverNotice";
import type {
  Doc,
  DocSummary,
  FileNode,
  FilesResponse,
  OpenTerminal,
  PlainFile,
  SearchHit,
  SearchResponse,
  FindFile,
  FindMatch,
} from "../api/types";
import { WorkspaceChat } from "./deferredAgent";
import { InsertMenu } from "../components/InsertMenu";
import { PlainFilePane } from "../components/PlainFilePane";
import { ScratchpadPane } from "../components/ScratchpadPane";
import { SearchPanel } from "../components/SearchPanel";
import { ReferencesPanel, type ReferenceReading } from "../components/ReferencesPanel";
import { WorkspaceProblems } from "../environments/WorkspaceProblems";
import type { ProblemRow } from "../components/ProblemsPanel";
import { diagnosticBoundSource } from "../lsp/diagnosticRevision";
import { useEnvironments } from "../environments/useEnvironments";
import { EnvironmentNotice } from "../environments/EnvironmentPanel";
import { PromptPanel, usePrompt } from "../components/PromptPanel";
import { bufferEditor, openBufferTarget } from "../lib/editorBufferTargets";
import { workspaceRibbonSources } from "./workspaceRibbonSources";
import { matchesRankedSearchHit, matchesWorkspaceSymbol, openPlainSearchFiles, searchWorkspace } from "../lib/workspaceSearch";
import { NavigationTrail, searchCharacter, semanticLocations, semanticPath, type SemanticLocation } from "../simple/Intelligence";
import { authoredSource } from "../simple/SourceText";
import { documentUri } from "../lsp/documentUri";
import { workspaceLspClient } from "../lsp/useLsp";
import type { MenuAction } from "../lib/menuBridge";
import { captureInsertion, insertionCurrent, insertCapturedElement } from "../editor/insertElement";
import {
  elementForExisting,
  initialValues,
  renderExistingElement,
  type FieldValues,
  type InsertElement,
} from "../lib/insertCatalog";
import { structureOf } from "../editor/wysiwyg";
import { useRibbonPresentation } from "../components/LineageSettings";
import { useWorkspaceAppearance, useWorkspaceSettings } from "./useWorkspaceAppearance";
import { workspaceIsVisible } from "../lib/workspaceVisibility";
import { windowTitle } from "../lib/windowTitle";
import { deriveRibbons } from "../lib/ribbons";
import { flashTab } from "../lib/flashTab";
import { SpatialShellView as ShellView } from "../shell/SpatialShellView";
import { requestSpatialRibbonOpen, revealSpatialSource } from "../shell/spatialNavigation";
import type { ShellPort } from "../shell/ShellView";
import {
  RibbonOverlay,
  type RibbonFile,
  type RibbonLink,
  type RibbonSource,
  type RibbonFamily,
} from "../shell/Ribbons";
import { samePath } from "../lib/paths";
import { WorkspaceStatusExtras } from "../environments/WorkspaceStatusExtras";
import { PaneHeaderContent } from "../shell/PaneHeader";
import { FileToolMenu } from "../shell/FileToolMenu";
import { plainSearchEditor } from "../lib/workspaceSearch";
import { ReviewEvents } from "../shell/ReviewEvents";
import { AgentStatusBar } from "../shell/AgentStatus";
import { findNodeByPath, joinRel } from "../shell/workspacePaths";
import { nextWorkspaceProblem, validProblemPosition } from "../shell/nextWorkspaceProblem";
import {
  loadProvenanceLayers,
  saveProvenanceLayers,
  toggleLayer,
  type ProvenanceLayer,
} from "../lib/provenanceLayers";
import type { ContextWrite, DeclaredCite } from "../api/types";
import {
  FILES_CHANGED_EVENT,
  FILES_DELETED_EVENT,
  FolderTreePane,
  useFolderTrees,
  fileAction,
} from "../shell/FolderTreePane";
import {
  paneById, open as openInLayout, openInBackground, retargetResourceTabs, tab as makeTab,
  panes as panesOf,
  type Layout,
  type Tab,
} from "../shell/layout";
import { paneBoxes, prepareRibbonOpen } from "../shell/ribbonNavigation";
import { regionsOf } from "../shell/layouts";
import type { Region } from "../shell/layout";
import { navigate, redirect, type Route, newDocument } from "../router";
import { beginsChord, isAction } from "../lib/keymap";
import { useNewDocument } from "./useNewDocument";
import { useUntitledSave } from "./useUntitledSave";
import { useUnsavedLifecycle } from "./useUnsavedLifecycle";
import { lensSources, onLensChange } from "../lib/lensSources";
import { welcomeActionsFor } from "./welcomeActions";
import {
  FLEET_TAB,
  MERGED_TAB,
  WELCOME_TAB,
  activateDocTab,
  adoptPlainFileTab,
  closeTabsForDeletedPaths,
  docIdsIn,
  findDocTab,
  findFileTab,
  focusedDocId,
  initialWorkspace,
  isWorkspaceEmpty,
  openChatTab,
  openDocTab,
  openFileTab,
  renameFileTab,
  openGeneratedTab,
  openGitTab,
  openIntoDeclared,
  openScratchpadTab,
  openStoryTab,
  openTerminalTab,
  openUntitledTab,
  openWelcomeTab,
} from "./workspaceState";
import {
  DocSessionHost,
  SessionRegistry,
  useSessionVersion,
} from "./documentSession";
import { AttentionCard } from "../terminal/AttentionCard";
import { MonitorDock } from "../terminal/MonitorDock";
// The terminal emulator is a quarter of a megabyte of JavaScript that a
// window without a terminal open never runs. Loaded the first time a terminal
// tab is drawn, so the document you opened the app for is on screen sooner.
import { AnchorBar } from "../terminal/AnchorBar";
const TerminalPane = lazy(() =>
  import("../terminal/TerminalPane").then((m) => ({ default: m.TerminalPane })),
);
import { sessionById, useTerminals } from "../terminal/useTerminals";
import { nextInQueue } from "../lib/attentionCursor";
import { DocTabBody, GeneratedTabBody, UntitledTab } from "./workspaceTabs";
import type { RecordedPatchSource, ResourceAdoptionStage } from "../simple/AgentProposalAdapter";
import { WorkspaceChangeHistory, planWorkspaceEdit, providerActionPaths, validateReferencedTargets, workspaceEditTargets, type ChangePlan, type WorkspaceHistoryToken } from "../simple/ChangePlan";
import { acquireReferenceSources, acquireSourceProposal } from "../simple/ReferenceSources";
import { validateProviderSourceProofs, providerSourceProofPaths } from "../simple/ProviderSourceProofs";
import { validateSourcePreparation } from "../simple/WorkspaceLanguages";
import { sourceDigest } from "../lsp/diagnosticRevision";
import { WsRealtime } from "../api/realtime";
import { ChangePreview } from "../components/ChangePreview";
import type { ResourcePublicationReview, ResourceTransport } from "../simple/ResourceEditors";
const ResourceConsequencesView = lazy(() => import("../simple/SourceRecovery").then(module => ({default: module.ResourceConsequencesView})));
import type { WindowHistoryPlan } from "./WorkspaceChat";
import type { UiOutcome } from "../lib/uiActions";
import { useUntitledRecovery } from "./useUntitledRecovery";
import { restoreFolderLayout } from "./restoreFolderLayout";
import { useWorkspaceUi } from "./useWorkspaceUi";
import { observeManagedStartup, observePlainStartup, openSelectedFile, useFolderPane, type StartupManagedDocument, type StartupPlainFile } from "./folderPane";
import { hickLangReady, useHickLangStatus } from "../editor/hickLang";
import { activeEditor, focusedEditor } from "../editor/activeEditor";
import { editorLanguageActions } from "../editor/languageActions";
import { offsetToPosition, type LocalSourceNavigation, type RelatedLocationReading } from "../lsp/cmLsp";
import { useZoom } from "./useZoom";
import { StatusBar } from "../shell/StatusBar";
import { WelcomePane, type WelcomeAction } from "./WelcomePane";
import { reviewTool, useRepresentationTabs } from "./representationTabs";
import { PlainDebugHosts } from "../debug/plainDebugHosts";
import { FleetPane } from "./FleetPane";
import { MergedView } from "./MergedView";
import {
  CommandBar,
  openCommandBar,
  type CommandItem,
  type CommandMode,
} from "../shell/CommandBar";
import { loadShowWelcome } from "../lib/welcomePref";
import { totalProblems } from "../lib/problems";
import { EditorView } from "@codemirror/view";
import { TAB_ZOOM_VAR } from "../lib/zoom";
import { requestFlushSaves } from "../lib/flushSaves";
import { afterSourceTabActivation, onOpenLocation, onShowTerminal, pathOfDocUri, revealLine, type SourceRevealTab } from "../lib/revealLine";
import { historySourceHash } from "../simple/SourceHistory";
import type { CoverageSurfaceClient } from "../coverage/CoverageSurface";
import { rankFiles } from "../lib/fileRanking";
import { allFileProblems, useFileProblemsVersion } from "../lib/fileProblems";
import { formatOnSave, loadFormatOnSave } from "../lib/formatOnSave";
import { formatView, type ProviderPreviewRequest } from "../lsp/cmLspFeatures";
import { codeActionMatchesKind, queryCodeActions, resolveCodeAction } from "../lsp/codeActionResolution";
import { LspClient, type CodeAction, type LspLocation } from "../lsp/client";
import { printText, printTitleFor } from "../lib/printing";

/** The routes the workspace answers. Everything else is App's. */
export type WorkspaceRoute = Extract<
  Route,
  { name: "doc" } | { name: "new" } | { name: "scratchpad" }
>;

const WorkspaceTools = lazy(() => import("../shell/WorkspaceTools").then(module => ({ default: module.WorkspaceTools })));

export function WorkspaceView({ route }: { route: WorkspaceRoute }) {
  const [tool, setToolKind] = useState<"connections" | "execution" | "review" | "import" | "export" | "help" | null>(null);
  const [toolTarget, setToolTarget] = useState<{ path: string | null; document?: string; source?: string; view?: EditorView | null }>({ path: null });
  const [paintedFamilies, setPaintedFamilies] = useState<ReadonlySet<RibbonFamily>>(new Set());
  const startup = useRef(route.name === "new" && route.introduction === true).current;
  const requestedFile = route.name === "new" ? route.file : undefined;
  const explicitFileStartup = useRef(!!requestedFile).current;
  const startupFile = useRef(requestedFile).current;
  const startupHint = useRef(new URLSearchParams(window.location.search).get("startupPath")).current;
  const [plainStartup, setPlainStartup] = useState<{id: string; observation: StartupPlainFile} | null>(null);
  const [managedStartup, setManagedStartup] = useState<{id: string; observation: StartupManagedDocument} | null>(null);
  const parserStatus = useHickLangStatus();
  const [layout, setLayout] = useState<Layout>(() =>
    startup ? openUntitledTab(initialWorkspace()) : initialWorkspace(),
  );
  const openLiterateLibrary = useRepresentationTabs(setLayout);
  const introductionTab = useRef(
    startup ? panesOf(layout.root).flatMap((pane) => pane.tabs).find((tab) => tab.kind === "untitled")?.id : undefined,
  ).current;
  const [treeFocusRequest, setTreeFocusRequest] = useState(0);
  useNewDocument(setLayout);
  // Re-derive ribbon sources and links when a lens comes or goes.
  const [lensTick, setLensTick] = useState(0);
  useEffect(() => onLensChange(() => setLensTick((n) => n + 1)), []);
  const layoutRef = useRef(layout);
  layoutRef.current = layout;
  const focusedPane = paneById(layout, layout.focus);
  const activeTab = focusedPane?.tabs[focusedPane.active];
  const focusKey = JSON.stringify([layout.focus, activeTab?.id, activeTab?.kind, activeTab?.target, activeTab?.docId]);
  const focusIntent = useRef({key: focusKey, tab: activeTab});
  const preparedFile = useRef<{file: PlainFile; current: ()=>boolean} | undefined>(undefined);
  const preparedOutput = useRef<{file: PlainFile; owner: string; current: ()=>boolean} | undefined>(undefined);
  const preparedDocument = useRef<{doc: Doc; current: ()=>boolean; workspaceCurrent: ()=>boolean} | undefined>(undefined);
  if (focusIntent.current.key !== focusKey) {
    focusIntent.current = {key: focusKey, tab: activeTab};
    if (preparedFile.current?.file.path !== activeTab?.target) preparedFile.current = undefined;
  }
  const takePreparedSource = useCallback((path: string) => {
    const prepared = preparedFile.current;
    if (prepared?.file.path !== path) return undefined;
    preparedFile.current = undefined;
    return prepared.current() ? prepared.file : undefined;
  }, []);
  // Refs let welcome and search actions reach callbacks declared later.
  const openTerminalRef = useRef<() => Promise<unknown> | void>(
    () => undefined,
  );
  const sourceVisit = useRef(0);
  const sourceActivation = useRef<(() => boolean) | undefined>(undefined);
  const flushSourceActivation = useCallback(() => {
    const poll = sourceActivation.current;
    if (poll?.() && sourceActivation.current === poll) sourceActivation.current = undefined;
  }, []);
  useLayoutEffect(flushSourceActivation, [focusKey, flushSourceActivation]);
  const onSourceActivation = useCallback((target: SourceRevealTab, current: () => boolean,
      deliver: (stillFocused: () => boolean) => void) => {
    sourceActivation.current = afterSourceTabActivation(target, () => focusIntent.current, current, deliver);
    flushSourceActivation();
  }, [flushSourceActivation]);
  const openHitRef = useRef<(path: string, line: number, into?: string, onOpen?: (target: SourceRevealTab)=>void) => boolean | void>(
    () => undefined,
  );
  const focusedPathRef = useRef<string | null>(null);
  const workspaceSearchRef = useRef<(query: string, limit: number) => Promise<SearchResponse>>(async () => ({ semantic: false, hits: [] }));
  const openSearchHitRef = useRef<(hit: SearchHit) => void>(() => {});
  const readSymbolCandidates = useRef<(term: string) => Promise<CommandItem[]>>(async () => []);
  // The tree and the outputs map are built further down; these let the
  // callbacks above read the current values without depending on declaration
  // order.
  const folderRootsRef = useRef<readonly FilesResponse[]>([]);
  const openableOutputsRef = useRef<ReadonlyMap<string, string>>(new Map());
  // The regions of a declared layout, when one was applied — only ever at a
  // fresh launch, into an empty workspace (see ensure below). They keep
  // routing generated files to their panes afterwards.
  const [regions, setRegions] = useState<readonly Region[]>([]);
  const regionsRef = useRef(regions);
  regionsRef.current = regions;

  // One live session per open document, published through the registry.
  const registry = useMemo(() => new SessionRegistry(), []);
  const documentSlots = useRef(new Map<string, string>());
  useSessionVersion(registry);

  // Project search (Mod-Shift-F). The folder's documents are kept alongside
  // so a hit in one of them can resolve to its route.
  const [searchOpen, setSearchOpen] = useState(false);
  const [folderDocs, setFolderDocs] = useState<DocSummary[]>([]);
  // The Insert panel: the element a native-menu pick named (null when it was
  // opened bare), and what was selected in the buffer at the moment it
  // opened. The selection is captured HERE, at open time, because the panel
  // takes the focus and CodeMirror's selection is no longer readable as
  // "what the person meant" once anything else has it.
  const [insertPanel, setInsertPanel] = useState<{
    id: string | null;
    selected: string;
    request: string;
    target: ReturnType<typeof captureInsertion>;
  } | null>(null);
  // The same form edits an existing element. Its source range is captured
  // with the parsed structure, rather than relying on the selection after
  // the dialog has taken focus.
  const [editPanel, setEditPanel] = useState<{
    element: InsertElement;
    request: string;
    target: ReturnType<typeof captureInsertion>;
    values: FieldValues;
    body: string;
    attrs: Readonly<Record<string, string>>;
    from: number;
    to: number;
    source: string;
  } | null>(null);
  // Why an insert could not happen. Rare — it needs a workspace with no
  // editor in it at all — but silence would look like a broken menu.
  const [insertNotice, setInsertNotice] = useState<string | null>(null);
  // The dock is part of the workspace, not a mode: it is always mounted and
  // remembers whether the log is expanded.
  const { style: ribbonStyle, visibility: ribbonVisibility } = useRibbonPresentation();
  // The folder's project id — what the zoomed-out lineage graph is keyed by.
  const [projectId, setProjectId] = useState<string | null>(null);
  // Which provenances the overlay draws — a live choice, remembered.
  const [layers, setLayers] = useState<ReadonlySet<ProvenanceLayer>>(() =>
    loadProvenanceLayers(),
  );
  const toggleProvenance = useCallback((layer: ProvenanceLayer) => {
    setLayers((current) => {
      const next = toggleLayer(current, layer);
      saveProvenanceLayers(next);
      return next;
    });
  }, []);
  const { tabStyle, channelWidth } = useWorkspaceAppearance();
  const [shellBox, setShellBox] = useState<HTMLElement | null>(null);
  // The workspace's own prompt, for the things that belong to the WINDOW
  // rather than to a document — asking for a worktree's branch name, now that
  // terminals have no pane of their own to ask on.
  const shellPrompt = usePrompt();
  const [untitledSources, setUntitledSources] = useState<Record<string, string>>(() => introductionTab ? { [introductionTab]: STARTUP_INTRODUCTION } : {});
  const plainSources = useRef(new Map<string, string>());
  const plainPatchOwners = useRef(new Map<string, () => RecordedPatchSource | null>());
  const [recordedDraftHistory] = useState(() => new WorkspaceChangeHistory());
  const changeTicket = useRef(0);
  const [changeReview, setChangeReview] = useState<{
    plan: ChangePlan; current: () => boolean; opensFiles: boolean; apply: () => Promise<void>;
  } | null>(null);
  const [resourceReview, setResourceReview] = useState<(ResourcePublicationReview & {history?: WorkspaceHistoryToken}) | null>(null);
  const [changePath, setChangePath] = useState(0);
  const [changePage, setChangePage] = useState(0);
  const [changePending, setChangePending] = useState(false);
  const [changeKind, setChangeKind] = useState<ProviderPreviewRequest["kind"] | "delete" | "move">("rename");
  const prepareFileResourceRef = useRef<(kind: "delete" | "move", view: EditorView, tab: Tab, current: () => boolean) => void>(() => {});
  const [changeError, setChangeError] = useState<string | null>(null);
  const changeApplying = useRef(false);
  const changeHistoryRef = useRef<(direction: "undo" | "redo") => void>(() => {});
  const untitledRevisions = useRef(new Map<string, number>());
  const registerPlainPatchOwner = useCallback((id: string, read: () => RecordedPatchSource | null) => {
    plainPatchOwners.current.set(id, read);
    return () => { if (plainPatchOwners.current.get(id) === read) plainPatchOwners.current.delete(id); };
  }, []);
  const agentFocusedEditor = useRef<string | null>(null);
  const agentActivePane = paneById(layout, layout.focus);
  const agentActiveTab = agentActivePane?.tabs[agentActivePane.active];
  if (agentActiveTab && ["document", "generated", "file", "untitled"].includes(agentActiveTab.kind)) agentFocusedEditor.current = agentActiveTab.id;
  const untitledSourcesRef = useRef(untitledSources);
  untitledSourcesRef.current = untitledSources;
  const forgetUntitled = useCallback((tabId: string) => {
    untitledRevisions.current.delete(tabId);
    setUntitledSources((current) => {
      if (!(tabId in current)) return current;
      const next = { ...current };
      delete next[tabId];
      return next;
    });
  }, []);
  const [retainSavedDrafts, setRetainSavedDrafts] = useState(false);
  const [plainDirtyTabs, setPlainDirtyTabs] = useState<ReadonlySet<string>>(new Set());
  const plainUnsavedActions = useRef(
    new Map<
      string,
      { save: () => Promise<boolean>; discard: () => void; retain: () => Promise<void>; revision?: () => unknown; isDirty?: () => boolean }
    >(),
  );
  const environments = useEnvironments();
  const fileProblemsVersion = useFileProblemsVersion();
  const problems = useMemo(
    () =>
      totalProblems([
        ...registry.all().map((open) => open.lspDiagnostics ?? []),
        ...allFileProblems().map((file) => file.diagnostics),
      ]),
    // registry.version (via useSessionVersion) is what actually changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [registry, registry.version, fileProblemsVersion],
  );
  // Find References asked from a plain file. A document keeps its own list
  // on its session; a file has no session, so the workspace holds it.
  const [fileReferences, setFileReferences] = useState<ReferenceReading | null>(null);
  const providerNavigation = useRef(0);
  // The list behind the status bar's count. Open/closed is all that is held;
  // the rows are read from the sessions when it draws, so a diagnostic that
  // arrives while it is open appears without a subscription of its own.
  const [problemsOpen, setProblemsOpen] = useState(false);
  // The branch, for the status bar. Refetched when files change — a commit,
  // a checkout, or a save can all move it.
  const [git, setGit] = useState<{ branch: string; dirty: number } | null>(
    null,
  );
  useEffect(() => {
    const read = () => {
      void api.gitStatus().then(
        (status) =>
          setGit(
            status.repository && status.branch
              ? {
                  branch: status.branch,
                  dirty:
                    (status.staged ?? 0) +
                    (status.unstaged ?? 0) +
                    (status.untracked ?? 0),
                }
              : null,
          ),
        // Not a repository, or no git: the status bar simply says nothing
        // about branches, which is the honest answer.
        () => setGit(null),
      );
    };
    read();
    window.addEventListener(FILES_CHANGED_EVENT, read);
    window.addEventListener("focus", read);
    return () => {
      window.removeEventListener(FILES_CHANGED_EVENT, read);
      window.removeEventListener("focus", read);
    };
  }, []);
  const focusedPath = useRecentFiles(layout);
  focusedPathRef.current = focusedPath;
  const welcomeActions: WelcomeAction[] = useMemo(
    () =>
      welcomeActionsFor({
        setLayout,
        openTerminal: () => openTerminalRef.current(),
        focusedPath: () => focusedPathRef.current,
        focusTree: () => focusTreeRef.current(),
      }),
    [],
  );
  const { roots: folderRoots, error: folderError, folderOpen } = useFolderTrees();
  folderRootsRef.current = folderRoots;
  const workspaceKey = JSON.stringify(folderRoots.map(root => [root.workspace_id, root.root_path ?? root.root]));
  const sourceWorkspace = useRef({key: workspaceKey});
  if (sourceWorkspace.current.key !== workspaceKey) sourceWorkspace.current = {key: workspaceKey};
  type HistoryPoint = SemanticLocation & {sourceHash: Promise<string | null>};
  const historyRef = useRef({workspace: sourceWorkspace.current, trail: new NavigationTrail<HistoryPoint>()});
  if (historyRef.current.workspace !== sourceWorkspace.current) {
    historyRef.current = {workspace: sourceWorkspace.current, trail: new NavigationTrail<HistoryPoint>()};
  }
  const navigateHistoryRef = useRef<(direction: -1 | 1, current?: () => boolean) => void>(() => {});
  const activeSourceEditor = useCallback(() => {
    const tab = focusIntent.current.tab;
    if (!tab || tab.retainedResource || !["file", "document", "generated"].includes(tab.kind)) return null;
    const body = [...(shellBox?.querySelectorAll<HTMLElement>("[data-workspace-tab]") ?? [])]
      .find(element => element.dataset.workspaceTab === tab.id);
    const focused = focusedEditor();
    if (focused && body?.contains(focused.dom)) return {tab, view: focused};
    const node = body?.querySelector<HTMLElement>(".cm-editor");
    const view = node && EditorView.findFromDOM(node);
    return view ? {tab, view} : null;
  }, [shellBox]);
  const makeHistoryPoint = useCallback((path: string, source: string, line: number, character: number,
    sourceHash = historySourceHash(source).catch(() => null)): HistoryPoint => {
    const start = {line, character};
    return {uri: documentUri(path), range: {start, end: start}, sourceHash};
  }, []);
  const readDeparture = useCallback(() => {
    const editor = activeSourceEditor();
    if (!editor) return null;
    const {tab, view: {state}} = editor;
    const {line, character} = offsetToPosition(state.doc, state.selection.main.head);
    return makeHistoryPoint(tab.target, authoredSource(state), line, character);
  }, [activeSourceEditor, makeHistoryPoint]);
  const recordLocalNavigation = useCallback<LocalSourceNavigation>((view, departure) => {
    const editor = activeSourceEditor();
    const {doc, selection} = view.state;
    if (editor?.view !== view || departure === selection.main.head) return;
    const from = offsetToPosition(doc, departure), to = offsetToPosition(doc, selection.main.head);
    const path = editor.tab.target, source = authoredSource(view.state);
    const entry = makeHistoryPoint(path, source, from.line, from.character);
    sourceVisit.current++;
    historyRef.current.trail.record(entry, makeHistoryPoint(path, source, to.line, to.character, entry.sourceHash));
  }, [activeSourceEditor, makeHistoryPoint]);
  const goToNextProblem = useCallback(() => {
    const active = activeSourceEditor();
    if (!active) return;
    const {tab, view} = active;
    const diagnostics = tab.kind === "file" ? allFileProblems().find(file => file.path === tab.target)?.diagnostics
      : tab.kind === "document" ? registry.get(tab.docId)?.lspDiagnostics : undefined;
    if (!diagnostics?.length) return;
    const departure = view.state.selection.main.head;
    nextWorkspaceProblem(view, diagnostics);
    recordLocalNavigation(view, departure);
  }, [activeSourceEditor, registry, recordLocalNavigation]);
  // F8 moves within the current source. The status-bar button opens Problems.
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!workspaceIsVisible()) return;
      // Reserve the first stroke of a workspace chord for its second stroke.
      if (beginsChord(event)) {
        event.preventDefault();
        return;
      }
      if (!isAction(event, "view.problems")) return;
      event.preventDefault();
      goToNextProblem();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [goToNextProblem]);
  const captureNavigationContext = useCallback(() => {
    const ticket = ++providerNavigation.current;
    const workspace = sourceWorkspace.current, focus = focusIntent.current;
    sourceVisit.current++;
    setFileReferences(null);
    return () => providerNavigation.current === ticket && sourceWorkspace.current === workspace && focusIntent.current === focus;
  }, []);
  const beginSourceOpen = useCallback((originCurrent?: () => boolean) => {
    const ticket = ++sourceVisit.current;
    const workspace = sourceWorkspace.current, focus = focusIntent.current;
    const workspaceCurrent = () => sourceWorkspace.current === workspace;
    const current = () => ticket === sourceVisit.current && workspaceCurrent();
    const beforeOpen = () => current() && focusIntent.current === focus && (!originCurrent || originCurrent());
    return {workspaceCurrent, current, beforeOpen};
  }, []);
  const prepareDocument = useCallback((doc: Doc, request: ReturnType<typeof beginSourceOpen>) => {
    if (!docIdsIn(layoutRef.current).includes(doc.id)) {
      preparedDocument.current = {doc, current: request.current, workspaceCurrent: request.workspaceCurrent};
    }
  }, []);
  const workspaceUi = useWorkspaceUi(layout, restored => setLayout(current => restoreFolderLayout(current, restored,
    folderOpen === true, introductionTab && untitledSourcesRef.current[introductionTab] === STARTUP_INTRODUCTION ? introductionTab : undefined)),
    folderOpen === null ? (folderError ? "window" : null) : folderOpen ? folderRoots[0]?.root_path ?? folderRoots[0]?.root ?? "folder" : "window",
    !explicitFileStartup);
  const untitledRecovery = useUntitledRecovery({
    hydrated: workspaceUi.hydrated, layoutRef, setLayout,
    sourcesRef: untitledSourcesRef, setSources: setUntitledSources,
    initialBaselines: introductionTab ? { [introductionTab]: STARTUP_INTRODUCTION } : {},
  });
  useUiReading("workspace restoration state", { layout_hydrated: workspaceUi.hydrated, untitled_buffers_restored: untitledRecovery.ready, message: untitledRecovery.ready ? null : "Restoring document…", source_read: false }, folderRoots.map(root => root.root));
  // ⌘+ / ⌘- / ⌘0 size the whole window; adding Alt sizes only the focused
  // tab. See views/useZoom.ts for why that split, and why it is the root's
  // font size rather than a transform.
  const zoom = useZoom({
    focusedTarget: () => {
      const pane = paneById(layoutRef.current, layoutRef.current.focus);
      const tab = pane?.tabs[pane.active];
      return tab && tab.kind !== "tree" && tab.kind !== "tool"
        ? tab.target
        : null;
    },
    zoomOfTab: (target) => workspaceUi.zoomFor(target),
    setTabZoom: workspaceUi.setZoom,
  });
  const derivedFocus = focusedDocId(layout);
  const [lastDocId, setLastDocId] = useState<string | null>(null);
  useEffect(() => {
    if (derivedFocus && derivedFocus !== lastDocId) setLastDocId(derivedFocus);
  }, [derivedFocus, lastDocId]);
  const focusedId = derivedFocus ?? lastDocId;
  const focusedIdRef = useRef(focusedId);
  focusedIdRef.current = focusedId;
  const focused = registry.get(focusedId);
  // ---- opening --------------------------------------------------------------
  const openGeneratedFor = useCallback((docId: string, path: string, into?: string) => {
    const already = panesOf(layoutRef.current.root).some((pane) =>
      pane.tabs.some((t) => t.kind === "generated" && t.target === path),
    );
    setLayout((current) =>
      openGeneratedTab(current, docId, path, regionsRef.current, into),
    );
    if (already) flashTab("generated", path);
  }, []);
  /** Open a plain file — same "ensure open" a document gets, no owner. */
  const openPlainFile = useCallback((path: string, into?: string) => {
    const already = findFileTab(layoutRef.current, path) !== null;
    setLayout((current) => openFileTab(current, path, into));
    if (already) flashTab("file", path);
  }, []);
  // Preserve the editor buffer while following a file rename.
  useEffect(() => {
    const onFilesChanged = (event: Event) => {
      const renames = (event as CustomEvent<{ renames?: { from: string; to: string }[] }>).detail
        ?.renames;
      if (!renames?.length) return;
      setLayout((current) =>
        renames.reduce((next, rename) => renameFileTab(next, rename.from, rename.to), current),
      );
    };
    window.addEventListener(FILES_CHANGED_EVENT, onFilesChanged);
    return () => window.removeEventListener(FILES_CHANGED_EVENT, onFilesChanged);
  }, []);
  // File operations announce only paths that actually left the host. Closing
  // here keeps all deletion entry points consistent, including Files drafts.
  useEffect(() => {
    const onFilesDeleted = (event: Event) => {
      const paths = (event as CustomEvent<{ paths?: string[] }>).detail?.paths;
      if (paths?.length) setLayout(current => closeTabsForDeletedPaths(current, paths));
    };
    window.addEventListener(FILES_DELETED_EVENT, onFilesDeleted);
    return () => window.removeEventListener(FILES_DELETED_EVENT, onFilesDeleted);
  }, []);
  /**
   * "Ensure open": activate the document's tab wherever it is, or fetch its
   * path and add a tab in the focused pane. The ONE exception to "add, never
   * arrange": a document that declares its own layout, opened into a still-
   * empty workspace (a fresh launch), gets its declared arrangement. A
   * workspace with anything open keeps its arrangement, always — merging a
   * declared layout into a busy workspace is out of scope, deliberately.
   */
  /** Pulse the tab that already shows `id` — "it's open, HERE". */
  const flashDocTab = useCallback((id: string) => {
    const existing = findDocTab(layoutRef.current, id);
    const tab = existing?.pane.tabs[existing.index];
    if (tab) flashTab("document", tab.target);
  }, []);
  const ensureDocOpen = useCallback((id: string, into?: string, recordHistory = false, revealAt?: number) => {
    const request = beginSourceOpen();
    const record = (target: string, source?: string) => {
      if (recordHistory || revealAt !== undefined) {
        onSourceActivation({kind: "document", target, docId: id}, request.current, stillFocused => {
          if (recordHistory) navigate(`/docs/${id}`);
          if (revealAt !== undefined && source !== undefined) {
            revealLine(target, revealAt, undefined, {source, current: stillFocused});
          }
        });
      }
    };
    const front = (target: string) => {
      setLayout(current => activateDocTab(current, id) ?? current);
      flashDocTab(id);
      record(target, registry.get(id)?.liveSource);
    };
    // A slow read can still finish, but only the original workspace and
    // foreground choice can put its tab on screen. The fetched document also
    // supplies that tab's read-only preview while the room connects.
    const existing = findDocTab(layoutRef.current, id);
    if (existing) {
      front(existing.tab.target);
      return;
    }
    const failed = () => {
      if (!request.beforeOpen()) return;
      // The session renders the load error in its own tab.
      setLayout(l => findDocTab(l, id) ? l : openDocTab(l, id, id, into));
      record(id);
    };
    const accept = (doc: Doc) => {
      if (!request.beforeOpen()) return;
      if (doc.id !== id) {
        setInsertNotice("The server returned a different document. Open the file again.");
        return;
      }
      const current = layoutRef.current;
      const existing = findDocTab(current, id);
      if (existing) {
        front(existing.tab.target);
        return;
      }
      prepareDocument(doc, request);
      if (!into && isWorkspaceEmpty(current) && hickLangReady()) {
        const declared = regionsOf(doc.source);
        if (declared.length > 0) {
          setRegions(declared);
          setLayout(openIntoDeclared(declared, id, doc.path));
          record(doc.path, doc.source);
          return;
        }
      }
      setLayout(l => activateDocTab(l, id) ?? openDocTab(l, id, doc.path, into));
      record(doc.path, doc.source);
    };
    const retained = registry.get(id)?.doc;
    if (retained) accept(retained);
    else void api.doc(id).then(accept, failed);
  }, [beginSourceOpen, flashDocTab, onSourceActivation, prepareDocument, registry]);
  /**
   * What the top field answers, in each of its modes.
   *
   * The bar itself knows nothing about this workspace — it parses a prefix
   * and asks. Which is what lets "go to line" mean the focused editor, and
   * "files" mean this folder, without the control having to be told.
   */
  const commandCandidates = useCallback(
    async (mode: CommandMode, term: string): Promise<CommandItem[]> => {
      if (mode === "line") {
        const line = Number(term);
        if (!Number.isFinite(line) || line < 1) return [];
        const view = focusedEditor();
        if (!view) return [];
        const focus = focusIntent.current;
        return [
          {
            id: `line-${line}`,
            label: `Go to line ${line}`,
            detail: focusedPathRef.current ?? "",
            run: () => {
              if (!view.dom.isConnected || focusIntent.current !== focus || focusedEditor() !== view) return;
              const departure = view.state.selection.main.head;
              const target = view.state.doc.line(
                Math.min(Math.floor(line), view.state.doc.lines),
              );
              view.dispatch({
                selection: { anchor: target.from },
                effects: EditorView.scrollIntoView(target.from, {
                  y: "center",
                }),
              });
              recordLocalNavigation(view, departure);
              view.focus();
            },
          },
        ];
      }
      if (mode === "command") {
        const focus = focusIntent.current;
        const history = historyRef.current;
        const view = activeSourceEditor()?.view;
        const editorCommands: CommandItem[] = [];
        if (view) {
          for (const action of view.state.facet(editorLanguageActions)) {
            const command = action.command;
            if (!command || !action.enabled(view)) continue;
            editorCommands.push({
              id: `editor:${command.id}`, label: command.label, agentAllowed: true,
              run: paletteCurrent => {
                if (!paletteCurrent?.() || focusIntent.current !== focus || !view.dom.isConnected ||
                    !view.state.facet(editorLanguageActions).includes(action) || !action.enabled(view)) return;
                command.run(view, paletteCurrent);
              },
            });
          }
        }
        const all: CommandItem[] = [...documentCommands(registry.get(focusedDocId(layoutRef.current))),
          ...editorCommands,
          ...(view && focus.tab && (focus.tab.kind === "file" || focus.tab.kind === "document") && !view.state.readOnly
            ? (["delete", "move"] as const).map(kind => ({
              id: `workspace-resources:${kind}`,
              label: kind === "delete" ? "Preview deletion of this file" : "Preview move of this file",
              run: (current?: () => boolean) => {
                if (current) prepareFileResourceRef.current(kind, view, focus.tab!, current);
              },
            })) : []),
          ...(["undo", "redo"] as const).filter(direction => recordedDraftHistory.preview(direction)).map(direction => ({
            id: `workspace-drafts:${direction}`, label: direction === "undo" ? "Undo workspace change" : "Redo workspace change",
            run: () => changeHistoryRef.current(direction),
          })),
          ...([[history.trail.canBack, -1, "back", "Navigate Back"], [history.trail.canForward, 1, "forward", "Navigate Forward"]] as const)
            .flatMap(([available, direction, id, label]): CommandItem[] => available ? [{
              id: `navigation.${id}`, label, agentAllowed: true, run: current => {
                if (current?.() && historyRef.current === history && focusIntent.current === focus) {
                  navigateHistoryRef.current(direction, current);
                }
              },
            }] : []),
          {id: "workspace-symbols", label: "Search workspace symbols", detail: "Type @ in search", agentAllowed: true, run: () => openCommandBar("symbols")},
          ...([ ["connections", "Connection display"], ["execution", "Tests and execution"], ["review", "Review this file"], ["import", "Import meeting"], ["export", "Export document snapshot"], ["help", "Workflow help"] ] as const).map(([id, label]) => ({ id: `tool-${id}`, label, detail: "Open a temporary panel", run: () => setTool(id) })),
          ...(projectId ? [{ id: "document-connections-graph", label: "Show document connections graph", detail: "Workspace lineage", run: () => navigate(`/projects/${encodeURIComponent(projectId)}/lineage`) }] : []),
          ...welcomeActions.map((action) => ({
            id: action.id,
            label: action.label,
            detail: action.hint,
            run: action.run,
            agentAllowed: true,
          })),
          {
            id: "settings",
            agentAllowed: true,
            label: "Settings",
            detail: "Appearance and agent settings",
            run: () => navigate("/settings"),
          },
          {
            id: "welcome",
            agentAllowed: true,
            label: "Welcome page",
            detail: "Start, and what is in this folder",
            run: () => setLayout(openWelcomeTab),
          },
        ];
        if (await hasCommandInstaller()) all.push({
          id: "install-hick-command", agentAllowed: true, label: "Shell Command: Install hick command in PATH…",
          detail: "Command line settings", run: () => navigate("/settings"),
        });
        const needle = term.toLowerCase();
        return all.filter((c) => c.label.toLowerCase().includes(needle));
      }
      if (mode === "symbols") return term ? readSymbolCandidates.current(term) : [];
      if (mode === "content") {
        if (!term) return [];
        const found = await workspaceSearchRef.current(term, 12);
        return found.hits.map((hit, index) => ({
          id: `${hit.path}:${index}`,
          label: hit.path,
          detail: hit.snippet,
          run: () => openSearchHitRef.current(hit),
        }));
      }
      // Rank all tree paths before capping the results.
      const files = workspaceTreeFiles(folderRootsRef.current);
      return rankFiles(files, term).map((node) => ({
        id: node.path,
        label: node.name,
        detail: node.path,
        run: () => openHitRef.current(node.path, 1),
      }));
    },
    [welcomeActions, registry, projectId, activeSourceEditor, recordLocalNavigation, recordedDraftHistory],
  );
  /**
   * Open a path and put the caret on a line.
   *
   * Shared by the tree's find results and the top field, and it reuses the
   * TREE's routing: a path may be a document, a generated output, or a plain
   * file, and the tree already knows which. Two answers to that question
   * would eventually disagree.
   */
  const openHit = useCallback(
    (path: string, line: number, into?: string, onOpen?: (target: SourceRevealTab)=>void) => {
      const publish = onOpen ?? ((tab: SourceRevealTab) => revealLine(tab.target, line));
      if (!onOpen && openBufferTarget(path, line, layoutRef.current, setLayout, setInsertNotice)) return true;
      const openDocument = (id: string, target: string, revealAt?: number) => {
        if (onOpen) {
          // The path is already verified. Let the session load in its own tab
          // rather than letting a second document request decide the focus.
          setLayout(current => activateDocTab(current, id) ?? openDocTab(current, id, target, into));
          onOpen({kind: "document", target, docId: id});
        } else {
          ensureDocOpen(id, into, true, revealAt);
        }
      };
      const document = registry.all().find((session) => session.doc && samePath(session.doc.path, path));
      if (document) {
        openDocument(document.docId, document.doc?.path ?? path);
        if (!onOpen) document.revealDocLine(Math.max(0, line - 1));
        return true;
      }
      const output = registry.all().find((session) => session.openOutputs.has(path));
      if (output) {
        openGeneratedFor(output.docId, path, into);
        publish({kind: "generated", target: path, docId: output.docId});
        return true;
      }
      const node = findNodeByPath(folderRootsRef.current, path);
      const action = node
        ? fileAction(node, new Set(openableOutputsRef.current.keys()))
        : ({ kind: "file", path } as const);
      if (action.kind === "doc") {
        openDocument(action.id, node?.path ?? path, line);
      } else if (action.kind === "generated") {
        const owner =
          action.docId ?? openableOutputsRef.current.get(action.path);
        if (!owner) return false;
        openGeneratedFor(owner, action.path, into);
        publish({kind: "generated", target: action.path, docId: owner});
      } else if (action.kind === "file") {
        openPlainFile(action.path, into);
        publish({kind: "file", target: action.path});
      } else return false;
      return true;
    },
    [registry, ensureDocOpen, openGeneratedFor, openPlainFile],
  );
  openHitRef.current = openHit;
  // Whether Save formats first, read once; Settings keeps it current.
  useEffect(() => {
    void loadFormatOnSave();
  }, []);
  // Provider results and coverage reports both acquire the destination before
  // activating a tab. Coverage can yield while hashing the text as well as
  // during the read. Recheck the report, the foreground and the live text at
  // the last point before activation. A source-only link leaves the caret alone.
  const visitSource = useCallback(async ({path, line, character, expectedSource, originCurrent, report, searchHit, findHit, provider, recordHistory = true, historyMove}: {
    path: string; line: number | null; character?: number; expectedSource?: string;
    originCurrent?: () => boolean; report?: {sourceHash: string | null}; searchHit?: SearchHit;
    findHit?: {sourceHash: string; text: string};
    provider?: {savedOnly: boolean; symbol?: string};
    recordHistory?: boolean;
    historyMove?: {sourceHash: Promise<string | null>; current: () => boolean; accept: () => boolean};
  }) => {
    const request = beginSourceOpen(originCurrent);
    const {current, beforeOpen} = request;
    if (!beforeOpen()) return false;
    const history = historyRef.current;
    const departure = recordHistory && line !== null ? readDeparture() : null;

    const retained = () => {
      const session = registry.all().find(item => item.doc?.path === path);
      const tab = panesOf(layoutRef.current.root).flatMap(item => item.tabs)
        .find(item => ["file", "generated", "document"].includes(item.kind) && item.target === path);
      return {session, tab, text: session?.liveSource ?? (tab && plainSources.current.get(tab.id))};
    };
    try {
      const first = retained();
      const listed = (node: FileNode | undefined | null, target: ReturnType<typeof retained>) =>
        !node?.dir && !!(node || target.tab || target.session);
      const firstNode = findNodeByPath(folderRootsRef.current, path);
      const documentId = (node: FileNode | undefined | null, target: ReturnType<typeof retained>) =>
        node?.doc_id ?? target.session?.docId ?? (target.tab?.kind === "document" ? target.tab.docId : undefined);
      const needsListedSource = !!(report || searchHit || findHit || provider || historyMove);
      if (needsListedSource && !listed(firstNode, first)) {
        throw Error("Source no longer listed. Refresh the workspace.");
      }
      const id = documentId(firstNode, first);
      // Managed documents have their own endpoint. Use its admitted source
      // to start a new session directly while rendering and the room connect.
      const readSaved = first.text === undefined || provider?.savedOnly;
      const doc = readSaved && id ? await api.doc(id) : undefined;
      const file = readSaved && !id ? await api.file(path) : undefined;
      if (!beforeOpen()) return false;
      if ((file && file.path !== path) || (doc && (doc.path !== path || doc.id !== id))) {
        throw Error("The server returned a different source. Select the exact location again.");
      }
      const saved = doc?.source ?? file?.content;
      const source = first.text ?? saved;
      if (source === undefined) return false;
      if (provider?.savedOnly && source !== saved) {
        throw Error("The target editor differs from the saved source. Save it and repeat the search.");
      }
      const hash = (report || findHit || historyMove) && line !== null ? await historySourceHash(source) : undefined;
      const rememberedHash = historyMove ? await historyMove.sourceHash : undefined;
      if (!beforeOpen()) return false;
      if (historyMove && (!rememberedHash || hash !== rememberedHash)) {
        throw Error("Source changed since this visit. Navigate from a current location to update history.");
      }
      const latest = retained();
      const node = findNodeByPath(folderRootsRef.current, path);
      if (needsListedSource && !listed(node, latest)) throw Error("Source no longer listed. Refresh the workspace.");
      const textChanged = latest.text === undefined ? first.text !== undefined : latest.text !== source;
      const lines = line === null ? [] : source.split("\n");
      const row = line === null ? undefined : lines[line - 1]?.replace(/\r$/, "");
      const column = character ?? 0;
      if (searchHit && !matchesRankedSearchHit(source, searchHit)) {
        throw Error("Search result changed. Search again to find its current location.");
      }
      if (findHit && (hash !== findHit.sourceHash || row !== findHit.text)) {
        throw Error("Saved search source changed. Find it again before opening the result.");
      }
      if (provider?.symbol && !matchesWorkspaceSymbol(row, column, provider.symbol)) {
        throw Error("The reported symbol has moved or changed. Search for it again.");
      }
      if (textChanged || documentId(node, latest) !== id || (expectedSource !== undefined && source !== expectedSource) ||
          (line !== null && !validProblemPosition(lines, {line: line - 1, character: column})) ||
          (report && hash !== undefined && hash !== report.sourceHash)) {
        throw Error(report ? "Source changed or report position is unavailable. Refresh coverage before navigating."
          : "Source or position changed. Query the location again.");
      }
      if (file && !latest.tab && !latest.session && (!node || (!node.doc_id && !node.generated_by && !node.dir))) {
        preparedFile.current = {file, current};
      }
      if (doc) prepareDocument(doc, request);
      return openHitRef.current(path, line ?? 1, undefined, target => {
        if (file && !latest.tab && target.kind === "generated" && target.docId && target.target === file.path) {
          preparedOutput.current = {file, owner: target.docId, current};
        }
        onSourceActivation(target, current, stillFocused => {
          if (target.kind === "document") navigate(`/docs/${target.docId}`);
          if (line !== null) {
            const tabId = focusIntent.current.tab?.id;
            const revealCurrent = () => stillFocused() && (!historyMove || historyMove.current());
            const onRevealed = recordHistory || historyMove ? () => {
              if (historyRef.current !== history || !revealCurrent()) return;
              if (historyMove) historyMove.accept();
              else history.trail.record(departure, makeHistoryPoint(path, source, line - 1, column));
            } : undefined;
            revealLine(target.target, line, character, {
              source, current: revealCurrent, onRevealed,
              accepts: editor => !!tabId && editor.closest<HTMLElement>("[data-workspace-tab]")?.dataset.workspaceTab === tabId,
            });
          }
        });
      }) === true;
    } catch (error) {
      if (!beforeOpen()) return false;
      throw error;
    }
  }, [beginSourceOpen, onSourceActivation, prepareDocument, registry, readDeparture, makeHistoryPoint]);
  const openProviderResult = useCallback(async (reading: Pick<ReferenceReading, "current" | "originUri">,
    location: LspLocation, symbol?: string) => {
    if (!reading.current()) {
      setInsertNotice("The source query is no longer current. Repeat it before opening a result.");
      return false;
    }
    const target = pathOfDocUri(location.uri.replace(/^hick-output:/, "hick:"));
    const origin = pathOfDocUri(reading.originUri.replace(/^hick-output:/, "hick:"));
    if (!target || !origin) {
      setInsertNotice("This result has no workspace source.");
      return false;
    }
    const {line, character} = location.range.start;
    try {
      return await visitSource({path: target, line: line + 1, character, originCurrent: reading.current,
        provider: {savedOnly: target !== origin, symbol}});
    } catch (error) {
      setInsertNotice(error instanceof Error ? error.message : String(error));
      return false;
    }
  }, [visitSource]);
  const showRelatedLocations = useCallback((reading: RelatedLocationReading) => {
    if (!reading.current()) return;
    const files = new Set(workspaceTreeFiles(folderRootsRef.current).map(file => file.path));
    const options = reading.locations.flatMap(location => {
      const path = semanticPath(location.uri.replace(/^hick-output:/, "hick:"));
      if (!path || !files.has(path)) return [];
      const {line, character} = location.range.start;
      const position = `${path}:${line + 1}:${character + 1}`;
      return [{label: location.name ? `${location.name}  ${position}` : position, value: location}];
    });
    void shellPrompt.askChoice(reading.title, options,
      {agent: true, filterable: true, onlyWhenIdle: true, currentRevision: reading.current}).then(location => {
      if (!location || !reading.current()) return;
      const name = location.name;
      const symbol = name && /^[\p{ID_Start}_$][\p{ID_Continue}$]*$/u.test(name) ? name : undefined;
      void openProviderResult(reading, location, symbol);
    });
  }, [openProviderResult, shellPrompt.askChoice]);
  const captureReadingPosition = useCallback(() => {
    const editor = focusedEditor();
    const document = editor?.state.doc, selection = editor?.state.selection;
    return () => !editor || (focusedEditor() === editor &&
      editor.state.doc === document && editor.state.selection === selection);
  }, []);
  navigateHistoryRef.current = (direction, paletteCurrent) => {
    const history = historyRef.current;
    const move = history.trail.prepareStep(direction, readDeparture());
    if (!move) return;
    const path = semanticPath(move.location.uri);
    if (!path) return;
    const {line, character} = move.location.range.start;
    const readingCurrent = captureReadingPosition();
    const historyCurrent = () => historyRef.current === history && move.current() && (!paletteCurrent || paletteCurrent());
    const originCurrent = () => historyCurrent() && readingCurrent();
    void visitSource({path, line: line + 1, character, recordHistory: false, originCurrent,
      historyMove: {sourceHash: move.location.sourceHash, current: historyCurrent, accept: move.accept}})
      .catch(error => setInsertNotice(error instanceof Error ? error.message : String(error)));
  };
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!workspaceIsVisible() || event.defaultPrevented || event.isComposing) return;
      const {trail} = historyRef.current;
      if (trail.canBack && isAction(event, "navigation.back")) {
        event.preventDefault();
        navigateHistoryRef.current(-1);
      } else if (trail.canForward && isAction(event, "navigation.forward")) {
        event.preventDefault();
        navigateHistoryRef.current(1);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
  // A provider change discovers its targets and captures each exact input
  // before preview. Apply needs a current draft owner for every patch, and
  // stages the whole group before changing any editor.
  const readChangeOwner = useCallback((path: string) => {
    const tabs = panesOf(layoutRef.current.root).flatMap(pane => pane.tabs)
      .filter(tab => (tab.kind === "document" || tab.kind === "file") && tab.target === path);
    if (tabs.length > 1) throw Error(`More than one tab owns ${path}. Close the duplicate first.`);
    const tab = tabs[0];
    const read = tab?.kind === "document" ? registry.get(tab.docId)?.recordedPatchSource
      : tab ? plainPatchOwners.current.get(tab.id) : undefined;
    return {tab, owner: read?.() ?? undefined};
  }, [registry]);
  const readChangeDisk = useCallback(async (path: string, singleFile?: Tab) => {
    const node = findNodeByPath(folderRootsRef.current, path);
    if ((!node && singleFile?.target !== path) || node?.dir || node?.generated_by) {
      throw Error(`Choose listed authored source: ${path}`);
    }
    const docId = node?.doc_id ?? singleFile?.docId;
    if (docId) {
      const doc = await api.doc(docId);
      if (doc.id !== docId || doc.path !== path || !doc.save_guard) throw Error(`Source authority changed: ${path}`);
      return {text: doc.source, guard: doc.save_guard, managed: true, docId: doc.id};
    }
    const file = await api.file(path);
    if (file.path !== path || !file.save_guard) throw Error(`Source authority changed: ${path}`);
    return {text: file.content, guard: file.save_guard, managed: false, docId: undefined};
  }, []);
  const captureResourceWindow = useCallback((tools: Pick<typeof import("./WorkspaceChat"), "captureWorkspacePatch">,
    scope: Parameters<typeof tools.captureWorkspacePatch>[1], current: () => boolean) => {
    const tabs = () => panesOf(layoutRef.current.root).flatMap(pane => pane.tabs)
      .filter(tab => tab.kind === "file" || tab.kind === "document" || tab.kind === "untitled");
    const originalTabs = tabs();
    const rosterCurrent = () => {
      const retained = tabs();
      return current() && retained.length === originalTabs.length &&
        retained.every((tab, index) => tab.id === originalTabs[index].id && tab.target === originalTabs[index].target);
    };
    return tools.captureWorkspacePatch(originalTabs, scope, tab => {
      if (tab.retainedResource) return tab.kind === "document"
        ? registry.get(tab.docId)?.recordedPatchSource() : plainPatchOwners.current.get(tab.id)?.();
      const owner = readChangeOwner(tab.target);
      return owner.tab?.id === tab.id ? owner.owner : undefined;
    }, id => {
      const view = bufferEditor(id);
      if (!view?.dom.isConnected) return null;
      const revision = untitledRevisions.current.get(id) ?? 0, doc = view.state.doc;
      return {text: doc.toString(), revision, current: () => bufferEditor(id) === view &&
        view.dom.isConnected && view.state.doc === doc && (untitledRevisions.current.get(id) ?? 0) === revision};
    }, rosterCurrent);
  }, [readChangeOwner, registry]);
  const observeResourceWindow = useCallback(async (capture: ReturnType<typeof captureResourceWindow>, edit: unknown,
    targets: string[], context: string, load: ResourceTransport, observe: typeof import("../simple/ResourceEditors").observeResourceInputs) => {
    const retained = await capture.retain();
    const paths = new Set(targets);
    const drafts = Object.fromEntries(Object.entries(retained.drafts).filter(([target]) => paths.has(target)));
    return observe({context, edit, ...retained, drafts}, load, capture.current);
  }, []);
  const prepareChangePreview = useCallback(async (request: ProviderPreviewRequest): Promise<UiOutcome> => {
    const operation = request.kind;
    const path = semanticPath(request.uri);
    if (changeApplying.current) throw Error("Wait for the current draft Apply to finish.");
    if (!path || !workspaceIsVisible() || activeSourceEditor()?.view !== request.view || !request.current()) {
      throw Error("Select a current authored source before preparing changes.");
    }
    const ticket = ++changeTicket.current, workspace = sourceWorkspace.current;
    const visible = () => changeTicket.current === ticket && sourceWorkspace.current === workspace;
    const snapshots = new Map<string, {text: string; base: string; guard: string; managed: boolean;
      tabId?: string; owner?: RecordedPatchSource}>();
    const current = () => visible() && request.current() && [...snapshots.values()].every(snapshot => !snapshot.owner || snapshot.owner.current());
    const check = () => { if (!current()) throw Error(`Source or editor changed. Prepare the ${operation} again.`); };
    setChangeKind(operation); setChangePending(true); setChangeReview(null); setResourceReview(null); setChangeError(null);
    let temporary: WsRealtime | undefined, secondary: LspClient | undefined;
    const capture = async (client: LspClient, uri: string, text: string, managed: boolean) => {
      check();
      const digest = await sourceDigest(text);
      check();
      const receipt = await client.request("hick/captureDocument", {textDocument: {uri, languageId: "hick", version: 1, text}});
      check();
      validateSourcePreparation(receipt, digest, managed);
      return {source_hash: digest, source_revision: (receipt as {source_revision: string}).source_revision};
    };
    try {
      const [
        {matchesScope, patchScope, captureWorkspacePatch, commitWorkspaceDrafts, invertWindowDraft},
        {isResourceEdit, observeResourceInputs, prepareResourceReview},
      ] = await Promise.all([import("./WorkspaceChat"), import("../simple/ResourceEditors")]);
      const files = folderRootsRef.current[0] ?? await api.files();
      check();
      const scope = patchScope(files);
      if (!scope) throw Error("The current workspace has no guarded source identity.");
      const initial = readChangeOwner(path);
      if (!initial.tab || !initial.owner || !matchesScope(scope, initial.owner.scope) || initial.owner.text !== authoredSource(request.view.state)) {
        throw Error("The originating editor has no exact guarded source. Reopen it and retry.");
      }
      const singleFile = files.folder_open === false ? initial.tab : undefined;
      const readDisk = (target: string) => readChangeDisk(target, singleFile?.target === target ? singleFile : undefined);
      const origin = {text: initial.owner.text, base: initial.owner.base, guard: initial.owner.guard,
        managed: initial.tab.kind === "document", tabId: initial.tab.id, owner: initial.owner};
      snapshots.set(path, origin);
      const acquire = async (target: string) => {
        const known = snapshots.get(target);
        if (known) return {text: known.text, writable: true};
        const retained = readChangeOwner(target);
        const disk = await readDisk(target);
        check();
        if (retained.owner && (!matchesScope(scope, retained.owner.scope) || retained.owner.guard !== disk.guard || !retained.owner.current())) {
          throw Error(`The saved source changed: ${target}. Refresh its editor.`);
        }
        const text = retained.owner?.text ?? disk.text;
        const held = retained.tab && plainSources.current.get(retained.tab.id);
        if (!retained.owner && held !== undefined && held !== disk.text) throw Error(`Open the retained draft for ${target} before preparing this change.`);
        snapshots.set(target, {text, base: disk.text, guard: disk.guard, managed: disk.managed,
          tabId: retained.tab?.id, owner: retained.owner});
        if (!secondary) {
          temporary = new WsRealtime("workspace");
          secondary = new LspClient(temporary.lsp());
        }
        const uri = documentUri(target);
        if (!retained.owner) secondary.didOpen(uri, text);
        await capture(secondary, uri, text, disk.managed);
        return {text, writable: true};
      };
      if (request.kind === "rename") await capture(request.client, request.uri, origin.text, origin.managed);
      const sources = new Map([[path, {text: origin.text, writable: true}]]);
      const reference = request.kind === "rename" ? await acquireReferenceSources(sources, async () =>
        (await request.client.references(request.uri, request.position)).map(location => {
          const target = semanticPath(location.uri);
          if (!target) throw Error("A reported reference has no editable workspace source.");
          return target;
        }), acquire, current) : null;
      let retainedChoice = request.kind === "code action" ? request.action : undefined;
      let resourceEditTargets: string[] | null = null;
      const load: ResourceTransport = (route, body) => callApi("POST", route, body);
      const proposal = await acquireSourceProposal(reference?.sources ?? sources, async () => {
        let value: CodeAction;
        if (request.kind !== "code action") {
          const expected = await capture(request.client, request.uri, origin.text, origin.managed);
          const document = {textDocument: {uri: request.uri}, expected};
          const bound = (request.kind === "rename"
            ? await request.client.request("hick/rename", {...document, position: request.position, newName: request.name})
            : await request.client.request("hick/formatting", {...document, options: request.options})
          ) as CodeAction & {source_files_modified?: boolean};
          if (!bound || bound.source_files_modified !== false) {
            throw Error(`The provider did not confirm a complete ${operation} preview.`);
          }
          value = bound;
        } else {
          // Resolve the retained token first. Re-sending its source would
          // restart provider diagnostics. Query again only after acquisition.
          let selected = retainedChoice;
          retainedChoice = undefined;
          if (!selected) {
            const choices = await queryCodeActions(request.client, request.uri, request.range, request.scope);
            check();
            const matches = choices.filter(action => action.title === request.action.title && action.kind === request.action.kind);
            if (matches.length !== 1) throw Error("The selected provider action changed. Request code actions again.");
            selected = matches[0];
          }
          if (selected.disabled) throw Error(selected.disabled.reason);
          value = await resolveCodeAction(request.client, selected, request.uri);
          if (request.scope && !codeActionMatchesKind(value, request.scope)) {
            throw Error("The provider changed the source action kind. Request code actions again.");
          }
        }
        check();
        if (value.disabled) throw Error(value.disabled.reason);
        if (value.command) throw Error(`"${value.title}" needs a language-server command this editor cannot execute.`);
        // A resource plan names missing destinations as well as existing
        // files. Only existing inputs can supply provider preimage proofs.
        if (isResourceEdit(value.edit)) {
          const {inputs} = await observeResourceInputs({
            context: crypto.randomUUID().replaceAll("-", ""), edit: value.edit,
            drafts: {}, ownership_sources: {}, unaddressed_sources: {}, saved_guards: {},
          }, load, current);
          if (inputs.workspace !== scope.workspace) throw Error("The resource proposal belongs to a different workspace.");
          resourceEditTargets = Object.entries(inputs.sources).flatMap(([target, source]) => source.kind === "file" ? [target] : []);
        } else resourceEditTargets = null;
        const paths = [...(resourceEditTargets ?? providerActionPaths(value)), ...providerSourceProofPaths(value)];
        // The action's proof binds the originating source already in sources.
        // It still participates in hash and saved-guard verification.
        return {paths: request.kind === "code action" ? paths.filter(target => target !== path) : paths, value};
      }, acquire, current);
      const requiredProofs = new Set(resourceEditTargets ?? workspaceEditTargets(proposal.value.edit).map(target => target.path));
      if (request.kind === "code action") requiredProofs.add(path);
      await validateProviderSourceProofs(proposal.value, proposal.sources, requiredProofs);
      check();
      const title = request.kind === "rename" ? `Rename ${request.previous} to ${request.name}`
        : request.kind === "code action" ? request.action.title : "Format document";
      if (isResourceEdit(proposal.value.edit)) {
        const edit = proposal.value.edit;
        const captured = captureResourceWindow({captureWorkspacePatch}, scope, current);
        const review = await prepareResourceReview(title, edit, crypto.randomUUID().replaceAll("-", ""), async target => {
          const source = proposal.sources.get(target);
          if (!source || !requiredProofs.has(target)) throw Error(`Resource source lacks an exact provider proof: ${target}`);
          return source.text;
        }, async (targets, context) => {
          const observation = await observeResourceWindow(captured, edit, targets, context, load, observeResourceInputs);
          if (observation.inputs.workspace !== scope.workspace) throw Error("Resource workspace changed. Prepare again.");
          return observation;
        }, load, captured.current);
        if (!review.current()) throw Error("Resource sources changed. Prepare the preview again.");
        setChangePath(0); setChangePage(0); setResourceReview({...review, operation: "save"});
        return {status: "completed", value: {preview_ready: true, source_files: review.patches.length, source_changed: false}};
      }
      const plan = planWorkspaceEdit(title, proposal.value.edit, proposal.sources);
      if (reference) validateReferencedTargets(plan, reference.references);
      const opensFiles = plan.patches.some(patch => !snapshots.get(patch.path)?.tabId);
      const apply = async () => {
        const readCurrentDisk = async (target: string) => {
          const snapshot = snapshots.get(target)!;
          const disk = await readDisk(target);
          check();
          if (disk.text !== snapshot.base || disk.guard !== snapshot.guard || disk.managed !== snapshot.managed) {
            throw Error(`Saved source changed: ${target}. Prepare the ${operation} again.`);
          }
          return disk;
        };
        const saved = new Map<string, Awaited<ReturnType<typeof readDisk>>>();
        for (const target of snapshots.keys()) saved.set(target, await readCurrentDisk(target));
        const bindings = new Map<string, {tabId: string; guard: string}>();
        const background: Tab[] = [];
        for (const patch of plan.patches) {
          const snapshot = snapshots.get(patch.path)!;
          const disk = saved.get(patch.path)!;
          const retained = readChangeOwner(patch.path).tab;
          if (snapshot.tabId && retained?.id !== snapshot.tabId) {
            throw Error(`The reviewed editor changed: ${patch.path}. Prepare the ${operation} again.`);
          }
          const tab = retained ?? makeTab(disk.managed ? "document" : "file",
            patch.path, patch.path.split("/").pop(), disk.docId);
          if (!retained) background.push(tab);
          bindings.set(patch.path, {tabId: tab.id, guard: snapshot.guard});
        }
        check();
        if (background.length) setLayout(layout => current() ? openInBackground(layout, background) : layout);
        const prepareStages = () => plan.patches.map(patch => {
          const snapshot = snapshots.get(patch.path)!;
          const binding = bindings.get(patch.path)!;
          const {tab, owner} = readChangeOwner(patch.path);
          if (tab && tab.id !== binding.tabId) throw Error(`Another editor took ownership of ${patch.path}. Prepare the ${operation} again.`);
          if (!owner) return null;
          if (!matchesScope(scope, owner.scope) || owner.guard !== binding.guard || owner.text !== patch.before ||
              owner.base !== snapshot.base || !owner.current()) {
            throw Error(`The editor source changed: ${patch.path}. Prepare the ${operation} again.`);
          }
          return owner.prepareDraft?.(patch.after) ?? null;
        });
        const until = Date.now() + 15_000;
        // A background tab can load its text, room and recovery at different
        // times. A stage is ready only when that editor can detach its saver.
        while (true) {
          check();
          if (prepareStages().every(stage => stage?.current())) break;
          if (Date.now() >= until) throw Error("An affected editor is still unavailable. Wait for it to load and retry Apply.");
          await new Promise(resolve => window.setTimeout(resolve, 50));
        }
        if (background.length) for (const target of snapshots.keys()) await readCurrentDisk(target);
        check();
        const stages = prepareStages().map(stage => {
          if (!stage?.current()) throw Error("An affected editor changed while checking saved source.");
          return stage;
        });
        const historyPlan = {...plan, scope, bindings};
        commitWorkspaceDrafts(stages);
        const inverse = invertWindowDraft(historyPlan);
        recordedDraftHistory.record({kind: "draft", plan: inverse, read: () => inverse});
      };
      setChangePath(0); setChangePage(0); setChangeReview({plan, current, opensFiles, apply});
      return {status: "completed", value: {preview_ready: true, source_files: plan.patches.length, source_changed: false}};
    } finally {
      secondary?.dispose();
      temporary?.close();
      if (visible()) setChangePending(false);
    }
  }, [activeSourceEditor, captureResourceWindow, observeResourceWindow, readChangeOwner, readChangeDisk, recordedDraftHistory]);
  prepareFileResourceRef.current = (kind, view, tab, paletteCurrent) => {
    const active = activeSourceEditor();
    if (changeApplying.current || !workspaceIsVisible() || active?.view !== view || active.tab.id !== tab.id ||
        view.composing || view.state.readOnly || !paletteCurrent()) return;
    const ticket = ++changeTicket.current, workspace = sourceWorkspace.current, focus = focusIntent.current;
    const {doc, selection} = view.state;
    const visible = () => ticket === changeTicket.current && sourceWorkspace.current === workspace;
    const current = () => visible() && workspaceIsVisible() && focusIntent.current === focus && paletteCurrent() &&
      view.dom.isConnected && !view.composing && !view.state.readOnly && view.state.doc === doc && view.state.selection === selection;
    setChangePending(false); setChangeReview(null); setResourceReview(null); setChangeError(null);
    const load: ResourceTransport = (route, body) => callApi("POST", route, body);
    void (async () => {
      const destination = kind === "move" ? await shellPrompt.askText("Move this file", tab.target) : null;
      if (kind === "move") {
        if (destination === null || destination === tab.target) return;
        if (semanticPath(documentUri(destination)) !== destination ||
            destination.endsWith(".md") !== (tab.kind === "document")) {
          throw Error("Choose a new workspace-relative path with the same document type.");
        }
      }
      if (!current()) return;
      setChangeKind(kind);
      setChangePending(true);
      const [tools, resources] = await Promise.all([import("./WorkspaceChat"), import("../simple/ResourceEditors")]);
      const files = folderRootsRef.current[0] ?? await api.files();
      const scope = tools.patchScope(files);
      if (!scope || !current()) throw Error("Choose a current authored file before reviewing this change.");
      const capture = captureResourceWindow(tools, scope, current);
      const source = capture.named.get(tab.target);
      if (!source || source.text !== authoredSource(view.state)) throw Error("The editor has no exact source to review.");
      const edit = {documentChanges: [destination === null ? {kind: "delete", uri: documentUri(source.path)}
        : {kind: "rename", oldUri: documentUri(source.path), newUri: documentUri(destination)}]};
      const title = destination === null ? `Delete ${source.path}` : `Move ${source.path} to ${destination}`;
      const review = await resources.prepareResourceReview(title, edit, crypto.randomUUID().replaceAll("-", ""),
        async path => {
          if (path !== source.path) throw Error("The resource preview named an unexpected file.");
          return source.text;
        }, async (paths, context) => {
          const observation = await observeResourceWindow(capture, edit, paths, context, load, resources.observeResourceInputs);
          if (observation.inputs.workspace !== scope.workspace) throw Error("The resource preview belongs to another workspace.");
          return observation;
        }, load, capture.current);
      if (!review.current()) throw Error("The source changed while preparing the review.");
      setChangePath(0); setChangePage(0); setResourceReview({...review, operation: "save"});
    })().catch(error => { if (visible()) setInsertNotice(String(error)); })
      .finally(() => { if (visible()) setChangePending(false); });
  };
  const saveResourceReview = async (review: NonNullable<typeof resourceReview>) => {
    if (changeApplying.current || !review.current()) {
      setChangeError("Source changed after review. Prepare the resource change again.");
      return;
    }
    changeApplying.current = true;
    setChangePending(true);
    setChangeError(null);
    const held: ResourceAdoptionStage[] = [];
    const load: ResourceTransport = (route, body) => callApi("POST", route, body);
    try {
      const {confirmedResourceFiles, publishReviewedResources} = await import("../simple/ResourceEditors");
      const token = review.history;
      const current = () => review.current() && (!token || recordedDraftHistory.current(token));
      if (!current()) throw Error("Workspace history or source changed. Open a fresh preview.");
      // Match the before and after addresses of each resource identity.
      // Closed files need no editor adoption, but any live editor must
      // remain attached to the same source through the reviewed move.
      const destinations = new Map(review.patches.flatMap(patch => patch.after ? [[patch.after.identity, {
        path: patch.path, text: patch.after.text,
      }] as const] : []));
      const retained = panesOf(layoutRef.current.root).flatMap(pane => pane.tabs)
        .filter(tab => tab.retainedResource);
      const stages = review.patches.flatMap(patch => {
        const live = readChangeOwner(patch.path);
        const restored = !patch.before && patch.after
          ? retained.find(tab => tab.retainedResource?.identity === patch.after?.identity) : undefined;
        const tab = live.tab ?? restored;
        if (!tab) return [];
        const identity = patch.before?.identity ?? restored?.retainedResource?.identity;
        const destination = identity ? destinations.get(identity) : undefined;
        const saved = destination && review.saved[destination.path];
        const owner = restored && !live.tab
          ? restored.kind === "document" ? registry.get(restored.docId)?.recordedPatchSource() : plainPatchOwners.current.get(restored.id)?.()
          : live.owner;
        const moved = destination && destination.path !== patch.path;
        const managed = tab.kind === "document";
        const canRetarget = !moved || (destination.path.endsWith(".md") === managed &&
          !readChangeOwner(destination.path).tab && (!managed || !retained.some(other =>
            other.id !== tab.id && other.kind === "document" && other.retainedResource?.path === destination.path)));
        if (!owner || !identity || (!patch.before && live.tab) || !canRetarget ||
            (destination ? saved?.kind !== "file" : tab.kind !== "file" && tab.kind !== "document")) {
          throw Error(`The open editor for ${patch.path} cannot adopt this resource transition in place.`);
        }
        const before = patch.before?.text ?? destination?.text;
        const stage = owner.text === before && owner.prepareResource?.(destination?.text ?? owner.text,
          saved?.kind === "file" ? saved.text : owner.base, destination?.path ?? null);
        if (!stage || !stage.current()) throw Error(`The open editor for ${patch.path} is unavailable for resource publication.`);
        return [{from: patch.path, to: destination?.path ?? null, identity, tab, stage}];
      });
      const owned = () => current() && stages.every(({stage}) => stage.current());
      // Retain the same review identity so a dispatched request can't be retried.
      if (!owned()) throw Error("Source changed after review. Prepare the resource change again.");
      const {recovery} = await publishReviewedResources(review, load, async (receipt, expected) => {
        const files = confirmedResourceFiles(review.patches, receipt, expected);
        if (!owned()) throw Error("Open editors changed during publication.");
        const adoptions = stages.map(({from, to, tab, stage, identity}) => {
          if (to === null) return {from, to, tab, stage, identity, authority: null};
          const file = files.get(to);
          const docId = file?.file.doc_id;
          if (!file?.save_guard || (tab.kind === "document") !== !!docId ||
              (from === to && docId !== tab.docId)) {
            throw Error(`The saved source has no matching editor authority: ${to}`);
          }
          return {from, to, tab, stage, identity, authority: {hash: file.hash, guard: file.save_guard, docId}};
        });
        await Promise.all(adoptions.map(({stage, authority}) => stage.waitForSavedSource?.(authority)));
        if (!owned()) throw Error("Open editors changed while confirming the saved resource sources.");
        const moved = new Map<string, Pick<Tab, "target" | "retainedResource" | "docId">>();
        for (const {from, to, tab, stage, identity, authority} of adoptions) {
          stage.adopt(authority);
          if (from === to && !tab.retainedResource) continue;
          if (to === null) {
            moved.set(tab.id, {target: `\0retained-file:${tab.id}`, retainedResource: {path: from, identity}});
            continue;
          }
          const docId = authority?.docId;
          if (tab.docId && docId && tab.docId !== docId) {
            const slot = documentSlots.current.get(tab.docId);
            if (slot) {
              documentSlots.current.delete(tab.docId);
              documentSlots.current.set(docId, slot);
            }
          }
          moved.set(tab.id, {target: to, docId});
        }
        if (moved.size) {
          providerNavigation.current++;
          setFileReferences(null);
          setLayout(layout => retargetResourceTabs(layout, moved));
        }
      }, () => {
        for (const {stage} of stages) if (stage.begin) {
          held.push(stage);
          stage.begin();
        }
        if (!owned()) throw Error("Open editors changed before resource publication.");
      });
      const entry = {kind: "resource" as const, recovery};
      if (token) recordedDraftHistory.commit(token, entry);
      else recordedDraftHistory.record(entry);
      setResourceReview(null);
      setInsertNotice("Saved reviewed source resources.");
      window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
    } catch (error) {
      setChangeError(error instanceof Error ? error.message : String(error));
      window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
    } finally {
      try {
        for (const stage of held) stage.release?.();
      } finally {
        changeApplying.current = false;
        setChangePending(false);
      }
    }
  };
  changeHistoryRef.current = direction => {
    const token = recordedDraftHistory.preview(direction), workspace = sourceWorkspace.current;
    if (!token || changeApplying.current) return;
    if (token.entry.kind === "resource") {
      const recovery = token.entry.recovery, ticket = ++changeTicket.current;
      const visible = () => ticket === changeTicket.current && sourceWorkspace.current === workspace;
      const current = () => visible() && recordedDraftHistory.current(token) && workspaceIsVisible();
      setChangeKind("code action");
      setChangePending(true); setChangeReview(null); setResourceReview(null); setChangeError(null);
      const load: ResourceTransport = (route, body) => callApi("POST", route, body);
      void (async () => {
        const [tools, resources] = await Promise.all([import("./WorkspaceChat"), import("../simple/ResourceEditors")]);
        const files = folderRootsRef.current[0] ?? await api.files();
        const scope = tools.patchScope(files);
        if (!scope || !current()) throw Error("Open the original workspace before preparing resource history.");
        const record = await callApi<import("../simple/SourceRecovery").RecoveryReading>("GET",
          "/api/changes/resource-recovery?id=" + encodeURIComponent(recovery));
        if (!current() || record.id !== recovery || record.workspace !== scope.workspace || record.entry_transition ||
            !record.paths.length || record.paths.length > 1000 || !/^[0-9a-f]{64}$/.test(record.fingerprint)) {
          throw Error("Current resource recovery evidence is unavailable.");
        }
        const capture = captureResourceWindow(tools, scope, current);
        const paths = record.paths.map(item => item.path);
        const edit = {changes: Object.fromEntries(paths.map(path => [documentUri(path), []]))};
        const observation = await observeResourceWindow(capture, edit, paths, crypto.randomUUID().replaceAll("-", ""), load, resources.observeResourceInputs);
        if (observation.inputs.workspace !== scope.workspace) throw Error("Resource workspace changed.");
        const review = await resources.prepareReviewedResourceRestoration(record, observation, load);
        if (!review.current()) throw Error("Resource history changed. Open a fresh preview.");
        setChangePath(0); setChangePage(0);
        setResourceReview({...review, operation: "restore", fingerprint: record.fingerprint, history: token});
      })().catch(error => { if (visible()) setInsertNotice(String(error)); })
        .finally(() => { if (visible()) setChangePending(false); });
      return;
    }
    void import("./WorkspaceChat").then(async ({matchesScope, patchScope, stageDrafts, commitWorkspaceDrafts, invertWindowDraft}) => {
      const files = folderRootsRef.current[0] ?? await api.files();
      const plan = recordedDraftHistory.readDraft(token) as WindowHistoryPlan;
      if (sourceWorkspace.current !== workspace || !matchesScope(patchScope(files), plan.scope)) {
        throw Error("Open the original workspace before changing these drafts.");
      }
      commitWorkspaceDrafts(stageDrafts(plan, path => {
        const {tab, owner} = readChangeOwner(path), binding = plan.bindings.get(path);
        return tab?.id === binding?.tabId && owner?.guard === binding?.guard ? owner : null;
      }));
      const inverse = invertWindowDraft(plan);
      recordedDraftHistory.commit(token, {kind: "draft", plan: inverse, read: () => inverse});
      setInsertNotice(direction === "undo" ? "Workspace change undone in drafts." : "Workspace change redone in drafts.");
    }).catch(error => setInsertNotice(String(error)));
  };
  readSymbolCandidates.current = async term => {
    const client = workspaceLspClient();
    if (!client) return [];
    const origin = focusIntent.current.tab?.target;
    const contextCurrent = captureNavigationContext();
    const readingCurrent = captureReadingPosition();
    const current = () => contextCurrent() && readingCurrent();
    const answer = await client.request("workspace/symbol", {query: term});
    if (!current()) return [];
    const candidates: CommandItem[] = [];
    for (const {name, uri, range: {start: {line, character}}} of semanticLocations(answer)) {
      const path = semanticPath(uri.replace(/^hick-output:/, "hick:"));
      const node = path && findNodeByPath(folderRootsRef.current, path);
      if (!name || !path || !node || node.dir) continue;
      candidates.push({
        id: JSON.stringify([path, line, character, name]),
        label: name,
        detail: `${path}:${line + 1}:${character + 1}`,
        run: paletteCurrent => {
          const originCurrent = () => !!paletteCurrent?.() && current();
          if (!originCurrent()) return;
          void visitSource({path, line: line + 1, character, originCurrent,
            provider: {savedOnly: path !== origin, symbol: name}})
            .catch(error => setInsertNotice(error instanceof Error ? error.message : String(error)));
        },
      });
      if (candidates.length === 100) break;
    }
    return candidates;
  };
  const openSearchHit = useCallback((hit: SearchHit) => {
    void visitSource({path: hit.path, line: hit.start_line, originCurrent: captureReadingPosition(), searchHit: hit})
      .catch(error => setInsertNotice(error instanceof Error ? error.message : String(error)));
  }, [captureReadingPosition, visitSource]);
  openSearchHitRef.current = openSearchHit;
  const openFolderHit = useCallback((file: FindFile, match: FindMatch) => {
    // The engine reports UTF-8 byte columns, while the editor uses UTF-16.
    // The visible row opens its first observed match.
    const sourceHash = file.source_hash?.replace(/^sha256:/i, "").toLowerCase() ?? "";
    const first = match.at[0];
    const character = first ? searchCharacter(match.text, first.column) : null;
    if (!/^[a-f0-9]{64}$/.test(sourceHash) || character === null) {
      setInsertNotice("Saved search source or position is unavailable. Search again with a current workspace server.");
      return;
    }
    void visitSource({path: file.path, line: match.line, character, originCurrent: captureReadingPosition(),
      findHit: {sourceHash, text: match.text}}).catch(error => setInsertNotice(error instanceof Error ? error.message : String(error)));
  }, [captureReadingPosition, visitSource]);
  const navigateCoverage = useCallback<CoverageSurfaceClient["navigate"]>((location, current) => {
    const readingCurrent = captureReadingPosition();
    const originCurrent = () => current() && readingCurrent();
    return visitSource({path: location.path, line: location.line, originCurrent, report: {sourceHash: location.sourceHash}});
  }, [captureReadingPosition, visitSource]);
  const openProblem = useCallback(async ({docId, path, diagnostic}: ProblemRow) => {
    const source = diagnosticBoundSource(diagnostic);
    const {line, character} = diagnostic.range.start;
    const readingCurrent = captureReadingPosition();
    const diagnosticCurrent = () => {
      const session = docId.startsWith("file:") ? undefined : registry.get(docId);
      const diagnostics = docId.startsWith("file:")
        ? allFileProblems().find(file => file.path === path)?.diagnostics
        : session?.doc?.path === path ? session.lspDiagnostics : undefined;
      return !!diagnostics?.includes(diagnostic) && diagnosticBoundSource(diagnostic) === source;
    };
    if (source === undefined || !diagnosticCurrent() || !validProblemPosition(source.split("\n"), {line, character})) {
      setInsertNotice("This problem has no current source position. Wait for fresh diagnostics before opening it.");
      return false;
    }
    try {
      return await visitSource({path, line: line + 1, character, expectedSource: source,
        originCurrent: () => readingCurrent() && diagnosticCurrent()});
    } catch (error) {
      setInsertNotice(error instanceof Error ? error.message : String(error));
      return false;
    }
  }, [captureReadingPosition, registry, visitSource]);
  useEffect(() => {
    const stop = onOpenLocation(location => {
      const {path, line, character, expectedSource, originCurrent} = location;
      if (character === undefined && expectedSource === undefined && !originCurrent) {
        sourceVisit.current++;
        openHitRef.current(path, line);
      } else {
        void visitSource(location).catch(error => setInsertNotice(String(error)));
      }
    });
    return () => {
      sourceVisit.current++;
      sourceActivation.current = undefined;
      preparedFile.current = undefined;
      preparedOutput.current = undefined;
      preparedDocument.current = undefined;
      stop();
    };
  }, [visitSource]);
  // The route is a REQUEST against the workspace, not its owner:
  // `#/docs/<id>` means "make sure this document is open and frontmost",
  // `#/new` means "make sure there is an untitled buffer". Back and forward
  // therefore just re-activate tabs that are already there.
  // Every non-document route needs its OWN key: collapsing them all to one
  // string means navigating from `#/new` to `#/scratchpad` looks like no
  // change at all, and the effect never runs.
  const routeKey = route.name === "doc" ? `doc:${route.id}` : route.name === "new" && route.file ? `file:${route.file}` : route.name;
  const hydrated = workspaceUi.hydrated;
  useEffect(() => {
    if (!requestedFile) return;
    let current = true;
    let settled = false;
    let selectionError: string | undefined;
    let observedTab: Tab | null = null;
    const focusedSource = () => {
      const tab = focusIntent.current.tab;
      return tab?.kind === "tree" ? undefined : tab?.id;
    };
    const originalFocus = focusedSource();
    // Refusing explicit Open File does not settle the independent text read.
    // A late preview can still take the original slot if the user stayed there.
    const canPresent = () => current && !settled && (!selectionError || focusedSource() === originalFocus);
    const selected = api.openFile(requestedFile);
    if (requestedFile === startupFile) {
      const observe = (startupHint ?? requestedFile).endsWith(".md") ? observeManagedStartup : observePlainStartup;
      void observe(requestedFile, startupHint, selected, () => current && (!!observedTab || canPresent()))
        .then(observation => {
          if (!canPresent() || !observation) return;
          const {path} = observation.source;
          if ("docId" in observation) {
            const {docId} = observation;
            if (findDocTab(layoutRef.current, docId)) return;
            const tab = makeTab("document", path, path.split("/").pop(), docId);
            observedTab = tab;
            setManagedStartup({id: tab.id, observation});
            setLayout(layout => openDocTab(layout, docId, path, undefined, tab));
          } else {
            if (findFileTab(layoutRef.current, path)) return;
            const tab = makeTab("file", path, path.split("/").pop());
            observedTab = tab;
            setPlainStartup({id: tab.id, observation});
            setLayout(layout => openFileTab(layout, path, undefined, tab));
          }
          if (selectionError) setInsertNotice(notice => notice === selectionError ? null : notice);
        }, () => { /* The ordinary explicit open owns the fallback. */ });
    }
    void selected.then(({doc_id, path}) => {
      settled = true;
      if (!current || observedTab) return;
      if (doc_id) setLayout(layout => activateDocTab(layout, doc_id) ?? openDocTab(layout, doc_id, path));
      else openPlainFile(path);
    }, error => {
      selectionError = error instanceof Error ? error.message : String(error);
      if (current && !observedTab) setInsertNotice(selectionError);
    });
    return () => { current = false; };
  }, [requestedFile, startupFile, startupHint, openPlainFile]);
  useEffect(() => {
    // Wait for the stored layout. Opening the routed document first would
    // leave the workspace non-empty when the restore lands, and the restore
    // would be dropped without a word.
    if (!hydrated) return;
    if (route.name === "new" && route.file) return;
    if (route.name === "new") {
      if (!(startup && folderOpen && workspaceUi.restored)) setLayout(current => openUntitledTab(current));
      return;
    }
    if (route.name === "scratchpad") {
      setLayout((current) => openScratchpadTab(current));
      return;
    }
    // Already looking at this document — through its own tab or one of its
    // generated files? Then the URL is just catching up with the focus (our
    // own redirect below), and activating would yank the focus to the doc
    // tab the person deliberately left.
    if (focusedDocId(layoutRef.current) === route.id) return;
    ensureDocOpen(route.id);
    // routeKey stands in for the route object, which is rebuilt per render.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [routeKey, ensureDocOpen, hydrated]);
  // The other direction: focusing a different document's pane makes the URL
  // follow, replacing the current entry — focus flips are not history the
  // Back button should have to chew through.
  useEffect(() => {
    if (!derivedFocus) return;
    if (requestedFile && managedStartup?.observation.docId === derivedFocus) return;
    if (route.name === "doc" && route.id === derivedFocus) return;
    redirect(`/docs/${derivedFocus}`);
    // Only ever triggered by a real focus change.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [derivedFocus]);
  const saveUntitled = useUntitledSave({
    layoutRef,
    setLayout,
    onError: setInsertNotice,
    sourceFor: (tabId) => untitledSourcesRef.current[tabId],
    onSaved: forgetUntitled,
  });
  // What the window calls itself: the custom override from Settings, else
  // the project folder's name, else the focused file (lib/windowTitle.ts).
  const untitledFocused = (() => {
    const pane = panesOf(layout.root).find(
      (candidate) => candidate.id === layout.focus,
    );
    return pane?.tabs[pane.active]?.kind === "untitled";
  })();
  const [customTitle, setCustomTitle] = useState<string | null>(null);
  useWorkspaceSettings(setCustomTitle, setRetainSavedDrafts);
  const focusedFileName = untitledFocused
    ? "Untitled"
    : (focused?.doc?.path.split("/").pop() ?? null);
  useEffect(() => {
    document.title = windowTitle({
      custom: customTitle,
      folder: folderRoots[0]?.root ?? null,
      file: focusedFileName,
    });
  }, [customTitle, folderRoots, focusedFileName]);
  // The folder's documents, for search resolution. Refreshed with the tree.
  useEffect(() => {
    let live = true;
    const load = () => {
      api.projects().then(
        async (projects) => {
          const project = projects[0];
          if (!project) return;
          if (live) setProjectId(project.id);
          const docs = await api.projectDocs(project.id).catch(() => []);
          if (live) setFolderDocs(docs);
        },
        () => {},
      );
    };
    load();
    window.addEventListener(FILES_CHANGED_EVENT, load);
    return () => {
      live = false;
      window.removeEventListener(FILES_CHANGED_EVENT, load);
    };
  }, []);
  // ---- sessions -------------------------------------------------------------
  //
  // One host per document the layout involves. A document stays alive while
  // ANY of its tabs is open — a generated file outliving its document's own
  // tab still needs the provenance, the saver, and the way back.
  const openDocIds = useMemo(() => docIdsIn(layout), [layout]);
  // Open sessions follow their editor across a reviewed change of address.
  // A later occupant of the original address gets its own session.
  documentSlots.current = new Map(openDocIds.map(docId =>
    [docId, documentSlots.current.get(docId) ?? crypto.randomUUID()]));

  const admittedOutput = preparedOutput.current;
  useLayoutEffect(() => {
    if (!admittedOutput || preparedOutput.current !== admittedOutput) return;
    const visible = panesOf(layout.root).some(pane => pane.tabs.some(tab =>
      tab.kind === "generated" && tab.docId === admittedOutput.owner && tab.target === admittedOutput.file.path));
    if (visible) preparedOutput.current = undefined;
  }, [layout, admittedOutput]);
  const admittedDocument = preparedDocument.current;
  useLayoutEffect(() => {
    if (admittedDocument && openDocIds.includes(admittedDocument.doc.id) &&
        preparedDocument.current === admittedDocument) preparedDocument.current = undefined;
  }, [openDocIds, admittedDocument]);
  const attachedManagedStartup = managedStartup &&
    findDocTab(layout, managedStartup.observation.docId)?.tab.id === managedStartup.id ? managedStartup.observation : undefined;
  // ---- ribbons and ports (the focused document's outputs) -----------------
  //
  // The overlay draws the focused document's lineage: its outputs, back to
  // the focused document AND to every other document those outputs carry
  // bytes from — a message that quotes a meeting turn gets a ribbon to the
  // meeting note, because that is where the bytes are. On focus change it
  // re-targets.
  //
  // Each other document is named by the path provenance reports (absolute,
  // from the engine) and drawn by the path the app knows it under (relative,
  // from the tree) — the tree resolves one to the other. A document that is
  // open contributes its text and editor; one that is not contributes only
  // its path, so its ribbons terminate on its tree row or an "open here" port.
  // The sources are memoized: the overlay re-measures whenever this array
  // changes identity, so an inline literal would re-measure every
  // relationship on screen on every render of this view.
  const docIdByPath = useCallback(
    (reported: string) => resolveDocumentPath(reported, folderDocs, folderRoots),
    [folderRoots, folderDocs],
  );
  const ribbonSources: RibbonSource[] = useMemo(() => workspaceRibbonSources(focused, registry, docIdByPath),
    // Registry and lens versions publish changes to the live editors.
    [lensTick, focused?.doc, focused?.docEditor, focused?.outputs, registry, registry.version, docIdByPath]);
  const ribbonFiles: RibbonFile[] = useMemo(
    () =>
      focused
        ? [...focused.outputs.values()].map((file) => ({
            file,
            view: focused.openOutputs.get(file.path),
          }))
        : [],
    [focused],
  );
  // ---- context provenance: what the model had in front of it -------------
  //
  // Fetched for the focused document and re-fetched when its text changes
  // (debounced — every keystroke changes the text). The derivation reads the
  // session files beside the document; the answer is where each agent-written
  // run of lines is NOW and which inputs preceded the write.
  const [contextWrites, setContextWrites] = useState<{
    docId: string;
    writes: ContextWrite[];
  } | null>(null);
  const contextDocId = focused?.docId ?? null;
  const contextDocSource = focused?.doc?.source ?? null;
  useEffect(() => {
    if (!contextDocId || contextDocSource === null || !layers.has("context"))
      return;
    let cancelled = false;
    const timer = window.setTimeout(() => {
      api.context(contextDocId).then(
        (r) => {
          if (!cancelled)
            setContextWrites({ docId: contextDocId, writes: r.writes });
        },
        () => {
          if (!cancelled) setContextWrites({ docId: contextDocId, writes: [] });
        },
      );
    }, 400);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [contextDocId, contextDocSource, layers]);
  // ---- declared provenance: what the author says it rests on ------------
  const [declared, setDeclared] = useState<{
    docId: string;
    cites: DeclaredCite[];
  } | null>(null);
  useEffect(() => {
    if (!contextDocId || contextDocSource === null || !layers.has("declared"))
      return;
    let cancelled = false;
    const timer = window.setTimeout(() => {
      api.cites(contextDocId).then(
        (r) => {
          if (!cancelled) setDeclared({ docId: contextDocId, cites: r.cites });
        },
        () => {
          if (!cancelled) setDeclared({ docId: contextDocId, cites: [] });
        },
      );
    }, 400);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [contextDocId, contextDocSource, layers]);
  // The links the overlay draws: one per (agent write, input) for the
  // focused document, from the lines as they stand now to the input's home
  // — a file's tree row, a session's element, a document's tab — and one
  // per declared citation, from the citing element to what it names.
  const ribbonLinks: RibbonLink[] = useMemo(() => {
    const doc = focused?.doc;
    if (!doc) return lensSources().flatMap(lens => lens.links);
    const declaredLinks: RibbonLink[] = [];
    if (declared && declared.docId === focused.docId) {
      declared.cites.forEach((cite, ci) => {
        if (!samePath(cite.from.path, doc.path) || cite.from.first_line === 0)
          return;
        cite.to.forEach((place, pi) => {
          const self = samePath(place.path, doc.path);
          declaredLinks.push({
            key: `cite:${ci}:${pi}`,
            family: "declared",
            from: {
              path: doc.path,
              lines: [cite.from.first_line, cite.from.last_line],
            },
            // A citation of this same document's fragment still needs a far
            // end the overlay can find; its own tab is that end.
            to: {
              path: place.path,
              lines: [place.first_line, place.last_line],
              kind: place.path.endsWith(".md") || self ? "document" : "file",
            },
            title: `Declared — this ${cite.from.element} says it cites ${place.id ? `#${place.id}` : place.element} in ${place.path} lines ${place.first_line}–${place.last_line} (cites="${cite.select}"). An assertion, not a derivation.`,
          });
        });
      });
    }
    if (!contextWrites || contextWrites.docId !== focused.docId)
      return declaredLinks;
    const docDir = doc.path.includes("/")
      ? doc.path.slice(0, doc.path.lastIndexOf("/") + 1)
      : "";
    const out: RibbonLink[] = [];
    contextWrites.writes.forEach((write, wi) => {
      const lines = write.current_lines;
      if (!lines) return;
      write.inputs.forEach((input, ii) => {
        if (input.kind === "file") {
          const path = joinRel(docDir, input.path);
          // The document reading itself before writing is not news.
          if (samePath(path, doc.path)) return;
          out.push({
            key: `${wi}:${ii}`,
            family: "context",
            from: { path: doc.path, lines },
            to: { path, lines: [input.first_line, input.last_line] },
            title: `Context — ${input.path} lines ${input.first_line}–${input.last_line} (sha256 ${input.sha256.slice(0, 12)}${
              input.commit ? `, commit ${input.commit.slice(0, 12)}` : ""
            }) was in front of the model when these lines were written`,
          });
        } else {
          out.push({
            key: `${wi}:${ii}`,
            family: "context",
            from: { path: doc.path, lines },
            to: {
              path: write.session,
              lines: [input.session_line, input.session_line],
              kind: "document",
            },
            title: `Context — ${input.element}${input.source ? ` ${input.source}` : ""} "${input.summary.slice(0, 60)}${
              input.summary.length > 60 ? "…" : ""
            }" (session line ${input.session_line}) was in front of the model when these lines were written`,
          });
        }
      });
    });
    const lensLinks = lensSources().flatMap((lens) => lens.links);
    return [...out, ...declaredLinks, ...lensLinks];
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [focused?.doc, focused?.docId, contextWrites, declared, lensTick]);
  // What is open, as `kind:target` — decided from the layout data, not the
  // DOM, because the layout is the truth about what has a tab.
  const openTargets = useMemo(() => {
    const set = new Set<string>();
    for (const pane of panesOf(layout.root)) {
      for (const t of pane.tabs) set.add(`${t.kind}:${t.target}`);
    }
    return set;
  }, [layout]);
  const reopenFocusedDocument = useCallback((into?: string) => {
    const session = registry.get(focusedIdRef.current);
    if (!session?.doc) return;
    const { id, path } = session.doc;
    const already = findDocTab(layoutRef.current, id) !== null;
    setLayout((current) => openDocTab(current, id, path, into));
    if (already) flashTab("document", path);
  }, [registry]);
  /**
   * Open another document by the path provenance reported for it — the
   * meeting note a message quotes — selecting `span` in it when its session
   * is already live. A document that is not loaded yet opens; the span is
   * not carried across the load (the click that brought it on screen is
   * answered by the ribbon that now reaches its prose).
   */
  // A span to select in a document that is still opening: kept until its
  // session publishes an editor, then consumed. This is what makes a click
  // on a cross-document ribbon land ON the bytes, not merely in the file.
  const pendingSelect = useRef<{ path: string; span: [number, number] } | null>(
    null,
  );
  useEffect(() => {
    const want = pendingSelect.current;
    if (!want) return;
    const live = registry
      .all()
      .find((s) => s.doc && samePath(s.doc.path, want.path) && s.docEditor);
    if (!live) return;
    pendingSelect.current = null;
    live.onSelectSpan(want.span);
    // registry.version is the signal; it is what changes when an editor mounts.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [registry, registry.version]);
  const openDocumentByPath = useCallback(
    (path: string, span?: [number, number], into?: string) => {
      const live = registry
        .all()
        .find((s) => s.doc && samePath(s.doc.path, path));
      if (live) {
        ensureDocOpen(live.docId, into);
        if (span) {
          if (live.docEditor) live.onSelectSpan(span);
          else pendingSelect.current = { path, span };
        }
        return;
      }
      void openIndexedDocument(path, docIdByPath(path), projectId, {
        refresh: api.projectDocs,
        update: setFolderDocs,
        open: (id, target) => {
          if (span) pendingSelect.current = { path: target, span };
          ensureDocOpen(id, into);
        },
        notice: setInsertNotice,
        focusTree: () => focusTreeRef.current(),
      });
    },
    [registry, ensureDocOpen, docIdByPath, projectId],
  );

  // Show Files activates its pane and puts keyboard point in its tree.
  const focusTreeRef = useRef<() => void>(() => undefined);
  const focusTree = useFolderPane(folderOpen, explicitFileStartup || workspaceUi.hydrated, layout, setLayout, setTreeFocusRequest, setInsertNotice, workspaceUi.restored);
  focusTreeRef.current = focusTree;
  openTerminalRef.current = () => openTerminal();
  // The "open here" ports: one per file the focused document's ribbons reach
  // but no pane shows. They live in the shell's divider (or edge rail) —
  // chrome, not floating labels over content.
  const ports: ShellPort[] = useMemo(() => {
    if (!focused?.doc) return [];
    const doc = focused.doc;
    const list: ShellPort[] = [];
    for (const file of focused.outputs.values()) {
      if (openTargets.has(`generated:${file.path}`)) continue;
      if (deriveRibbons(file, doc.path, doc.source).length === 0) continue;
      const name = file.path.split("/").pop() ?? file.path;
      list.push({
        id: `generated:${file.path}`,
        label: name,
        title: `Open ${file.path} here`,
        onOpen: () => openGeneratedFor(doc.id, file.path),
      });
    }
    // The document itself can be closed too; its back-ribbons need a
    // terminal that brings it back.
    if (!openTargets.has(`document:${doc.path}`)) {
      const name = doc.path.split("/").pop() ?? doc.path;
      list.push({
        id: `document:${doc.path}`,
        label: name,
        title: `Reopen ${doc.path}`,
        onOpen: reopenFocusedDocument,
      });
    }
    // And every OTHER document the outputs carry bytes from — the meeting a
    // message quotes — when no tab shows it: the ribbon needs somewhere to
    // end, and the end is the way there.
    for (const source of ribbonSources.slice(1)) {
      if (openTargets.has(`document:${source.docPath}`)) continue;
      const name = source.docPath.split("/").pop() ?? source.docPath;
      list.push({
        id: `document:${source.docPath}`,
        label: name,
        title: `Open ${source.docPath} — text here came from it`,
        onOpen: () => openDocumentByPath(source.docPath),
      });
    }
    // And the far ends of context/declared links: a file the model read, the
    // session that recorded it. The overlay prefers a tab, then a visible tree
    // row, then the port — so the port is there for when the tree has the
    // folder collapsed, and idle otherwise.
    const seen = new Set(list.map((p) => p.id));
    for (const link of ribbonLinks) {
      if (!layers.has(link.family)) continue;
      const isDoc =
        link.to.kind === "document" || link.to.path.endsWith(".md");
      const kind = isDoc ? "document" : "file";
      const id = `${kind}:${link.to.path}`;
      if (
        seen.has(id) ||
        openTargets.has(id) ||
        openTargets.has(`generated:${link.to.path}`)
      )
        continue;
      seen.add(id);
      const name = link.to.path.split("/").pop() ?? link.to.path;
      list.push({
        id,
        label: name,
        title: `Open ${link.to.path} — ${link.family === "context" ? "it was in front of the model" : "this cites it"}`,
        onOpen: () =>
          isDoc
            ? openDocumentByPath(link.to.path)
            : openPlainFile(link.to.path),
      });
    }
    return list;
  }, [
    focused,
    openTargets,
    openGeneratedFor,
    reopenFocusedDocument,
    ribbonSources,
    openDocumentByPath,
    ribbonLinks,
    layers,
    openPlainFile,
  ]);
  // ---- the Insert menu -----------------------------------------------------
  //
  // The vocabulary of the language, as a thing you pick. It writes into
  // whichever document editor was focused last (editor/activeEditor.ts) —
  // not the focused SESSION, because the untitled buffer has no session and
  // is exactly where a first `<hick:exec>` most wants to be typed.
  const openInsert = useCallback((id: string | null) => {
    const view = activeEditor();
    if (!view) {
      setInsertNotice(
        "There is no document open to insert into. Open one from the Files tree, or start a new one.",
      );
      return;
    }
    const { from, to } = view.state.selection.main;
    setInsertPanel({ id, selected: view.state.sliceDoc(from, to), request: crypto.randomUUID(), target: captureInsertion(view) });
  }, []);
  const applyInsert = useCallback((element: InsertElement, values: FieldValues, body: string, actor?: import("../lib/uiActions").UiActor) => {
    if (!insertPanel || !insertCapturedElement(insertPanel.target, element, values, body, actor?.kind === "agent")) {
      setInsertNotice("The originating editor changed, closed or became read-only. Reopen Insert from its current source."); return false;
    }
    return true;
  }, [insertPanel]);
  const openEditElement = useCallback(() => {
    const view = activeEditor();
    if (!view) {
      setInsertNotice("Open a document, then put the caret in the element you want to edit.");
      return;
    }
    const caret = view.state.selection.main.head;
    const block = structureOf(view.state).blocks
      .filter((candidate) => candidate.from <= caret && caret <= candidate.to)
      .sort((a, b) => a.to - a.from - (b.to - b.from))[0];
    if (!block) {
      setInsertNotice("Put the caret inside a hick element to edit its properties.");
      return;
    }
    const element = elementForExisting(block.name, block.attrs);
    if (!element) {
      setInsertNotice(`hick:${block.name} does not have an editable form yet; edit its source directly.`);
      return;
    }
    const values = { ...initialValues(element) };
    for (const field of element.fields) values[field.name] = block.attrs[field.name] ?? "";
    const rawBody = view.state.sliceDoc(block.contentFrom, block.contentTo);
    const body = element.body === "block"
      ? rawBody.replace(/^\n/, "").replace(/\n$/, "")
      : rawBody;
    setEditPanel({
      element, request: crypto.randomUUID(), target: captureInsertion(view),
      values,
      body,
      attrs: block.attrs,
      from: block.from,
      to: block.to,
      source: view.state.sliceDoc(block.from, block.to),
    });
  }, []);
  const applyEditElement = useCallback(
    (element: InsertElement, values: FieldValues, body: string, actor?: import("../lib/uiActions").UiActor) => {
      const edit = editPanel;
      const view = edit?.target.view;
      if (!edit || !view || !insertionCurrent(edit.target)) { setInsertNotice("The originating element editor changed, closed or became read-only. Reopen its current source."); return false; }
      // A collaborator may have changed this document while the form was
      // open. Offsets are not durable identities, so refuse rather than
      // replacing whichever bytes subsequently landed at this range.
      if (view.state.sliceDoc(edit.from, edit.to) !== edit.source) {
        setInsertNotice("This element changed while its editor was open. Reopen Edit Element to start from its current text.");
        return false;
      }
      view.dispatch({
        changes: {
          from: edit.from,
          to: edit.to,
          insert: renderExistingElement(element, values, body, edit.attrs),
        },
        selection: { anchor: edit.from },
        scrollIntoView: true,
        userEvent: actor?.kind === "agent" ? "input.uiAgentEditElement" : "input.editElement",
      });
      view.focus();
      return true;
    },
    [editPanel],
  );
  /**
   * Print whatever buffer has the focus.
   *
   * The text comes from the editor's STATE rather than its DOM: CodeMirror
   * only keeps the lines near the viewport in the document, so printing what
   * is rendered would print one screenful and the paper would look fine. See
   * lib/printing.ts.
   */
  const printFocusedBuffer = useCallback(() => {
    const view = focusedEditor();
    if (!view) {
      setInsertNotice("Open a document or a file to print.");
      return false;
    }
    const pane = panesOf(layoutRef.current.root).find(pane => pane.id === layoutRef.current.focus);
    const tab = pane?.tabs[pane.active];
    printText({
      title: printTitleFor(tab?.target ?? "document"),
      text: view.state.doc.toString(),
    });
    return true;
  }, []);
  const welcomed = useRef(false);
  useEffect(() => {
    if (!workspaceUi.hydrated || welcomed.current) return;
    welcomed.current = true;
    if (startup || workspaceUi.restored || (route.name === "new" && route.file) || !loadShowWelcome()) return;
    setLayout(openWelcomeTab);
  }, [workspaceUi.hydrated]);
  useEffect(() => {
    if (insertNotice === null) return;
    const timer = window.setTimeout(() => setInsertNotice(null), 4000);
    return () => window.clearTimeout(timer);
  }, [insertNotice]);
  // ---- native menu: Save / Save As ---------------------------------------
  //
  // The desktop menu's Save and Save As arrive from App as one window event;
  // they act on the FOCUSED document's session, whichever that is.
  const menuSnapshot = () => {
    const pane = panesOf(layoutRef.current.root).find(p => p.id === layoutRef.current.focus);
    const active = pane?.tabs[pane.active];
    const session = registry.get(focusedIdRef.current);
    const view = focusedEditor();
    const tabs = panesOf(layoutRef.current.root).flatMap(p => p.tabs);
    const insertView = activeEditor();
    const insertTab = tabs.find(t => t.kind === "document" ? registry.get(t.docId)?.docEditor === insertView : t.kind === "untitled" ? bufferEditor(t.id) === insertView : false);
    const status = { target: active ? { id: active.id, kind: active.kind, path: active.target } : null, document: session?.doc?.id ?? null, document_status: session?.syncState ?? null, banner: session?.banner ? { kind: session.banner.kind, text: session.banner.text } : null, notice: insertNotice, format_on_save: formatOnSave(), insert_target: insertTab ? { id: insertTab.id, kind: insertTab.kind, path: insertTab.target } : null, unnamed_buffers: tabs.filter(t => t.kind === "untitled").map(t => t.id) };
    return { visible: workspaceIsVisible(), guard: [status, view?.state.doc.toString(), view?.state.selection.toJSON(), insertView?.state.doc.toString(), insertView?.state.selection.toJSON(), registry.all().map(s => [s.doc?.id, s.doc?.path, s.liveSource]), tabs.map(t => [t.id, t.kind, t.target, t.docId]), [...plainSources.current], untitledSourcesRef.current], status };
  };
  const handleDocumentMenu = async (detail: MenuAction, actor: import("../lib/uiActions").UiActor, destination?: string) => {
    const pane = panesOf(layoutRef.current.root).find(p => p.id === layoutRef.current.focus);
    const active = pane?.tabs[pane.active];
    const session = registry.get(focusedIdRef.current);
    const view = focusedEditor();
    const unnamedSource = (id: string) => untitledSourcesRef.current[id] ?? bufferEditor(id)?.state.doc.toString();
    const unnamed = panesOf(layoutRef.current.root).flatMap(p => p.tabs).filter(t => t.kind === "untitled").map(t => t.id);
    const wrapSession = (session: NonNullable<ReturnType<typeof registry.get>>) => ({ id: session.doc?.id ?? null, save: session.menuSave, saveAs: session.menuSaveAs });
    return runWorkspaceMenu(detail, actor, {
      target: { id: active?.id ?? null, kind: active?.kind ?? null },
      untitled: unnamed, untitledNeedingDestination: unnamed.filter(id => unnamedSource(id) !== "" || formatOnSave()), unnamedEmpty: active?.kind === "untitled" && unnamedSource(active.id) === "" && !formatOnSave(),
      sessions: registry.all().map(wrapSession), session: session ? wrapSession(session) : undefined,
      working: active ? viewBuffers.get(active.id) ?? plainUnsavedActions.current.get(active.id) : undefined,
    }, {
      formatFirst: async () => {
        if (!formatOnSave() || !view) return;
        const original = view.state.doc;
        const formatted = await formatView(view).catch(() => false);
        if ((!formatted && view.state.doc !== original) || !view.dom.isConnected) throw Error("The originating buffer changed or closed during formatting. Save was not submitted; inspect its current source before retrying.");
      },
      saveUntitled, flush: requestFlushSaves, print: printFocusedBuffer, insert: openInsert, editElement: openEditElement,
    }, destination);
  };
  useEffect(() => {
    const openTool = (event: Event) => {
      const value = (event as CustomEvent<string>).detail;
      if (value === "import-meeting") setTool("import");
      if (value === "export-snapshot") setTool("export");
    };
    window.addEventListener("hickory-file-tool", openTool);
    return () => window.removeEventListener("hickory-file-tool", openTool);
  }, []);
  const invokeDocumentMenu = useWorkspaceMenuActions(menuSnapshot, handleDocumentMenu);
  const menuHandler = useRef(invokeDocumentMenu); menuHandler.current = invokeDocumentMenu;
  useEffect(() => {
    const onCommand = (event: Event) => { void menuHandler.current((event as CustomEvent).detail as MenuAction).catch(error => setInsertNotice(String(error))); };
    // The tree pane's reopen affordances, now that there is no toolbar:
    // File > Show Files in the native menu, and the explorer key everywhere.
    const onKey = (e: KeyboardEvent) => {
      if (!workspaceIsVisible()) return;
      if (!(e.metaKey || e.ctrlKey)) return;
      if (e.shiftKey && e.key.toLowerCase() === "e") {
        e.preventDefault();
        focusTreeRef.current();
      } else if (!e.shiftKey && !e.altKey && e.key.toLowerCase() === "i") {
        // The same key in the browser build, where there is no native menu.
        e.preventDefault();
        openInsert(null);
      } else if (e.altKey && !e.shiftKey && e.key.toLowerCase() === "i") {
        e.preventDefault();
        openEditElement();
      }
    };
    const onFiles = () => focusTreeRef.current();
    // View → Agent, its key, and the welcome verb: the conversation about
    // the focused document, in its pane.
    const onAgent = () => setLayout(openChatTab);
    // File > Open File… picked something inside this folder: find its tree
    // entry and open it in place — one more tab, never a session restart.
    const onOpenPath = (e: Event) => {
      const absolute = (e as CustomEvent<string>).detail;
      if (typeof absolute !== "string" || absolute.length === 0) return;
      const {beforeOpen} = beginSourceOpen();
      void openSelectedFile(absolute, id => ensureDocOpen(id, undefined, true), openPlainFile, setInsertNotice, beforeOpen);
    };
    window.addEventListener("hickory-doc-command", onCommand);
    window.addEventListener("keydown", onKey);
    window.addEventListener("hickory-show-files", onFiles);
    window.addEventListener("hickory-show-agent", onAgent);
    window.addEventListener("hickory-open-path", onOpenPath);
    return () => {
      window.removeEventListener("hickory-doc-command", onCommand);
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("hickory-show-files", onFiles);
      window.removeEventListener("hickory-show-agent", onAgent);
      window.removeEventListener("hickory-open-path", onOpenPath);
    };
  }, [registry, beginSourceOpen, ensureDocOpen, openPlainFile, openInsert, openEditElement, saveUntitled]);
  const terminals = useTerminals();
  // A terminal a pane started — a test run from the gutter — shown here,
  // because only the workspace can open a tab.
  useEffect(
    () =>
      onShowTerminal(({ id, title }) => {
        terminals.refresh();
        setLayout((current) => openTerminalTab(current, id, title));
      }),
    // `terminals` is a stable hook result; the setter is stable too.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );
  const [attentionAt, setAttentionAt] = useState<string | null>(null);
  const [nothingWaiting, setNothingWaiting] = useState(false);
  // The queue and the sessions change on every poll; the handlers below must
  // not be rebuilt (and their listeners re-subscribed) once a second.
  const attentionRef = useRef(terminals.attention);
  attentionRef.current = terminals.attention;
  const sessionsRef = useRef(terminals.sessions);
  sessionsRef.current = terminals.sessions;
  const attentionAtRef = useRef(attentionAt);
  attentionAtRef.current = attentionAt;
  const terminalsRef = useRef(terminals);
  terminalsRef.current = terminals;
  const openTerminal = useCallback(async (spec: OpenTerminal = {}) => {
    const session = await terminalsRef.current.open(spec);
    if (!session) return;
    setLayout((current) => openTerminalTab(current, session.id, session.title));
  }, []);
  const showTerminal = useCallback((id: string) => {
    const session = sessionsRef.current.find((s) => s.id === id);
    setLayout((current) =>
      openTerminalTab(current, id, session?.title ?? "Terminal"),
    );
  }, []);
  /** ⌘J: the next thing claiming attention, or the news that there is none. */
  const nextAttention = useCallback(() => {
    const next = nextInQueue(attentionRef.current, attentionAtRef.current);
    attentionAtRef.current = next;
    setAttentionAt(next);
    setNothingWaiting(next === null);
  }, []);
  useEffect(() => {
    if (!nothingWaiting) return;
    const timer = window.setTimeout(() => setNothingWaiting(false), 2000);
    return () => window.clearTimeout(timer);
  }, [nothingWaiting]);
  // The native menu's terminal verbs, arriving from App as one window event.
  useEffect(() => {
    const onTerminalCommand = (event: Event) => {
      const detail = (event as CustomEvent).detail;
      if (detail === "terminal") {
        // No list to open any more: the terminal appears as an icon on its
        // directory's row in the one tree this window has.
        void openTerminal();
      } else if (detail === "attention") {
        nextAttention();
      }
    };
    window.addEventListener("hickory-terminal-command", onTerminalCommand);
    return () =>
      window.removeEventListener("hickory-terminal-command", onTerminalCommand);
  }, [openTerminal, nextAttention]);
  // The card follows the cursor, and lets go when what it was showing stops
  // claiming anything — answered here, answered in its own terminal, or
  // closed. A card for a settled session is a card you learn to ignore.
  const attentionSession = sessionById(terminals.sessions, attentionAt);
  const attentionPlace =
    attentionAt === null ? -1 : terminals.attention.indexOf(attentionAt);
  useEffect(() => {
    if (attentionAt !== null && !terminals.attention.includes(attentionAt)) {
      setAttentionAt(null);
    }
  }, [attentionAt, terminals.attention]);
  // ---- project search -----------------------------------------------------
  //
  // The shell owns the Mod-Shift-F key while it is mounted; this listener is
  // the fallback for the moments it is not. Both can fire — the shell's
  // preventDefault marks the event as handled, and opening an open panel
  // again is a no-op either way.
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!workspaceIsVisible()) return;
      if (!(event.metaKey || event.ctrlKey) || !event.shiftKey) return;
      if (event.key !== "f" && event.key !== "F") return;
      if (event.defaultPrevented) return;
      event.preventDefault();
      setSearchOpen(true);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
  const workspaceSearch = useCallback((query: string, limit: number) => {
    const files = openPlainSearchFiles();
    for (const session of registry.all()) {
      if (session.doc) files.push({ path: session.doc.path, content: session.docEditor?.state.doc.toString() ?? session.liveSource });
      for (const [path, view] of session.openOutputs) files.push({ path, content: view.state.doc.toString() });
    }
    return searchWorkspace(query, limit, files, folderOpen ? api.search : undefined, folderRoots[0]?.root);
  }, [registry, folderOpen, folderRoots]);
  workspaceSearchRef.current = workspaceSearch;
  // Generated paths map to their owning open document.
  const openableOutputs = useMemo(() => {
    const map = new Map<string, string>(); // path -> owning docId
    for (const session of registry.all()) {
      for (const path of session.outputs.keys()) {
        if (!map.has(path)) map.set(path, session.docId);
      }
    }
    return map;
    // registry.version (via useSessionVersion) is what actually changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [registry, registry.version]);
  openableOutputsRef.current = openableOutputs;
  const { onTabContextMenu, fileTabMenu } = useFileTabMenu(folderRoots, focusedId, openableOutputs, ensureDocOpen, panesOf(layout.root).flatMap(pane => pane.tabs));
  // Hide continuity until the server answers.
  const [continuity, setContinuity] = useState<boolean | undefined>(undefined);
  useEffect(() => {
    let live = true;
    api.continuity().then(
      (answer) => live && setContinuity(answer.enabled),
      () => {},
    );
    return () => {
      live = false;
    };
  }, []);
  const setContinuityEnabled = useCallback((enabled: boolean) => {
    setContinuity(enabled);
    api.setContinuity(enabled).catch(() => setContinuity(!enabled));
  }, []);
  const drawnLayers = useMemo(
    () => new Set([...layers].filter((l) => l !== "continuity") as RibbonFamily[]),
    [layers],
  );
  const { dirtyTabIds, dirtyPaths, requestCloseTab } = useUnsavedLifecycle({
    flushUi: workspaceUi.flush,
    layout,
    layoutRef,
    setLayout,
    registry,
    retainSavedDrafts,
    untitledSources,
    untitledBaselines: untitledRecovery.baselines,
    untitledSourcesRef,
    forgetUntitled,
    saveUntitled,
    prompt: shellPrompt, onError: setInsertNotice, windowGuard: () => [sessionsRef.current.map(session => session.id).sort(), Object.entries(terminals.anchors).map(([id, anchor]) => [id, anchor.doc, anchor.container]).sort()],
    plainDirtyTabs,
    plainUnsavedActions,
  });
  /**
   * What one tab shows.
   *
   * Lifted out of the JSX so the shell's `render` can wrap every body in
   * its tab's zoom box without this switch growing another level of
   * indentation.
   */
  function setTool(kind: typeof tool, tab?: Tab) {
    if (kind) {
      const pane = panesOf(layoutRef.current.root).find(pane => pane.id === layoutRef.current.focus);
      const current = tab ?? pane?.tabs[pane.active];
      const session = registry.get(current?.docId);
      const path = current && ["document", "generated", "file"].includes(current.kind) ? current.target : null;
      const view = current?.kind === "generated" ? session?.openOutputs.get(current.target)
        : current?.kind === "file" ? plainSearchEditor(current.target)
        : current?.kind === "untitled" ? bufferEditor(current.id) : session?.docEditor;
      setToolTarget({ path, document: current?.kind === "document" ? current.docId : undefined, source: view?.state.doc.toString() ?? session?.liveSource, view });
    }
    setToolKind(kind);
  }
  const renderTabBody = (tab: Tab): ReactNode => {
    if (tab.kind === "document" && tab.docId) {
      const startup = managedStartup?.id === tab.id ? managedStartup.observation : undefined;
      return (
        <DocTabBody
          reflowOnOpen={!tab.id.startsWith("untitled-")}
          registry={registry}
          docId={tab.docId}
          slot={documentSlots.current.get(tab.docId)}
          startup={startup}
          onRetryStartup={startup?.retry ? () => {
            if (startup.current()) {
              startup.retry?.();
              setManagedStartup({id: tab.id, observation: startup});
            }
          } : undefined}
          wrapColumn={workspaceUi.wrapFor(tab.target)}
          onWrapColumn={(column) => workspaceUi.setWrap(tab.target, column)}
          tableLayouts={workspaceUi.tables}
          onTableLayout={workspaceUi.setTableLayout}
        />
      );
    }
    if (tab.kind === "generated" && tab.docId) {
      return (
        <GeneratedTabBody
          initialDiskFile={admittedOutput?.owner === tab.docId && admittedOutput.file.path === tab.target &&
            admittedOutput.current() ? admittedOutput.file : undefined}
          bufferId={tab.id}
          registry={registry}
          docId={tab.docId}
          path={tab.target}
          onSource={(_, source) => plainSources.current.set(tab.id, source)}
        />
      );
    }
    if (tab.kind === "file") {
      // Each tab owns a buffer; another tab's save arrives as an external change.
      const startup = plainStartup?.id === tab.id && plainStartup.observation.source.path === tab.target
        ? plainStartup.observation : undefined;
      return (
        <PlainFilePane
          key={tab.id}
          path={tab.retainedResource?.path ?? tab.target}
          retainedResource={!!tab.retainedResource}
          proposalTabId={tab.id} proposalOwner={registerPlainPatchOwner}
          startup={startup}
          onRetryStartup={startup?.retry ? () => {
            if (startup.current()) {
              startup.retry?.();
              setPlainStartup({id: tab.id, observation: startup});
            }
          } : undefined}
          takePreparedSource={takePreparedSource}
          onSource={(_, source) => plainSources.current.set(tab.id, source)}
          onReferences={setFileReferences}
          captureNavigationContext={captureNavigationContext}
          onRelatedLocations={showRelatedLocations}
          onLocalNavigation={recordLocalNavigation}
          onProviderPreview={prepareChangePreview}
          askText={shellPrompt.askText}
          askChoice={shellPrompt.askChoice}
          retainUnsaved={retainSavedDrafts}
          onUnsaved={(dirty, actions) => {
            plainUnsavedActions.current.set(tab.id, actions);
            setPlainDirtyTabs((current) => {
              const next = new Set(current);
              if (dirty) next.add(tab.id);
              else next.delete(tab.id);
              return next;
            });
          }}
          onAdopted={(adopted) => {
            // Adoption converts this tab in place and opens its new owner beside it.
            const {current: currentRequest} = beginSourceOpen();
            setLayout(current => openDocTab(
              adoptPlainFileTab(
                current,
                tab.target,
                adopted.doc_id,
                adopted.output_path,
              ),
              adopted.doc_id,
              adopted.doc_path,
            ));
            onSourceActivation({kind: "document", target: adopted.doc_path, docId: adopted.doc_id},
              currentRequest, () => navigate(`/docs/${adopted.doc_id}`));
            window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
          }}
        />
      );
    }
    if (tab.kind === "scratchpad") {
      return <ScratchpadPane key={tab.id} />;
    }
    if (tab.kind === "untitled") {
      if (!untitledRecovery.ready) return <p className="muted">Restoring document…</p>;
      return (
        <UntitledTab key={tab.id}
          tabId={tab.id}
          wrapColumn={workspaceUi.wrapFor(tab.target)}
          onWrapColumn={(column) => workspaceUi.setWrap(tab.target, column)}
          initialSource={untitledSources[tab.id] ?? ""}
          onSource={(tabId, source) => {
            untitledRevisions.current.set(tabId, (untitledRevisions.current.get(tabId) ?? 0) + 1);
            setUntitledSources((current) =>
              current[tabId] === source ? current : { ...current, [tabId]: source },
            );
          }}
        />
      );
    }
    if (tab.kind === "terminal") {
      // Canvas terminals need explicit zoom and refitting.
      return (
        <div className="terminal-tab">
          {/* Above the emulator, and present for exactly as long as the
              terminal is writing: the difference between "this disappears"
              and "this is being committed" is the most important thing on
              the screen, so it is never a toast. */}
          <AnchorBar
            anchor={terminals.anchors[tab.target] ?? null}
            onResume={() => terminals.resumeAnchor(tab.target)}
            onUnanchor={() => terminals.unanchor(tab.target)}
          />
          <Suspense fallback={null}>
            <TerminalPane
              sessionId={tab.target}
              zoom={workspaceUi.zoomFor(tab.target)}
            />
          </Suspense>
        </div>
      );
    }
    const review = reviewTool(tab, path => openHit(path, 1), () => setLayout(openStoryTab), openLiterateLibrary);
    if (review) return review;
    if (tab.kind === "tool" && tab.target === FLEET_TAB) {
      return <FleetPane />;
    }
    if (tab.kind === "tool" && tab.target.startsWith(MERGED_TAB)) {
      return <MergedView path={tab.target.slice(MERGED_TAB.length)} />;
    }
    if (tab.kind === "tool" && tab.target === WELCOME_TAB) {
      return (
        <WelcomePane
          actions={welcomeActions}
          recent={folderDocs}
          onOpenRecent={(id) => {
            ensureDocOpen(id, undefined, true);
          }}
        />
      );
    }
    if (tab.kind === "chat") {
      if (parserStatus !== "ready") return <p role="status" className="muted">
        {parserStatus === "loading" ? "Loading the Agent document reader…" : "The Agent document reader is unavailable."}
      </p>;
      return (
        <WorkspaceChat
          layout={layout}
          registry={registry}
          focusedEditor={agentFocusedEditor.current}
          untitledSources={untitledSourcesRef}
          plainSources={plainSources}
          plainPatchOwners={plainPatchOwners}
          untitledRevisions={untitledRevisions}
          patchRoot={folderOpen && !folderError ? folderRoots[0] : undefined}
          folder={folderOpen ? folderRoots[0]?.root ?? "Open folder" : null}
          onOpenSession={openDocumentByPath}
          draftHistory={recordedDraftHistory}
        />
      );
    }

    if (tab.kind === "tree") {
      return (
        <FolderTreePane
          focusRequest={tab.target === "folder:right" ? 0 : treeFocusRequest}
          roots={folderRoots}
          error={folderError}
          openable={new Set(openableOutputs.keys())}
          activeDocId={focusedId ?? undefined}
          dirtyPaths={dirtyPaths}
          onNewDocument={newDocument}
          sessions={terminals.sessions.map((session) => ({
            id: session.id,
            title: session.title,
            cwd: session.cwd,
            state: session.state,
            monitor: session.monitor,
            cwdIsLive: session.cwd_is_live ?? false,
          }))}
          onOpenTerminal={showTerminal}
          // Terminal tree rows own their directory launch and session-stop actions.
          onNewTerminal={(path) => void openTerminal({ cwd: path })}
          onNewWorktree={(path) => {
            void shellPrompt
              .askText("Branch for the new worktree:", "")
              .then((branch) => {
                if (branch) {
                  void openTerminal({
                    title: branch,
                    worktree_branch: branch,
                    cwd: path,
                  });
                }
              });
          }}
          onCloseTerminal={(id) => void terminals.close(id)}
          // Find opens through the tree only after checking the observed saved bytes.
          onOpenHit={openFolderHit}
          onOpen={(action) => {
            // Documents open/front a tab; generated files open beside their owner.
            if (action.kind === "doc") {
              ensureDocOpen(action.id, undefined, true);
            } else if (action.kind === "generated") {
              // Prefer the server's owner; openableOutputs only knows open documents.
              const owner = action.docId ?? openableOutputs.get(action.path);
              if (owner) openGeneratedFor(owner, action.path);
            } else {
              openPlainFile(action.path);
            }
          }}
        />
      );
    }
    return null;
  };

  return (
    <div className="doc-page with-chat wide-mode">
      {/* Plain files' debug sessions, held above their panes so a session
          survives the tab that started it being fronted away. */}
      <PlainDebugHosts />
      {tool !== "review" && <ReviewEvents />}
      <CommandBar candidates={commandCandidates} overlay />
      <div className="doc-main">
        <div className="shell-host" ref={setShellBox}>
          {/* The machinery, one per open document, none of it visible: the
              tabs below subscribe to what these publish. */}
          {openDocIds.map((docId) => (
            <DocSessionHost
              key={documentSlots.current.get(docId)}
              slot={documentSlots.current.get(docId)}
              registry={registry}
              docId={docId}
              startup={attachedManagedStartup?.docId === docId ? attachedManagedStartup : undefined}
              startupAuthority={attachedManagedStartup?.docId === docId ? attachedManagedStartup.authority : undefined}
              initialDocument={admittedDocument?.doc.id === docId && admittedDocument.current() ? admittedDocument : undefined}
              captureNavigationContext={captureNavigationContext}
              onRelatedLocations={showRelatedLocations}
              onLocalNavigation={recordLocalNavigation}
              onProviderPreview={prepareChangePreview}
              openGenerated={openGeneratedFor}
              retainUnsaved={retainSavedDrafts}
            />
          ))}
          {fileTabMenu}
          <ShellView
            onTabContextMenu={onTabContextMenu}
            sources={ribbonSources} files={ribbonFiles} links={ribbonLinks} folderRoot={folderRoots[0]?.root_path ?? folderRoots[0]?.root} hasFiles={!!folderOpen && folderRoots.some(root => root.tree.length > 0)}
            layout={layout} spatialState={workspaceUi.spatial} onSpatialState={workspaceUi.setSpatial} spatialHydrated={explicitFileStartup || workspaceUi.hydrated}
            onLayout={setLayout}
            onSearch={() => setSearchOpen(true)}
            ports={ports}
            tabStyle={tabStyle}
            channelWidth={channelWidth}
            dirtyTabIds={dirtyTabIds}
            onRequestCloseTab={requestCloseTab}
            empty={
              <span>
                Nothing open here.
                <br />
                Open a file from the Files tree, or split another pane.
              </span>
            }
            render={(tab) => (
              <div
                className="tab-zoom"
                data-workspace-tab={tab.id}
                style={
                  {
                    [TAB_ZOOM_VAR]: workspaceUi.zoomFor(tab.target),
                  } as React.CSSProperties
                }
              >
                {renderTabBody(tab)}
                {["document", "generated", "file", "untitled"].includes(tab.kind) && <PaneHeaderContent part="actions">
                  <FileToolMenu name={tab.title ?? tab.target} document={tab.kind === "document"}
                    readRevision={() => [tab.id, registry.get(tab.docId)?.liveSource, plainSources.current.get(tab.id)]} onOpen={kind => setTool(kind, tab)} />
                </PaneHeaderContent>}
              </div>
            )}
          />
          <RibbonOverlay
            container={shellBox}
            sources={ribbonSources}
            files={ribbonFiles}
            links={ribbonLinks}
            // Only drawn ribbon families reach the overlay.
            layers={drawnLayers}
            ribbonStyle={ribbonStyle}
            visibility={ribbonVisibility}
            onVisibleFamilies={setPaintedFamilies}
            onNavigate={(target, intent) => {
              const spatial = requestSpatialRibbonOpen(shellBox, target, intent);
              const routed = spatial ? layoutRef.current : prepareRibbonOpen(layoutRef.current, paneBoxes(shellBox), intent);
              layoutRef.current = routed;
              setLayout(routed);
              const into = !spatial && intent ? routed.focus : undefined;
              const session = registry.get(focusedIdRef.current);
              if (target.kind === "path") {
                if (revealSpatialSource(shellBox, ribbonSources, ribbonFiles, target)) return;
                openHit(target.path, target.lines?.[0] ?? 1, into);
                return;
              }
              if (!session) return;
              if (target.kind === "document") {
                if (session.doc && !samePath(target.path, session.doc.path)) {
                  openDocumentByPath(target.path, target.span, into);
                  return;
                }
                if (!session.docEditor) reopenFocusedDocument(into);
                session.onSelectSpan(target.span);
                return;
              }
              openGeneratedFor(session.docId, target.path, into);
              session.revealOutput(target.path, target.range);
            }}
          />
        </div>
      </div>
      {insertNotice && (
        <div className="menu-notice" role="status">
          {insertNotice}
        </div>
      )}
      {insertPanel && (
        <InsertMenu
          key={insertPanel.request}
          sourceCurrent={() => insertionCurrent(insertPanel.target)}
          initialId={insertPanel.id}
          selectedText={insertPanel.selected}
          onInsert={applyInsert}
          onClose={() => setInsertPanel(null)}
        />
      )}
      {editPanel && (
        <InsertMenu
          key={editPanel.request}
          sourceCurrent={() => insertionCurrent(editPanel.target)}
          initialId={editPanel.element.id}
          selectedText=""
          edit={{ values: editPanel.values, body: editPanel.body }}
          onInsert={applyEditElement}
          onClose={() => setEditPanel(null)}
        />
      )}
      {searchOpen && (
        <SearchPanel
          search={workspaceSearch}
          folderOpen={folderOpen === true}
          onNavigate={openSearchHit}
          onClose={() => setSearchOpen(false)}
        />
      )}
      <EnvironmentNotice environments={environments} />
      {problemsOpen && (
        <WorkspaceProblems
          environments={environments}
          registry={registry}
          onPick={openProblem}
          onClose={() => setProblemsOpen(false)}
        />
      )}
      {fileReferences?.current() ? (
        <ReferencesPanel
          locations={fileReferences.locations}
          query={fileReferences.query}
          onPick={location => { void openProviderResult(fileReferences, location).then(opened => {
            if (opened) setFileReferences(current => current === fileReferences ? null : current);
          }); }}
          onClose={() => setFileReferences(null)}
        />
      ) : (
        focused?.references?.current() && (
          <ReferencesPanel
            locations={focused.references.locations}
            query={focused.references.query}
            onPick={location => { void openProviderResult(focused.references!, location).then(opened => {
              if (opened && registry.get(focused.docId)?.references === focused.references) focused.clearReferences();
            }); }}
            onClose={focused.clearReferences}
          />
        )
      )}
      {focused && (
        <PromptPanel prompt={focused.prompt} onSettle={focused.settle} />
      )}
      <PromptPanel prompt={shellPrompt.prompt} onSettle={shellPrompt.settle} />
      {(changePending || changeReview || resourceReview) && <aside className="prompt-panel prompt-panel--filtered" role="dialog" aria-label={`${changeKind[0].toUpperCase()}${changeKind.slice(1)} preview`}
        onKeyDown={event => {
          if (event.key !== "Escape" || changeApplying.current) return;
          changeTicket.current++;
          setChangeReview(null); setResourceReview(null);
          setChangePending(false);
        }}>
        <button type="button" aria-label={`Close ${changeKind} preview`} disabled={changeApplying.current}
          onClick={() => { changeTicket.current++; setChangeReview(null); setResourceReview(null); setChangePending(false); }}>×</button>
        {changePending && <p role="status">{changeApplying.current ? (resourceReview ? "Saving reviewed source resources…" : "Checking and applying source drafts…") : `Preparing ${changeKind} preview…`}</p>}
        {resourceReview && (() => {
          const patch = resourceReview.patches[changePath];
          const savedBefore = resourceReview.savedBefore[patch.path], savedAfter = resourceReview.saved[patch.path];
          const sources = [
            ["Current draft", patch.before?.text],
            ["After draft", patch.after?.text],
            ["Current saved baseline", savedBefore?.kind === "file" ? savedBefore.text : undefined],
            ["After saved baseline", savedAfter?.kind === "file" ? savedAfter.text : undefined],
          ] as const;
          const offset = changePage * 16000;
          const more = sources.some(([, text]) => text !== undefined && text.length > offset + 16000);
          return <section aria-label="Resource change preview">
            <ChangePreview heading={resourceReview.plan.title}>
              <label>Resource preview source <select aria-label="Resource preview source" value={changePath}
                onChange={event => { setChangePath(Number(event.target.value)); setChangePage(0); }}>
                {resourceReview.patches.map((item, index) => <option key={item.path} value={index}>{item.path}</option>)}
              </select></label>
              {sources.map(([label, text]) => <div key={label}><strong>{label}</strong>
                <pre aria-label={label}>{text === undefined ? "Source absent" : text.slice(offset, offset + 16000)}</pre></div>)}
              {(more || changePage > 0) && <>
                <button type="button" disabled={!changePage} onClick={() => setChangePage(page => page - 1)}>Previous text</button>
                <button type="button" disabled={!more} onClick={() => setChangePage(page => page + 1)}>Next text</button>
              </>}
              <Suspense fallback={<p role="status">Opening construction consequences…</p>}>
                <ResourceConsequencesView review={resourceReview} />
              </Suspense>
            </ChangePreview>
            {changeError && <p role="alert">{changeError}</p>}
            <button type="button" disabled={changePending || !!changeError}
              onClick={() => { void saveResourceReview(resourceReview); }}>Save reviewed source resources</button>
          </section>;
        })()}
        {changeReview && (() => {
          const patch = changeReview.plan.patches[changePath];
          const before = patch.before.split("\n"), after = patch.after.split("\n");
          const pageCount = Math.max(1, Math.ceil(Math.max(before.length, after.length) / 100));
          const offset = changePage * 100;
          return <>
            <ChangePreview heading={changeReview.plan.title}>
              <p>Provider-reported changes in {changeReview.plan.patches.length} source {changeReview.plan.patches.length === 1 ? "file" : "files"}.</p>
              <label>Preview source file <select aria-label="Preview source file" value={changePath}
                onChange={event => { setChangePath(Number(event.target.value)); setChangePage(0); }}>
                {changeReview.plan.patches.map((item, index) => <option key={item.path} value={index}>{item.path}</option>)}
              </select></label>
              <strong>Before</strong><pre aria-label="Before">{before.slice(offset, offset + 100).join("\n")}</pre>
              <strong>After</strong><pre aria-label="After">{after.slice(offset, offset + 100).join("\n")}</pre>
              {pageCount > 1 && <>
                <button type="button" disabled={!changePage} onClick={() => setChangePage(page => page - 1)}>Previous lines</button>
                <button type="button" disabled={changePage + 1 >= pageCount} onClick={() => setChangePage(page => page + 1)}>Next lines</button>
              </>}
            </ChangePreview>
            {changeReview.opensFiles && <p>Apply opens affected files in background tabs.</p>}
            {changeError && <p role="alert">{changeError}</p>}
            <button type="button" disabled={changePending} onClick={() => {
              if (changeApplying.current) return;
              if (!changeReview.current()) { setChangeError(`Source or editor changed. Prepare the ${changeKind} again.`); return; }
              changeApplying.current = true;
              setChangePending(true); setChangeError(null);
              void changeReview.apply().then(() => {
                setChangeReview(null);
                const applied = changeKind[0].toUpperCase() + changeKind.slice(1);
                setInsertNotice(`${applied} applied to source drafts. Use Save All to publish or Undo workspace change to revert.`);
              }, error => setChangeError(error instanceof Error ? error.message : String(error)))
                .finally(() => { changeApplying.current = false; setChangePending(false); });
            }}>Apply to source drafts</button>
          </>;
        })()}
      </aside>}
      {/* The dock: things that run so you can work. Always visible, never
          focused, amber when one of them has fallen over. */}
      <MonitorDock
        sessions={terminals.sessions}
        onOpen={showTerminal}
        onClose={(id) => terminals.close(id)}
      />
      {/* The top of the queue, brought to you. Answering it costs you
          nothing: the pane you were in keeps the focus. */}
      {attentionSession && (
        <AttentionCard
          session={attentionSession}
          position={attentionPlace + 1}
          total={terminals.attention.length}
          onAnswer={(send) => terminals.answer(attentionSession.id, send)}
          onOpen={() => showTerminal(attentionSession.id)}
          onInterrupt={() => terminals.interrupt(attentionSession.id)}
          onNext={nextAttention}
          onDismiss={() => setAttentionAt(null)}
        />
      )}
      {nothingWaiting && (
        <p className="attention-empty" role="status">
          Nothing is waiting on you.
        </p>
      )}
      {tool && parserStatus === "ready" && <Suspense fallback={<span role="status">Opening tool…</span>}><WorkspaceTools
        key={`${tool}:${toolTarget.path}`} tool={tool} {...toolTarget}
        provenance={{ layers, onToggle: toggleProvenance, continuity, onContinuity: setContinuityEnabled }} projectId={projectId ?? null}
        onGraph={() => navigate(`/projects/${encodeURIComponent(projectId ?? "")}/lineage`)}
        navigateCoverage={navigateCoverage} onOpen={(path, line) => openHit(path, line ?? 1)}
        onOpenNote={id => ensureDocOpen(id, undefined, true)} close={() => setTool(null)}
      /></Suspense>}
      <StatusBar
        git={git}
        onChecks={() => setLayout(current => openInLayout(current, makeTab("tool", "checks", "Checks")))} onGit={() => setLayout(openGitTab)}
        problems={problems}
        needsAttention={terminals.attention.length}
        path={focusedPath}
        zoom={zoom.uiZoom}
        extra={<>
          {folderOpen && <MergeDriverNotice />}
          <WorkspaceStatusExtras
            provenance={{ layers, onToggle: toggleProvenance, displayed: paintedFamilies, continuity, onContinuity: setContinuityEnabled }}
            environmentCount={environments.notices.length}
            onEnvironments={() => setProblemsOpen(true)}
          />
          <AgentStatusBar />
        </>}
        onProblems={() => setProblemsOpen((open) => !open)}
        onAttention={nextAttention}
      />
    </div>
  );
}
