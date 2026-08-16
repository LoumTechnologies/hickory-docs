// What the app does when it has no document open yet.
//
// There is no project, no list, and nothing to select. The app was started on
// a file, or on a folder, or on nothing — and on nothing it does what an
// editor does: gives you a blank document. See
// docs/specs/freeform/shell-layouts.md.

import { useEffect, useState } from "react";

import { api } from "../api/client";
import { navigate } from "../router";
import type { DocSummary } from "../api/types";

/** A document that does not exist yet, so there is something to type into. */
const BLANK = `<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="untitled.md">
# Untitled

`;

export function StartView() {
  const [docs, setDocs] = useState<DocSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);

  useEffect(() => {
    let live = true;
    void (async () => {
      try {
        const projects = await api.projects();
        const first = projects[0];
        const found = first ? await api.projectDocs(first.id) : [];
        if (!live) return;
        setDocs(found);
        // One document is what "opened on a file" looks like from here, and
        // the file is what the person asked for.
        if (found.length === 1) navigate(`/docs/${found[0].id}`);
      } catch (e) {
        if (live) setError(e instanceof Error ? e.message : String(e));
      }
    })();
    return () => {
      live = false;
    };
  }, []);

  const startBlank = async () => {
    setCreating(true);
    try {
      const projects = await api.projects();
      const project = projects[0];
      if (!project) throw new Error("no folder is open");
      const name = `untitled-${new Date().toISOString().slice(0, 10)}.hick`;
      const created = await api.createDoc(project.id, name, BLANK);
      navigate(`/docs/${created.id}`);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setCreating(false);
    }
  };

  if (error) {
    return (
      <div className="start">
        <p className="error">{error}</p>
      </div>
    );
  }

  return (
    <div className="start">
      <div className="start__inner">
        <h1 className="start__title">Hickory Docs</h1>
        <p className="start__lede">
          A document that writes files, runs them, and keeps what it says about
          them true.
        </p>

        <button className="btn btn-primary start__new" onClick={() => void startBlank()} disabled={creating}>
          {creating ? "Creating…" : "New document"}
        </button>

        {/* Whatever this folder already holds. Not a project list — a folder
            was opened, and these are the documents in it. */}
        {docs && docs.length > 0 && (
          <section className="start__docs">
            <h2>In this folder</h2>
            <ul>
              {docs.map((entry) => (
                <li key={entry.id}>
                  <button className="start__doc mono" onClick={() => navigate(`/docs/${entry.id}`)}>
                    {entry.path}
                  </button>
                </li>
              ))}
            </ul>
          </section>
        )}
        {docs && docs.length === 0 && (
          <p className="muted start__empty">
            This folder has no documents yet. A new one starts empty and saves
            itself as you type.
          </p>
        )}
      </div>
    </div>
  );
}
