<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/gitHistoryTypes.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-085c28edfb47"></a>
## `apps/web/src/api/gitHistoryTypes.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-085c28edfb47-GitCommitDetail"></a>
### `GitCommitDetail`

<storycode:copy id="f-085c28edfb47-GitCommitDetail">
import type { GitRecipe, GitFileChange } from "./types";

export interface GitCommitDetail {
  sha: string;
  diff: string;
  files: string[];
  edited_since: { path: string; sha: string; short: string; subject: string }[];
}

</storycode:copy>
<a id="f-085c28edfb47-GitCommit"></a>
### `GitCommit`

<storycode:copy id="f-085c28edfb47-GitCommit">
export interface GitCommit {
  /** Above the publication floor: still a draft. */
  draft?: boolean;
  /** The command that produced this commit's tree, when its trailers say. */
  recipe?: GitRecipe;
  sha: string;
  short: string;
  parents: string[];
  author: string;
  email: string;
  /** Unix seconds; formatted by the client in the reader's own locale. */
  time: number;
  subject: string;
  body: string;
  /** Branch and tag names pointing here. */
  refs: string[];
  files: GitFileChange[];
  added: number;
  removed: number;
}

</storycode:copy>
<a id="f-085c28edfb47-GitLog"></a>
### `GitLog`

<storycode:copy id="f-085c28edfb47-GitLog">
export interface GitLog {
  /** False for a folder that is not under version control, which is an
   * entirely normal thing for a folder of notes to be. */
  repository: boolean;
  commits: GitCommit[];
  /** Null where the repository has published nothing to compute one against.
   * See docs/specs/freeform/expression-and-log.md. */
  floor: PublicationFloor | null;
}

</storycode:copy>
<a id="f-085c28edfb47-PublicationFloor"></a>
### `PublicationFloor`

The publication floor: `merge-base(HEAD, <published ref>)`.
Below it, commits are RECORDS — someone else may be holding them, and
nothing may re-produce one. Above it is the frontier, which is derived:
edit the document, re-emit, and those commits are replaced. Computed on
every read, never stored — merging moves the floor, and a recorded one
would be a claim the next fetch falsifies.

<storycode:copy id="f-085c28edfb47-PublicationFloor">
/** The publication floor: `merge-base(HEAD, <published ref>)`.
 *
 * Below it, commits are RECORDS — someone else may be holding them, and
 * nothing may re-produce one. Above it is the frontier, which is derived:
 * edit the document, re-emit, and those commits are replaced. Computed on
 * every read, never stored — merging moves the floor, and a recorded one
 * would be a claim the next fetch falsifies. */
export interface PublicationFloor {
  /** The ref the floor was computed against, when there is one. */
  published_ref?: string;
  /** The merge-base itself. Absent when nothing is published. */
  sha?: string;
  source: "upstream" | "remote-default" | "remote-named" | "none";
  /** Commits reachable from HEAD but not from the floor. */
  drafts: string[];
  /** One sentence a person can read. */
  summary: string;
}

</storycode:copy>

### Assemble `apps/web/src/api/gitHistoryTypes.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/gitHistoryTypes.ts"><storycode:paste select="#f-085c28edfb47-GitCommitDetail"/><storycode:paste select="#f-085c28edfb47-GitCommit"/><storycode:paste select="#f-085c28edfb47-GitLog"/><storycode:paste select="#f-085c28edfb47-PublicationFloor"/></storycode:file>

</storycode:doc>
