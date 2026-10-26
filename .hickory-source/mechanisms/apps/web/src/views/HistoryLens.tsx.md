<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/HistoryLens.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b01a7a57f285"></a>
## `apps/web/src/views/HistoryLens.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Recipe Commit Can Be Replayed, Into A New Commit, Never Over The Old One](../../../../../guarantees/lenses/a-recipe-commit-can-be-replayed/contract.md), [A view declares where edits land](../../../../../guarantees/lenses/a-view-declares-where-edits-land/contract.md), [The History Lens Reads The Repository As A Story](../../../../../guarantees/lenses/the-history-lens-reads-the-repository-as-a-story/contract.md), [The Past Is Edited By Rebase, Above The Floor Only](../../../../../guarantees/lenses/the-past-is-edited-by-rebase-above-the-floor/contract.md), [The Tail Of The Story Is The Next Commit](../../../../../guarantees/lenses/the-tail-of-the-story-is-the-next-commit/contract.md).

<a id="f-b01a7a57f285-OPEN_TAIL"></a>
### `OPEN_TAIL`

<storycode:copy id="f-b01a7a57f285-OPEN_TAIL">
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

</storycode:copy>
<a id="f-b01a7a57f285-FOLD_STEP"></a>
### `FOLD_STEP`

<storycode:copy id="f-b01a7a57f285-FOLD_STEP">
/** How many more the fold reveals per click. */
const FOLD_STEP = 25;

</storycode:copy>
<a id="f-b01a7a57f285-storyOrder"></a>
### `storyOrder`

The story's order: oldest first, so the newest commit is at the bottom
beside the working tree. The log arrives newest-first, as `git log` does.

<storycode:copy id="f-b01a7a57f285-storyOrder">
/**
 * The story's order: oldest first, so the newest commit is at the bottom
 * beside the working tree. The log arrives newest-first, as `git log` does.
 */
export function storyOrder(commits: readonly GitCommit[]): GitCommit[] {
  return [...commits].reverse();
}

</storycode:copy>
<a id="f-b01a7a57f285-floorIndex"></a>
### `floorIndex`

Where the publication floor falls in story order: the index of the first
draft, or null when nothing is a draft. Everything before it is a record
someone else may hold; everything from it on is still rewritable.

<storycode:copy id="f-b01a7a57f285-floorIndex">
/**
 * Where the publication floor falls in story order: the index of the first
 * draft, or null when nothing is a draft. Everything before it is a record
 * someone else may hold; everything from it on is still rewritable.
 */
export function floorIndex(commits: readonly GitCommit[]): number | null {
  const index = commits.findIndex((commit) => commit.draft);
  return index < 0 ? null : index;
}

</storycode:copy>
<a id="f-b01a7a57f285-proseOf"></a>
### `proseOf`

The prose of a message: the body without its trailer paragraph.

<storycode:copy id="f-b01a7a57f285-proseOf">
/** The prose of a message: the body without its trailer paragraph. */
export function proseOf(body: string): string {
  return body.split(/\n\n(?=Hick-)/)[0].trim();
}

</storycode:copy>
<a id="f-b01a7a57f285-Verbs"></a>
### `Verbs`

<storycode:copy id="f-b01a7a57f285-Verbs">
type Verbs = {
  /** A verb ran; the story re-reads itself. */
  refresh: () => void;
  /** What a verb said — a result, a refusal, or a stopped join, in git's words. */
  say: (text: string | null) => void;
  pending: boolean;
  activity: (pending: boolean) => void;
};

</storycode:copy>
<a id="f-b01a7a57f285-runVerb"></a>
### `runVerb`

Run a verb, tell the story what it said, and re-read.

<storycode:copy id="f-b01a7a57f285-runVerb">
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

</storycode:copy>
<a id="f-b01a7a57f285-CommitDetail"></a>
### `CommitDetail`

One commit's card, fetched on expand: the diff and what was edited since.

<storycode:copy id="f-b01a7a57f285-CommitDetail">
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

</storycode:copy>
<a id="f-b01a7a57f285-RecipeCell"></a>
### `RecipeCell`

The recipe cell inside a card: the command, the chips, and Replay.

<storycode:copy id="f-b01a7a57f285-RecipeCell">
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

</storycode:copy>
<a id="f-b01a7a57f285-DraftVerbs"></a>
### `DraftVerbs`

