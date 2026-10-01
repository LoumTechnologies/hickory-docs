// The repository's history, read as a story — and, above the floor, edited.
//
// A LENS, not a document (docs/specs/freeform/lenses.md): the commits drawn
// top to bottom with the cards a document uses — the message as prose, a
// recipe-bearing commit as a cell whose output is its diff, the working tree
// as the last card. None of it exists on disk as text; there is no save
// path, no `.md` extension, and no place in the folder tree.
//
// Oldest first, because a narrative reads forward and a document already
// does. The past folds by default and the view opens at the tail, the way a
// session folds the agent's work: a thousand commits is not a page anyone
// reads from the top.
//
// The verbs, each one git operation run as itself on the server, with git's
// own words when it refuses:
//  * Replay, on a recipe card whose tree matches its trailer: a NEW commit,
//    never a rewrite — rebase above the floor, merge below.
//  * The tail: prose typed there is the next commit's message; a command
//    typed there runs in a clean worktree and its output is committed with
//    the recipe.
//  * Reword, move, drop on a DRAFT card: a rebase above the floor. Below
//    it, cards are records and carry no verbs at all.
//
// Three provenances, kept apart on every card (three-provenances.md): the
// message is DECLARED and unverifiable; the diff is DERIVED, git's own; a
// recipe is a declared claim until a replay verifies it, and a replay's
// "same"/"differs" is EVIDENCE.

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

/** The prose of a message: the body without its trailer paragraph. */
export function proseOf(body: string): string {
  return body.split(/\n\n(?=Hick-)/)[0].trim();
}

type Verbs = {
  /** A verb ran; the story re-reads itself. */
  refresh: () => void;
  /** What a verb said — a result, a refusal, or a stopped join, in git's words. */
  say: (text: string | null) => void;
};

/** Run a verb, tell the story what it said, and re-read. */
function runVerb(
  verbs: Verbs,
  setBusy: (busy: boolean) => void,
  work: Promise<string | null>,
  then?: () => void,
) {
  setBusy(true);
  verbs.say(null);
  work
    .then((said) => {
      then?.();
      verbs.say(said);
    })
    .catch((e: unknown) => verbs.say(e instanceof Error ? e.message : String(e)))
    .finally(() => {
      setBusy(false);
      verbs.refresh();
    });
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
      <DiffView
        path={`${detail.files.length} file${detail.files.length === 1 ? "" : "s"}`}
        diff={detail.diff}
        binary={false}
        staged={false}
      />
    </div>
  );
}

/** The recipe cell inside a card: the command, the chips, and Replay. */
function RecipeCell({
  commit,
  verbs,
  busy,
  setBusy,
}: {
  commit: GitCommit;
  verbs: Verbs;
  busy: boolean;
  setBusy: (busy: boolean) => void;
}) {
  const recipe = commit.recipe!;
  const replay = () =>
    runVerb(
      verbs,
      setBusy,
      api.gitReplay(commit.sha).then((result) =>
        result.same
          ? `Replayed ${commit.short} as ${result.short}: the same tree, so nothing about it changed. Joined by ${result.moved}.`
          : `Replayed ${commit.short} as ${result.short}: the output differs, and its diff says how. Joined by ${result.moved}.`,
      ),
    );

  return (
    <div className="story-cell" role="group" aria-label="Recipe">
      <div className="story-cell__bar">
        <span className="story-cell__verb">recipe</span>
        {recipe.image && <span className="mono story-cell__image">{recipe.image}</span>}
        {/* Two questions, kept apart. First: is this commit's tree exactly
            what its trailer says the command wrote? Derived — git checked
            it — and it decides whether the commit can be upgraded at all.
            Second: has a replay produced evidence? */}
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
            data-tip="The trailer names no path to check, so whether this tree is the command's cannot be told without a replay."
          >
            no recorded output
          </span>
        )}
        {recipe.replay_of ? (
          <span
            className={`story-chip ${recipe.replay_same ? "story-chip--same" : "story-chip--differs"}`}
            data-tip={`A replay of ${recipe.replay_of.slice(0, 7)}: the recipe was run again, and this is what it produced. Evidence, not a claim.`}
          >
            {recipe.replay_same
              ? `replayed · same as ${recipe.replay_of.slice(0, 7)}`
              : `replayed · differs from ${recipe.replay_of.slice(0, 7)}`}
          </span>
        ) : (
          <span
            className="story-chip story-chip--unrecorded"
            data-tip="A declared claim in the commit's own words. Replay is what would verify it; nothing has."
          >
            unrecorded · no evidence of drift
          </span>
        )}
      </div>
      <pre className="story-cell__command mono">{recipe.command}</pre>
      {commit.body && proseOf(commit.body) && <p className="story-card__body">{proseOf(commit.body)}</p>}
      {recipe.output_matches === true && (
        <div className="story-cell__verbs">
          <button
            type="button"
            className="btn"
            disabled={busy}
            onClick={replay}
            data-tip="Run this recipe again and commit what it makes as a NEW commit — never a rewrite of this one. Above the publication floor the commits after it are rebased onto the result; below it, the result is merged. Needs a clean working tree."
          >
            {busy ? "Replaying…" : "Replay"}
          </button>
        </div>
      )}
    </div>
  );
}

