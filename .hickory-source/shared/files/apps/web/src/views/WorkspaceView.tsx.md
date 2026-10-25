<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/WorkspaceView.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [An Answer In The Agent Pane Has Ribbons](../../../../../../guarantees/agent/an-answer-in-the-agent-pane-has-ribbons/contract.md)
- [A Folder Remembers Its Window](../../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md)
- [A Keystroke Redraws Only The Editor](../../../../../../guarantees/authoring/a-keystroke-redraws-only-the-editor/contract.md)
- [Inserting An Element Writes Hick You Could Have Typed Yourself](../../../../../../guarantees/authoring/inserting-an-element-writes-hick-you-could-have-typed/contract.md)
- [New Document Creates Another Unnamed Buffer](../../../../../../guarantees/authoring/new-document-is-an-act/contract.md)
- [Open File Tabs Follow Renames](../../../../../../guarantees/authoring/open-file-tabs-follow-renames/contract.md)
- [Shared print requests do not attest physical output](../../../../../../guarantees/authoring/print-requests-do-not-attest-physical-output/contract.md)
- [Settings preserves the open workspace](../../../../../../guarantees/authoring/settings-preserves-the-open-workspace/contract.md)
- [The App Starts As A Lightweight Editor](../../../../../../guarantees/authoring/the-app-starts-as-a-lightweight-editor/contract.md)
- [The engineer tour distinguishes replay from execution](../../../../../../guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution/contract.md)
- [The Measure Says What It Measures](../../../../../../guarantees/authoring/the-measure-says-what-it-measures/contract.md)
- [Unsaved Work Survives Closing The App](../../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md)
- [Pair identity without silently granting execution](../../../../../../guarantees/collaboration/a-machine-is-a-keypair/contract.md)
- [A Plain File Has The Same Language Server A Document Has](../../../../../../guarantees/editor-intelligence/a-plain-file-has-the-same-language-server/contract.md)
- [A Typed Path Opens That File](../../../../../../guarantees/editor-intelligence/a-typed-path-opens-that-file/contract.md)
- [Actions belong to their subject](../../../../../../guarantees/editor-intelligence/actions-belong-to-their-subject/contract.md)
- [Every Shortcut Is A Setting](../../../../../../guarantees/editor-intelligence/every-shortcut-is-a-setting/contract.md)
- [Save Can Format First, With The File's Own Formatter](../../../../../../guarantees/editor-intelligence/save-can-format-first/contract.md)
- [The Document Toolbar Uses Words An IDE User Already Knows](../../../../../../guarantees/editor-intelligence/the-toolbar-uses-words-an-ide-user-knows/contract.md)
- [A Test Runs From The Line It Is Written On](../../../../../../guarantees/execution/a-test-runs-from-the-line-it-is-written-on/contract.md)
- [The History Lens Reads The Repository As A Story](../../../../../../guarantees/lenses/the-history-lens-reads-the-repository-as-a-story/contract.md)
- [A Ribbon Crosses Documents, Because Lineage Does](../../../../../../guarantees/lineage/a-ribbon-crosses-documents/contract.md)
- [A Ribbon Is Drawn Where It Is Asked For](../../../../../../guarantees/lineage/a-ribbon-is-drawn-where-it-is-asked-for/contract.md)
- [Three Provenances Are Drawn Apart, And Any Subset Can Be Shown](../../../../../../guarantees/lineage/three-provenances-are-drawn-apart/contract.md)
- [A Local Install Is a Launchable macOS App](../../../../../../guarantees/release/a-local-install-is-a-launchable-macos-app/contract.md)
- [Spatial Views Preserve The Editor](../../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md)
- [A Session Appears Where It Is Working](../../../../../../guarantees/terminal/a-session-appears-where-it-is-working/contract.md)

Captured SHA-256: `5a8419dded62eda79aa0f20d325c4b19d893e4e4c7500733bed137fdfe0e2395`.

<book:entry path="apps/web/src/views/WorkspaceView.tsx">
import { CoverageTools } from "../coverage/CoverageTools";
import { workspaceTreeFiles } from "./workspaceTreeFiles";
import { useUiReading } from "../lib/uiReading";
import { runWorkspaceMenu } from "./workspaceMenuActions";
import { useWorkspaceMenuActions } from "./uiWorkspaceMenuActions";
import { NativeRuntimeHistory } from "../debug/RuntimeRuns";
import { MeetingImport } from "../components/MeetingImport";
import { SnapshotExport } from "../components/SnapshotExport";
import { WorkflowTour } from "../components/WorkflowTour";
import { ReviewPanel } from "../components/ReviewPanel";
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
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { useRecentFiles } from "./useRecentFiles";
import { STARTUP_INTRODUCTION } from "../lib/newDoc";
import { api } from "../api/client";
import { MergeDriverNotice } from "../components/MergeDriverNotice";
import type {
  DocSummary,
  FileNode,
  OpenTerminal,
  SearchResponse,
} from "../api/types";
import { WorkspaceChat } from "./deferredAgent";
import { InsertMenu } from "../components/InsertMenu";
import { MoreActions } from "../components/MoreActions";
import { PlainFilePane } from "../components/PlainFilePane";
import { ScratchpadPane } from "../components/ScratchpadPane";
import { SearchPanel } from "../components/SearchPanel";
import { ReferencesPanel } from "../components/ReferencesPanel";
import { WorkspaceProblems } from "../environments/WorkspaceProblems";
import { useEnvironments } from "../environments/useEnvironments";
import { EnvironmentNotice } from "../environments/EnvironmentPanel";
import { PromptPanel, usePrompt } from "../components/PromptPanel";
import { bufferEditor, openBufferTarget } from "../lib/editorBufferTargets";
import { workspaceRibbonSources } from "./workspaceRibbonSources";
import { openPlainSearchFiles, searchWorkspace } from "../lib/workspaceSearch";
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
import { findNodeByPath, joinRel } from "../shell/workspacePaths";
import { nextWorkspaceProblem } from "../shell/nextWorkspaceProblem";
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
  paneById,
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
import { useUntitledRecovery } from "./useUntitledRecovery";
import { restoreFolderLayout } from "./restoreFolderLayout";
import { useWorkspaceUi } from "./useWorkspaceUi";
import { openSelectedFile, useFolderPane } from "./folderPane";
import { activeEditor, focusedEditor } from "../editor/activeEditor";
import { useZoom } from "./useZoom";
import { StatusBar } from "../shell/StatusBar";
import { WelcomePane, type WelcomeAction } from "./WelcomePane";
import { reviewTool, useRepresentationTabs } from "./representationTabs";
import { PlainDebugHosts } from "../debug/plainDebugHosts";
import { FleetPane } from "./FleetPane";
import { MergedView } from "./MergedView";
import {
  CommandBar,
  type CommandItem,
  type CommandMode,
} from "../shell/CommandBar";
import { loadShowWelcome } from "../lib/welcomePref";
import { totalProblems } from "../lib/problems";
import { EditorView } from "@codemirror/view";
import { TAB_ZOOM_VAR } from "../lib/zoom";
import { requestFlushSaves } from "../lib/flushSaves";
import { onOpenLocation, onShowTerminal, pathOfDocUri, revealLine } from "../lib/revealLine";
import { rankFiles } from "../lib/fileRanking";
import { allFileProblems, useFileProblemsVersion } from "../lib/fileProblems";
import { formatOnSave, loadFormatOnSave } from "../lib/formatOnSave";
import { formatView } from "../lsp/cmLspFeatures";
import type { LspLocation } from "../lsp/client";
import { printText, printTitleFor } from "../lib/printing";

/** The routes the workspace answers. Everything else is App's. */
export type WorkspaceRoute = Extract<
  Route,
  { name: "doc" } | { name: "new" } | { name: "scratchpad" }
>;

export function WorkspaceView({ route }: { route: WorkspaceRoute }) {
  const startup = useRef(route.name === "new" && route.introduction === true).current;
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
  // Refs let welcome and search actions reach callbacks declared later.
  const openTerminalRef = useRef<() => Promise<unknown> | void>(
    () => undefined,
  );
  const openHitRef = useRef<(path: string, line: number) => void>(
    () => undefined,
  );
  const focusedPathRef = useRef<string | null>(null);
  const workspaceSearchRef = useRef<(query: string, limit: number) => Promise<SearchResponse>>(async () => ({ semantic: false, hits: [] }));
  // The tree and the outputs map are built further down; these let the
  // callbacks above read the current values without depending on declaration
  // order.
  const folderRootsRef = useRef<readonly { tree: FileNode[] }[]>([]);
  const openableOutputsRef = useRef<ReadonlyMap<string, string>>(new Map());
  // The regions of a declared layout, when one was applied — only ever at a
  // fresh launch, into an empty workspace (see ensure below). They keep
  // routing generated files to their panes afterwards.
  const [regions, setRegions] = useState<readonly Region[]>([]);
  const regionsRef = useRef(regions);
  regionsRef.current = regions;

  // One live session per open document, published through the registry.
  const registry = useMemo(() => new SessionRegistry(), []);
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
  const agentFocusedEditor = useRef<string | null>(null);
  const agentActivePane = paneById(layout, layout.focus);
  const agentActiveTab = agentActivePane?.tabs[agentActivePane.active];
  if (agentActiveTab && ["document", "generated", "file", "untitled"].includes(agentActiveTab.kind)) agentFocusedEditor.current = agentActiveTab.id;
  const untitledSourcesRef = useRef(untitledSources);
  untitledSourcesRef.current = untitledSources;
  const forgetUntitled = useCallback((tabId: string) => {
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
  const [fileReferences, setFileReferences] = useState<{
    locations: LspLocation[];
    query: string;
  } | null>(null);
  /** Put the caret on the next error or warning in the focused document. */
  // The list behind the status bar's count. Open/closed is all that is held;
  // the rows are read from the sessions when it draws, so a diagnostic that
  // arrives while it is open appears without a subscription of its own.
  const [problemsOpen, setProblemsOpen] = useState(false);
  const goToNextProblem = useCallback(() => nextWorkspaceProblem(registry, focusedIdRef.current), [registry]);
  // F8 — "go to the next problem" in every editor that has the idea, and the
  // verb the status-bar click used to be. Keeping it as a key rather than
  // dropping it is the point: the click now answers "what is wrong", and this
  // still answers "take me to the next one".
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!workspaceIsVisible()) return;
      // The first stroke of any chord is swallowed here so it reaches no
      // editor as a stray Ctrl+K; the second completes the action.
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
  const workspaceUi = useWorkspaceUi(layout, restored => setLayout(current => restoreFolderLayout(current, restored,
    folderOpen === true, introductionTab && untitledSourcesRef.current[introductionTab] === STARTUP_INTRODUCTION ? introductionTab : undefined)),
    folderOpen === null ? (folderError ? "window" : null) : folderOpen ? folderRoots[0]?.root_path ?? folderRoots[0]?.root ?? "folder" : "window");
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
  const ensureDocOpen = useCallback((id: string, into?: string) => {
    if (findDocTab(layoutRef.current, id)) {
      setLayout((current) => activateDocTab(current, id) ?? current);
      flashDocTab(id);
      return;
    }
    void api.doc(id).then(
      (doc) => {
        const current = layoutRef.current;
        if (findDocTab(current, id)) {
          setLayout((l) => activateDocTab(l, id) ?? l);
          flashDocTab(id);
          return;
        }
        if (!into && isWorkspaceEmpty(current)) {
          const declared = regionsOf(doc.source);
          if (declared.length > 0) {
            setRegions(declared);
            setLayout(openIntoDeclared(declared, id, doc.path));
            return;
          }
        }
        setLayout((l) =>
          findDocTab(l, id)
            ? (activateDocTab(l, id) ?? l)
            : openDocTab(l, id, doc.path, into),
        );
      },
      () => {
        // A document that will not load still gets a tab: the session's
        // error, with its message, renders there — a workspace is not the
        // right thing to take down over one bad id in a URL.
        setLayout((l) => (findDocTab(l, id) ? l : openDocTab(l, id, id, into)));
      },
    );
  }, []);
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
        return [
          {
            id: `line-${line}`,
            label: `Go to line ${line}`,
            detail: focusedPathRef.current ?? "",
            run: () => {
              const target = view.state.doc.line(
                Math.min(Math.floor(line), view.state.doc.lines),
              );
              view.dispatch({
                selection: { anchor: target.from },
                effects: EditorView.scrollIntoView(target.from, {
                  y: "center",
                }),
              });
              view.focus();
            },
          },
        ];
      }
      if (mode === "command") {
        const all: CommandItem[] = [...documentCommands(registry.get(focusedDocId(layoutRef.current))),
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
      if (mode === "content") {
        if (!term) return [];
        const found = await workspaceSearchRef.current(term, 12);
        return found.hits.map((hit, index) => ({
          id: `${hit.path}:${index}`,
          label: hit.path,
          detail: hit.snippet,
          run: () => openHitRef.current(hit.path, hit.start_line),
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
    [welcomeActions, registry],
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
    (path: string, line: number, into?: string) => {
      if (openBufferTarget(path, line, layoutRef.current, setLayout, setInsertNotice)) return;
      const document = registry.all().find((session) => session.doc && samePath(session.doc.path, path));
      if (document) {
        ensureDocOpen(document.docId, into);
        navigate(`/docs/${document.docId}`);
        document.revealDocLine(Math.max(0, line - 1));
        return;
      }
      const output = registry.all().find((session) => session.openOutputs.has(path));
      if (output) {
        openGeneratedFor(output.docId, path, into);
        revealLine(path, line);
        return;
      }
      const node = findNodeByPath(folderRootsRef.current, path);
      const action = node
        ? fileAction(node, new Set(openableOutputsRef.current.keys()))
        : ({ kind: "file", path } as const);
      if (action.kind === "doc") {
        ensureDocOpen(action.id, into);
        navigate(`/docs/${action.id}`);
      } else if (action.kind === "generated") {
        const owner =
          action.docId ?? openableOutputsRef.current.get(action.path);
        if (owner) openGeneratedFor(owner, action.path, into);
      } else if (action.kind === "file") {
        openPlainFile(action.path, into);
      }
      revealLine(path, line);
    },
    [registry, ensureDocOpen, openGeneratedFor, openPlainFile],
  );
  openHitRef.current = openHit;
  // Whether Save formats first, read once; Settings keeps it current.
  useEffect(() => {
    void loadFormatOnSave();
  }, []);
  // A definition that landed in another file, asked from an editor that
  // cannot open tabs. The workspace can. See lib/revealLine.ts.
  useEffect(
    () => onOpenLocation(({ path, line }) => openHitRef.current?.(path, line)),
    [],
  );
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
    // Wait for the stored layout. Opening the routed document first would
    // leave the workspace non-empty when the restore lands, and the restore
    // would be dropped without a word.
    if (!hydrated) return;
    if (route.name === "new" && route.file) {
      void openSelectedFile(route.file, ensureDocOpen, openPlainFile, setInsertNotice);
      return;
    }
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
  const focusTree = useFolderPane(folderOpen, workspaceUi.hydrated, layout, setLayout, setTreeFocusRequest, setInsertNotice, workspaceUi.restored);
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
      void openSelectedFile(absolute, ensureDocOpen, openPlainFile, setInsertNotice);
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
  }, [registry, openPlainFile, openInsert, openEditElement, saveUntitled]);
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
  const renderTabBody = (tab: Tab): ReactNode => {
    if (tab.kind === "document" && tab.docId) {
      return (
        <DocTabBody
          registry={registry}
          docId={tab.docId}
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
      return (
        <PlainFilePane
          key={tab.id}
          path={tab.target}
          onSource={(_, source) => plainSources.current.set(tab.id, source)}
          onReferences={setFileReferences}
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
            setLayout((current) =>
              adoptPlainFileTab(
                current,
                tab.target,
                adopted.doc_id,
                adopted.output_path,
              ),
            );
            ensureDocOpen(adopted.doc_id);
            navigate(`/docs/${adopted.doc_id}`);
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
          onSource={(tabId, source) =>
            setUntitledSources((current) =>
              current[tabId] === source ? current : { ...current, [tabId]: source },
            )
          }
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
            ensureDocOpen(id);
            navigate(`/docs/${id}`);
          }}
        />
      );
    }
    if (tab.kind === "chat") {
      return (
        <WorkspaceChat
          layout={layout}
          registry={registry}
          focusedEditor={agentFocusedEditor.current}
          untitledSources={untitledSourcesRef}
          plainSources={plainSources}
          folder={folderOpen ? folderRoots[0]?.root ?? "Open folder" : null}
          onOpenSession={openDocumentByPath}
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
          // Find uses the tree's document/generated/plain-file routing.
          onOpenHit={openHit}
          onOpen={(action) => {
            // Documents open/front a tab; generated files open beside their owner.
            if (action.kind === "doc") {
              ensureDocOpen(action.id);
              navigate(`/docs/${action.id}`);
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
      <CommandBar candidates={commandCandidates} overlay />
      <div className="doc-main">
        {/* Asked at open, because a clone that never ran `hick init` merges
            `.md` documents with git's line merge and is told nothing. */}
        {folderOpen && <MergeDriverNotice />}
        <div className="shell-host" ref={setShellBox}>
          {/* The machinery, one per open document, none of it visible: the
              tabs below subscribe to what these publish. */}
          {openDocIds.map((docId) => (
            <DocSessionHost
              key={docId}
              registry={registry}
              docId={docId}
              openGenerated={openGeneratedFor}
              retainUnsaved={retainSavedDrafts}
            />
          ))}
          {fileTabMenu}
          <ShellView
            onTabContextMenu={onTabContextMenu}
            sources={ribbonSources} files={ribbonFiles} links={ribbonLinks} folderRoot={folderRoots[0]?.root_path ?? folderRoots[0]?.root} hasFiles={!!folderOpen && folderRoots.some(root => root.tree.length > 0)}
            layout={layout} spatialState={workspaceUi.spatial} onSpatialState={workspaceUi.setSpatial} spatialHydrated={workspaceUi.hydrated}
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
                style={
                  {
                    [TAB_ZOOM_VAR]: workspaceUi.zoomFor(tab.target),
                  } as React.CSSProperties
                }
              >
                {renderTabBody(tab)}
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
          onNavigate={(hit) => openHit(hit.path, hit.start_line)}
          onClose={() => setSearchOpen(false)}
        />
      )}
      <EnvironmentNotice environments={environments} />
      {problemsOpen && (
        <WorkspaceProblems
          environments={environments}
          registry={registry}
          openHit={openHit}
          openDocument={ensureDocOpen}
          navigate={navigate}
          onClose={() => setProblemsOpen(false)}
        />
      )}
      {fileReferences ? (
        <ReferencesPanel
          locations={fileReferences.locations}
          query={fileReferences.query}
          onPick={(location) => {
            setFileReferences(null);
            const target =
              pathOfDocUri(location.uri) ??
              location.uri.replace(/^hick-output:\/\/\//, "");
            openHit(target, location.range.start.line + 1);
          }}
          onClose={() => setFileReferences(null)}
        />
      ) : (
        focused?.references && (
          <ReferencesPanel
            locations={focused.references.locations}
            query={focused.references.query}
            onPick={focused.openTarget}
            onClose={focused.clearReferences}
          />
        )
      )}
      {focused && (
        <PromptPanel prompt={focused.prompt} onSettle={focused.settle} />
      )}
      <PromptPanel prompt={shellPrompt.prompt} onSettle={shellPrompt.settle} />
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
      <StatusBar
        git={git}
        onGit={() => setLayout(openGitTab)}
        problems={problems}
        needsAttention={terminals.attention.length}
        path={focusedPath}
        zoom={zoom.uiZoom}
        extra={<>
          <WorkspaceStatusExtras
            provenance={{ layers, onToggle: toggleProvenance, continuity, onContinuity: setContinuityEnabled }}
            environmentCount={environments.notices.length}
            onEnvironments={() => setProblemsOpen(true)}
            projectId={projectId ?? null}
            onGraph={() => navigate(`/projects/${encodeURIComponent(projectId ?? "")}/lineage`)}
          />
          <MoreActions label="Workspace tools">
            <CoverageTools path={focusedPath} />
            <ReviewPanel document={focused?.doc?.path} source={focused?.liveSource} view={focused?.docEditor} />
            <MeetingImport onOpen={(path, line) => openHit(path, line ?? 1)} onOpenNote={(id) => { ensureDocOpen(id); navigate(`/docs/${id}`); }} />
            <SnapshotExport docId={focused?.docId} />
            <NativeRuntimeHistory path={focused?.doc?.path} source={focused?.liveSource} view={focused?.docEditor} />
            <WorkflowTour surface="app" onOpen={(path, line) => openHit(path, line ?? 1)} focused={{ path: focused?.doc?.path, source: focused?.liveSource, view: focused?.docEditor }} />
          </MoreActions>
        </>}
        onProblems={() => setProblemsOpen((open) => !open)}
        onAttention={nextAttention}
      />
    </div>
  );
}
</book:entry>

</book:doc>