The verbs a draft card carries: reword, move, drop. Records carry none.

<storycode:copy id="f-b01a7a57f285-DraftVerbs">
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

</storycode:copy>
<a id="f-b01a7a57f285-CommitCard"></a>
### `CommitCard`

One commit as a card: prose for an ordinary one, a cell for a recipe.

<storycode:copy id="f-b01a7a57f285-CommitCard">
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

</storycode:copy>
<a id="f-b01a7a57f285-TailCard"></a>
### `TailCard`

The last card: the working tree, and the two ways the next card gets
written — prose for what is uncommitted, or a command whose output is
committed with its recipe.

<storycode:copy id="f-b01a7a57f285-TailCard">
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

</storycode:copy>
<a id="f-b01a7a57f285-HistoryLens"></a>
### `HistoryLens`

<storycode:copy id="f-b01a7a57f285-HistoryLens">
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
</storycode:copy>

### Assemble `apps/web/src/views/HistoryLens.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/HistoryLens.tsx"><storycode:paste select="#f-b01a7a57f285-OPEN_TAIL"/><storycode:paste select="#f-b01a7a57f285-FOLD_STEP"/><storycode:paste select="#f-b01a7a57f285-storyOrder"/><storycode:paste select="#f-b01a7a57f285-floorIndex"/><storycode:paste select="#f-b01a7a57f285-proseOf"/><storycode:paste select="#f-b01a7a57f285-Verbs"/><storycode:paste select="#f-b01a7a57f285-runVerb"/><storycode:paste select="#f-b01a7a57f285-CommitDetail"/><storycode:paste select="#f-b01a7a57f285-RecipeCell"/><storycode:paste select="#f-b01a7a57f285-DraftVerbs"/><storycode:paste select="#f-b01a7a57f285-CommitCard"/><storycode:paste select="#f-b01a7a57f285-TailCard"/><storycode:paste select="#f-b01a7a57f285-HistoryLens"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-8206e3ef4d86"></a>
## `apps/web/src/views/HistoryLens.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [A Recipe Commit Can Be Replayed, Into A New Commit, Never Over The Old One](../../../../../guarantees/lenses/a-recipe-commit-can-be-replayed/contract.md), [The History Lens Reads The Repository As A Story](../../../../../guarantees/lenses/the-history-lens-reads-the-repository-as-a-story/contract.md), [The Past Is Edited By Rebase, Above The Floor Only](../../../../../guarantees/lenses/the-past-is-edited-by-rebase-above-the-floor/contract.md), [The Tail Of The Story Is The Next Commit](../../../../../guarantees/lenses/the-tail-of-the-story-is-the-next-commit/contract.md).

<a id="f-8206e3ef4d86-commit"></a>
### `commit`

<storycode:copy id="f-8206e3ef4d86-commit">
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
// The history lens: the repository read as a story.
//
// Protects docs/guarantees/lenses/the-history-lens-reads-the-repository-as-a-story.md

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";

import { HistoryLens, OPEN_TAIL, floorIndex, storyOrder } from "./HistoryLens";
import { api } from "../api/client";
import type { GitCommit, GitLog } from "../api/types";

function commit(over: Partial<GitCommit> & { sha: string }): GitCommit {
  return {
    short: over.sha.slice(0, 7),
    parents: [],
    author: "Nate",
    email: "n@example.com",
    time: 1_700_000_000,
    subject: `Commit ${over.sha}`,
    body: "",
    refs: [],
    files: [{ path: "a.txt", status: "M", added: 1, removed: 0 }],
    added: 1,
    removed: 0,
    ...over,
  };
}

</storycode:copy>
<a id="f-8206e3ef4d86-LOG"></a>
### `LOG`

<storycode:copy id="f-8206e3ef4d86-LOG">
/** Newest first, as `git log` answers. */
const LOG: GitLog = {
  repository: true,
  commits: [
    commit({ sha: "c3333333", subject: "Change Main", draft: true }),
    commit({
      sha: "b2222222",
      subject: "Scaffold a web API",
      body: "Scaffolded.\n\nHick-Recipe: dotnet new webapi -o app\nHick-Image: sdk:9.0",
      recipe: { command: "dotnet new webapi -o app", image: "sdk:9.0", output: "4b825dc", output_path: "app", output_matches: true },
      files: [
        { path: "app/Program.cs", status: "A", added: 10, removed: 0 },
        { path: "app/app.csproj", status: "A", added: 5, removed: 0 },
      ],
      added: 15,
    }),
    commit({ sha: "a1111111", subject: "First", body: "The beginning." }),
  ],
  floor: {
    source: "upstream",
    sha: "b2222222",
    drafts: ["c3333333"],
    summary: "1 draft above origin/master.",
  },
};