/** The verbs a draft card carries: reword, move, drop. Records carry none. */
function DraftVerbs({
  commit,
  first,
  last,
  verbs,
  busy,
  setBusy,
  onReword,
}: {
  commit: GitCommit;
  first: boolean;
  last: boolean;
  verbs: Verbs;
  busy: boolean;
  setBusy: (busy: boolean) => void;
  onReword: () => void;
}) {
  return (
    <span className="story-card__verbs" role="group" aria-label="Edit this draft">
      <button
        type="button"
        className="btn"
        disabled={busy}
        onClick={onReword}
        data-tip="Change this commit's message — a rebase, allowed because this is a draft"
      >
        Reword
      </button>
      <button
        type="button"
        className="btn"
        disabled={busy || first}
        onClick={() => runVerb(verbs, setBusy, api.gitMove(commit.sha, "earlier").then(() => null))}
        data-tip="Swap this commit with the one before it"
        aria-label="Move earlier"
      >
        ↑
      </button>
      <button
        type="button"
        className="btn"
        disabled={busy || last}
        onClick={() => runVerb(verbs, setBusy, api.gitMove(commit.sha, "later").then(() => null))}
        data-tip="Swap this commit with the one after it"
        aria-label="Move later"
      >
        ↓
      </button>
      <button
        type="button"
        className="btn"
        disabled={busy}
        onClick={() => runVerb(verbs, setBusy, api.gitDrop(commit.sha).then(() => null))}
        data-tip="Remove this commit from the history. Its changes go with it; the commits after it are rebased over the gap."
      >
        Drop
      </button>
    </span>
  );
}

/** One commit as a card: prose for an ordinary one, a cell for a recipe. */
function CommitCard({
  commit,
  expanded,
  onToggle,
  verbs,
  draftPosition,
}: {
  commit: GitCommit;
  expanded: boolean;
  onToggle: () => void;
  verbs: Verbs;
  /** Where among the drafts this card sits, when it is one. Null for a record. */
  draftPosition: { first: boolean; last: boolean } | null;
}) {
  const recipe = commit.recipe ?? null;
  const [busy, setBusy] = useState(false);
  const [rewording, setRewording] = useState<string | null>(null);
  const saveReword = () => {
    if (rewording === null) return;
    runVerb(verbs, setBusy, api.gitReword(commit.sha, rewording).then(() => null), () =>
      setRewording(null),
    );
  };
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
            <span
              className="git-draft"
              data-tip="Above the publication floor: still rewritable, because nobody else can be holding it"
            >
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
      {rewording !== null ? (
        <div className="story-card__reword">
          <textarea
            className="story-card__textarea mono"
            aria-label="New message"
            value={rewording}
            rows={Math.max(3, rewording.split("\n").length + 1)}
            onChange={(event) => setRewording(event.target.value)}
          />
          <span className="story-card__verbs">
            <button type="button" className="btn btn-primary" disabled={busy} onClick={saveReword}>
              {busy ? "Rewording…" : "Save"}
            </button>
            <button type="button" className="btn" disabled={busy} onClick={() => setRewording(null)}>
              Cancel
            </button>
          </span>
        </div>
      ) : (
        <>
          {commit.body && !recipe && <p className="story-card__body">{commit.body}</p>}
          {recipe && <RecipeCell commit={commit} verbs={verbs} busy={busy} setBusy={setBusy} />}
        </>
      )}
      <div className="story-card__foot">
        <button type="button" className="story-card__toggle btn" onClick={onToggle} aria-expanded={expanded}>
          {expanded
            ? "Hide changes"
            : `${commit.files.length} file${commit.files.length === 1 ? "" : "s"}, +${commit.added} −${commit.removed}`}
        </button>
        {draftPosition && rewording === null && (
          <DraftVerbs
            commit={commit}
            first={draftPosition.first}
            last={draftPosition.last}
            verbs={verbs}
            busy={busy}
            setBusy={setBusy}
            onReword={() =>
              setRewording(commit.body ? `${commit.subject}\n\n${commit.body}` : commit.subject)
            }
          />
        )}
      </div>
      {expanded && <CommitDetail sha={commit.sha} recipe={recipe !== null} />}
    </article>
  );
}

/**
 * The last card: the working tree, and the two ways the next card gets
 * written — prose for what is uncommitted, or a command whose output is
 * committed with its recipe.
 */
