import { useEffect, useState } from "react";
import { api } from "../api/client";
import type { DocSummary, Project } from "../api/types";
import { navigate } from "../router";

export function ProjectsView({ projectId }: { projectId?: string }) {
  const [projects, setProjects] = useState<Project[] | null>(null);
  const [docs, setDocs] = useState<DocSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [newName, setNewName] = useState("");

  useEffect(() => {
    api.projects().then(setProjects, (e) => setError(String(e.message ?? e)));
  }, []);

  useEffect(() => {
    setDocs(null);
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
  };

  const selected = projects?.find((p) => p.id === projectId);

  return (
    <div className="projects-page">
      <section className="project-list">
        <h1>Projects</h1>
        {error && <p className="error">{error}</p>}
        {projects === null ? (
          <p className="muted">Loading…</p>
        ) : (
          <ul>
            {projects.map((p) => (
              <li key={p.id}>
                <button
                  className={`row-btn${p.id === projectId ? " selected" : ""}`}
                  onClick={() => navigate(`/projects/${p.id}`)}
                >
                  <span className="row-name">{p.name}</span>
                  <span className={`chip chip-${p.visibility}`}>{p.visibility}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
        <form className="inline-form" onSubmit={createProject}>
          <input
            placeholder="New project name"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
          />
          <button className="btn">Create</button>
        </form>
      </section>
      <section className="doc-list">
        {selected ? (
          <>
            <h2>{selected.name}</h2>
            {docs === null ? (
              <p className="muted">Loading…</p>
            ) : docs.length === 0 ? (
              <p className="muted">No documents yet.</p>
            ) : (
              <ul>
                {docs.map((d) => (
                  <li key={d.id}>
                    <button className="row-btn" onClick={() => navigate(`/docs/${d.id}`)}>
                      <span className="row-name mono">{d.path}</span>
                      <span className="muted">
                        {new Date(d.updated_at).toLocaleDateString()}
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </>
        ) : (
          <p className="muted">Select a project to see its documents.</p>
        )}
      </section>
    </div>
  );
}