</storycode:copy>
<a id="f-8206e3ef4d86-serve"></a>
### `serve`

<storycode:copy id="f-8206e3ef4d86-serve">
function serve(log: GitLog = LOG) {
  vi.spyOn(api, "gitLog").mockResolvedValue(log);
  vi.spyOn(api, "gitStatus").mockResolvedValue({ repository: true, branch: "master", staged: 0, unstaged: 2, untracked: 0 });
  vi.spyOn(api, "gitStage").mockResolvedValue({ ok: true } as never);
  vi.spyOn(api, "gitCommit").mockResolvedValue({ sha: "d4444444", short: "d444444" } as never);
  return vi.spyOn(api, "gitCommitDetail").mockResolvedValue({
    sha: "b2222222",
    diff: "diff --git a/app/Program.cs b/app/Program.cs\n+// scaffolded\n",
    files: ["app/Program.cs", "app/app.csproj"],
    edited_since: [{ path: "app/Program.cs", sha: "c3333333", short: "c333333", subject: "Change Main" }],
  });
}

</storycode:copy>
<a id="f-8206e3ef4d86-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-8206e3ef4d86-expression_statement">
beforeEach(() => vi.restoreAllMocks());
</storycode:copy>
<a id="f-8206e3ef4d86-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-8206e3ef4d86-expression_statement-2">
afterEach(() => cleanup());

</storycode:copy>
<a id="f-8206e3ef4d86-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-8206e3ef4d86-expression_statement-3">
describe("the story's shape", () => {
  it("reads oldest first", () => {
    expect(storyOrder(LOG.commits).map((c) => c.subject)).toEqual(["First", "Scaffold a web API", "Change Main"]);
  });

  it("puts the floor before the first draft", () => {
    expect(floorIndex(storyOrder(LOG.commits))).toBe(2);
    expect(floorIndex(storyOrder(LOG.commits).map((c) => ({ ...c, draft: false })))).toBeNull();
  });
});

