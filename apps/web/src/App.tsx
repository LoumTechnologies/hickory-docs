import { lazy, Suspense, useEffect, useRef, useState } from "react";
import { navigate, useRoute, newDocument } from "./router";
import { WorkspaceView, type WorkspaceRoute } from "./views/WorkspaceView";
import { api } from "./api/client";
import { insertTarget, onMenuAction, type MenuAction } from "./lib/menuBridge";
import { UiActionsVisibility, useUiCommands } from "./lib/uiActionBridge";
import { TooltipLayer } from "./components/TooltipLayer";
import { showTerminalRequest } from "./lib/revealLine";
import type { ScaffoldStarted } from "./api/types";
import { FILES_CHANGED_EVENT } from "./shell/FolderTreePane";
import { BlankWelcome } from "./views/BlankWelcome";
import { SETTINGS_CLOSED } from "./lib/workspaceVisibility";
import { StartupNotice } from "./components/StartupNotice";

const LineageView = lazy(() => import("./views/LineageView").then((m) => ({ default: m.LineageView })));
const SettingsView = lazy(() => import("./views/SettingsView").then((m) => ({ default: m.SettingsView })));
const NewProjectDialog = lazy(() => import("./components/NewProjectDialog").then((m) => ({ default: m.NewProjectDialog })));

/** How often the app asks whether a scaffold has landed, and for how long.
 * The person is watching the terminal; this is only so the tree and the
 * history pane catch up on their own. Ten minutes is longer than any
 * `dotnet new` and shorter than forever. */
const SCAFFOLD_POLL_MS = 400;
const SCAFFOLD_POLL_TRIES = 1500;

