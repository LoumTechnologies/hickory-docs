// The repository's history, read as a story.
//
// A LENS, not a document (docs/specs/freeform/lenses.md): the commits drawn
// top to bottom with the cards a document uses — the message as prose, a
// recipe-bearing commit as a cell whose output is its diff, the working tree
// as the last card. None of it exists on disk as text; there is no save
// path, no `.hick` extension, and no place in the folder tree.
//
// Oldest first, because a narrative reads forward and a document already
// does. The past folds by default and the view opens at the tail, the way a
// session folds the agent's work: a thousand commits is not a page anyone
// reads from the top.
//
// Read-only in this step. Nothing here rewords, reorders, replays or
// commits; those verbs come later and are gated by the publication floor,
// which is already drawn here as the line between records and drafts.
//
// Three provenances, kept apart on every card (three-provenances.md): the
// message is DECLARED and unverifiable; the diff is DERIVED, git's own; a
// recipe is a declared claim until a replay verifies it, so a recipe card is
// drawn as *unrecorded* — say "no evidence of drift", never "reproducible".

import { useCallback, useEffect, useMemo, useState } from "react";

import { api } from "../api/client";
import type { GitCommit, GitCommitDetail, GitLog, GitStatus } from "../api/types";
import { DiffView } from "../components/DiffView";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { when } from "./GitPane";

/** How many of the newest commits open unfolded. The rest are one fold. */
export const OPEN_TAIL = 12;

/** How many more the fold reveals per click. */
const FOLD_STEP = 25;

/**
 * The story's order: oldest first, so the newest commit is at the bottom
 * beside the working tree. The log arrives newest-first, as `git log` does.
 */
export function storyOrder(commits: readonly GitCommit[]): GitCommit[] {
  return [...commits].reverse();
}

/**
 * Where the publication floor falls in story order: the index of the first
 * draft, or null when nothing is a draft. Everything before it is a record
 * someone else may hold; everything from it on is still rewritable.
 */
export function floorIndex(commits: readonly GitCommit[]): number | null {
  const index = commits.findIndex((commit) => commit.draft);
  return index < 0 ? null : index;
}

/** One commit's card, fetched on expand: the diff and what was edited since. */
function CommitDetail({ sha, recipe }: { sha: string; recipe: boolean }) {
  const [detail, setDetail] = useState<GitCommitDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    api.gitCommitDetail(sha).then(
      (answer) => live && setDetail(answer),
      (e: unknown) => live && setError(e instanceof Error ? e.message : String(e)),
    );
    return () => {
      live = false;
    };
  }, [sha]);
  if (error) return <p className="error">{error}</p>;
  if (!detail) return <p className="muted">Reading the change…</p>;
  return (
    <div className="story-card__detail">
      {/* The fact the scaffold cell could never show: the output stays, AND
          it says which later commit changed it. Only worth saying on a
          recipe, where "what the command wrote" and "what is there now" are
          different questions. */}
      {recipe && detail.edited_since.length > 0 && (
        <p className="story-card__edited" role="status">
          Edited since:{" "}
          {detail.edited_since.map((edit, index) => (
            <span key={edit.path}>
              {index > 0 ? ", " : ""}
              <span className="mono">{edit.path}</span> in <span className="mono">{edit.short}</span>{" "}
              ({edit.subject})
            </span>
          ))}
        </p>
      )}
      {recipe && detail.edited_since.length === 0 && (
        <p className="story-card__edited muted" role="status">
          Not edited since — the tree still holds what the command wrote.
        </p>
      )}
      <DiffView path={`${detail.files.length} file${detail.files.length === 1 ? "" : "s"}`} diff={detail.diff} binary={false} staged={false} />
    </div>
  );
}

/** One commit as a card: prose for an ordinary one, a cell for a recipe. */
function CommitCard({ commit, expanded, onToggle }: { commit: GitCommit; expanded: boolean; onToggle: () => void }) {
  const recipe = commit.recipe ?? null;
  return (
    <article
      className={`story-card${recipe ? " story-card--recipe" : ""}${commit.draft ? " story-card--draft" : ""}`}
      aria-label={commit.subject}
      data-sha={commit.sha}
    >
      <header className="story-card__head">
        <h3 className="story-card__subject">{commit.subject}</h3>
        <span className="story-card__meta">
          <span className="mono">{commit.short}</span>
          <span>{commit.author}</span>
          <span>{when(commit.time)}</span>
          {commit.draft && (
            <span className="git-draft" data-tip="Above the publication floor: still rewritable, because nobody else can be holding it">
              draft
            </span>
          )}
          {commit.refs.map((ref) => (
            <span key={ref} className="git-ref mono">
              {ref}
            </span>
          ))}
        </span>
      </header>
      {commit.body && !recipe && <p className="story-card__body">{commit.body}</p>}
      {recipe && (
        <div className="story-cell" role="group" aria-label="Recipe">
          <div className="story-cell__bar">
            <span className="story-cell__verb">recipe</span>
            {recipe.image && <span className="mono story-cell__image">{recipe.image}</span>}
            {/* Two chips, two questions, kept apart. First: is this commit's
                tree exactly what its trailer says the scaffolder wrote?
                Derived — git checked it, no replay — and it decides whether
                the commit can be upgraded at all: an edit made before the
                commit is fused into it and cannot be separated. Second: has
                anything replayed the recipe? Nothing has, so the answer is
                "unrecorded", however confident the prose. */}
            {recipe.output_matches === true && (
              <span
                className="story-chip story-chip--matches"
                data-tip="git holds exactly the tree the trailer recorded at this path: nothing was edited before it was committed, so replay and rebase can upgrade it"
              >
                matches its recorded output · upgradeable
              </span>
            )}
            {recipe.output_matches === false && (
              <span
                className="story-chip story-chip--edited"
                role="status"
                data-tip="The tree at the recorded path is not the one the trailer names: something was edited before this was committed, or the trailer was written by hand. The edits cannot be separated from the scaffold, so this cannot be upgraded by replay."
              >
                edited before it was committed · not upgradeable
              </span>
            )}
            {recipe.output_matches === undefined && (
              <span
                className="story-chip"
                data-tip="The trailer names no path to check, so whether this tree is the scaffolder's cannot be told without a replay."
              >
                no recorded output
              </span>
            )}
            <span
              className="story-chip story-chip--unrecorded"
              data-tip="A declared claim in the commit's own words. Replay is what would verify it; nothing has."
            >
              unrecorded · no evidence of drift
            </span>
          </div>
          <pre className="story-cell__command mono">{recipe.command}</pre>
          {commit.body && (
            <p className="story-card__body">{commit.body.split(/\n\n(?=Hick-)/)[0]}</p>
          )}
        </div>
      )}
      <button type="button" className="story-card__toggle btn" onClick={onToggle} aria-expanded={expanded}>
        {expanded ? "Hide changes" : `${commit.files.length} file${commit.files.length === 1 ? "" : "s"}, +${commit.added} −${commit.removed}`}
      </button>
      {expanded && <CommitDetail sha={commit.sha} recipe={recipe !== null} />}
    </article>
  );
}

