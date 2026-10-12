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