</storycode:copy>
<a id="f-8206e3ef4d86-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-8206e3ef4d86-expression_statement-4">
describe("the history lens", () => {
  it("draws the commits as cards, oldest first, ending at the working tree", async () => {
    serve();
    render(<HistoryLens />);
    await waitFor(() => expect(screen.getByRole("article", { name: "First" })).toBeTruthy());
    const cards = screen.getAllByRole("article").map((a) => a.getAttribute("aria-label"));
    expect(cards).toEqual(["First", "Scaffold a web API", "Change Main", "Working tree"]);
    // It says what it is: a lens, on disk nowhere, read-only.
    expect(screen.getByRole("note").textContent).toMatch(/lens/);
    expect(screen.getByRole("note").textContent).toMatch(/cannot be saved/);
    // The tail is the working tree, with what is uncommitted.
    expect(screen.getByRole("article", { name: "Working tree" }).textContent).toMatch(/2 changes not yet committed/);
  });

  it("marks the publication floor between records and drafts", async () => {
    serve();
    render(<HistoryLens />);
    await waitFor(() => expect(screen.getByRole("separator", { name: "Publication floor" })).toBeTruthy());
    const floor = screen.getByRole("separator", { name: "Publication floor" });
    // The floor sits immediately before the first draft card.
    const next = floor.parentElement?.querySelector("article");
    expect(next?.getAttribute("aria-label")).toBe("Change Main");
    expect(screen.getByRole("article", { name: "Change Main" }).className).toContain("story-card--draft");
  });

  it("draws a recipe commit as a cell, unrecorded until replayed", async () => {
    serve();
    render(<HistoryLens />);
    const card = await screen.findByRole("article", { name: "Scaffold a web API" });
    expect(card.className).toContain("story-card--recipe");
    const cell = screen.getByRole("group", { name: "Recipe" });
    expect(cell.textContent).toContain("dotnet new webapi -o app");
    expect(cell.textContent).toContain("sdk:9.0");
    // A declared claim: nothing has replayed it, and the chip says so in
    // the words the spec allows.
    expect(cell.textContent).toContain("no evidence of drift");
    expect(cell.textContent).not.toMatch(/reproducible/);
    // The trailers are not repeated as prose.
    expect(cell.textContent).not.toContain("Hick-Recipe");
    // git checked the tree against the trailer: this one is the scaffold
    // exactly, so it can be upgraded.
    expect(cell.textContent).toContain("matches its recorded output");
    expect(cell.textContent).toContain("upgradeable");
  });

  it("says when a recipe commit was edited before it was committed", async () => {
    // The visual answer to "is this upgradeable?": a tree that is not the
    // trailer's is a scaffold with edits fused into it, and replay cannot
    // separate them.
    const edited = {
      ...LOG,
      commits: LOG.commits.map((c) =>
        c.recipe ? { ...c, recipe: { ...c.recipe, output_matches: false } } : c,
      ),
    };
    serve(edited);
    render(<HistoryLens />);
    const cell = await screen.findByRole("group", { name: "Recipe" });
    expect(cell.textContent).toContain("edited before it was committed");
    expect(cell.textContent).toContain("not upgradeable");
    expect(cell.textContent).not.toContain("matches its recorded output");
  });

  it("shows the diff as the card's output, and what was edited since", async () => {
    const detail = serve();
    render(<HistoryLens />);
    const card = await screen.findByRole("article", { name: "Scaffold a web API" });
    fireEvent.click(card.querySelector(".story-card__toggle")!);
    await waitFor(() => expect(detail).toHaveBeenCalledWith("b2222222"));
    await waitFor(() => expect(screen.getByRole("status").textContent).toMatch(/Edited since/));
    const edited = screen.getByRole("status").textContent ?? "";
    expect(edited).toContain("app/Program.cs");
    expect(edited).toContain("Change Main");
    expect(card.textContent).toContain("+// scaffolded");
  });

  it("folds the past and opens at the tail", async () => {
    const many: GitCommit[] = [];
    for (let i = 0; i < OPEN_TAIL + 5; i += 1) many.push(commit({ sha: `${i}`.padStart(8, "0"), subject: `Commit ${i}` }));
    serve({ repository: true, commits: [...many].reverse(), floor: null });
    render(<HistoryLens />);
    const fold = await screen.findByRole("button", { name: "5 earlier commits, folded" });
    expect(screen.queryByRole("article", { name: "Commit 0" })).toBeNull();
    expect(screen.getByRole("article", { name: `Commit ${OPEN_TAIL + 4}` })).toBeTruthy();
    fireEvent.click(fold);
    await waitFor(() => expect(screen.getByRole("article", { name: "Commit 0" })).toBeTruthy());
  });

  it("says so when the folder is not a repository", async () => {
    serve({ repository: false, commits: [], floor: null });
    render(<HistoryLens />);
    await waitFor(() => expect(screen.getByText(/not a git repository/)).toBeTruthy());
  });
});

</storycode:copy>
<a id="f-8206e3ef4d86-expression_statement-5"></a>
### `expression_statement`

Protects docs/guarantees/lenses/a-recipe-commit-can-be-replayed.md,

