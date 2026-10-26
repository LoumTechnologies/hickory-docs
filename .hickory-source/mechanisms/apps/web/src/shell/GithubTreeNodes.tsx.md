<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/GithubTreeNodes.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8c9096f3895b"></a>
## `apps/web/src/shell/GithubTreeNodes.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A GitHub Issue Belongs To Its Associated Folder](../../../../../guarantees/integrations/a-github-issue-belongs-to-its-associated-folder/contract.md), [A Review Belongs To The Worktree For Its Head](../../../../../guarantees/integrations/a-review-belongs-to-the-worktree-for-its-head/contract.md), [An Unread Item Notifies At Its Nearest Visible Ancestor](../../../../../guarantees/integrations/an-unread-item-notifies-at-its-nearest-visible-ancestor/contract.md).

<a id="f-8c9096f3895b-githubKey"></a>
### `githubKey`

<storycode:copy id="f-8c9096f3895b-githubKey">
import { useCallback, useEffect, useState, type KeyboardEvent } from "react";

import { useGithubTreeDisclosure } from "./useGithubTreeDisclosure";
import { useUiReading } from "../lib/uiReading";
import { api } from "../api/client";
import type {
  GithubCheck,
  GithubIssue,
  GithubPullRequest,
  GithubWorkspace,
} from "../api/github";
import { workspaceNodeKey } from "../lib/workspaceTree";
import { isAction } from "../lib/keymap";
import { GithubNotificationRead, GithubTreeLink } from "./GithubTreeControls";
import { GithubBodyAndComment as BodyAndComment } from "./GithubBodyAndComment";
import { TreeRenameEditor } from "./TreeRenameEditor";

const githubKey = (id: string) => workspaceNodeKey({ provider: "github", id });
</storycode:copy>
<a id="f-8c9096f3895b-textOf"></a>
### `textOf`

<storycode:copy id="f-8c9096f3895b-textOf">
const textOf = (error: unknown) => error instanceof Error ? error.message : String(error);

</storycode:copy>
<a id="f-8c9096f3895b-parseGithubIssueReference"></a>
### `parseGithubIssueReference`

<storycode:copy id="f-8c9096f3895b-parseGithubIssueReference">
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

</storycode:copy>
<a id="f-8c9096f3895b-githubUnreadUnder"></a>
### `githubUnreadUnder`

<storycode:copy id="f-8c9096f3895b-githubUnreadUnder">
export function githubUnreadUnder(issues: readonly GithubIssue[], folder: string): number {
  const prefix = folder ? `${folder.replace(/\/+$/, "")}/` : "";
  return issues.reduce((total, issue) =>
    total + ((issue.folder === folder || (prefix && issue.folder.startsWith(prefix))) ? issue.unread ?? 0 : 0), 0);
}

</storycode:copy>
<a id="f-8c9096f3895b-GithubTreeReading"></a>
### `GithubTreeReading`

<storycode:copy id="f-8c9096f3895b-GithubTreeReading">
function GithubTreeReading({ label, value }: { label: string; value: unknown }) {
  useUiReading(`GitHub tree content ${label}`, value);
  return null;
}

</storycode:copy>
<a id="f-8c9096f3895b-UnreadBadge"></a>
### `UnreadBadge`

<storycode:copy id="f-8c9096f3895b-UnreadBadge">
function UnreadBadge({ count }: { count?: number }) {
  return count ? <span className="github-tree__unread" data-tip={`${count} unread GitHub notification${count === 1 ? "" : "s"}`}>{count}</span> : null;
}

</storycode:copy>
<a id="f-8c9096f3895b-MarkRead"></a>
### `MarkRead`

<storycode:copy id="f-8c9096f3895b-MarkRead">
function MarkRead({ thread, onChanged }: { thread?: string | null; onChanged: () => void }) {
  return thread ? <GithubNotificationRead thread={thread} onChanged={onChanged} /> : null;
}

</storycode:copy>
<a id="f-8c9096f3895b-useGithubWorkspaceNodes"></a>
### `useGithubWorkspaceNodes`

<storycode:copy id="f-8c9096f3895b-useGithubWorkspaceNodes">
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

</storycode:copy>
<a id="f-8c9096f3895b-statusText"></a>
### `statusText`

<storycode:copy id="f-8c9096f3895b-statusText">
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

</storycode:copy>
<a id="f-8c9096f3895b-RenameTitle"></a>
### `RenameTitle`

<storycode:copy id="f-8c9096f3895b-RenameTitle">
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

</storycode:copy>
<a id="f-8c9096f3895b-CheckRow"></a>
### `CheckRow`

