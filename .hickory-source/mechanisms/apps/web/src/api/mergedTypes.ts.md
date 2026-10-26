<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/mergedTypes.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-250d9c7b22e5"></a>
## `apps/web/src/api/mergedTypes.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-250d9c7b22e5-WorktreeInfo"></a>
### `WorktreeInfo`

One real worktree available to the synthesized merged view.
See docs/specs/freeform/the-merged-view.md.

<storycode:copy id="f-250d9c7b22e5-WorktreeInfo">
/** One real worktree available to the synthesized merged view.
 * See docs/specs/freeform/the-merged-view.md. */
export interface WorktreeInfo {
  path: string;
  name: string;
  branch?: string;
  current: boolean;
}

</storycode:copy>
<a id="f-250d9c7b22e5-MergedRegion"></a>
### `MergedRegion`

Peer sources agree on shared regions; variants retain each source's text.

<storycode:copy id="f-250d9c7b22e5-MergedRegion">
/** Peer sources agree on shared regions; variants retain each source's text. */
export type MergedRegion =
  | { kind: "shared"; text: string }
  | { kind: "variant"; by_source: Record<string, string> };

</storycode:copy>
<a id="f-250d9c7b22e5-MergedViewResponse"></a>
### `MergedViewResponse`

<storycode:copy id="f-250d9c7b22e5-MergedViewResponse">
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
</storycode:copy>

### Assemble `apps/web/src/api/mergedTypes.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/mergedTypes.ts"><storycode:paste select="#f-250d9c7b22e5-WorktreeInfo"/><storycode:paste select="#f-250d9c7b22e5-MergedRegion"/><storycode:paste select="#f-250d9c7b22e5-MergedViewResponse"/></storycode:file>

</storycode:doc>
