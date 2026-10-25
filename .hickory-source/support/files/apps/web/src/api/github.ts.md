<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/api/github.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `72740f4b1ecb87b6d0c4aaee6e33b773bef5d1a58d14417ec1930cf2cce44e12`.

<book:entry path="apps/web/src/api/github.ts">
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

export interface GithubComment {
  id?: string;
  author?: { login?: string } | null;
  body: string;
  createdAt?: string;
  updatedAt?: string;
  url?: string;
}

export interface GithubReview extends GithubComment {
  state?: string;
  submittedAt?: string;
}

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

export interface GithubWorkspace {
  status: "available" | "unavailable" | "not_repository" | "not_github" | "detached";
  repository?: string;
  branch?: string;
  reason?: string;
  reviews: GithubPullRequest[];
  issues: GithubIssue[];
}
</book:entry>

</book:doc>