<storycode:copy id="f-8c9096f3895b-CheckRow">
function CheckRow({ check, parent }: { check: GithubCheck; parent: string }) {
  const key = githubKey(`${parent}/check/${check.name}`);
  const canLog = Boolean(check.run && check.job);
  const { open, detail: log, changed, error, toggle } = useGithubTreeDisclosure(
    `GitHub check ${parent}/${check.workflow ?? ""}/${check.name}`, [parent, check],
    () => api.githubCheckLog(check.run!, check.job!), canLog ? undefined : "This check has no addressable run/job log", false);
  return (
    <li role="treeitem" aria-expanded={open}>
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
      {open && <ul role="group"><GithubTreeReading label={key} value={{ check, log, error, capture_matches_source: !changed, available: canLog }} /><li role="treeitem" className="github-tree__log"><pre>{changed ? "Earlier check capture; refreshing current check…\n" : ""}{error ?? log?.text ?? (canLog ? "Loading log…" : "No addressable run/job log")}</pre>{log?.truncated && <span>Log truncated at 512 KiB.</span>}</li></ul>}
    </li>
  );
}

</storycode:copy>
<a id="f-8c9096f3895b-GithubReviewRows"></a>
### `GithubReviewRows`

<storycode:copy id="f-8c9096f3895b-GithubReviewRows">
export function GithubReviewRows({ workspace, onChanged }: { workspace: GithubWorkspace | null; onChanged: () => void }) {
  if (!workspace) return null;
  return <>{workspace.reviews.map((review) => <ReviewRow key={`${review.repository}#${review.number}`} review={review} onChanged={onChanged} />)}</>;
}

</storycode:copy>
<a id="f-8c9096f3895b-GithubProviderStatusRow"></a>
### `GithubProviderStatusRow`