function TailCard({ status, verbs }: { status: GitStatus | null; verbs: Verbs }) {
  const dirty = (status?.staged ?? 0) + (status?.unstaged ?? 0) + (status?.untracked ?? 0);
  const [message, setMessage] = useState("");
  const [command, setCommand] = useState("");
  const [output, setOutput] = useState("");
  const [busy, setBusy] = useState(false);
  const commit = () =>
    runVerb(
      verbs,
      setBusy,
      api
        .gitStage({ all: true })
        .then(() => api.gitCommit(message))
        .then(() => null),
      () => setMessage(""),
    );
  const emit = () =>
    runVerb(
      verbs,
      setBusy,
      api
        .gitRecipe(command, output)
        .then((run) => `Ran it and committed ${output.trim()}/ as ${run.short}, with the recipe in its trailers.`),
      () => {
        setCommand("");
        setOutput("");
      },
    );
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
          : `${dirty} change${dirty === 1 ? "" : "s"} not yet committed. Say what happened, and this becomes the next card.`}
      </p>
      {dirty > 0 && (
        <div className="story-tail__form">
          <textarea
            className="story-card__textarea mono"
            aria-label="What happened"
            placeholder="What happened, in a sentence — the next commit's message"
            value={message}
            rows={3}
            onChange={(event) => setMessage(event.target.value)}
          />
          <span className="story-card__verbs">
            <button
              type="button"
              className="btn btn-primary"
              disabled={busy || message.trim() === ""}
              onClick={commit}
              data-tip="Stage everything and commit it with this message — one git commit, run as itself"
            >
              {busy ? "Committing…" : "Commit"}
            </button>
          </span>
        </div>
      )}
      <div className="story-tail__form story-tail__recipe" role="group" aria-label="Run a command as a recipe">
        <p className="insert-menu__hint">
          Or run a command and commit what it writes, with the command in the commit’s trailers so it
          can be replayed. It runs in a clean checkout of HEAD, never in your working tree, and its
          output lands in the folder you name — which must be empty.
        </p>
        <input
          className="insert-menu__input mono"
          aria-label="Command"
          placeholder="dotnet new webapi -o app"
          value={command}
          onChange={(event) => setCommand(event.target.value)}
        />
        <input
          className="insert-menu__input mono"
          aria-label="Output folder"
          placeholder="app"
          value={output}
          onChange={(event) => setOutput(event.target.value)}
        />
        <span className="story-card__verbs">
          <button
            type="button"
            className="btn"
            disabled={busy || command.trim() === "" || output.trim() === ""}
            onClick={emit}
          >
            {busy ? "Running…" : "Run and commit"}
          </button>
        </span>
      </div>
    </article>
  );
}

export function HistoryLens() {
  const [log, setLog] = useState<GitLog | null>(null);
  const [status, setStatus] = useState<GitStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [open, setOpen] = useState<ReadonlySet<string>>(new Set());
  // How many commits before the open tail are unfolded. Grows by clicks.
  const [unfolded, setUnfolded] = useState(0);
  const [tick, setTick] = useState(0);
  const refresh = useCallback(() => setTick((n) => n + 1), []);
  const verbs = useMemo<Verbs>(() => ({ refresh, say: setNotice }), [refresh]);

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
        <p className="muted">
          This folder is not a git repository, so it has no history to read as a story.
        </p>
      </div>
    );
  }

  const shown = Math.min(story.length, OPEN_TAIL + unfolded);
  const folded = story.length - shown;
  const visible = story.slice(folded);
  const drafts = story.filter((commit) => commit.draft);

  return (
    <div className="story" role="region" aria-label="History, as a story">
      <p className="story__banner muted" role="note">
        A lens over this repository’s history, oldest first. It exists on disk nowhere and cannot
        be saved. Cards below the publication floor are records and carry no verbs; drafts above it
        can be reworded, moved and dropped, and a recipe can be replayed.
      </p>
      {notice && (
        <p className="story__notice" role="alert">
          {notice}
        </p>
      )}
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
            <p
              key={`floor-${commit.sha}`}
              className="story__floor"
              role="separator"
              aria-label="Publication floor"
            >
              <span className="git-floor__count">Publication floor</span>{" "}
              {log.floor?.summary ??
                "Below this line, commits are records someone else may hold. Above it they are drafts."}
            </p>
          ) : null;
        const draftIndex = commit.draft ? drafts.indexOf(commit) : -1;
        return (
          <div key={commit.sha}>
            {marker}
            <CommitCard
              commit={commit}
              expanded={open.has(commit.sha)}
              onToggle={() => toggle(commit.sha)}
              verbs={verbs}
              draftPosition={
                draftIndex >= 0
                  ? { first: draftIndex === 0, last: draftIndex === drafts.length - 1 }
                  : null
              }
            />
          </div>
        );
      })}
      <TailCard status={status} verbs={verbs} />
    </div>
  );
}
