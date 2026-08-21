// The workspace: ONE layout, above document identity.
//
// This component mounts once for every document-shaped route and stays
// mounted as `#/docs/<id>` comes and goes. That inversion is the whole
// point: the tile tree, the focus, the collapse state and every open tab
// are session state a navigation must never reset. Opening a document —
// from the tree, from a URL, from search, from the untitled buffer growing
// a name — ADDS a tab (or activates the one that exists) and touches
// nothing else.
//
// Per-document machinery (CRDT room, runs, LSP, debugger, outputs) lives in
// one session per open document — see views/documentSession.tsx. The chrome
// that is genuinely singular stays here: the toolbar (the style toggles now
// live on the Settings page and are only read here),
// the search panel, the folder tree, the menu Save routing, the agent dock,
// and the ribbon overlay — the last two following the FOCUSED document.
//
// See docs/specs/freeform/shell-layouts.md.

import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";

import { api } from "../api/client";
import type { DocSummary, FileNode, OpenTerminal, SearchHit } from "../api/types";
import { ChatDock } from "../components/ChatDock";
import { InsertMenu } from "../components/InsertMenu";
import { PlainFilePane } from "../components/PlainFilePane";
import { ScratchpadPane } from "../components/ScratchpadPane";
import { SearchPanel } from "../components/SearchPanel";
import { ReferencesPanel } from "../components/ReferencesPanel";
import { PromptPanel, usePrompt } from "../components/PromptPanel";
import { resolveSearchHit, type SearchNavigation } from "../lib/searchNavigation";
import { insertTarget, type MenuAction } from "../lib/menuBridge";
import { insertElement } from "../editor/insertElement";
import { loadRibbonStyle, type RibbonStyle } from "../lib/ribbonStyle";
import { loadTabStyle, type TabStyle } from "../lib/tabStyle";
import { loadChannelWidth } from "../lib/channelWidth";
import { windowTitle } from "../lib/windowTitle";
import { deriveRibbons } from "../lib/ribbons";
import { flashTab } from "../lib/flashTab";
import { nodeForAbsolutePath } from "../lib/openPath";
import { ShellView, type ShellPort } from "../shell/ShellView";
import { RibbonOverlay, type RibbonFile } from "../shell/Ribbons";
import {
  FILES_CHANGED_EVENT,
  FolderTreePane,
  isLikelyBinaryPath,
  useFolderTrees, fileAction } from "../shell/FolderTreePane";
import {
  activate,
  paneById,
  panes as panesOf,
  tab as makeTab,
  treePane,
  withTree,
  type Layout,
  type Tab,
} from "../shell/layout";
import { regionsOf } from "../shell/layouts";
import type { Region } from "../shell/layout";
import { navigate, redirect, type Route } from "../router";
import {
  activateDocTab,
  adoptPlainFileTab,
  adoptUntitledTab,
  docIdsIn,
  findDocTab,
  findFileTab,
  focusedDocId,
  initialWorkspace,
  isWorkspaceEmpty,
  openDocTab,
  openFileTab,
  openGeneratedTab,
  openIntoDeclared,
  openTerminalTab,
  openScratchpadTab,
  openUntitledTab,
  openWelcomeTab,
  openGitTab,
  WELCOME_TAB,
  GIT_TAB,
} from "./workspaceState";
import { DocSessionHost, SessionRegistry, useSessionVersion } from "./documentSession";
import { AttentionCard } from "../terminal/AttentionCard";
import { MonitorDock } from "../terminal/MonitorDock";
import { TerminalPane } from "../terminal/TerminalPane";
import { sessionById, useTerminals } from "../terminal/useTerminals";
import { nextInQueue } from "../lib/attentionCursor";
import { DocTabBody, GeneratedTabBody, UntitledTab } from "./workspaceTabs";
import { useWorkspaceUi } from "./useWorkspaceUi";
import { focusedEditor } from "../editor/activeEditor";
import { useZoom } from "./useZoom";
import { StatusBar } from "../shell/StatusBar";
import { WelcomePane, type WelcomeAction } from "./WelcomePane";
import { GitPane } from "./GitPane";
import { CommandBar, type CommandItem, type CommandMode } from "../shell/CommandBar";
import { loadShowWelcome } from "../lib/welcomePref";
import { severityOf, totalProblems } from "../lib/problems";
import { positionToUtf16 } from "../lsp/positions";
import { EditorView } from "@codemirror/view";
import { TAB_ZOOM_VAR } from "../lib/zoom";
import { requestFlushSaves } from "../lib/flushSaves";
import { revealLine } from "../lib/revealLine";
import { printText, printTitleFor } from "../lib/printing";

