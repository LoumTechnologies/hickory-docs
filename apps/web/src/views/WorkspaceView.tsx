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

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { api } from "../api/client";
import type { DocSummary, OpenTerminal, SearchHit } from "../api/types";
import { ChatDock } from "../components/ChatDock";
import { SearchPanel } from "../components/SearchPanel";
import { ReferencesPanel } from "../components/ReferencesPanel";
import { PromptPanel } from "../components/PromptPanel";
import { resolveSearchHit, type SearchNavigation } from "../lib/searchNavigation";
import { loadRibbonStyle, type RibbonStyle } from "../lib/ribbonStyle";
import { loadTabStyle, type TabStyle } from "../lib/tabStyle";
import { loadChannelWidth } from "../lib/channelWidth";
import { windowTitle } from "../lib/windowTitle";
import { deriveRibbons } from "../lib/ribbons";
import { flashTab } from "../lib/flashTab";
import { nodeForAbsolutePath } from "../lib/openPath";
import { ShellView, type ShellPort } from "../shell/ShellView";
import { RibbonOverlay, type RibbonFile } from "../shell/Ribbons";
import { FILES_CHANGED_EVENT, FolderTreePane, useFolderTrees } from "../shell/FolderTreePane";
import { activate, panes as panesOf, tab as makeTab, treePane, withTree, type Layout } from "../shell/layout";
import { regionsOf } from "../shell/layouts";
import type { Region } from "../shell/layout";
import { navigate, redirect, type Route } from "../router";
import {
  activateDocTab,
  adoptUntitledTab,
  docIdsIn,
  findDocTab,
  focusedDocId,
  initialWorkspace,
  isWorkspaceEmpty,
  openDocTab,
  openGeneratedTab,
  openIntoDeclared,
  openSessionsTab,
  openTerminalTab,
  openUntitledTab,
  SESSIONS_TAB,
} from "./workspaceState";
import { DocSessionHost, SessionRegistry, useSessionVersion } from "./documentSession";
import { AttentionCard } from "../terminal/AttentionCard";
import { MonitorDock } from "../terminal/MonitorDock";
import { SessionsPane } from "../terminal/SessionsPane";
import { TerminalPane } from "../terminal/TerminalPane";
import { sessionById, useTerminals } from "../terminal/useTerminals";
import { nextInQueue } from "../lib/attentionCursor";
import { DocTabBody, GeneratedTabBody, UntitledTab } from "./workspaceTabs";

/** The routes the workspace answers. Everything else is App's. */
export type WorkspaceRoute = Extract<Route, { name: "doc" } | { name: "new" }>;

export function WorkspaceView({ route }: { route: WorkspaceRoute }) {
  // What the window is arranged as. Session state, owned HERE, above any
  // document: navigating between documents must leave it untouched.
  const [layout, setLayout] = useState<Layout>(initialWorkspace);
  const layoutRef = useRef(layout);
  layoutRef.current = layout;
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
  // The dock is part of the workspace, not a mode: it is always mounted and
  // remembers whether the log is expanded.
  const [chatCollapsed, setChatCollapsed] = useState(
    () => localStorage.getItem("hickory.chatCollapsed") === "1",
  );
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

  // The route is a REQUEST against the workspace, not its owner:
  // `#/docs/<id>` means "make sure this document is open and frontmost",
  // `#/new` means "make sure there is an untitled buffer". Back and forward
  // therefore just re-activate tabs that are already there.
  const routeKey = route.name === "doc" ? `doc:${route.id}` : "new";
  useEffect(() => {
    if (route.name === "new") {
      setLayout((current) => openUntitledTab(current));
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
  }, [routeKey, ensureDocOpen]);

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

  // ---- native menu: Save / Save As ---------------------------------------
  //
  // The desktop menu's Save and Save As arrive from App as one window event;
  // they act on the FOCUSED document's session, whichever that is.
  useEffect(() => {
    const onCommand = (event: Event) => {
      const session = registry.get(focusedIdRef.current);
      if (!session) return;
      const detail = (event as CustomEvent).detail;
      if (detail === "save") session.menuSave();
      else if (detail === "save-as") session.menuSaveAs();
    };
    // The tree pane's reopen affordances, now that there is no toolbar:
    // File > Show Files in the native menu, and the explorer key everywhere.
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === "e") {
        e.preventDefault();
        focusTreeRef.current();
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
        } else {
          // Not a document (or not indexed): the tree is the way to it.
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
  }, [registry]);

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
        setLayout(openSessionsTab);
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

  const banner = focused?.banner ?? null;

  return (
    <div className="doc-page with-chat wide-mode">
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
            render={(tab) => {
              if (tab.kind === "document" && tab.docId) {
                return <DocTabBody registry={registry} docId={tab.docId} />;
              }
              if (tab.kind === "generated" && tab.docId) {
                return <GeneratedTabBody registry={registry} docId={tab.docId} path={tab.target} />;
              }
              if (tab.kind === "untitled") {
                return <UntitledTab tabId={tab.id} onCreated={onUntitledCreated} />;
              }
              if (tab.kind === "terminal") {
                return <TerminalPane sessionId={tab.target} />;
              }
              if (tab.kind === "tool" && tab.target === SESSIONS_TAB) {
                return (
                  <SessionsPane
                    sessions={terminals.sessions}
                    turbo={terminals.turbo}
                    error={terminals.error}
                    activeId={attentionAt}
                    onOpen={showTerminal}
                    onClose={(id) => void terminals.close(id)}
                    onNew={(monitor) => void openTerminal({ monitor })}
                    onNewWorktree={(branch) =>
                      void openTerminal({ title: branch, worktree_branch: branch })
                    }
                    onSetTurbo={(enabled) => void terminals.setTurbo(enabled)}
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
                    onOpen={(action) => {
                      // A document ADDS a tab (or fronts its existing one);
                      // a generated file opens beside its owner. Nothing
                      // closes, nothing rebuilds.
                      if (action.kind === "doc") {
                        ensureDocOpen(action.id);
                        navigate(`/docs/${action.id}`);
                      } else {
                        const owner = openableOutputs.get(action.path);
                        if (owner) openGeneratedFor(owner, action.path);
                      }
                    }}
                  />
                );
              }
              return null;
            }}
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
      {focused && (
        <ChatDock
          // Keyed by document: the dock docks to the focused document, one
          // dock re-targeting on focus change (v1 — a dock per document is
          // the follow-up if switching proves disruptive).
          key={focused.docId}
          docId={focused.docId}
          realtime={focused.realtime}
          collapsed={chatCollapsed}
          onToggleCollapsed={() =>
            setChatCollapsed((v) => {
              localStorage.setItem("hickory.chatCollapsed", v ? "0" : "1");
              return !v;
            })
          }
          onAgentFinished={focused.refresh}
        />
      )}
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
    </div>
  );
}
