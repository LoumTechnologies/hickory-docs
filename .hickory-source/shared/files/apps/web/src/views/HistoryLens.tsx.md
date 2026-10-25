<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/HistoryLens.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Recipe Commit Can Be Replayed, Into A New Commit, Never Over The Old One](../../../../../../guarantees/lenses/a-recipe-commit-can-be-replayed/contract.md)
- [A view declares where edits land](../../../../../../guarantees/lenses/a-view-declares-where-edits-land/contract.md)
- [The History Lens Reads The Repository As A Story](../../../../../../guarantees/lenses/the-history-lens-reads-the-repository-as-a-story/contract.md)
- [The Past Is Edited By Rebase, Above The Floor Only](../../../../../../guarantees/lenses/the-past-is-edited-by-rebase-above-the-floor/contract.md)
- [The Tail Of The Story Is The Next Commit](../../../../../../guarantees/lenses/the-tail-of-the-story-is-the-next-commit/contract.md)

Captured SHA-256: `65d201419d635281911d6ccc724505854c44964db35a0ace97267a91a4f5db6d`.

<book:entry path="apps/web/src/views/HistoryLens.tsx">
import { useHistoryActions } from "./historyActions";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome, UiValue } from "../lib/uiActions";
import { SurfaceInfo } from "../components/SurfaceInfo";
import { openCommit } from "../lib/readingViews";
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
  pending: boolean;
  activity: (pending: boolean) => void;
};