/** The routes the workspace answers. Everything else is App's. */
export type WorkspaceRoute = Extract<
  Route,
  { name: "doc" } | { name: "new" } | { name: "scratchpad" }
>;

/** The tree node for a root-relative path, across every open root. */
function findNodeByPath(
  roots: readonly { tree: FileNode[] }[],
  path: string,
): FileNode | null {
  const walk = (nodes: readonly FileNode[]): FileNode | null => {
    for (const node of nodes) {
      if (node.path === path) return node;
      const found = node.children ? walk(node.children) : null;
      if (found) return found;
    }
    return null;
  };
  for (const root of roots) {
    const found = walk(root.tree);
    if (found) return found;
  }
  return null;
}

export function WorkspaceView({ route }: { route: WorkspaceRoute }) {
  // What the window is arranged as. Session state, owned HERE, above any
  // document: navigating between documents must leave it untouched.
  const [layout, setLayout] = useState<Layout>(initialWorkspace);
  const layoutRef = useRef(layout);
  layoutRef.current = layout;
  // Read through refs by the welcome page's actions and the top field's
  // candidates, which are built before the callbacks they reach for exist.
  const openTerminalRef = useRef<() => Promise<unknown> | void>(() => undefined);
  const openHitRef = useRef<(path: string, line: number) => void>(() => undefined);
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
  const [insertPanel, setInsertPanel] = useState<{ id: string | null; selected: string } | null>(
    null,
  );
  // Why an insert could not happen. Rare — it needs a workspace with no
  // editor in it at all — but silence would look like a broken menu.
  const [insertNotice, setInsertNotice] = useState<string | null>(null);
  // The dock is part of the workspace, not a mode: it is always mounted and
  // remembers whether the log is expanded.
  // The presentation preferences: how lineage draws (bands or braces), where
  // tabs live, how wide the inter-pane channel is. All EDITED on the Settings
  // page ("#/settings" — see SettingsView's Appearance section) and only READ
  // here. Reading once at mount is enough: App swaps this view out for
  // SettingsView while settings are open, so coming back remounts the
  // workspace and re-reads whatever was just saved to localStorage.
  const [ribbonStyle] = useState<RibbonStyle>(() => loadRibbonStyle());
  const [tabStyle] = useState<TabStyle>(() => loadTabStyle());
  const [channelWidth] = useState<number>(() => loadChannelWidth());
  const [shellBox, setShellBox] = useState<HTMLElement | null>(null);
  // The workspace's own prompt, for the things that belong to the WINDOW
  // rather than to a document — asking for a worktree's branch name, now that
  // terminals have no pane of their own to ask on.
  const shellPrompt = usePrompt();

  // What the window looked like last time, and where each tab's prose measure
  // sits. Restored into an untouched workspace only — the same rule a
  // document's own declared layout follows — and the route's opener waits for
  // `hydrated` so the two cannot race. See views/useWorkspaceUi.ts.
  // Where the caret is, for the status bar. Held as state rather than read
  // during render because the editors deliberately do not re-render on every
  // keystroke — this is subscribed to instead, and it is the only thing in
  // the window that wants a per-keystroke update.
  const [caret, setCaret] = useState<{ line: number; column: number } | null>(null);
  useEffect(() => {
    let frame: number | null = null;
    const read = () => {
      frame = null;
      const view = focusedEditor();
      if (!view || !view.dom.isConnected) {
        setCaret(null);
        return;
      }
      const head = view.state.selection.main.head;
      const line = view.state.doc.lineAt(head);
      setCaret({ line: line.number, column: head - line.from + 1 });
    };
    // Polled on a frame rather than hooked into every editor: there is no one
    // editor to hook, panes come and go, and a status bar that is one frame
    // behind the caret is indistinguishable from one that is not.
    const tick = () => {
      if (frame === null) frame = requestAnimationFrame(read);
    };
    const timer = window.setInterval(tick, 120);
    tick();
    return () => {
      window.clearInterval(timer);
      if (frame !== null) cancelAnimationFrame(frame);
    };
  }, []);

  // How much is wrong, across every open document. Recomputed from the
  // sessions' own diagnostics rather than kept as a second copy: two counts
  // that can disagree is worse than no count at all.
  const problems = useMemo(
    () => totalProblems(registry.all().map((open) => open.lspDiagnostics ?? [])),
    // registry.version (via useSessionVersion) is what actually changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [registry, registry.version],
  );

  /** Put the caret on the next error or warning in the focused document. */
  const goToNextProblem = useCallback(() => {
    const session = registry.get(focusedIdRef.current);
    const view = session?.docEditor;
    const diagnostics = session?.lspDiagnostics ?? [];
    if (!view || diagnostics.length === 0) return;
    const ranked = [...diagnostics]
      .filter((d) => severityOf(d) <= 2)
      .map((d) => ({
        d,
        at: positionToUtf16(view.state.doc.toString(), d.range.start),
      }))
      .sort((a, b) => a.at - b.at);
    if (ranked.length === 0) return;
    const head = view.state.selection.main.head;
    // Wraps: pressing it at the last problem takes you back to the first,
    // which is what "next" means in a list you are working through.
    const next = ranked.find((r) => r.at > head) ?? ranked[0];
    view.dispatch({
      selection: { anchor: next.at },
      effects: EditorView.scrollIntoView(next.at, { y: "center" }),
    });
    view.focus();
  }, [registry]);

  // The branch, for the status bar. Refetched when files change — a commit,
  // a checkout, or a save can all move it.
  const [git, setGit] = useState<{ branch: string; dirty: number } | null>(null);
  useEffect(() => {
    const read = () => {
      void api.gitStatus().then(
        (status) =>
          setGit(
            status.repository && status.branch
              ? {
                  branch: status.branch,
                  dirty: (status.staged ?? 0) + (status.unstaged ?? 0) + (status.untracked ?? 0),
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
    return tab && tab.kind !== "tree" && tab.kind !== "tool" ? tab.target : null;
  })();

  /** What the welcome page offers. Every one of them does something — a row
   * here is a verb, never a link to a tour. */
  const welcomeActions: WelcomeAction[] = useMemo(
    () => [
      {
        id: "new",
        label: "New document…",
        hint: "An untitled buffer, adopted into a document on its first save",
        run: () => navigate("/new"),
      },
      {
        id: "scratchpad",
        label: "Scratchpad",
        hint: "Text on its way to becoming a note",
        run: () => navigate("/scratchpad"),
      },
      {
        id: "terminal",
        label: "Open a terminal",
        hint: "In this folder; it appears on the folder's row in the tree",
        run: () => {
          void openTerminalRef.current();
        },
      },
      {
        id: "history",
        label: "History",
        hint: "The commit graph, with every commit's files",
        run: () => setLayout(openGitTab),
      },
      {
        id: "find",
        label: "Find in folder…",
        hint: "Exhaustive find and replace across every file",
        run: () => focusTreeRef.current(),
      },
    ],
    [],
  );

  const workspaceUi = useWorkspaceUi(layout, (restored) => {
    setLayout((current) => (isWorkspaceEmpty(current) ? restored : current));
  });

  // ⌘+ / ⌘- / ⌘0 size the whole window; adding Alt sizes only the focused
  // tab. See views/useZoom.ts for why that split, and why it is the root's
  // font size rather than a transform.
  const zoom = useZoom({
    focusedTarget: () => {
      const pane = paneById(layoutRef.current, layoutRef.current.focus);
      const tab = pane?.tabs[pane.active];
      return tab && tab.kind !== "tree" && tab.kind !== "tool" ? tab.target : null;
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
    setLayout((current) => openGeneratedTab(current, docId, path, regionsRef.current));
    if (already) flashTab("generated", path);
  }, []);

  /** Open a plain file — same "ensure open" a document gets, no owner. */
  const openPlainFile = useCallback((path: string) => {
    const already = findFileTab(layoutRef.current, path) !== null;
    setLayout((current) => openFileTab(current, path));
    if (already) flashTab("file", path);
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
        setLayout((l) => (findDocTab(l, id) ? (activateDocTab(l, id) ?? l) : openDocTab(l, id, doc.path)));
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
                effects: EditorView.scrollIntoView(target.from, { y: "center" }),
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
      // Files: everything the tree lists, matched on the path.
      const needle = term.toLowerCase();
      const out: CommandItem[] = [];
      const walk = (nodes: readonly FileNode[]) => {
        for (const node of nodes) {
          if (node.dir) {
            if (node.children) walk(node.children);
            continue;
          }
          if (needle && !node.path.toLowerCase().includes(needle)) continue;
          out.push({
            id: node.path,
            label: node.name,
            detail: node.path,
            run: () => openHitRef.current(node.path, 1),
          });
          if (out.length >= 40) return;
        }
      };
      for (const root of folderRootsRef.current) walk(root.tree);
      return out;
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
        const owner = action.docId ?? openableOutputsRef.current.get(action.path);
        if (owner) openGeneratedFor(owner, action.path);
      } else if (action.kind === "file") {
        openPlainFile(action.path);
      }
      revealLine(path, line);
    },
    [ensureDocOpen, openGeneratedFor, openPlainFile],
  );
  openHitRef.current = openHit;

  // The route is a REQUEST against the workspace, not its owner:
  // `#/docs/<id>` means "make sure this document is open and frontmost",
  // `#/new` means "make sure there is an untitled buffer". Back and forward
  // therefore just re-activate tabs that are already there.
  // Every non-document route needs its OWN key: collapsing them all to one
  // string means navigating from `#/new` to `#/scratchpad` looks like no
  // change at all, and the effect never runs.
  const routeKey = route.name === "doc" ? `doc:${route.id}` : route.name;
  const hydrated = workspaceUi.hydrated;
  useEffect(() => {
    // Wait for the stored layout. Opening the routed document first would
    // leave the workspace non-empty when the restore lands, and the restore
    // would be dropped without a word.
    if (!hydrated) return;
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

  // The untitled buffer earned a name: adopt its tab in place, then let the
  // URL say so (a redirect, so Back never returns to a buffer that no
  // longer exists).
  const onUntitledCreated = useCallback((tabId: string, docId: string, path: string) => {
    setLayout((current) => adoptUntitledTab(current, tabId, docId, path));
    redirect(`/docs/${docId}`);
  }, []);

  // The folder tree's data: one root today, an array so several folders can
  // sit side by side later without this view changing shape. Hoisted above
  // the title effect: the root also names the window.
  const { roots: folderRoots, error: folderError } = useFolderTrees();
  folderRootsRef.current = folderRoots;

  // What the window calls itself: the custom override from Settings, else
  // the project folder's name, else the focused file (lib/windowTitle.ts).
  // The override is fetched once per mount — Settings is a different route,
  // so returning from it remounts this view and picks up a fresh value.
  const untitledFocused = (() => {
    const pane = panesOf(layout.root).find((candidate) => candidate.id === layout.focus);
    return pane?.tabs[pane.active]?.kind === "untitled";
  })();
  const [customTitle, setCustomTitle] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    api.settingsUi().then(
      (ui) => {
        if (live) setCustomTitle(ui.window_title);
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

  // ---- ribbons and ports (the focused document's, v1) ----------------------
  //
  // The overlay draws ONE document's lineage: the focused one's source pane,
  // its outputs, its tabs and tree rows and ports. On focus change it
  // re-targets. The seam for drawing several documents' overlays at once is
  // already here — RibbonOverlay takes {source, files} — that is the
  // follow-up, not this change.
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

  // The toolbar's Files button: bring the tree pane back, or the eye to it.
  // Never a navigation — the folder is a pane of this window, not a page
  // somewhere else.
  const focusTreeRef = useRef<() => void>(() => undefined);
  const focusTree = useCallback(() => {
    setLayout((current) => {
      const pane = treePane(current);
      if (!pane) return withTree(current, makeTab("tree", "folder", "Files"));
      const index = pane.tabs.findIndex((t) => t.kind === "tree");
      return activate(current, pane.id, Math.max(0, index));
    });
  }, []);
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
    return list;
  }, [focused, openTargets, openGeneratedFor, reopenFocusedDocument]);

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
    (element: Parameters<typeof insertElement>[1], values: Parameters<typeof insertElement>[2], body: string) => {
      const view = focusedEditor();
      if (!view) {
        setInsertNotice("The document this was going into was closed. Open it again and retry.");
        return;
      }
      insertElement(view, element, values, body);
    },
    [],
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
      .flatMap((pane) => (pane.tabs[pane.active] ? [pane.tabs[pane.active]] : []))
      .find((t) => t.kind !== "tree" && t.kind !== "tool");
    printText({
      title: printTitleFor(tab?.target ?? "document"),
      text: view.state.doc.toString(),
    });
  }, []);

  // The welcome page, once, and only into a workspace that has nothing open.
  // A workspace restored with work in it does not want a welcome screen in
  // front of it; that is the whole difference between "just launched" and
  // "came back".
  const welcomed = useRef(false);
  useEffect(() => {
    if (!workspaceUi.hydrated || welcomed.current) return;
    welcomed.current = true;
    if (!loadShowWelcome()) return;
    setLayout((current) => (isWorkspaceEmpty(current) ? openWelcomeTab(current) : current));
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
      // Save All is the one command that is deliberately NOT about the
      // focused buffer: every open document, plus every pane holding a file
      // it saves itself. The panes are reached by an event rather than by a
      // registry because a plain file has no session — it owns its own saver,
      // and only it knows whether anything is pending.
      if (detail === "save-all") {
        for (const open of registry.all()) open.menuSave();
        requestFlushSaves();
        return;
      }
      if (detail === "print") {
        printFocusedBuffer();
        return;
      }
      const session = registry.get(focusedIdRef.current);
      if (!session) return;
      if (detail === "save") session.menuSave();
      else if (detail === "save-as") session.menuSaveAs();
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
      }
    };
    const onFiles = () => focusTreeRef.current();
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
    window.addEventListener("hickory-open-path", onOpenPath);
    return () => {
      window.removeEventListener("hickory-doc-command", onCommand);
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("hickory-show-files", onFiles);
      window.removeEventListener("hickory-open-path", onOpenPath);
    };
  }, [registry, openPlainFile, openInsert]);

  // ---- terminals ----------------------------------------------------------
  //
  // The window's sessions, the queue across them, and the two keys that reach
  // it. A pane SHOWS a session; the session lives on the server, which is why
  // closing a terminal tab here never stops the work inside it.
  const terminals = useTerminals();
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
    setLayout((current) => openTerminalTab(current, id, session?.title ?? "Terminal"));
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
    return () => window.removeEventListener("hickory-terminal-command", onTerminalCommand);
  }, [openTerminal, nextAttention]);

  // The card follows the cursor, and lets go when what it was showing stops
  // claiming anything — answered here, answered in its own terminal, or
  // closed. A card for a settled session is a card you learn to ignore.
  const attentionSession = sessionById(terminals.sessions, attentionAt);
  const attentionPlace = attentionAt === null ? -1 : terminals.attention.indexOf(attentionAt);
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
          ? [...focused.outputs.values()].map((file) => ({ path: file.path, content: file.content }))
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
        />
      );
    }
    if (tab.kind === "generated" && tab.docId) {
      return <GeneratedTabBody registry={registry} docId={tab.docId} path={tab.target} />;
    }
    if (tab.kind === "file") {
      // Keyed by tab: two panes showing the same path are two
      // buffers, each saving whole and each hearing the other's
      // save as an external change on the next refresh signal.
      return (
        <PlainFilePane
          key={tab.id}
          path={tab.target}
          onAdopted={(adopted) => {
            // The file gained an owner: this tab becomes a
            // generated tab in place, and the owning document
            // opens beside it — the comparison adoption exists
            // for. The tree refreshes to show the new document.
            setLayout((current) =>
              adoptPlainFileTab(current, tab.target, adopted.doc_id, adopted.output_path),
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
      return <UntitledTab tabId={tab.id} onCreated={onUntitledCreated} />;
    }
    if (tab.kind === "terminal") {
      // The emulator draws to a canvas, so the CSS zoom around it does
      // nothing; it is told its level and re-fits itself.
      return <TerminalPane sessionId={tab.target} zoom={workspaceUi.zoomFor(tab.target)} />;
    }
    if (tab.kind === "tool" && tab.target === GIT_TAB) {
      return <GitPane onOpenFile={(path) => openHit(path, 1)} />;
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
          roots={folderRoots}
          error={folderError}
          openable={new Set(openableOutputs.keys())}
          activeDocId={focusedId ?? undefined}
          onNewDocument={() => navigate("/new")}
          // What is running, shown where it is running. The tree
          // already knows the folder; the sessions already know
          // their directory; this is the join.
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
            void shellPrompt.askText("Branch for the new worktree:", "").then((branch) => {
              if (branch) {
                void openTerminal({ title: branch, worktree_branch: branch, cwd: path });
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
      {/* One field across the top, understanding four prefixes, rather than
          four separate controls to learn and four places to look. */}
      <div className="workspace-top">
        <CommandBar candidates={commandCandidates} />
      </div>
      <div className="doc-main">
        {banner && (
          <div className={`banner banner-${banner.kind}`} role="status">
            {banner.text}
          </div>
        )}
        {focused?.renderError && (
          <div className="banner banner-fail" role="status">
            Could not weave this document — editing still works. {focused.renderError}
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
            />
          ))}
          <ShellView
            layout={layout}
            onLayout={setLayout}
            onSearch={() => setSearchOpen(true)}
            ports={ports}
            tabStyle={tabStyle}
            channelWidth={channelWidth}
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
                style={{ [TAB_ZOOM_VAR]: workspaceUi.zoomFor(tab.target) } as React.CSSProperties}
              >
                {renderTabBody(tab)}
              </div>
            )}
          />
          {/* Where this text came from, drawn between the panes showing it.
              Not a layout: an overlay, following the focused document. */}
          <RibbonOverlay
            container={shellBox}
            source={
              // The view is optional: a closed document pane still leaves
              // bands pointing back to it, which is how you find it again.
              focused?.doc
                ? {
                    view: focused.docEditor ?? undefined,
                    docPath: focused.doc.path,
                    docSource: focused.doc.source,
                  }
                : null
            }
            files={ribbonFiles}
            documentVisible={!!focused?.docEditor}
            ribbonStyle={ribbonStyle}
            onNavigate={(target) => {
              const session = registry.get(focusedIdRef.current);
              if (!session) return;
              // Clicking a band IS the navigation.
              if (target.kind === "document") {
                // Back the way it came: the document's own bytes, selected —
                // first bringing the document on screen when the band ended
                // at a port or an inactive tab rather than at visible prose.
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
      {searchOpen && (
        <SearchPanel
          resolve={resolveHit}
          onNavigate={onSearchNavigate}
          onClose={() => setSearchOpen(false)}
        />
      )}
      {focused?.references && (
        <ReferencesPanel
          locations={focused.references.locations}
          query={focused.references.query}
          onPick={focused.openTarget}
          onClose={focused.clearReferences}
        />
      )}
      {focused && <PromptPanel prompt={focused.prompt} onSettle={focused.settle} />}
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
        caret={caret}
        zoom={zoom.uiZoom}
        onProblems={goToNextProblem}
        onAttention={nextAttention}
      />
    </div>
  );
}
