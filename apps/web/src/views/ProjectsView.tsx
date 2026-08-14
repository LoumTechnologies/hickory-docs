import { useEffect, useState } from "react";
import { api } from "../api/client";
import type { DocSummary, Project } from "../api/types";
import { navigate } from "../router";
import { ProjectDocTree } from "../components/ProjectDocTree";
import { TreeMark } from "../components/icons";

const STARTER_SOURCE = `<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">

</h:doc>
`;

export function ProjectsView({ projectId }: { projectId?: string }) {
  const [projects, setProjects] = useState<Project[] | null>(null);
  const [docs, setDocs] = useState<DocSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [showNewProject, setShowNewProject] = useState(false);
  const [newName, setNewName] = useState("");
  const [newDocPrefix, setNewDocPrefix] = useState<string | null>(null);
  const [newDocName, setNewDocName] = useState("");

  useEffect(() => {
    api.projects().then(setProjects, (e) => setError(String(e.message ?? e)));
  }, []);

  useEffect(() => {
    setDocs(null);
    setNewDocPrefix(null);
    if (projectId) {
      api.projectDocs(projectId).then(setDocs, (e) => setError(String(e.message ?? e)));
    }
  }, [projectId]);

  const createProject = async (e: React.FormEvent) => {
    e.preventDefault();
    const name = newName.trim();
    if (!name) return;
    const project = await api.createProject(name, "private");
    setProjects((prev) => [...(prev ?? []), project]);
    setNewName("");
    setShowNewProject(false);
    navigate(`/projects/${project.id}`);
  };

  const createDoc = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!projectId || newDocPrefix === null) return;
    const leaf = newDocName.trim();
    if (!leaf) return;
    const name = leaf.includes(".") ? leaf : `${leaf}.hick`;
    const path = newDocPrefix ? `${newDocPrefix}/${name}` : name;
    try {
      const doc = await api.createDoc(projectId, path, STARTER_SOURCE);
      setDocs((prev) => [...(prev ?? []), { id: doc.id, path: doc.path, updated_at: doc.updated_at }]);
      setNewDocPrefix(null);
      setNewDocName("");
      navigate(`/docs/${doc.id}`);
    } catch (err: any) {
      setError(String(err.message ?? err));
    }
  };

  const selected = projects?.find((p) => p.id === projectId);

  return (
    <div className="drive-page">
      <aside className="drive-rail">
        <div className="drive-rail-header">
          <TreeMark size={16} />
          <span>My Projects</span>
        </div>

        <button
          type="button"
          className="drive-new-btn"
          onClick={() => setShowNewProject((v) => !v)}
        >
          + New
        </button>
        {showNewProject && (
          <form className="inline-form drive-new-form" onSubmit={createProject}>
            <input
              autoFocus
              placeholder="Project name"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
            />
            <button className="btn">Create</button>
          </form>
        )}

        {error && <p className="error">{error}</p>}
        {projects === null ? (
          <p className="muted drive-rail-loading">Loading…</p>
        ) : projects.length === 0 ? (
          <p className="muted drive-rail-loading">No projects yet.</p>
        ) : (
          <ul className="drive-project-list">
            {projects.map((p) => (
              <li key={p.id}>
                <button
                  className={`drive-project-row${p.id === projectId ? " selected" : ""}`}
                  onClick={() => navigate(`/projects/${p.id}`)}
                >
                  <span className="drive-project-icon" aria-hidden="true">
                    ▸
                  </span>
                  <span className="drive-project-name">{p.name}</span>
                  <span
                    className={`drive-visibility-dot drive-visibility-${p.visibility}`}
                    title={p.visibility}
                  />
                </button>
              </li>
            ))}
          </ul>
        )}
      </aside>

      <main className="drive-main">
        {selected ? (
          <>
            <div className="drive-crumbline">
              <p className="drive-breadcrumb">
                My Projects <span className="drive-breadcrumb-sep">/</span> {selected.name}
              </p>
              {/* The chain crosses documents, so this is project-scoped: a
                  doc-scoped view could only ever show one link of it. */}
              <button
                className="btn btn-quiet"
                onClick={() => navigate(`/projects/${selected.id}/lineage`)}
                title="See every stage side by side, with the links between them"
              >
                Lineage
              </button>
            </div>
            {docs === null ? (
              <p className="muted">Loading…</p>
            ) : (
              <ProjectDocTree
                docs={docs}
                activeDocId={null}
                onSelect={(id) => navigate(`/docs/${id}`)}
                onCreate={(prefix) => {
                  setNewDocPrefix(prefix);
                  setNewDocName("");
                }}
              />
            )}
            {newDocPrefix !== null && (
              <form className="inline-form" onSubmit={createDoc}>
                <input
                  autoFocus
                  placeholder={newDocPrefix ? `New document in ${newDocPrefix}/` : "New document name"}
                  value={newDocName}
                  onChange={(e) => setNewDocName(e.target.value)}
                />
                <button className="btn">Create</button>
                <button type="button" className="btn-link" onClick={() => setNewDocPrefix(null)}>
                  Cancel
                </button>
              </form>
            )}
          </>
        ) : (
          <div className="drive-empty">
            <TreeMark size={28} />
            <p>Select a project to see its documents.</p>
          </div>
        )}
      </main>
    </div>
  );
}