<storycode:copy id="f-8206e3ef4d86-expression_statement-5">
// Protects docs/guarantees/lenses/a-recipe-commit-can-be-replayed.md,
// docs/guarantees/lenses/the-tail-of-the-story-is-the-next-commit.md and
// docs/guarantees/lenses/the-past-is-edited-by-rebase-above-the-floor.md
describe("the story's verbs", () => {
  it("replays a recipe whose tree matches, and says what came of it", async () => {
    serve();
    const replay = vi.spyOn(api, "gitReplay").mockResolvedValue({
      of: "b2222222",
      sha: "e5555555",
      short: "e555555",
      same: false,
      moved: "rebase",
      head: "f6666666",
      said: [],
    });
    render(<HistoryLens />);
    const cell = await screen.findByRole("group", { name: "Recipe" });
    fireEvent.click(within(cell).getByRole("button", { name: "Replay" }));
    await waitFor(() => expect(replay).toHaveBeenCalledWith("b2222222"));
    await waitFor(() => expect(screen.getByRole("alert").textContent).toMatch(/Replayed b222222 as e555555/));
    expect(screen.getByRole("alert").textContent).toMatch(/differs/);
    expect(screen.getByRole("alert").textContent).toMatch(/rebase/);
  });

  it("offers no replay on a commit edited before it was committed, and shows replay evidence apart", async () => {
    const log = {
      ...LOG,
      commits: LOG.commits.map((c) =>
        c.recipe
          ? { ...c, recipe: { ...c.recipe, output_matches: false, replay_of: "a1111111", replay_same: true } }
          : c,
      ),
    };
    serve(log);
    render(<HistoryLens />);
    const cell = await screen.findByRole("group", { name: "Recipe" });
    expect(within(cell).queryByRole("button", { name: "Replay" })).toBeNull();
    // Evidence, worded as evidence: a replay happened and matched.
    expect(cell.textContent).toContain("replayed · same as a111111");
    expect(cell.textContent).not.toContain("no evidence of drift");
  });

  it("gives drafts reword, move and drop, and records nothing", async () => {
    serve();
    const reword = vi.spyOn(api, "gitReword").mockResolvedValue({ head: "x" });
    const drop = vi.spyOn(api, "gitDrop").mockResolvedValue({ head: "x" });
    render(<HistoryLens />);
    const draft = await screen.findByRole("article", { name: "Change Main" });
    const record = screen.getByRole("article", { name: "First" });
    expect(within(record).queryByRole("group", { name: "Edit this draft" })).toBeNull();
    const edit = within(draft).getByRole("group", { name: "Edit this draft" });
    // The only draft is both first and last: nowhere to move.
    expect(within(edit).getByRole("button", { name: "Move earlier" })).toHaveProperty("disabled", true);
    expect(within(edit).getByRole("button", { name: "Move later" })).toHaveProperty("disabled", true);

    fireEvent.click(within(edit).getByRole("button", { name: "Reword" }));
    const textarea = within(draft).getByRole("textbox", { name: "New message" }) as HTMLTextAreaElement;
    expect(textarea.value).toBe("Change Main");
    fireEvent.change(textarea, { target: { value: "Change Main, said better" } });
    fireEvent.click(within(draft).getByRole("button", { name: "Save" }));
    await waitFor(() => expect(reword).toHaveBeenCalledWith("c3333333", "Change Main, said better"));

    fireEvent.click(within(screen.getByRole("article", { name: "Change Main" })).getByRole("button", { name: "Drop" }));
    await waitFor(() => expect(drop).toHaveBeenCalledWith("c3333333"));
  });

  it("shows git's words when a verb is refused, and re-reads the story", async () => {
    serve();
    vi.spyOn(api, "gitDrop").mockRejectedValue(new Error("the working tree has uncommitted changes"));
    render(<HistoryLens />);
    const draft = await screen.findByRole("article", { name: "Change Main" });
    const calls = vi.mocked(api.gitLog).mock.calls.length;
    fireEvent.click(within(draft).getByRole("button", { name: "Drop" }));
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("uncommitted changes"));
    await waitFor(() => expect(vi.mocked(api.gitLog).mock.calls.length).toBeGreaterThan(calls));
  });

  it("commits the working tree from the tail, and runs a command as a recipe", async () => {
    serve();
    const recipe = vi.spyOn(api, "gitRecipe").mockResolvedValue({ sha: "e5555555", short: "e555555", output_tree: "t", said: [] });
    render(<HistoryLens />);
    const tail = await screen.findByRole("article", { name: "Working tree" });
    const message = within(tail).getByRole("textbox", { name: "What happened" });
    expect(within(tail).getByRole("button", { name: "Commit" })).toHaveProperty("disabled", true);
    fireEvent.change(message, { target: { value: "Fix the thing" } });
    fireEvent.click(within(tail).getByRole("button", { name: "Commit" }));
    await waitFor(() => expect(api.gitCommit).toHaveBeenCalledWith("Fix the thing"));
    expect(api.gitStage).toHaveBeenCalledWith({ all: true });

    fireEvent.change(within(tail).getByRole("textbox", { name: "Command" }), { target: { value: "dotnet new webapi -o app" } });
    fireEvent.change(within(tail).getByRole("textbox", { name: "Output folder" }), { target: { value: "app" } });
    fireEvent.click(within(tail).getByRole("button", { name: "Run and commit" }));
    await waitFor(() => expect(recipe).toHaveBeenCalledWith("dotnet new webapi -o app", "app"));
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("committed app/ as e555555"));
  });
});