<storycode:copy id="f-8c9096f3895b-GithubProviderStatusRow">
export function GithubProviderStatusRow({ workspace }: { workspace: GithubWorkspace | null }) {
  useUiReading("GitHub provider status", workspace ? { status: workspace.status, reason: workspace.reason ?? null,
    repository: workspace.repository ?? null, branch: workspace.branch ?? null } : { status: "not_loaded" });
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

</storycode:copy>
<a id="f-8c9096f3895b-ReviewRow"></a>
### `ReviewRow`

<storycode:copy id="f-8c9096f3895b-ReviewRow">
function ReviewRow({ review, onChanged }: { review: GithubPullRequest; onChanged: () => void }) {
  useUiReading(`GitHub PR row ${review.repository}#${review.number}`, { title: review.title, state: review.state,
    draft: review.draft, status: statusText(review), head: review.head, head_sha: review.head_sha, base: review.base,
    unread: review.unread ?? 0, updated_at: review.updated_at ?? null });
  const id = `${review.repository}#${review.number}`;
  const key = githubKey(id);
  const { open, detail, changed, error, renaming, setRenaming, load, toggle, rename } = useGithubTreeDisclosure(
    `GitHub PR ${id}`, [review.repository, review.number, review.head_sha, review.updated_at], () => api.githubPullRequest(review.number));
  const renameKey = (event: KeyboardEvent) => {
    if (isAction(event, "tree.rename")) {
      event.preventDefault(); event.stopPropagation();
      rename();
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
        {changed && <li role="treeitem">Showing an earlier provider capture while the current source refreshes. Submission is disabled.</li>}
        {detail && <>
          <GithubTreeReading label={key} value={{ detail, capture_matches_source: !changed }} />
          <MarkRead thread={review.notification_thread} onChanged={onChanged} />
          <li role="treeitem" className="github-tree__facts">{statusText(detail)} · {detail.author ?? "unknown author"} · <GithubTreeLink url={detail.url} label="Open on GitHub" /></li>
          <BodyAndComment readOnly={changed} kind="pr" repository={detail.repository} number={detail.number} body={detail.body ?? ""} comments={detail.comments ?? []} onChanged={load} />
          <li role="treeitem" className="github-tree__section">Reviews ({detail.reviews?.length ?? 0})
            <ul role="group">{(detail.reviews ?? []).map((item, index) => <li role="treeitem" key={item.id ?? index}><span className="github-tree__byline">{item.author?.login ?? "unknown"} · {item.state?.toLowerCase()}</span>{item.body && <p>{item.body}</p>}</li>)}</ul>
          </li>
          <li role="treeitem" className="github-tree__section">Checks ({detail.checks.length})
            <ul role="group">{detail.checks.map((check, index) => <CheckRow key={`${check.workflow}:${check.name}:${check.run}:${check.job}:${index}`} check={check} parent={id} />)}</ul>
          </li>
          <li role="treeitem" className="github-tree__section">Current-head conflict analysis ({detail.current_head_conflicts?.length ?? 0})
            <ul role="group">{(detail.current_head_conflicts ?? []).map((other) => <li role="treeitem" key={other.number}>
              <GithubTreeLink url={other.url} label={`PR #${other.number}: ${other.title}`} />
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

</storycode:copy>
<a id="f-8c9096f3895b-GithubIssueRows"></a>
### `GithubIssueRows`

<storycode:copy id="f-8c9096f3895b-GithubIssueRows">
export function GithubIssueRows({ workspace, folder, parentKey, onChanged }: { workspace: GithubWorkspace | null; folder: string; parentKey?: string; onChanged: () => void }) {
  if (!workspace) return null;
  return <>{workspace.issues.filter((issue) => issue.folder === folder).map((issue) => <IssueRow key={`${issue.repository}#${issue.number}`} issue={issue} parentKey={parentKey} onChanged={onChanged} />)}</>;
}

</storycode:copy>
<a id="f-8c9096f3895b-IssueRow"></a>
### `IssueRow`

<storycode:copy id="f-8c9096f3895b-IssueRow">
function IssueRow({ issue, parentKey, onChanged }: { issue: GithubIssue; parentKey?: string; onChanged: () => void }) {
  useUiReading(`GitHub issue row ${issue.repository}#${issue.number}`, { title: issue.title, state: issue.state,
    folder: issue.folder, unread: issue.unread ?? 0, freshness: issue.freshness ?? null, reason: issue.reason ?? null });
  const key = githubKey(`${issue.repository}#${issue.number}`);
  const { open, detail, changed, error, renaming, setRenaming, load, toggle, rename } = useGithubTreeDisclosure(
    `GitHub issue ${issue.repository}#${issue.number}`, [issue.repository, issue.number, issue.updated_at, issue.freshness],
    () => api.githubIssue(issue.repository, issue.number), issue.freshness === "unavailable" ? issue.reason ?? "Provider unavailable" : undefined);
  return (
    <li role="treeitem" aria-expanded={open}>
      <button type="button" className="github-tree__node mono" onClick={toggle}
        onKeyDown={(event) => { if (isAction(event, "tree.rename")) { event.preventDefault(); event.stopPropagation(); rename(); } }}
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
        {changed && <li role="treeitem">Showing an earlier provider capture while the current source refreshes. Submission is disabled.</li>}
        {detail && <>
          <GithubTreeReading label={key} value={{ detail, capture_matches_source: !changed }} />
          <MarkRead thread={issue.notification_thread} onChanged={onChanged} />
          <li role="treeitem" className="github-tree__facts">{detail.author ?? "unknown author"} · {(detail.labels ?? []).join(", ") || "no labels"} · <GithubTreeLink url={detail.url} label="Open on GitHub" /></li>
          <BodyAndComment readOnly={changed} kind="issue" repository={detail.repository} number={detail.number} body={detail.body ?? ""} comments={detail.comments ?? []} onChanged={load} />
        </>}
      </ul>}
    </li>
  );
}
</storycode:copy>

### Assemble `apps/web/src/shell/GithubTreeNodes.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/GithubTreeNodes.tsx"><storycode:paste select="#f-8c9096f3895b-githubKey"/><storycode:paste select="#f-8c9096f3895b-textOf"/><storycode:paste select="#f-8c9096f3895b-parseGithubIssueReference"/><storycode:paste select="#f-8c9096f3895b-githubUnreadUnder"/><storycode:paste select="#f-8c9096f3895b-GithubTreeReading"/><storycode:paste select="#f-8c9096f3895b-UnreadBadge"/><storycode:paste select="#f-8c9096f3895b-MarkRead"/><storycode:paste select="#f-8c9096f3895b-useGithubWorkspaceNodes"/><storycode:paste select="#f-8c9096f3895b-statusText"/><storycode:paste select="#f-8c9096f3895b-RenameTitle"/><storycode:paste select="#f-8c9096f3895b-CheckRow"/><storycode:paste select="#f-8c9096f3895b-GithubReviewRows"/><storycode:paste select="#f-8c9096f3895b-GithubProviderStatusRow"/><storycode:paste select="#f-8c9096f3895b-ReviewRow"/><storycode:paste select="#f-8c9096f3895b-GithubIssueRows"/><storycode:paste select="#f-8c9096f3895b-IssueRow"/></storycode:file>

</storycode:doc>
