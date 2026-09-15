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
