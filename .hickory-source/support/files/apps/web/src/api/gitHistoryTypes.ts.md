<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/api/gitHistoryTypes.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `52002b517222f6aa1c5e355618ecebb027f561a88af8c4945196c1c1bc187e4c`.

<book:entry path="apps/web/src/api/gitHistoryTypes.ts">
import type { GitRecipe, GitFileChange } from "./types";

export interface GitCommitDetail {
  sha: string;
  diff: string;
  files: string[];
  edited_since: { path: string; sha: string; short: string; subject: string }[];
}

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

export interface GitLog {
  /** False for a folder that is not under version control, which is an
   * entirely normal thing for a folder of notes to be. */
  repository: boolean;
  commits: GitCommit[];
  /** Null where the repository has published nothing to compute one against.
   * See docs/specs/freeform/expression-and-log.md. */
  floor: PublicationFloor | null;
}

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

</book:entry>

</book:doc>
