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

import { STARTUP_INTRODUCTION } from "../lib/newDoc";
import { api } from "../api/client";
import { MergeDriverNotice } from "../components/MergeDriverNotice";
import type {
  DocSummary,
  FileNode,
  OpenTerminal,
  SearchHit,
} from "../api/types";
import { ChatDock } from "../components/ChatDock";
import { InsertMenu } from "../components/InsertMenu";
import { PlainFilePane } from "../components/PlainFilePane";
import { ScratchpadPane } from "../components/ScratchpadPane";
import { SearchPanel } from "../components/SearchPanel";
import { ReferencesPanel } from "../components/ReferencesPanel";
import { WorkspaceProblems } from "../environments/WorkspaceProblems";
import { useEnvironments } from "../environments/useEnvironments";
import { EnvironmentNotice } from "../environments/EnvironmentPanel";
import { PromptPanel, usePrompt } from "../components/PromptPanel";
import {
  resolveSearchHit,
  type SearchNavigation,
} from "../lib/searchNavigation";
import { insertTarget, type MenuAction } from "../lib/menuBridge";
import { insertElement } from "../editor/insertElement";
import {
  elementForExisting,
  initialValues,
  renderExistingElement,
  type FieldValues,
  type InsertElement,
} from "../lib/insertCatalog";
import { structureOf } from "../editor/wysiwyg";
import { useRibbonPresentation } from "../components/LineageSettings";

