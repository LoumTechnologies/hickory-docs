// The Git pane's contract: status, changes, diffs, branches, and what one
// git command said. Split out of types.ts by name, not by accident — the
// file-length ratchet (scripts/check-file-length.sh) is what asked for a
// smaller file, and these are one subject.

/** Whether `.hick` documents merge through hick in THIS clone.
 *
 * The routing (`*.hick merge=hick`) is committed; the driver definition
 * cannot be, because git will not let a repository hand a clone an executable
 * command. An undefined driver makes git fall back to its line merge
 * silently, which is why this is checked at project open. */
export interface MergeDriverStatus {
  repository: boolean;
  attributes: boolean;
  configured: boolean;
  summary: string;
}

/** What `POST /api/git/merge-driver` (the banner's "Run hick init") did. */
export interface InitOutcome {
  changed: {
    hook: boolean;
    gitignore: boolean;
    gitattributes: boolean;
    merge_driver: boolean;
    agents_md: boolean;
    mcp_json: boolean;
  };
  hook_path: string;
  status: MergeDriverStatus;
  ok: boolean;
}

export interface GitStatus {
  repository: boolean;
  branch?: string;
  staged?: number;
  unstaged?: number;
  untracked?: number;
}

/** One changed file in the working tree: what the index says of it and what
 * the tree says, as git's two porcelain columns (` `, `M`, `A`, `D`, `R`,
 * `?`). A file can be on both sides at once. */
export interface GitChangeFile {
  path: string;
  from?: string;
  index: string;
  tree: string;
}

/** GET /api/git/changes. */
export interface GitChanges {
  repository: boolean;
  branch?: string;
  upstream?: string | null;
  ahead?: number;
  behind?: number;
  files: GitChangeFile[];
}

/** GET /api/git/diff — one file's diff, as git prints it. */
export interface GitDiff {
  path: string;
  diff: string;
  binary: boolean;
}

export interface GitBranch {
  name: string;
  upstream: string | null;
  current: boolean;
}

/** What a git verb answered: `ok`, and what git said on the way. */
export interface GitSaid {
  ok: boolean;
  said?: string;
  branch?: string;
}

export interface GitCommitResult {
  sha: string;
  short: string;
  subject: string;
}
