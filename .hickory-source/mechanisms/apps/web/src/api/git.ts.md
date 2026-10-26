<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/git.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-06e4a0df030b"></a>
## `apps/web/src/api/git.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-06e4a0df030b-MergeDriverStatus"></a>
### `MergeDriverStatus`

The Git pane's contract: status, changes, diffs, branches, and what one
git command said. Split out of types.ts by name, not by accident — the
file-length ratchet (scripts/check-file-length.sh) is what asked for a
smaller file, and these are one subject.
Whether `.md` documents merge through hick in THIS clone.
The routing (`*.md merge=hick`) is committed; the driver definition
cannot be, because git will not let a repository hand a clone an executable
command. An undefined driver makes git fall back to its line merge
silently, which is why this is checked at project open.

<storycode:copy id="f-06e4a0df030b-MergeDriverStatus">
// The Git pane's contract: status, changes, diffs, branches, and what one
// git command said. Split out of types.ts by name, not by accident — the
// file-length ratchet (scripts/check-file-length.sh) is what asked for a
// smaller file, and these are one subject.

/** Whether `.md` documents merge through hick in THIS clone.
 *
 * The routing (`*.md merge=hick`) is committed; the driver definition
 * cannot be, because git will not let a repository hand a clone an executable
 * command. An undefined driver makes git fall back to its line merge
 * silently, which is why this is checked at project open. */
export interface MergeDriverStatus {
  repository: boolean;
  attributes: boolean;
  configured: boolean;
  summary: string;
}

</storycode:copy>
<a id="f-06e4a0df030b-InitOutcome"></a>
### `InitOutcome`

What `POST /api/git/merge-driver` (the banner's "Run hick init") did.

<storycode:copy id="f-06e4a0df030b-InitOutcome">
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

</storycode:copy>
<a id="f-06e4a0df030b-GitStatus"></a>
### `GitStatus`

<storycode:copy id="f-06e4a0df030b-GitStatus">
export interface GitStatus {
  repository: boolean;
  branch?: string;
  staged?: number;
  unstaged?: number;
  untracked?: number;
}

</storycode:copy>
<a id="f-06e4a0df030b-GitChangeFile"></a>
### `GitChangeFile`

One changed file in the working tree: what the index says of it and what
the tree says, as git's two porcelain columns (` `, `M`, `A`, `D`, `R`,
`?`). A file can be on both sides at once.

<storycode:copy id="f-06e4a0df030b-GitChangeFile">
/** One changed file in the working tree: what the index says of it and what
 * the tree says, as git's two porcelain columns (` `, `M`, `A`, `D`, `R`,
 * `?`). A file can be on both sides at once. */
export interface GitChangeFile {
  path: string;
  from?: string;
  index: string;
  tree: string;
}

</storycode:copy>
<a id="f-06e4a0df030b-GitChanges"></a>
### `GitChanges`

<storycode:copy id="f-06e4a0df030b-GitChanges">
/** GET /api/git/changes. */
export interface GitChanges {
  repository: boolean;
  branch?: string;
  upstream?: string | null;
  ahead?: number;
  behind?: number;
  files: GitChangeFile[];
}

</storycode:copy>
<a id="f-06e4a0df030b-GitDiff"></a>
### `GitDiff`

GET /api/git/diff — one file's diff, as git prints it.

<storycode:copy id="f-06e4a0df030b-GitDiff">
/** GET /api/git/diff — one file's diff, as git prints it. */
export interface GitDiff {
  path: string;
  diff: string;
  binary: boolean;
}

</storycode:copy>
<a id="f-06e4a0df030b-GitBranch"></a>
### `GitBranch`

<storycode:copy id="f-06e4a0df030b-GitBranch">
export interface GitBranch {
  name: string;
  upstream: string | null;
  current: boolean;
}

</storycode:copy>
<a id="f-06e4a0df030b-GitSaid"></a>
### `GitSaid`

What a git verb answered: `ok`, and what git said on the way.

<storycode:copy id="f-06e4a0df030b-GitSaid">
/** What a git verb answered: `ok`, and what git said on the way. */
export interface GitSaid {
  ok: boolean;
  said?: string;
  branch?: string;
}

</storycode:copy>
<a id="f-06e4a0df030b-GitCommitResult"></a>
### `GitCommitResult`

<storycode:copy id="f-06e4a0df030b-GitCommitResult">
export interface GitCommitResult {
  sha: string;
  short: string;
  subject: string;
}

</storycode:copy>
<a id="f-06e4a0df030b-CommitReadingData"></a>
### `CommitReadingData`

An immutable reading, with exact blob bytes and the full commit message.

<storycode:copy id="f-06e4a0df030b-CommitReadingData">
/** An immutable reading, with exact blob bytes and the full commit message. */
export interface CommitReadingData {
  sha: string; parent: string | null; message: string;
  files: { path: string; from: string; status: string; before: string; after: string; binary: boolean }[];
}
</storycode:copy>

### Assemble `apps/web/src/api/git.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/git.ts"><storycode:paste select="#f-06e4a0df030b-MergeDriverStatus"/><storycode:paste select="#f-06e4a0df030b-InitOutcome"/><storycode:paste select="#f-06e4a0df030b-GitStatus"/><storycode:paste select="#f-06e4a0df030b-GitChangeFile"/><storycode:paste select="#f-06e4a0df030b-GitChanges"/><storycode:paste select="#f-06e4a0df030b-GitDiff"/><storycode:paste select="#f-06e4a0df030b-GitBranch"/><storycode:paste select="#f-06e4a0df030b-GitSaid"/><storycode:paste select="#f-06e4a0df030b-GitCommitResult"/><storycode:paste select="#f-06e4a0df030b-CommitReadingData"/></storycode:file>

</storycode:doc>