</storycode:copy>
<a id="f-8206e3ef4d86-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-8206e3ef4d86-expression_statement-6">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares progressive history controls, actual replay receipts and repository mutation exclusion", async () => {
  const detail = serve();
  let complete!: (value: Awaited<ReturnType<typeof api.gitReplay>>) => void;
  const replay = vi.spyOn(api, "gitReplay").mockImplementation(() => new Promise(done => { complete = done; }));
  const drop = vi.spyOn(api, "gitDrop").mockResolvedValue({ head: "new-head" });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><HistoryLens /></UiActionContext.Provider>);
  await screen.findByRole("article", { name: "Scaffold a web API" });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
  expect(actions().some(a => a.label === "Read history commit b2222222")).toBe(false);
  await run("Set history changes visibility b2222222", { open: true });
  await waitFor(() => expect(detail).toHaveBeenCalledWith("b2222222"));
  await waitFor(() => expect(actions().some(a => a.label === "Read history commit b2222222")).toBe(true));
  const handle = actions().find(a => a.label === "Replay history recipe b2222222")!.handle;
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle, arguments: {}, request_id: "replay" }); });
  await waitFor(() => expect(replay).toHaveBeenCalledTimes(1));
  const dropping = actions().find(a => a.label === "Drop history draft c3333333")!;
  expect(dropping.enabled).toBe(false);
  expect(await registry.act(registry.scope, { id: "acp:other", kind: "agent" }, { handle: dropping.handle, arguments: {}, request_id: "other-operation" })).toMatchObject({ status: "disabled" });
  expect(drop).not.toHaveBeenCalled();
  const actual: Awaited<ReturnType<typeof api.gitReplay>> = { of: "b2222222", sha: "actual-replay", short: "actual-r", same: false, moved: "rebase", head: "actual-head", said: [] };
  await act(async () => complete(actual));
  expect(await pending).toMatchObject({ status: "result", outcome: { status: "completed", value: { reply: JSON.stringify(actual) } } });
  await run("Set history changes visibility b2222222", { open: false });
  expect(actions().some(a => a.label === "Read history commit b2222222")).toBe(false);
});

</storycode:copy>
<a id="f-8206e3ef4d86-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-8206e3ef4d86-expression_statement-7">
it("keeps a rejected history commit message and returns the actual commit on retry", async () => {
  serve();
  const commit = vi.mocked(api.gitCommit).mockRejectedValueOnce(new Error("hook refused")).mockResolvedValue({ sha: "actual-commit", short: "actual-c", subject: "Keep this rejected draft" });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><HistoryLens /></UiActionContext.Provider>);
  await screen.findByRole("article", { name: "Working tree" });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  await run("Set history tail commit message", { value: "Keep this rejected draft" });
  expect(await run("Commit current history working tree")).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: "hook refused" } });
  expect((screen.getByRole("textbox", { name: "What happened" }) as HTMLTextAreaElement).value).toBe("Keep this rejected draft");
  expect(await run("Commit current history working tree")).toMatchObject({ status: "result", outcome: { status: "completed", value: { commit_reply: JSON.stringify({ sha: "actual-commit", short: "actual-c", subject: "Keep this rejected draft" }) } } });
  expect(commit).toHaveBeenCalledTimes(2);
});
</storycode:copy>

### Assemble `apps/web/src/views/HistoryLens.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/HistoryLens.test.tsx"><storycode:paste select="#f-8206e3ef4d86-commit"/><storycode:paste select="#f-8206e3ef4d86-LOG"/><storycode:paste select="#f-8206e3ef4d86-serve"/><storycode:paste select="#f-8206e3ef4d86-expression_statement"/><storycode:paste select="#f-8206e3ef4d86-expression_statement-2"/><storycode:paste select="#f-8206e3ef4d86-expression_statement-3"/><storycode:paste select="#f-8206e3ef4d86-expression_statement-4"/><storycode:paste select="#f-8206e3ef4d86-expression_statement-5"/><storycode:paste select="#f-8206e3ef4d86-expression_statement-6"/><storycode:paste select="#f-8206e3ef4d86-expression_statement-7"/></storycode:file>


</storycode:doc>
