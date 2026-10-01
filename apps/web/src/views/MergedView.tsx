// One tab, several worktrees, and the merge already done.
//
// The file as it exists in several places at once: regions every source
// agrees on appear ONCE, regions that differ appear as variants in the shape
// `<hick:when>` already has. None of that text exists on disk — there is no
// file containing those conditionals, nothing to commit, nothing to merge.
//
// **It is a lens, not a document.** No save path, no `.md` extension, no
// place in the folder tree. The same category as a diff view, and nobody
// mistakes `git diff` output for a source artifact. Getting this wrong
// reverses the product's central claim: a document is the source of its
// files, and a synthetic document assembled FROM files points the other way.
//
// The sources are PEERS (decided 2026-08-23): shared means agreed by ALL of
// them, and there is no order, because the view removes the question.
//
// Read-only in this step. Aligning N sources so shared regions genuinely
// correspond is the main technical risk in the whole idea, and a bad
// alignment routes an edit silently into the wrong file — so nothing writes
// through it until the alignment is proved.
//
// docs/specs/freeform/the-merged-view.md

import { useEffect, useMemo, useState } from "react";

import { api } from "../api/client";
import type { MergedViewResponse, WorktreeInfo } from "../api/types";

export function MergedView({ path }: { path: string }) {
  const [worktrees, setWorktrees] = useState<WorktreeInfo[] | null>(null);
  const [chosen, setChosen] = useState<ReadonlySet<string> | null>(null);
  const [view, setView] = useState<MergedViewResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    api.worktrees().then(
      (answer) => live && setWorktrees(answer.worktrees),
      () => live && setWorktrees([]),
    );
    return () => {
      live = false;
    };
  }, []);

  const targets = useMemo(
    () => (chosen ? [...chosen] : undefined),
    [chosen],
  );

  useEffect(() => {
    let live = true;
    setError(null);
    api.merged(path, targets).then(
      (answer) => live && setView(answer),
      (e: unknown) => live && setError(e instanceof Error ? e.message : String(e)),
    );
    return () => {
      live = false;
    };
  }, [path, targets]);

  if (error) return <p className="error">{error}</p>;
  if (!worktrees || !view) return <p className="muted">Reading the worktrees…</p>;

  if (!view.repository || worktrees.length === 0) {
    return (
      <div className="merged-view merged-view--none">
        <p className="muted">
          This folder is not a git repository with worktrees, so there is
          nothing to merge a view over.
        </p>
        <p className="muted">
          A merged view shows one file as it exists in several worktrees at
          once. “Across branches” means across worktrees — a branch that is
          not checked out cannot be written to without going behind the
          working tree.
        </p>
      </div>
    );
  }

  if (worktrees.length < 2) {
    return (
      <div className="merged-view merged-view--none">
        <p className="muted">
          This repository has one worktree, so a merged view over it would just
          be the file.
        </p>
        <p className="muted">
          Add one with <span className="mono">git worktree add</span>, and this
          shows what the two agree on and where they differ.
        </p>
      </div>
    );
  }

  const shown = view.sources.map((s) => s.name);

  return (
    <div className="merged-view">
      <header className="merged-view__head">
        <span className="merged-view__path mono">{path}</span>
        {/* Said at the top rather than left to be discovered. */}
        <span className="merged-view__lens" data-tip="A lens, not a document: this text exists on disk nowhere, there is nothing to save, and nothing to commit. Editing through it is the next step, once the alignment is proved.">
          lens · read-only
        </span>
      </header>

      <div className="merged-view__sources">
        {worktrees.map((tree) => {
          const on = chosen ? chosen.has(tree.name) : true;
          return (
            <label key={tree.name} className="merged-view__source">
              <input
                type="checkbox"
                checked={on}
                onChange={() => {
                  const next = new Set(chosen ?? worktrees.map((w) => w.name));
                  if (!next.delete(tree.name)) next.add(tree.name);
                  setChosen(next);
                }}
              />
              <span className="mono">{tree.name}</span>
              {tree.branch && <span className="muted"> · {tree.branch}</span>}
              {tree.current && <span className="muted"> · here</span>}
            </label>
          );
        })}
      </div>

      {view.missing.length > 0 && (
        <p className="lineage-note" role="status">
          {view.missing.join(", ")} {view.missing.length === 1 ? "does" : "do"} not
          have this file at all — which is an answer, not an omission.
        </p>
      )}

      <p className="merged-view__summary muted">
        {view.shared_lines ?? 0} line{(view.shared_lines ?? 0) === 1 ? "" : "s"} agreed by
        all {shown.length} source{shown.length === 1 ? "" : "s"}
        {(view.variants ?? 0) > 0
          ? `, ${view.variants} place${view.variants === 1 ? "" : "s"} where they differ.`
          : ". They agree everywhere."}{" "}
        A shared region is agreed by construction: it never diverged, so it
        cannot conflict later.
      </p>

      <div className="merged-view__body">
        {view.regions.map((region, i) =>
          region.kind === "shared" ? (
            <pre key={i} className="merged-region merged-region--shared">
              {region.text}
            </pre>
          ) : (
            <div key={i} className="merged-region merged-region--variant">
              {Object.entries(region.by_source).map(([name, text]) => (
                <div key={name} className="merged-variant">
                  <span className="merged-variant__label mono">{name}</span>
                  {text ? (
                    <pre className="merged-variant__text">{text}</pre>
                  ) : (
                    <p className="muted merged-variant__empty">nothing here</p>
                  )}
                </div>
              ))}
            </div>
          ),
        )}
      </div>
    </div>
  );
}
