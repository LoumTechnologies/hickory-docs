import { useEffect, useState } from "react";

import { api } from "./api/client";
import { navigate, useRoute } from "./router";
import { ProjectsView } from "./views/ProjectsView";
import { DocumentView } from "./views/DocumentView";
import { LineageView } from "./views/LineageView";
import { TreeMark } from "./components/icons";

/// The desktop app's shell.
///
/// There is no authentication, no landing page, and no account menu. This UI
/// only ever talks to the local server running in the same process, which
/// answers one person: the one whose machine it is. The marketing site is a
/// separate build — see `docs/specs/freeform/local-only.md`.
export function App() {
  const route = useRoute();
  useOpenTheOnlyDocument(route.name);

  return (
    <div className="app">
      <nav className="topnav">
        <button className="wordmark" onClick={() => navigate("/projects")}>
          <TreeMark size={17} />
          Hickory Docs
        </button>
      </nav>
      <main className="content">
        {route.name === "doc" ? (
          <DocumentView docId={route.id} />
        ) : route.name === "lineage" ? (
          <LineageView projectId={route.id} />
        ) : route.name === "project" ? (
          <ProjectsView projectId={route.id} />
        ) : (
          <ProjectsView />
        )}
      </main>
    </div>
  );
}


/**
 * A folder with one document in it opens that document.
 *
 * The app can be pointed at a single file, and when it has been, a list of
 * one thing standing between a person and their file is ceremony: Notepad
 * shows you the file. A folder with several documents still lists them,
 * because then the list is the answer to a real question.
 *
 * Only from the list itself, and only once — navigating back is a person
 * saying they want the list, and bouncing them out of it would be the app
 * arguing.
 */
function useOpenTheOnlyDocument(routeName: string) {
  const [checked, setChecked] = useState(false);
  useEffect(() => {
    // Both names land on the list: an empty hash parses as `landing`, which
    // this app has no page for, and renders the projects list instead.
    if (checked || (routeName !== "projects" && routeName !== "landing")) return;
    let live = true;
    void (async () => {
      try {
        const projects = await api.projects();
        const first = projects[0];
        if (!first) return;
        const docs = await api.projectDocs(first.id);
        if (live && docs.length === 1) navigate(`/docs/${docs[0].id}`);
      } catch {
        // A list that cannot load is the list's problem to report.
      } finally {
        if (live) setChecked(true);
      }
    })();
    return () => {
      live = false;
    };
  }, [checked, routeName]);
}
