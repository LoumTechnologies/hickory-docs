<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/api/mergedTypes.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3582509f88341343883ab5cb3d19eb0e15f4a1dc2348a1b3d41f97f0b546f6ea`.

<book:entry path="apps/web/src/api/mergedTypes.ts">
/** One real worktree available to the synthesized merged view.
 * See docs/specs/freeform/the-merged-view.md. */
export interface WorktreeInfo {
  path: string;
  name: string;
  branch?: string;
  current: boolean;
}

/** Peer sources agree on shared regions; variants retain each source's text. */
export type MergedRegion =
  | { kind: "shared"; text: string }
  | { kind: "variant"; by_source: Record<string, string> };

export interface MergedViewResponse {
  /** Exact displayed source bytes used to guard writes and undo. */
  snapshot?: Record<string, string>;
  repository: boolean;
  path?: string;
  sources: WorktreeInfo[];
  regions: MergedRegion[];
  /** Worktrees without this file are named rather than dropped. */
  missing: string[];
  shared_lines?: number;
  variants?: number;
  /** Comparison membership alone grants no write permission. */
  read_only?: boolean;
}
</book:entry>

</book:doc>
