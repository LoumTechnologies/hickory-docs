import { useCallback, useEffect, useState, type KeyboardEvent } from "react";

import { api } from "../api/client";
import type {
  GithubCheck,
  GithubComment,
  GithubIssue,
  GithubPullRequest,
  GithubWorkspace,
} from "../api/github";
import { workspaceNodeKey } from "../lib/workspaceTree";
import { isAction } from "../lib/keymap";
import { TreeRenameEditor } from "./TreeRenameEditor";

const githubKey = (id: string) => workspaceNodeKey({ provider: "github", id });
const textOf = (error: unknown) => error instanceof Error ? error.message : String(error);

export function parseGithubIssueReference(
  input: string,
  defaultRepository?: string,
): { repository: string; number: number } | null {
  const text = input.trim();
  const url = text.match(/^https:\/\/github\.com\/([^/]+\/[^/]+)\/issues\/(\d+)\/?$/i);
  const short = text.match(/^([A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+)#(\d+)$/);
  const local = text.match(/^#?(\d+)$/);
  const repository = url?.[1] ?? short?.[1] ?? (local ? defaultRepository : undefined);
  const number = Number(url?.[2] ?? short?.[2] ?? local?.[1]);
  return repository && Number.isSafeInteger(number) && number > 0 ? { repository, number } : null;
}

export function githubUnreadUnder(issues: readonly GithubIssue[], folder: string): number {
  const prefix = folder ? `${folder.replace(/\/+$/, "")}/` : "";
  return issues.reduce((total, issue) =>
    total + ((issue.folder === folder || (prefix && issue.folder.startsWith(prefix))) ? issue.unread ?? 0 : 0), 0);
}

function UnreadBadge({ count }: { count?: number }) {
  return count ? <span className="github-tree__unread" data-tip={`${count} unread GitHub notification${count === 1 ? "" : "s"}`}>{count}</span> : null;
}

function MarkRead({ thread, onChanged }: { thread?: string | null; onChanged: () => void }) {
  const [error, setError] = useState<string | null>(null);
  if (!thread) return null;
  return <li role="treeitem" className="github-tree__mark-read"><button type="button" onClick={() => {
    setError(null);
    void api.markGithubNotificationRead(thread).then(onChanged, (cause) => setError(textOf(cause)));
  }}>Mark notification read</button>{error && <span>{error}</span>}</li>;
}

export function useGithubWorkspaceNodes() {
  const [value, setValue] = useState<GithubWorkspace | null>(null);
  const [error, setError] = useState<string | null>(null);
  const refresh = useCallback(() => {
    setError(null);
    void api.githubWorkspace().then(
      (response) => setValue({ ...response, reviews: response.reviews ?? [], issues: response.issues ?? [] }),
      (cause) => setError(textOf(cause)),
    );
  }, []);
  useEffect(refresh, [refresh]);
  return { value, error, refresh };
}

function statusText(review: GithubPullRequest): string {
  const checks = review.checks ?? [];
  const failing = checks.filter((check) =>
    ["FAILURE", "ERROR", "TIMED_OUT", "CANCELLED"].includes(check.conclusion ?? ""),
  ).length;
  const pending = checks.filter((check) => check.status !== "COMPLETED").length;
  return [
    review.draft ? "draft" : review.state.toLowerCase(),
    review.review_decision?.toLowerCase().replaceAll("_", " "),
    failing ? `${failing} failing` : pending ? `${pending} pending` : checks.length ? "checks passed" : null,
    review.mergeable === "CONFLICTING" ? "conflicts" : null,
  ].filter(Boolean).join(" · ");
}

function RenameTitle({
  kind,
  repository,
  number,
  initialTitle,
  onDone,
}: {
  kind: "pr" | "issue";
  repository: string;
  number: number;
  initialTitle: string;
  onDone: (changed: boolean) => void;
}) {
  return (
    <TreeRenameEditor
      items={[{ key: githubKey(`${repository}#${number}`), context: `${repository}#${number}`, value: initialTitle }]}
      validate={(value) => value.trim() ? null : "A GitHub title cannot be empty."}
      apply={async (_item, value) => { await api.editGithubObject(kind, repository, number, "title", value); }}
      onClose={onDone}
    />
  );
}

function BodyAndComment({
  kind,
  repository,
  number,
  body,
  comments,
  onChanged,
}: {
  kind: "pr" | "issue";
  repository: string;
  number: number;
  body: string;
  comments: GithubComment[];
  onChanged: () => void;
}) {
  const [bodyText, setBodyText] = useState(body);
  const [comment, setComment] = useState("");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  useEffect(() => setBodyText(body), [body]);

  const saveBody = () => {
    setBusy(true);
    setNotice(null);
    void api.editGithubObject(kind, repository, number, "body", bodyText).then(
      () => { setBusy(false); setNotice("Body saved."); onChanged(); },
      (cause) => { setBusy(false); setNotice(textOf(cause)); },
    );
  };
  const addComment = () => {
    if (!comment.trim()) return;
    setBusy(true);
    setNotice(null);
    void api.commentOnGithubObject(kind, repository, number, comment).then(
      () => { setBusy(false); setComment(""); setNotice("Comment added."); onChanged(); },
      (cause) => { setBusy(false); setNotice(textOf(cause)); },
    );
  };

  return (
    <>
      <li role="treeitem" className="github-tree__field">
        <label>Body<textarea value={bodyText} disabled={busy} onChange={(event) => setBodyText(event.target.value)} /></label>
        <button type="button" disabled={busy || bodyText === body} onClick={saveBody}>Save body</button>
      </li>
      <li role="treeitem" className="github-tree__section">
        <span>Comments ({comments.length})</span>
        <ul role="group" className="github-tree__comments">
          {comments.map((item, index) => (
            <li role="treeitem" key={item.id ?? item.url ?? index}>
              <span className="github-tree__byline">{item.author?.login ?? "unknown"}</span>
              <p>{item.body}</p>
            </li>
          ))}
          <li role="treeitem" className="github-tree__new-comment">
            <label>New comment<textarea value={comment} disabled={busy} onChange={(event) => setComment(event.target.value)} /></label>
            <button type="button" disabled={busy || !comment.trim()} onClick={addComment}>Add comment</button>
          </li>
        </ul>
      </li>
      {notice && <li role="treeitem" className="github-tree__notice">{notice}</li>}
    </>
  );
}

function CheckRow({ check, parent }: { check: GithubCheck; parent: string }) {
  const [open, setOpen] = useState(false);
  const [log, setLog] = useState<{ text: string; truncated: boolean } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const key = githubKey(`${parent}/check/${check.name}`);
  const canLog = Boolean(check.run && check.job);
  const toggle = () => {
    if (!canLog) return;
    setOpen((before) => !before);
    if (!log && !open) {
      void api.githubCheckLog(check.run!, check.job!).then(setLog, (cause) => setError(textOf(cause)));
    }
  };
  return (
    <li role="treeitem" aria-expanded={canLog ? open : undefined}>
      <button
        type="button"
        className="github-tree__check mono"
        data-workspace-node-key={key}
        data-workspace-node-kind="check"
        data-workspace-parent={githubKey(parent)}
        data-workspace-expandable={String(canLog)}
        data-workspace-expanded={String(open)}
        tabIndex={-1}
        onClick={toggle}
      >
        <span aria-hidden>{open ? "▾" : canLog ? "▸" : "·"}</span>
        {check.workflow ? `${check.workflow}: ` : ""}{check.name}
        <span className="github-tree__state">{(check.conclusion ?? check.status).toLowerCase()}</span>
      </button>
      {open && <ul role="group"><li role="treeitem" className="github-tree__log"><pre>{error ?? log?.text ?? "Loading log…"}</pre>{log?.truncated && <span>Log truncated at 512 KiB.</span>}</li></ul>}
    </li>
  );
}

export function GithubReviewRows({ workspace, onChanged }: { workspace: GithubWorkspace | null; onChanged: () => void }) {
  if (!workspace) return null;
  return <>{workspace.reviews.map((review) => <ReviewRow key={`${review.repository}#${review.number}`} review={review} onChanged={onChanged} />)}</>;
}

export function GithubProviderStatusRow({ workspace }: { workspace: GithubWorkspace | null }) {
  if (!workspace || workspace.status !== "unavailable") return null;
  return (
    <li role="treeitem">
      <span className="github-tree__node github-tree__notice mono"
        data-workspace-node-key={githubKey("provider-status")}
        data-workspace-node-kind="field" tabIndex={-1}
        data-tip={workspace.reason ?? "GitHub is unavailable"}>
        GitHub unavailable{workspace.reason ? ` — ${workspace.reason}` : ""}
      </span>
    </li>
  );
}

function ReviewRow({ review, onChanged }: { review: GithubPullRequest; onChanged: () => void }) {
  const [open, setOpen] = useState(false);
  const [detail, setDetail] = useState<GithubPullRequest | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [renaming, setRenaming] = useState(false);
  const id = `${review.repository}#${review.number}`;
  const key = githubKey(id);
  const load = useCallback(() => {
    setError(null);
    void api.githubPullRequest(review.number).then(setDetail, (cause) => setError(textOf(cause)));
  }, [review.number]);
  const toggle = () => {
    setOpen((before) => !before);
    if (!open && !detail) load();
  };
  const renameKey = (event: KeyboardEvent) => {
    if (isAction(event, "tree.rename")) {
      event.preventDefault(); event.stopPropagation();
      setOpen(true); setRenaming(true); if (!detail) load();
    }
  };
  return (
    <li role="treeitem" aria-expanded={open}>
      <button type="button" className="github-tree__node mono" onClick={toggle} onKeyDown={renameKey}
        data-workspace-node-key={key} data-workspace-node-kind="review"
        data-workspace-expandable="true" data-workspace-expanded={String(open)} tabIndex={-1}
        data-tip={`${review.repository} ${review.head} → ${review.base}`}>
        <span aria-hidden>{open ? "▾" : "▸"}</span>
        <span className="github-tree__provider">PR</span> #{review.number}: {review.title}
        <UnreadBadge count={review.unread} />
        <span className="github-tree__state">{statusText(review)}</span>
      </button>
      {open && <ul role="group" className="github-tree__children">
        {renaming && <li role="treeitem"><RenameTitle kind="pr" repository={review.repository} number={review.number} initialTitle={review.title} onDone={(changed) => { setRenaming(false); if (changed) onChanged(); }} /></li>}
        {error && <li role="treeitem" className="github-tree__notice">{error}</li>}
        {!detail && !error && <li role="treeitem" className="muted">Loading pull request…</li>}
        {detail && <>
          <MarkRead thread={review.notification_thread} onChanged={onChanged} />
          <li role="treeitem" className="github-tree__facts">{statusText(detail)} · {detail.author ?? "unknown author"} · <a href={detail.url} target="_blank" rel="noreferrer">Open on GitHub</a></li>
          <BodyAndComment kind="pr" repository={detail.repository} number={detail.number} body={detail.body ?? ""} comments={detail.comments ?? []} onChanged={load} />
          <li role="treeitem" className="github-tree__section">Reviews ({detail.reviews?.length ?? 0})
            <ul role="group">{(detail.reviews ?? []).map((item, index) => <li role="treeitem" key={item.id ?? index}><span className="github-tree__byline">{item.author?.login ?? "unknown"} · {item.state?.toLowerCase()}</span>{item.body && <p>{item.body}</p>}</li>)}</ul>
          </li>
          <li role="treeitem" className="github-tree__section">Checks ({detail.checks.length})
            <ul role="group">{detail.checks.map((check) => <CheckRow key={`${check.workflow}:${check.name}`} check={check} parent={id} />)}</ul>
          </li>
          <li role="treeitem" className="github-tree__section">Current-head conflict analysis ({detail.current_head_conflicts?.length ?? 0})
            <ul role="group">{(detail.current_head_conflicts ?? []).map((other) => <li role="treeitem" key={other.number}>
              <a href={other.url} target="_blank" rel="noreferrer">PR #{other.number}: {other.title}</a>
              <span> — {other.status}; provider author {other.author ?? "unknown"}; commit authors {other.commit_authors.join(", ") || "unknown"}</span>
              {other.status === "conflicting" && <p>{other.claim}.</p>}
              {other.reason && <p>{other.reason}</p>}
            </li>)}</ul>
          </li>
        </>}
      </ul>}
    </li>
  );
}

export function GithubIssueRows({ workspace, folder, parentKey, onChanged }: { workspace: GithubWorkspace | null; folder: string; parentKey?: string; onChanged: () => void }) {
  if (!workspace) return null;
  return <>{workspace.issues.filter((issue) => issue.folder === folder).map((issue) => <IssueRow key={`${issue.repository}#${issue.number}`} issue={issue} parentKey={parentKey} onChanged={onChanged} />)}</>;
}

function IssueRow({ issue, parentKey, onChanged }: { issue: GithubIssue; parentKey?: string; onChanged: () => void }) {
  const [open, setOpen] = useState(false);
  const [detail, setDetail] = useState<GithubIssue | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [renaming, setRenaming] = useState(false);
  const key = githubKey(`${issue.repository}#${issue.number}`);
  const load = useCallback(() => {
    setError(null);
    void api.githubIssue(issue.repository, issue.number).then(setDetail, (cause) => setError(textOf(cause)));
  }, [issue.repository, issue.number]);
  const toggle = () => { setOpen((before) => !before); if (!open && !detail && issue.freshness !== "unavailable") load(); };
  return (
    <li role="treeitem" aria-expanded={open}>
      <button type="button" className="github-tree__node mono" onClick={toggle}
        onKeyDown={(event) => { if (isAction(event, "tree.rename")) { event.preventDefault(); event.stopPropagation(); setOpen(true); setRenaming(true); if (!detail && issue.freshness !== "unavailable") load(); } }}
        data-workspace-node-key={key} data-workspace-node-kind="work-item"
        data-workspace-parent={parentKey} data-workspace-expandable="true"
        data-workspace-expanded={String(open)} tabIndex={-1}
        data-tip={issue.reason ?? issue.repository}>
        <span aria-hidden>{open ? "▾" : "▸"}</span>
        <span className="github-tree__provider">Issue</span> #{issue.number}: {issue.title}
        <UnreadBadge count={issue.unread} />
        <span className="github-tree__state">{issue.state.toLowerCase()}</span>
      </button>
      {open && <ul role="group" className="github-tree__children">
        {renaming && <li role="treeitem"><RenameTitle kind="issue" repository={issue.repository} number={issue.number} initialTitle={issue.title} onDone={(changed) => { setRenaming(false); if (changed) onChanged(); }} /></li>}
        {issue.reason && <li role="treeitem" className="github-tree__notice">{issue.reason}</li>}
        {error && <li role="treeitem" className="github-tree__notice">{error}</li>}
        {!detail && !error && !issue.reason && <li role="treeitem" className="muted">Loading issue…</li>}
        {detail && <>
          <MarkRead thread={issue.notification_thread} onChanged={onChanged} />
          <li role="treeitem" className="github-tree__facts">{detail.author ?? "unknown author"} · {(detail.labels ?? []).join(", ") || "no labels"} · <a href={detail.url ?? "#"} target="_blank" rel="noreferrer">Open on GitHub</a></li>
          <BodyAndComment kind="issue" repository={detail.repository} number={detail.number} body={detail.body ?? ""} comments={detail.comments ?? []} onChanged={load} />
        </>}
      </ul>}
    </li>
  );
}