/// The desktop app's shell.
///
/// There is no authentication, no landing page, and no account menu. This UI
/// only ever talks to the local server running in the same process, which
/// answers one person: the one whose machine it is. The marketing site is a
/// separate build — see `docs/specs/freeform/local-only.md`.
export function App() {
  const requestedRoute = useRoute();
  const route = requestedRoute.name === "landing"
    ? { name: "new" as const, introduction: true }
    : requestedRoute;
  const inWorkspace = route.name === "doc" || route.name === "new" || route.name === "scratchpad";
  const workspace = useRef<{ route: WorkspaceRoute; hash: string } | null>(null);
  const pendingWorkspaceClose = useRef(false);
  if (inWorkspace) workspace.current = { route: route as WorkspaceRoute, hash: location.hash || "#/" };
  if (route.name === "blank") workspace.current = null;
  useEffect(() => {
    if (!inWorkspace || !pendingWorkspaceClose.current) return;
    pendingWorkspaceClose.current = false;
    window.dispatchEvent(new Event("hickory-workspace-close-request"));
  }, [inWorkspace]);

  // The desktop shell's native menu, arriving as DOM events (menuBridge).
  // Navigation is answered here; Save / Save As need the focused document,
  // which only the workspace knows — they are re-dispatched as a window
  // event it listens for. The subscription is mounted once, so the current route is
  // read through a ref rather than re-subscribing (which would reset the
  // bridge's duplicate-keypress guard).
  const [notice, setNotice] = useState<string | null>(null);
  // New Project lives here rather than in the workspace: it needs no buffer,
  // no focused document and no layout — it writes a file into the folder —
  // so hanging it off the workspace would have made it unreachable from
  // Settings and from the landing redirect for no reason.
  const [newProject, setNewProject] = useState(false);

  // What happens after New Project's button: the scaffolder is running in a
  // terminal, and that terminal is what the person watches.
  //
  // The app has its own reason to know how it ended — the tree has a new
  // folder in it and the history a new commit — so it polls for the verdict
  // while the terminal shows the person the same act in `dotnet`'s own words.
  // Two readers of one thing, neither pretending to be the other; this one
  // says one sentence and stops, because everything worth reading is in the
  // tab. See docs/guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal.md.
  const watchScaffold = (started: ScaffoldStarted) => {
    showTerminalRequest(started.session.id, started.session.title);
    let tries = 0;
    const poll = () => {
      void api
        .scaffoldResult(started.session.id)
        .then((result) => {
          if (result.state === "running") {
            // Bounded, because a `dotnet new` that never exits is a terminal
            // the person is already looking at — not a poll to keep forever.
            if (tries++ < SCAFFOLD_POLL_TRIES) setTimeout(poll, SCAFFOLD_POLL_MS);
            return;
          }
          window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
          if (result.state === "committed") {
            setNotice(
              `${result.output}/ scaffolded and committed as ${result.short} — ` +
                `${result.files.length} file${result.files.length === 1 ? "" : "s"}. ` +
                "Read it as a story in the History pane.",
            );
          } else {
            // The terminal has the whole of it; this only says where to look.
            setNotice(
              `Nothing was committed — the ${started.session.title} terminal says why.`,
            );
          }
        })
        .catch(() => {
          /* The terminal is the record. A poll that cannot reach the server
             has nothing useful to add to it. */
        });
    };
    setTimeout(poll, SCAFFOLD_POLL_MS);
  };

  const routeRef = useRef(route);
  routeRef.current = route;
  const handleMenuAction = (action: MenuAction) => {
      // Insert — bare, or naming one element — wants a buffer to write into,
      // which only the workspace has. Handled before the switch because the
      // element-carrying form is a whole family of actions, not one case.
      if (action === "insert" || insertTarget(action) !== null || action === "edit-element") {
        if (routeRef.current.name === "doc" || routeRef.current.name === "new") {
          window.dispatchEvent(new CustomEvent("hickory-doc-command", { detail: action }));
        } else {
          setNotice("Open a document first.");
        }
        return;
      }
      switch (action) {
        case "new":
          if (routeRef.current.name === "blank") {
            setNotice("Open a folder before creating a document.");
            return;
          }
          newDocument();
          return;
        case "new-project":
          if (routeRef.current.name === "blank") {
            setNotice("Open a folder before creating a project.");
            return;
          }
          setNewProject(true);
          return;
        case "files":
          if (routeRef.current.name === "blank") {
            setNotice("Open a folder to show Files.");
            return;
          }
          window.dispatchEvent(new CustomEvent("hickory-show-files"));
          break;
        case "show-agent":
          if (routeRef.current.name === "blank") {
            setNotice("Open a file or folder to use the agent.");
            return;
          }
          window.dispatchEvent(new CustomEvent("hickory-show-agent"));
          break;
        case "terminal":
        case "attention":
          // Terminals belong to the workspace, which owns the layout they
          // open into and the queue cursor ⌘J walks.
          if (routeRef.current.name === "doc" || routeRef.current.name === "new") {
            window.dispatchEvent(
              new CustomEvent("hickory-terminal-command", { detail: action }),
            );
          } else if (routeRef.current.name === "blank") {
            setNotice("Open a file or folder to use a terminal.");
          }
          return;
        case "settings":
          if (routeRef.current.name === "blank") {
            setNotice("Open a file or folder to use Settings.");
            return;
          }
          navigate("/settings");
          return;
        case "save":
        case "save-as":
        case "save-all":
        case "print":
          // The workspace routes the command to whichever document is
          // focused; it is mounted for both the doc and untitled routes.
          if (routeRef.current.name === "doc" || routeRef.current.name === "new") {
            window.dispatchEvent(new CustomEvent("hickory-doc-command", { detail: action }));
          } else {
            setNotice("Open a document first.");
          }
          return;
      }
  };
  const menuHandler = useRef(handleMenuAction); menuHandler.current = handleMenuAction;
  useEffect(() => onMenuAction(action => menuHandler.current(action)), []);
  useUiCommands([
    ...([['new', 'Create untitled document'], ['new-project', 'Open new project dialog'], ['files', 'Show workspace Files'], ['show-agent', 'Show workspace agent'], ['settings', 'Open settings']] as const).map(([action, label]) => ({
      key: `application-navigation/${action}`, command: {
        label, parent: 'application-navigation', subject: 'application-navigation', parameters: {}, prerequisites: ['An open workspace is required'],
        read: () => ({ revision: JSON.stringify([route, newProject]), enabled: route.name !== 'blank', reason: route.name === 'blank' ? 'Open a workspace first' : undefined }),
        execute: async () => { handleMenuAction(action); return { status: 'completed' as const, value: { dispatch: 'requested', result: 'Observe the actual route, pane or dialog' } }; },
      },
    })),
    { key: 'application-navigation/state', command: {
      label: 'Read application route and notice', parent: 'application-navigation', subject: 'application-navigation', parameters: {}, prerequisites: [],
      read: () => ({ revision: JSON.stringify([route, notice, newProject]), enabled: true }),
      execute: async () => ({ status: 'completed', value: JSON.parse(JSON.stringify({ route, notice, new_project_dialog: newProject, retained_workspace: workspace.current?.route ?? null })) }),
    } },
  ]);
  useEffect(() => {
    const onOpenPath = (event: Event) => {
      const path = (event as CustomEvent<string>).detail;
      if (typeof path !== "string" || !path) return;
      if (routeRef.current.name === "new" || routeRef.current.name === "doc") return;
      navigate(`/file/${encodeURIComponent(path)}`);
    };
    window.addEventListener("hickory-open-path", onOpenPath);
    return () => window.removeEventListener("hickory-open-path", onOpenPath);
  }, []);
  useEffect(() => {
    const onClose = () => {
      const current = routeRef.current;
      if (workspace.current && current.name !== "doc" && current.name !== "new" && current.name !== "scratchpad") {
        pendingWorkspaceClose.current = true;
        navigate(workspace.current.hash.replace(/^#/, ""));
        return;
      }
      if (current.name === "doc" || current.name === "new" || current.name === "scratchpad") {
        window.dispatchEvent(new Event("hickory-workspace-close-request"));
      } else {
        void api.closeWindow();
      }
    };
    window.addEventListener("hickory-window-close-request", onClose);
    return () => window.removeEventListener("hickory-window-close-request", onClose);
  }, []);
  useEffect(() => {
    if (notice === null) return;
    const timer = setTimeout(() => setNotice(null), 2500);
    return () => clearTimeout(timer);
  }, [notice]);

  return (
    <div className="app">
      <main className="content">
        {workspace.current && <div data-workspace hidden={!inWorkspace} style={{ display: inWorkspace ? "contents" : "none" }}>
          <UiActionsVisibility visible={inWorkspace}><WorkspaceView route={workspace.current.route} /></UiActionsVisibility>
        </div>}
        <Suspense fallback={<div role="status">Opening…</div>}>
          {inWorkspace ? null : route.name === "lineage" ? (
            <LineageView projectId={route.id} />
          ) : route.name === "settings" ? (
            <SettingsView onBack={() => { navigate(workspace.current?.hash.replace(/^#/, "") ?? "/"); window.dispatchEvent(new Event(SETTINGS_CLOSED)); }} />
          ) : route.name === "blank" ? (
            <BlankWelcome />
          ) : (
            <BlankWelcome />
          )}
        </Suspense>
      </main>
      {notice && (
        <div className="menu-notice" role="status">
          {notice}
        </div>
      )}
      {newProject && (
        <Suspense fallback={<div role="status">Opening New Project…</div>}>
          <NewProjectDialog onStarted={watchScaffold} onClose={() => setNewProject(false)} />
        </Suspense>
      )}
      {/* One tooltip for the whole app; every `data-tip` in it lands here. */}
      <TooltipLayer />
      <StartupNotice />
    </div>
  );
}