import { loadTabStyle, type TabStyle } from "../lib/tabStyle";
import { loadChannelWidth } from "../lib/channelWidth";
import { windowTitle } from "../lib/windowTitle";
import { deriveRibbons } from "../lib/ribbons";
import { flashTab } from "../lib/flashTab";
import { nodeForAbsolutePath } from "../lib/openPath";
import { ShellView, type ShellPort } from "../shell/ShellView";
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
  FolderTreePane,
  isLikelyBinaryPath,
  useFolderTrees,
  fileAction,
} from "../shell/FolderTreePane";
import {
  paneById,
  panes as panesOf,
  type Layout,
  type Tab,
} from "../shell/layout";
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
  GIT_TAB,
  MERGED_TAB,
  STORY_TAB,
  WELCOME_TAB,
  activateDocTab,
  adoptPlainFileTab,
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
import { useWorkspaceUi } from "./useWorkspaceUi";
import { openSelectedFile, useFolderPane } from "./folderPane";
import { focusedEditor } from "../editor/activeEditor";
import { useZoom } from "./useZoom";
import { StatusBar } from "../shell/StatusBar";
import { WelcomePane, type WelcomeAction } from "./WelcomePane";
import { GitPane } from "./GitPane";
import { HistoryLens } from "./HistoryLens";
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
  // What the window is arranged as. Session state, owned HERE, above any
  // document: navigating between documents must leave it untouched.
  const startup = useRef(route.name === "new" && route.introduction === true).current;
  const [layout, setLayout] = useState<Layout>(() =>
    startup ? openUntitledTab(initialWorkspace()) : initialWorkspace(),
  );
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
  // Read through refs by the welcome page's actions and the top field's
  // candidates, which are built before the callbacks they reach for exist.
  const openTerminalRef = useRef<() => Promise<unknown> | void>(
    () => undefined,
  );
  const openHitRef = useRef<(path: string, line: number) => void>(
    () => undefined,
  );
  const focusedPathRef = useRef<string | null>(null);
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
  } | null>(null);
  // The same form edits an existing element. Its source range is captured
  // with the parsed structure, rather than relying on the selection after
  // the dialog has taken focus.
  const [editPanel, setEditPanel] = useState<{
    element: InsertElement;
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
  const [tabStyle] = useState<TabStyle>(() => loadTabStyle());
  const [channelWidth] = useState<number>(() => loadChannelWidth());
  const [shellBox, setShellBox] = useState<HTMLElement | null>(null);
  // The workspace's own prompt, for the things that belong to the WINDOW
  // rather than to a document — asking for a worktree's branch name, now that
  // terminals have no pane of their own to ask on.
  const shellPrompt = usePrompt();
  // Untitled has no document session, so the workspace owns its explicit
  // save baseline. One exists at a time today; keyed by tab id so the data
  // model stays correct if that policy ever changes.
  const [untitledSources, setUntitledSources] = useState<Record<string, string>>(() => introductionTab ? { [introductionTab]: STARTUP_INTRODUCTION } : {});
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
      { save: () => Promise<boolean>; discard: () => void; retain: () => Promise<void> }
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
  /** The path of whatever tab is active in the focused pane. */
  const focusedPath: string | null = (() => {
    const pane = paneById(layout, layout.focus);
    const tab = pane?.tabs[pane.active];
    return tab && tab.kind !== "tree" && tab.kind !== "tool"
      ? tab.target
      : null;
  })();
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
  const workspaceUi = useWorkspaceUi(layout, (restored) => {
    if (!startup) setLayout((current) => (isWorkspaceEmpty(current) ? restored : current));
  });
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
  // ---- which document the chrome follows ---------------------------------
  //
  // The focused pane's active tab names a document, or the one focused last
  // does — the toolbar, dock and ribbons must not flicker to nothing when
  // the focus lands on the tree.
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
  const openGeneratedFor = useCallback((docId: string, path: string) => {
    const already = panesOf(layoutRef.current.root).some((pane) =>
      pane.tabs.some((t) => t.kind === "generated" && t.target === path),
    );
    setLayout((current) =>
      openGeneratedTab(current, docId, path, regionsRef.current),
    );
    if (already) flashTab("generated", path);
  }, []);
  /** Open a plain file — same "ensure open" a document gets, no owner. */
  const openPlainFile = useCallback((path: string) => {
    const already = findFileTab(layoutRef.current, path) !== null;
    setLayout((current) => openFileTab(current, path));
    if (already) flashTab("file", path);
  }, []);
  // The tree tells us the identity change, not merely that its listing is
  // stale. Keep the existing editor tab (and its buffer) in place while its
  // path and caption follow the renamed file.
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
  const ensureDocOpen = useCallback((id: string) => {
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
        if (isWorkspaceEmpty(current)) {
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
            : openDocTab(l, id, doc.path),
        );
      },
      () => {
        // A document that will not load still gets a tab: the session's
        // error, with its message, renders there — a workspace is not the
        // right thing to take down over one bad id in a URL.
        setLayout((l) => (findDocTab(l, id) ? l : openDocTab(l, id, id)));
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
        const all: CommandItem[] = [
          ...welcomeActions.map((action) => ({
            id: action.id,
            label: action.label,
            detail: action.hint,
            run: action.run,
          })),
          {
            id: "settings",
            label: "Settings",
            detail: "Appearance, provider keys",
            run: () => navigate("/settings"),
          },
          {
            id: "welcome",
            label: "Welcome page",
            detail: "Start, and what is in this folder",
            run: () => setLayout(openWelcomeTab),
          },
        ];
        const needle = term.toLowerCase();
        return all.filter((c) => c.label.toLowerCase().includes(needle));
      }
      if (mode === "content") {
        if (!term) return [];
        // The RANKED engine, deliberately: "where is the bit about invoices"
        // is a question with a best answer, unlike find-and-replace.
        const found = await api.search(term, 12);
        return found.hits.map((hit, index) => ({
          id: `${hit.path}:${index}`,
          label: hit.path,
          detail: hit.snippet,
          run: () => openHitRef.current(hit.path, hit.start_line),
        }));
      }
      // Files: everything the tree lists, ranked on the path.
      //
      // Collected whole and ranked afterwards. Filtering and capping inside
      // the walk — which is what this replaced — made the answer depend on
      // the order the tree was built in, and threw the best match away
      // without looking at it.
      const files: FileNode[] = [];
      const walk = (nodes: readonly FileNode[]) => {
        for (const node of nodes) {
          if (node.dir) {
            if (node.children) walk(node.children);
          } else {
            files.push(node);
          }
        }
      };
      for (const root of folderRootsRef.current) walk(root.tree);
      return rankFiles(files, term).map((node) => ({
        id: node.path,
        label: node.name,
        detail: node.path,
        run: () => openHitRef.current(node.path, 1),
      }));
    },
    [welcomeActions],
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
    (path: string, line: number) => {
      const node = findNodeByPath(folderRootsRef.current, path);
      const action = node
        ? fileAction(node, new Set(openableOutputsRef.current.keys()))
        : ({ kind: "file", path } as const);
      if (action.kind === "doc") {
        ensureDocOpen(action.id);
        navigate(`/docs/${action.id}`);
      } else if (action.kind === "generated") {
        const owner =
          action.docId ?? openableOutputsRef.current.get(action.path);
        if (owner) openGeneratedFor(owner, action.path);
      } else if (action.kind === "file") {
        openPlainFile(action.path);
      }
      revealLine(path, line);
    },
    [ensureDocOpen, openGeneratedFor, openPlainFile],
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
      setLayout((current) => openUntitledTab(current));
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
  // The folder tree's data: one root today, an array so several folders can
  // sit side by side later without this view changing shape. Hoisted above
  // the title effect: the root also names the window.
  const { roots: folderRoots, error: folderError, folderOpen } = useFolderTrees();
  folderRootsRef.current = folderRoots;
  // What the window calls itself: the custom override from Settings, else
  // the project folder's name, else the focused file (lib/windowTitle.ts).
  // The override is fetched once per mount — Settings is a different route,
  // so returning from it remounts this view and picks up a fresh value.
  const untitledFocused = (() => {
    const pane = panesOf(layout.root).find(
      (candidate) => candidate.id === layout.focus,
    );
    return pane?.tabs[pane.active]?.kind === "untitled";
  })();
  const [customTitle, setCustomTitle] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    api.settingsUi().then(
      (ui) => {
        if (live) {
          setCustomTitle(ui.window_title);
          setRetainSavedDrafts(ui.retain_unsaved_saved_files === true);
        }
      },
      // A server without the route (or unreachable) means no override.
      () => {},
    );
    return () => {
      live = false;
    };
  }, []);
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
    (reported: string): { path: string; id?: string } => {
      // Absolute (the engine's way of naming a file) or tree-relative (the
      // app's, and a session path from the dock): both resolve.
      const relative = (nodes: readonly FileNode[]): FileNode | null => {
        for (const n of nodes) {
          if (!n.dir && samePath(n.path, reported)) return n;
          if (n.children) {
            const hit = relative(n.children);
            if (hit) return hit;
          }
        }
        return null;
      };
      for (const root of folderRoots) {
        const node =
          nodeForAbsolutePath(root.tree, reported) ?? relative(root.tree);
        if (node) return { path: node.path, id: node.doc_id };
      }
      return { path: reported };
    },
    [folderRoots],
  );
  const ribbonSources: RibbonSource[] = useMemo(() => {
    if (!focused?.doc) return [];
    const primary: RibbonSource = {
      // The view is optional: a closed document pane still leaves bands
      // pointing back to it, which is how you find it again.
      view: focused.docEditor ?? undefined,
      docPath: focused.doc.path,
      docSource: focused.doc.source,
    };
    const others = new Map<string, RibbonSource>();
    for (const file of focused.outputs.values()) {
      for (const p of file.provenance) {
        if (p.origin.kind === "synthetic") continue;
        if (samePath(p.origin.doc_path, focused.doc.path)) continue;
        const { path } = docIdByPath(p.origin.doc_path);
        if (others.has(path)) continue;
        const open = registry
          .all()
          .find((s) => s.doc && samePath(s.doc.path, path));
        others.set(path, {
          view: open?.docEditor ?? undefined,
          docPath: path,
          docSource: open?.doc?.source ?? "",
        });
      }
    }
    // The lenses on screen — the agent pane's session — draw from their own
    // lines, with the links their elements declared.
    const lenses: RibbonSource[] = lensSources()
      .filter((lens) => !others.has(lens.path) && !samePath(lens.path, primary.docPath))
      .map((lens) => ({ view: lens.view, docPath: lens.path, docSource: lens.source }));
    return [primary, ...others.values(), ...lenses];
    // registry.version is what changes when another document opens or edits.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    lensTick,
    focused?.doc,
    focused?.docEditor,
    focused?.outputs,
    registry,
    registry.version,
    docIdByPath,
  ]);
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
    if (!doc) return [];
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
  const reopenFocusedDocument = useCallback(() => {
    const session = registry.get(focusedIdRef.current);
    if (!session?.doc) return;
    const { id, path } = session.doc;
    const already = findDocTab(layoutRef.current, id) !== null;
    setLayout((current) => openDocTab(current, id, path));
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
    (path: string, span?: [number, number]) => {
      const live = registry
        .all()
        .find((s) => s.doc && samePath(s.doc.path, path));
      if (live) {
        ensureDocOpen(live.docId);
        if (span) {
          if (live.docEditor) live.onSelectSpan(span);
          else pendingSelect.current = { path, span };
        }
        return;
      }
      const { id } = docIdByPath(path);
      if (span) pendingSelect.current = { path, span };
      if (id) ensureDocOpen(id);
      else focusTreeRef.current();
    },
    [registry, ensureDocOpen, docIdByPath],
  );

  // Show Files activates its pane and puts keyboard point in its tree.
  const focusTreeRef = useRef<() => void>(() => undefined);
  const focusTree = useFolderPane(folderOpen, workspaceUi.hydrated, layout, setLayout, setTreeFocusRequest, setInsertNotice);
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
    const view = focusedEditor();
    if (!view) {
      setInsertNotice(
        "There is no document open to insert into. Open one from the Files tree, or start a new one.",
      );
      return;
    }
    const { from, to } = view.state.selection.main;
    setInsertPanel({ id, selected: view.state.sliceDoc(from, to) });
  }, []);
  const applyInsert = useCallback(
    (
      element: Parameters<typeof insertElement>[1],
      values: Parameters<typeof insertElement>[2],
      body: string,
    ) => {
      const view = focusedEditor();
      if (!view) {
        setInsertNotice(
          "The document this was going into was closed. Open it again and retry.",
        );
        return;
      }
      insertElement(view, element, values, body);
    },
    [],
  );
  const openEditElement = useCallback(() => {
    const view = focusedEditor();
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
      element,
      values,
      body,
      attrs: block.attrs,
      from: block.from,
      to: block.to,
      source: view.state.sliceDoc(block.from, block.to),
    });
  }, []);
  const applyEditElement = useCallback(
    (element: InsertElement, values: FieldValues, body: string) => {
      const edit = editPanel;
      const view = focusedEditor();
      if (!edit || !view) return;
      // A collaborator may have changed this document while the form was
      // open. Offsets are not durable identities, so refuse rather than
      // replacing whichever bytes subsequently landed at this range.
      if (view.state.sliceDoc(edit.from, edit.to) !== edit.source) {
        setInsertNotice("This element changed while its editor was open. Reopen Edit Element to start from its current text.");
        return;
      }
      view.dispatch({
        changes: {
          from: edit.from,
          to: edit.to,
          insert: renderExistingElement(element, values, body, edit.attrs),
        },
        selection: { anchor: edit.from },
        scrollIntoView: true,
        userEvent: "input.editElement",
      });
      view.focus();
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
      return;
    }
    const tab = panesOf(layoutRef.current.root)
      .flatMap((pane) =>
        pane.tabs[pane.active] ? [pane.tabs[pane.active]] : [],
      )
      .find((t) => t.kind !== "tree" && t.kind !== "tool");
    printText({
      title: printTitleFor(tab?.target ?? "document"),
      text: view.state.doc.toString(),
    });
  }, []);
  // Honor "Show this page when a folder opens" after restoring the workspace.
  // Welcome becomes active; the person's restored tabs stay available.
  const welcomed = useRef(false);
  useEffect(() => {
    if (!workspaceUi.hydrated || welcomed.current) return;
    welcomed.current = true;
    if (startup || (route.name === "new" && route.file) || !loadShowWelcome()) return;
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
  useEffect(() => {
    const onCommand = (event: Event) => {
      const detail = (event as CustomEvent).detail as MenuAction;
      // Insert first: it belongs to the focused BUFFER, and the untitled one
      // has no session for the lookup below to find.
      if (detail === "insert" || insertTarget(detail) !== null) {
        openInsert(insertTarget(detail));
        return;
      }
      if (detail === "edit-element") {
        openEditElement();
        return;
      }
      // Format first, when asked to: the focused buffer, through whatever
      // formatter its language server offers. Then the save — every save,
      // since a plain file's autosave and a document's room both answer
      // to the flush, and the formatted text must be what lands on disk.
      const formatFirst = async () => {
        if (!formatOnSave()) return;
        const view = focusedEditor();
        if (view) await formatView(view).catch(() => false);
      };
      if (detail === "save-all") {
        void formatFirst().then(() => {
          for (const open of registry.all()) open.menuSave();
          requestFlushSaves();
        });
        return;
      }
      if (detail === "print") {
        printFocusedBuffer();
        return;
      }
      const focusedPane = panesOf(layoutRef.current.root).find(
        (pane) => pane.id === layoutRef.current.focus,
      );
      if (
        (detail === "save" || detail === "save-as") &&
        focusedPane?.tabs[focusedPane.active]?.kind === "untitled"
      ) {
        void formatFirst().then(() => saveUntitled(detail === "save-as"));
        return;
      }
      const session = registry.get(focusedIdRef.current);
      if (!session) return;
      if (detail === "save") {
        void formatFirst().then(() => {
          session.menuSave();
          requestFlushSaves();
        });
      } else if (detail === "save-as") session.menuSaveAs();
    };
    // The tree pane's reopen affordances, now that there is no toolbar:
    // File > Show Files in the native menu, and the explorer key everywhere.
    const onKey = (e: KeyboardEvent) => {
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
      void api.files().then((files) => {
        const node = nodeForAbsolutePath(files.tree, absolute);
        if (node?.doc_id) {
          navigate(`/docs/${node.doc_id}`);
        } else if (node && !node.dir && !isLikelyBinaryPath(node.path)) {
          // Any other text file opens as a plain-file pane, same as a
          // click on its tree row would.
          openPlainFile(node.path);
        } else {
          // A binary, a directory, or nothing the walk saw: the tree is
          // the way to it.
          focusTreeRef.current();
        }
      });
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
      if (!(event.metaKey || event.ctrlKey) || !event.shiftKey) return;
      if (event.key !== "f" && event.key !== "F") return;
      if (event.defaultPrevented) return;
      event.preventDefault();
      setSearchOpen(true);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
  const resolveHit = useCallback(
    (hit: SearchHit) =>
      resolveSearchHit(hit, {
        currentDocPath: focused?.doc?.path ?? null,
        docs: folderDocs,
        outputs: focused
          ? [...focused.outputs.values()].map((file) => ({
              path: file.path,
              content: file.content,
            }))
          : [],
      }),
    [focused, folderDocs],
  );
  const onSearchNavigate = useCallback(
    (target: SearchNavigation) => {
      const session = registry.get(focusedIdRef.current);
      switch (target.kind) {
        case "current-doc":
          session?.revealDocLine(target.line);
          return;
        case "doc":
          // The route effect turns this into "ensure open + activate";
          // navigating to the doc already routed still needs the ensure,
          // because its tab may have been closed since.
          ensureDocOpen(target.id);
          navigate(`/docs/${target.id}`);
          return;
        case "generated": {
          if (!session) return;
          // The ribbons' own mechanism: open the pane, then reveal — waiting
          // for the editor when the file was not open yet.
          openGeneratedFor(session.docId, target.path);
          session.revealOutput(target.path, target.range);
          return;
        }
        case "none":
          return;
      }
    },
    [registry, ensureDocOpen, openGeneratedFor],
  );
  // What a tree click can open beyond documents: any open document's
  // generated files, each owned by the document that made it.
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
  // Continuity, for THIS project. `undefined` until the server answers (or
  // forever, if it cannot) — the toggle is hidden rather than shown in a
  // state nobody chose. Off is the answer to "I could not tell".
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
    layout,
    layoutRef,
    setLayout,
    registry,
    retainSavedDrafts,
    untitledSources,
    untitledSourcesRef,
    forgetUntitled,
    saveUntitled,
    prompt: shellPrompt,
    plainDirtyTabs,
    plainUnsavedActions,
  });
  const banner = focused?.banner ?? null;
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
          // The measure belongs to the TAB, keyed by its path, and
          // is restored with the arrangement it was set in.
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
          registry={registry}
          docId={tab.docId}
          path={tab.target}
        />
      );
    }
    if (tab.kind === "file") {
      // Keyed by tab: two panes showing the same path are two
      // buffers, each saving whole and each hearing the other's
      // save as an external change on the next refresh signal.
      return (
        <PlainFilePane
          key={tab.id}
          path={tab.target}
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
            // The file gained an owner: this tab becomes a
            // generated tab in place, and the owning document
            // opens beside it — the comparison adoption exists
            // for. The tree refreshes to show the new document.
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
      return (
        <UntitledTab
          tabId={tab.id}
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
      // The emulator draws to a canvas, so the CSS zoom around it does
      // nothing; it is told its level and re-fits itself.
      return (
        <div className="terminal-tab">
          {/* Above the emulator, and present for exactly as long as the
              terminal is writing: the difference between "this disappears"
              and "this is being committed" is the most important thing on
              the screen, so it is never a toast. */}
          <AnchorBar
            anchor={terminals.anchors[tab.target] ?? null}
            onResume={() => void terminals.resumeAnchor(tab.target)}
            onUnanchor={() => void terminals.unanchor(tab.target)}
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
    if (tab.kind === "tool" && tab.target === GIT_TAB) {
      return (
        <GitPane
          onOpenFile={(path) => openHit(path, 1)}
          onOpenStory={() => setLayout(openStoryTab)}
        />
      );
    }
    if (tab.kind === "tool" && tab.target === STORY_TAB) {
      return <HistoryLens />;
    }
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
      // The conversation follows the FOCUSED document — one agent pane
      // re-targeting rather than one per document, which is the same choice
      // the dock made and the one worth keeping: the question you are asking
      // is almost always about what you are looking at.
      if (!focused) {
        return (
          <p className="muted chat-pane__empty">
            Open a document to talk to the agent about it.
          </p>
        );
      }
      return (
        <ChatDock
          onOpenSession={openDocumentByPath}
          key={focused.docId}
          docId={focused.docId}
          realtime={focused.realtime}
          onAgentFinished={focused.refresh}
        />
      );
    }
    if (tab.kind === "tree") {
      return (
        <FolderTreePane
          focusRequest={treeFocusRequest}
          roots={folderRoots}
          error={folderError}
          openable={new Set(openableOutputs.keys())}
          activeDocId={focusedId ?? undefined}
          dirtyPaths={dirtyPaths}
          onNewDocument={newDocument}
          // The session cwd places each terminal among the files where it runs.
          sessions={terminals.sessions.map((session) => ({
            id: session.id,
            title: session.title,
            cwd: session.cwd,
            state: session.state,
            monitor: session.monitor,
            cwdIsLive: session.cwd_is_live ?? false,
          }))}
          onOpenTerminal={showTerminal}
          // Terminals live in the tree now, so the verbs that used to sit on
          // the terminals pane live on the rows they act on: a directory
          // opens one IN that directory, and an icon's own menu is the only
          // place a session can be stopped.
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
          // A find hit opens its file and puts the caret on the line. The
          // file may be a document, a generated output, or a plain file —
          // the tree already knows which, so this reuses its own routing.
          onOpenHit={openHit}
          onOpen={(action) => {
            // A document ADDS a tab (or fronts its existing one);
            // a generated file opens beside its owner. Nothing
            // closes, nothing rebuilds.
            if (action.kind === "doc") {
              ensureDocOpen(action.id);
              navigate(`/docs/${action.id}`);
            } else if (action.kind === "generated") {
              // The server names the owner for any document in the folder;
              // `openableOutputs` only knows the OPEN ones, so it is the
              // fallback rather than the first answer.
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
      {/* One field across the top, understanding four prefixes, rather than
          four separate controls to learn and four places to look. */}
      <div className="workspace-top">
        <CommandBar candidates={commandCandidates} />
      </div>
      <div className="doc-main">
        {/* Asked at open, because a clone that never ran `hick init` merges
            `.md` documents with git's line merge and is told nothing. */}
        <MergeDriverNotice />
        {banner && (
          <div className={`banner banner-${banner.kind}`} role="status">
            {banner.text}
          </div>
        )}
        {focused?.renderError && (
          <div className="banner banner-fail" role="status">
            Could not weave this document — editing still works.{" "}
            {focused.renderError}
          </div>
        )}
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
          <ShellView
            layout={layout}
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
            // Every tab body is wrapped in its own zoom box. A pane's level
            // is a font size on that box and nothing else moves — which is
            // the point of a tab-scoped zoom: the furniture staying put is
            // what makes it usable for one dense file.
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
          {/* Where this text came from, drawn between the panes showing it.
              Not a layout: an overlay, following the focused document. */}
          <RibbonOverlay
            container={shellBox}
            sources={ribbonSources}
            files={ribbonFiles}
            links={ribbonLinks}
            // Only the DRAWN families reach the overlay. Continuity is a
            // fourth family with no overlay yet — the ribbon is the last step
            // of that design, after the journal has something in it worth
            // drawing — so it is filtered here rather than widening
            // `RibbonFamily` to a kind nothing emits.
            layers={drawnLayers}
            ribbonStyle={ribbonStyle}
            visibility={ribbonVisibility}
            onNavigate={(target) => {
              const session = registry.get(focusedIdRef.current);
              if (!session) return;
              // Clicking a band IS the navigation.
              if (target.kind === "path") {
                // A context/declared far end: a document or a plain file.
                if (target.path.endsWith(".md"))
                  openDocumentByPath(target.path);
                else openPlainFile(target.path);
                return;
              }
              if (target.kind === "document") {
                // Back the way it came: the document's own bytes, selected —
                // first bringing the document on screen when the band ended
                // at a port or an inactive tab rather than at visible prose.
                // Another document's bytes open THAT document.
                if (session.doc && !samePath(target.path, session.doc.path)) {
                  openDocumentByPath(target.path, target.span);
                  return;
                }
                if (!session.docEditor) reopenFocusedDocument();
                session.onSelectSpan(target.span);
                return;
              }
              openGeneratedFor(session.docId, target.path);
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
          initialId={insertPanel.id}
          selectedText={insertPanel.selected}
          onInsert={applyInsert}
          onClose={() => setInsertPanel(null)}
        />
      )}
      {editPanel && (
        <InsertMenu
          initialId={editPanel.element.id}
          selectedText=""
          edit={{ values: editPanel.values, body: editPanel.body }}
          onInsert={applyEditElement}
          onClose={() => setEditPanel(null)}
        />
      )}
      {searchOpen && (
        <SearchPanel
          resolve={resolveHit}
          onNavigate={onSearchNavigate}
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
        onClose={(id) => void terminals.close(id)}
      />
      {/* The top of the queue, brought to you. Answering it costs you
          nothing: the pane you were in keeps the focus. */}
      {attentionSession && (
        <AttentionCard
          session={attentionSession}
          position={attentionPlace + 1}
          total={terminals.attention.length}
          onAnswer={(send) => void terminals.answer(attentionSession.id, send)}
          onOpen={() => showTerminal(attentionSession.id)}
          onInterrupt={() => void terminals.interrupt(attentionSession.id)}
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
        extra={<WorkspaceStatusExtras
          provenance={{ layers, onToggle: toggleProvenance, continuity, onContinuity: setContinuityEnabled }}
          environmentCount={environments.notices.length}
          onEnvironments={() => setProblemsOpen(true)}
          projectId={projectId ?? null}
          onGraph={() => navigate(`/projects/${encodeURIComponent(projectId ?? "")}/lineage`)}
        />}
        onProblems={() => setProblemsOpen((open) => !open)}
        onAttention={nextAttention}
      />
    </div>
  );
}