/** The last card: the working tree, an unrecorded cell nothing has committed. */
function TailCard({ status }: { status: GitStatus | null }) {
  const dirty = (status?.staged ?? 0) + (status?.unstaged ?? 0) + (status?.untracked ?? 0);
  return (
    <article className="story-card story-card--tail" aria-label="Working tree">
      <header className="story-card__head">
        <h3 className="story-card__subject">Working tree</h3>
        <span className="story-card__meta">
          {status?.branch && <span className="git-ref mono">{status.branch}</span>}
        </span>
      </header>
      <p className="story-card__body muted">
        {dirty === 0
          ? "Nothing uncommitted. The story ends at the commit above."
          : `${dirty} change${dirty === 1 ? "" : "s"} not yet committed. This is the next card, once it is written.`}
      </p>
    </article>
  );
}

export function HistoryLens() {
  const [log, setLog] = useState<GitLog | null>(null);
  const [status, setStatus] = useState<GitStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [open, setOpen] = useState<ReadonlySet<string>>(new Set());
  // How many commits before the open tail are unfolded. Grows by clicks.
  const [unfolded, setUnfolded] = useState(0);
  const [tick, setTick] = useState(0);
  const refresh = useCallback(() => setTick((n) => n + 1), []);

  useEffect(() => {
    let live = true;
    api.gitLog(1000).then(
      (answer) => live && setLog(answer),
      (e: unknown) => live && setError(e instanceof Error ? e.message : String(e)),
    );
    api.gitStatus().then(
      (answer) => live && setStatus(answer),
      () => {},
    );
    return () => {
      live = false;
    };
  }, [tick]);

  useEffect(() => {
    window.addEventListener(FILES_CHANGED_EVENT, refresh);
    window.addEventListener("focus", refresh);
    return () => {
      window.removeEventListener(FILES_CHANGED_EVENT, refresh);
      window.removeEventListener("focus", refresh);
    };
  }, [refresh]);

  const story = useMemo(() => storyOrder(log?.commits ?? []), [log]);
  const floorAt = useMemo(() => floorIndex(story), [story]);
  const toggle = (sha: string) =>
    setOpen((current) => {
      const next = new Set(current);
      if (next.has(sha)) next.delete(sha);
      else next.add(sha);
      return next;
    });

  if (error) return <p className="error git-pane__error">{error}</p>;
  if (!log) return <p className="muted git-pane__loading">Reading history…</p>;
  if (!log.repository) {
    return (
      <div className="story story--none">
        <p className="muted">This folder is not a git repository, so it has no history to read as a story.</p>
      </div>
    );
  }

  const shown = Math.min(story.length, OPEN_TAIL + unfolded);
  const folded = story.length - shown;
  const visible = story.slice(folded);

  return (
    <div className="story" role="region" aria-label="History, as a story">
      <p className="story__banner muted" role="note">
        A lens over this repository’s history, oldest first. It exists on disk nowhere and cannot be saved; every card is a commit, read-only here.
      </p>
      {folded > 0 && (
        <button
          type="button"
          className="btn story__fold"
          onClick={() => setUnfolded((n) => n + FOLD_STEP)}
          aria-label={`${folded} earlier commits, folded`}
        >
          ⋯ {folded} earlier commit{folded === 1 ? "" : "s"}
        </button>
      )}
      {visible.map((commit, index) => {
        const absolute = folded + index;
        const marker =
          floorAt !== null && absolute === floorAt ? (
            <p key={`floor-${commit.sha}`} className="story__floor" role="separator" aria-label="Publication floor">
              <span className="git-floor__count">Publication floor</span>{" "}
              {log.floor?.summary ?? "Below this line, commits are records someone else may hold. Above it they are drafts."}
            </p>
          ) : null;
        return (
          <div key={commit.sha}>
            {marker}
            <CommitCard commit={commit} expanded={open.has(commit.sha)} onToggle={() => toggle(commit.sha)} />
          </div>
        );
      })}
      <TailCard status={status} />
    </div>
  );
}
