import { useEffect, useRef, useState } from "react";
import { navigate, redirect, useRoute } from "./router";
import { LineageView } from "./views/LineageView";
import { SettingsView } from "./views/SettingsView";
import { WorkspaceView } from "./views/WorkspaceView";
import { api } from "./api/client";
import { insertTarget, onMenuAction } from "./lib/menuBridge";
import { landingTarget } from "./lib/newDoc";
import { TooltipLayer } from "./components/TooltipLayer";
import { NewProjectDialog } from "./components/NewProjectDialog";
import { FILES_CHANGED_EVENT } from "./shell/FolderTreePane";

/// The desktop app's shell.
///
/// There is no authentication, no landing page, and no account menu. This UI
/// only ever talks to the local server running in the same process, which
/// answers one person: the one whose machine it is. The marketing site is a
/// separate build — see `docs/specs/freeform/local-only.md`.
export function App() {
  const route = useRoute();

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
  const routeRef = useRef(route);
  routeRef.current = route;
  useEffect(() => {
    return onMenuAction((action) => {
      // Insert — bare, or naming one element — wants a buffer to write into,
      // which only the workspace has. Handled before the switch because the
      // element-carrying form is a whole family of actions, not one case.
      if (action === "insert" || insertTarget(action) !== null) {
        if (routeRef.current.name === "doc" || routeRef.current.name === "new") {
          window.dispatchEvent(new CustomEvent("hickory-doc-command", { detail: action }));
        } else {
          setNotice("Open a document first.");
        }
        return;
      }
      switch (action) {
        case "new":
          navigate("/new");
          return;
        case "new-project":
          setNewProject(true);
          return;
        case "files":
          window.dispatchEvent(new CustomEvent("hickory-show-files"));
          break;
        case "terminal":
        case "attention":
          // Terminals belong to the workspace, which owns the layout they
          // open into and the queue cursor ⌘J walks.
          if (routeRef.current.name === "doc" || routeRef.current.name === "new") {
            window.dispatchEvent(
              new CustomEvent("hickory-terminal-command", { detail: action }),
            );
          }
          return;
        case "settings":
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
    });
  }, []);
  useEffect(() => {
    if (notice === null) return;
    const timer = setTimeout(() => setNotice(null), 2500);
    return () => clearTimeout(timer);
  }, [notice]);

  return (
    <div className="app">
      <main className="content">
        {route.name === "doc" ||
        route.name === "new" ||
        route.name === "scratchpad" ? (
          // ONE workspace for every document-shaped route. It owns the tile
          // layout and stays mounted as `#/docs/<id>` changes, which is what
          // lets several documents be open at once: navigation asks it to
          // ensure a tab, never to rebuild the world.
          <WorkspaceView route={route} />
        ) : route.name === "lineage" ? (
          <LineageView projectId={route.id} />
        ) : route.name === "settings" ? (
          <SettingsView />
        ) : (
          <Landing />
        )}
      </main>
      {notice && (
        <div className="menu-notice" role="status">
          {notice}
        </div>
      )}
      {newProject && (
        <NewProjectDialog
          onCreated={(created) => {
            // The tree has a new document and, when the scaffold ran, a new
            // directory of files beside it.
            window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
            navigate(`/docs/${created.id}`);
            setNotice(
              created.ingested
                ? `${created.path} — ${created.ingested.files.length} file(s) ingested` +
                  (created.ingested.skipped.length > 0
                    ? `, ${created.ingested.skipped.length} skipped by .gitignore`
                    : "")
                : created.note
                  ? `${created.path} written; the scaffold did not run — ${created.note}`
                  : `${created.path} written`,
            );
          }}
          onClose={() => setNewProject(false)}
        />
      )}
      {/* One tooltip for the whole app; every `data-tip` in it lands here. */}
      <TooltipLayer />
    </div>
  );
}

/// The front door. The app opens like an editor: in a document, always — the
/// most recently updated one, or a fresh untitled buffer when the folder has
/// none. A redirect rather than a page, so Back never revisits the decision.
function Landing() {
  useEffect(() => {
    let live = true;
    void (async () => {
      try {
        const projects = await api.projects();
        const first = projects[0];
        const docs = first ? await api.projectDocs(first.id) : [];
        if (!live) return;
        const target = landingTarget(docs);
        redirect(target.kind === "doc" ? `/docs/${target.id}` : "/new");
      } catch {
        // A server that cannot list documents can still hold a buffer: the
        // untitled editor works before any file exists.
        if (live) redirect("/new");
      }
    })();
    return () => {
      live = false;
    };
  }, []);

  return (
    <div className="start">
      <p className="muted">Opening…</p>
    </div>
  );
}
