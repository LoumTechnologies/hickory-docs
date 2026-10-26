<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/github.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4b914f32344b"></a>
## `apps/web/src/api/github.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-4b914f32344b-GithubCheck"></a>
### `GithubCheck`

Hickory's normalized GitHub contract, not the JSON shapes emitted by `gh`.

<storycode:copy id="f-4b914f32344b-GithubCheck">
// Hickory's normalized GitHub contract, not the JSON shapes emitted by `gh`.

export interface GithubCheck {
  name: string;
  workflow?: string | null;
  status: string;
  conclusion?: string | null;
  url?: string | null;
  run?: number | null;
  job?: number | null;
}

</storycode:copy>
<a id="f-4b914f32344b-GithubComment"></a>
### `GithubComment`

<storycode:copy id="f-4b914f32344b-GithubComment">
export interface GithubComment {
  id?: string;
  author?: { login?: string } | null;
  body: string;
  createdAt?: string;
  updatedAt?: string;
  url?: string;
}

</storycode:copy>
<a id="f-4b914f32344b-GithubReview"></a>
### `GithubReview`

<storycode:copy id="f-4b914f32344b-GithubReview">
export interface GithubReview extends GithubComment {
  state?: string;
  submittedAt?: string;
}

</storycode:copy>
<a id="f-4b914f32344b-GithubHeadConflict"></a>
### `GithubHeadConflict`

<storycode:copy id="f-4b914f32344b-GithubHeadConflict">
export interface GithubHeadConflict {
  number: number;
  title: string;
  url: string;
  author?: string | null;
  commit_authors: string[];
  head: string;
  head_sha: string;
  status: "clean" | "conflicting" | "unavailable";
  reason?: string | null;
  claim: "these current head commits conflict if merged together";
}

</storycode:copy>
<a id="f-4b914f32344b-GithubPullRequest"></a>
### `GithubPullRequest`

<storycode:copy id="f-4b914f32344b-GithubPullRequest">
export interface GithubPullRequest {
  repository: string;
  number: number;
  title: string;
  state: string;
  draft: boolean;
  url: string;
  author?: string | null;
  review_decision?: string | null;
  mergeable?: string | null;
  merge_state?: string | null;
  head: string;
  head_sha: string;
  base: string;
  updated_at?: string | null;
  checks: GithubCheck[];
  unread?: number;
  notification_thread?: string | null;
  body?: string;
  reviews?: GithubReview[];
  comments?: GithubComment[];
  current_head_conflicts?: GithubHeadConflict[];
}

</storycode:copy>
<a id="f-4b914f32344b-GithubIssue"></a>
### `GithubIssue`

<storycode:copy id="f-4b914f32344b-GithubIssue">
export interface GithubIssue {
  repository: string;
  number: number;
  folder: string;
  title: string;
  state: string;
  url?: string | null;
  author?: string | null;
  labels?: string[];
  updated_at?: string | null;
  freshness?: "live" | "unavailable";
  reason?: string;
  unread?: number;
  notification_thread?: string | null;
  body?: string;
  state_reason?: string | null;
  assignees?: { login: string }[];
  comments?: GithubComment[];
}

</storycode:copy>
<a id="f-4b914f32344b-GithubWorkspace"></a>
### `GithubWorkspace`

<storycode:copy id="f-4b914f32344b-GithubWorkspace">
export interface GithubWorkspace {
  status: "available" | "unavailable" | "not_repository" | "not_github" | "detached";
  repository?: string;
  branch?: string;
  reason?: string;
  reviews: GithubPullRequest[];
  issues: GithubIssue[];
}
</storycode:copy>

### Assemble `apps/web/src/api/github.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/github.ts"><storycode:paste select="#f-4b914f32344b-GithubCheck"/><storycode:paste select="#f-4b914f32344b-GithubComment"/><storycode:paste select="#f-4b914f32344b-GithubReview"/><storycode:paste select="#f-4b914f32344b-GithubHeadConflict"/><storycode:paste select="#f-4b914f32344b-GithubPullRequest"/><storycode:paste select="#f-4b914f32344b-GithubIssue"/><storycode:paste select="#f-4b914f32344b-GithubWorkspace"/></storycode:file>

</storycode:doc>