/** Run a verb, tell the story what it said, and re-read. */
async function runVerb(
  verbs: Verbs,
  setBusy: (busy: boolean) => void,
  work: () => Promise<{ said: string | null; value: UiValue }>,
  then?: () => void,
): Promise<UiOutcome> {
  if (verbs.pending) return { status: "unavailable", reason: "Another history operation is pending" };
  verbs.pending = true; verbs.activity(true); setBusy(true); verbs.say(null);
  try {
    const result = await work();
    then?.(); verbs.say(result.said);
    return { status: "completed", value: result.value };
  } catch (e) {
    const reason = e instanceof Error ? e.message : String(e);
    verbs.say(reason); return { status: "unavailable", reason };
  } finally { verbs.pending = false; verbs.activity(false); setBusy(false); verbs.refresh(); }
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
  const human = useHistoryActions(`detail:${sha}`, () => [sha, detail, error], [{ id: "read", label: `Read history commit ${sha}`, enabled: () => !!detail && !error, execute: async () => { openCommit(sha); return { status: "completed", value: { navigation_requested: sha } }; } }], setError);
  useUiReading("history commit detail status", { sha, error, loading: !detail && !error, files: detail?.files, edited_since: detail?.edited_since });
  if (error) return <p className="error">{error}</p>;
  if (!detail) return <p className="muted">Reading the change…</p>;
  return (
    <div className="story-card__detail">
      <button className="btn" onClick={() => human("read")}>Read commit</button>
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
  const replay = () => runVerb(verbs, setBusy, async () => {
    const result = await api.gitReplay(commit.sha);
    return { said: result.same
      ? `Replayed ${commit.short} as ${result.short}: the same tree, so nothing about it changed. Joined by ${result.moved}.`
      : `Replayed ${commit.short} as ${result.short}: the output differs, and its diff says how. Joined by ${result.moved}.`,
      value: { reply: JSON.stringify(result), scope: "Actual Git replay equality and join reply, not independent runtime or prose verification" } };
  });
  const human = useHistoryActions(`recipe:${commit.sha}`, () => [commit, busy, verbs.pending], recipe.output_matches === true ? [{ id: "replay", label: `Replay history recipe ${commit.sha}`, enabled: () => !busy && !verbs.pending, execute: replay }] : [], verbs.say);

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
            disabled={busy || verbs.pending}
            onClick={() => human("replay")}
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
  const human = useHistoryActions(`draft:${commit.sha}`, () => [commit, first, last, busy, verbs.pending], [
    { id: "reword", label: `Edit history commit message ${commit.sha}`, enabled: () => !busy && !verbs.pending, execute: async () => { onReword(); return { status: "completed", value: { message_editor_opened: true, history_rewritten: false } }; } },
    { id: "earlier", label: `Move history draft earlier ${commit.sha}`, enabled: () => !busy && !first && !verbs.pending, execute: () => runVerb(verbs, setBusy, async () => ({ said: null, value: await api.gitMove(commit.sha, "earlier") })) },
    { id: "later", label: `Move history draft later ${commit.sha}`, enabled: () => !busy && !last && !verbs.pending, execute: () => runVerb(verbs, setBusy, async () => ({ said: null, value: await api.gitMove(commit.sha, "later") })) },
    { id: "drop", label: `Drop history draft ${commit.sha}`, enabled: () => !busy && !verbs.pending, execute: () => runVerb(verbs, setBusy, async () => ({ said: null, value: await api.gitDrop(commit.sha) })) },
  ], verbs.say);
  return (
    <span className="story-card__verbs" role="group" aria-label="Edit this draft">
      <button
        type="button"
        className="btn"
        disabled={busy || verbs.pending}
        onClick={() => human("reword")}
        data-tip="Change this commit's message — a rebase, allowed because this is a draft"
      >
        Reword
      </button>
      <button
        type="button"
        className="btn"
        disabled={busy || verbs.pending || first}
        onClick={() => human("earlier")}
        data-tip="Swap this commit with the one before it"
        aria-label="Move earlier"
      >
        ↑
      </button>
      <button
        type="button"
        className="btn"
        disabled={busy || verbs.pending || last}
        onClick={() => human("later")}
        data-tip="Swap this commit with the one after it"
        aria-label="Move later"
      >
        ↓
      </button>
      <button
        type="button"
        className="btn"
        disabled={busy || verbs.pending}
        onClick={() => human("drop")}
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
  const saveReword = () => runVerb(verbs, setBusy, async () => {
    if (rewording === null) throw new Error("Open the draft message editor first");
    const value = await api.gitReword(commit.sha, rewording);
    return { said: null, value };
  }, () => setRewording(null));
  const actions: Parameters<typeof useHistoryActions>[2] = [
    { id: "toggle", label: `Set history changes visibility ${commit.sha}`, parameters: { open: { type: "boolean" as const } }, enabled: () => true, execute: async (args: Record<string, UiValue>): Promise<UiOutcome> => { if (expanded !== args.open) onToggle(); return { status: "completed", value: { expanded: args.open } }; } },
  ];
  if (rewording !== null) actions.push(
    { id: "message", label: `Set history draft message ${commit.sha}`, parameters: { value: { type: "string", maxLength: 65536 } }, enabled: () => !busy && !verbs.pending, execute: async args => { setRewording(args.value as string); return { status: "completed", value: { draft_changed: true, history_rewritten: false } }; } },
    { id: "save", label: `Save history draft message ${commit.sha}`, parameters: {}, enabled: () => !busy && !verbs.pending, execute: saveReword },
    { id: "cancel", label: `Cancel history draft message ${commit.sha}`, parameters: {}, enabled: () => !busy && !verbs.pending, execute: async () => { setRewording(null); return { status: "completed", value: { draft_cancelled: true, history_rewritten: false } }; } },
  );
  const human = useHistoryActions(`card:${commit.sha}`, () => [commit, expanded, rewording, busy, verbs.pending], actions, verbs.say);
  useUiReading("history story card", { sha: commit.sha, subject: commit.subject, author: commit.author, time: commit.time, refs: commit.refs, draft: !!commit.draft, body: recipe ? proseOf(commit.body ?? "") : commit.body, recipe: recipe && { command: recipe.command, image: recipe.image, output_matches: recipe.output_matches, replay_of: recipe.replay_of, replay_same: recipe.replay_same }, file_count: commit.files.length, added: commit.added, removed: commit.removed, expanded, message_draft: rewording, pending: busy, scope: "Loaded Git card; commit messages and recipes are declared claims, distinct from replay and diff evidence" });
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
            disabled={busy}
            value={rewording}
            rows={Math.max(3, rewording.split("\n").length + 1)}
            onChange={(event) => setRewording(event.target.value)}
          />
          <span className="story-card__verbs">
            <button type="button" className="btn btn-primary" disabled={busy || verbs.pending} onClick={() => human("save")}>
              {busy ? "Rewording…" : "Save"}
            </button>
            <button type="button" className="btn" disabled={busy || verbs.pending} onClick={() => human("cancel")}>
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
        <button type="button" className="story-card__toggle btn" onClick={() => human("toggle", { open: !expanded })} aria-expanded={expanded}>
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
  const commit = () => runVerb(verbs, setBusy, async () => {
    const stage = await api.gitStage({ all: true });
    const result = await api.gitCommit(message);
    return { said: null, value: { stage_reply: JSON.stringify(stage), commit_reply: JSON.stringify(result), scope: "Actual stage-all and commit replies; not a test or verification verdict" } };
  }, () => setMessage(""));
  const emit = () => runVerb(verbs, setBusy, async () => {
    const result = await api.gitRecipe(command, output);
    return { said: `Ran it and committed ${output.trim()}/ as ${result.short}, with the recipe in its trailers.`, value: { reply: JSON.stringify(result), scope: "Actual recipe commit reply; independent behavioral verification is separate" } };
  }, () => { setCommand(""); setOutput(""); });
  const actions: Parameters<typeof useHistoryActions>[2] = [
    { id: "message", label: "Set history tail commit message", parameters: { value: { type: "string", maxLength: 65536 } }, enabled: () => !busy && dirty > 0, execute: async args => { setMessage(args.value as string); return { status: "completed", value: { message_changed: true, committed: false } }; } },
    { id: "command", label: "Set history recipe command", parameters: { value: { type: "string", maxLength: 65536 } }, enabled: () => !busy, execute: async args => { setCommand(args.value as string); return { status: "completed", value: { command_changed: true, executed: false } }; } },
    { id: "output", label: "Set history recipe output folder", parameters: { value: { type: "string", maxLength: 4096 } }, enabled: () => !busy, execute: async args => { setOutput(args.value as string); return { status: "completed", value: { output_changed: true, executed: false } }; } },
    { id: "commit", label: "Commit current history working tree", enabled: () => !busy && !verbs.pending && dirty > 0 && !!message.trim(), execute: commit },
    { id: "recipe", label: "Run and commit history recipe", enabled: () => !busy && !verbs.pending && !!command.trim() && !!output.trim(), execute: emit },
  ];
  const human = useHistoryActions("tail", () => [status, message, command, output, busy, verbs.pending], actions.filter(action => dirty > 0 || !["message", "commit"].includes(action.id)), verbs.say);
  useUiReading("history working tree tail", { status, message: dirty > 0 ? message : null, command, output, pending: busy, scope: "Loaded working tree state and draft inputs; stage-all/commit act on the actual repository at dispatch" });
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
            disabled={busy}
            placeholder="What happened, in a sentence — the next commit's message"
            value={message}
            rows={3}
            onChange={(event) => setMessage(event.target.value)}
          />
          <span className="story-card__verbs">
            <button
              type="button"
              className="btn btn-primary"
              disabled={busy || verbs.pending || message.trim() === ""}
              onClick={() => human("commit")}
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
          disabled={busy}
          placeholder="dotnet new webapi -o app"
          value={command}
          onChange={(event) => setCommand(event.target.value)}
        />
        <input
          className="insert-menu__input mono"
          aria-label="Output folder"
          disabled={busy}
          placeholder="app"
          value={output}
          onChange={(event) => setOutput(event.target.value)}
        />
        <span className="story-card__verbs">
          <button
            type="button"
            className="btn"
            disabled={busy || verbs.pending || command.trim() === "" || output.trim() === ""}
            onClick={() => human("recipe")}
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
  const [repositoryBusy, setRepositoryBusy] = useState(false);
  const refresh = useCallback(() => setTick((n) => n + 1), []);
  const verbs = useMemo<Verbs>(() => ({ refresh, say: setNotice, pending: false, activity: setRepositoryBusy }), [refresh]);

  useEffect(() => {
    let live = true;
    api.gitLog(1000).then(
      (answer) => { if (live) { setLog(answer); setError(null); } },
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

  const human = useHistoryActions("story", () => [log, status, error, notice, unfolded, [...open], verbs.pending], [
    { id: "refresh", label: "Refresh history story", enabled: () => !verbs.pending, execute: async () => { refresh(); return { status: "completed", value: { refresh_requested: true } }; } },
    ...((log?.commits.length ?? 0) > OPEN_TAIL + unfolded ? [{ id: "unfold", label: "Show earlier history story commits", enabled: () => true, execute: async (): Promise<UiOutcome> => { setUnfolded(n => n + FOLD_STEP); return { status: "completed", value: { additional_cards_requested: FOLD_STEP } }; } }] : []),
  ], setNotice);
  useUiReading("history story status", { repository: log?.repository, loading: !log && !error, error, notice, floor: log?.floor && { source: log.floor.source, sha: log.floor.sha, summary: log.floor.summary, draft_count: log.floor.drafts.length }, total_loaded: log?.commits.length ?? 0, disclosed_cards: Math.min(log?.commits.length ?? 0, OPEN_TAIL + unfolded), repository_busy: repositoryBusy, scope: "Loaded story window; folded earlier cards require explicit expansion" });
  if (error) return <div><p className="error git-pane__error">{error}</p><button className="btn" onClick={() => human("refresh")}>Retry history</button></div>;
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
      <button className="btn" disabled={verbs.pending} onClick={() => human("refresh")}>Refresh history</button>
      <SurfaceInfo name="Repository history" editable destination="Actions change git history above the publication floor or create a new commit. Published commits are read-only." />
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
          onClick={() => human("unfold")}
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
</book:entry>

</book:doc>
