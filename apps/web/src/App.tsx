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
