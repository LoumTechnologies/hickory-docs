<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/workspaceTree.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-59ed104225da"></a>
## `apps/web/src/lib/workspaceTree.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Workspace Tree Is A Lens](../../../../../guarantees/authoring/the-workspace-tree-is-a-lens/contract.md), [An Unread Item Notifies At Its Nearest Visible Ancestor](../../../../../guarantees/integrations/an-unread-item-notifies-at-its-nearest-visible-ancestor/contract.md).

<a id="f-59ed104225da-WorkspaceNodeKind"></a>
### `WorkspaceNodeKind`

The provider-neutral half of the workspace tree.
A node is a projection of something another substrate owns: the filesystem,
a terminal registry, another Hickory process, or a configured remote
provider. It is deliberately not editable serialized text. `capabilities`
says which semantic edits its owner can answer, and `source` contains routing
identity but never a credential.

<storycode:copy id="f-59ed104225da-WorkspaceNodeKind">
// The provider-neutral half of the workspace tree.
//
// A node is a projection of something another substrate owns: the filesystem,
// a terminal registry, another Hickory process, or a configured remote
// provider. It is deliberately not editable serialized text. `capabilities`
// says which semantic edits its owner can answer, and `source` contains routing
// identity but never a credential.

export type WorkspaceNodeKind =
  | "folder"
  | "file"
  | "terminal"
  | "window"
  | "work-item"
  | "review"
  | "field"
  | "comment"
  | "check"
  | "log";

</storycode:copy>
<a id="f-59ed104225da-WorkspaceCapability"></a>
### `WorkspaceCapability`

<storycode:copy id="f-59ed104225da-WorkspaceCapability">
export type WorkspaceCapability =
  | "activate"
  | "expand"
  | "rename"
  | "rename-title"
  | "edit-body"
  | "comment"
  | "focus"
  | "close"
  | "refresh"
  | "mark-read"
  | "copy";

</storycode:copy>
<a id="f-59ed104225da-WorkspaceFreshness"></a>
### `WorkspaceFreshness`

<storycode:copy id="f-59ed104225da-WorkspaceFreshness">
export type WorkspaceFreshness =
  | { kind: "live" }
  | { kind: "cached"; at: string }
  | { kind: "refreshing" }
  | { kind: "unavailable"; reason: string }
  | { kind: "stale"; at: string; reason: string };

</storycode:copy>
<a id="f-59ed104225da-WorkspaceNodeSource"></a>
### `WorkspaceNodeSource`

<storycode:copy id="f-59ed104225da-WorkspaceNodeSource">
export interface WorkspaceNodeSource {
  /** Filesystem, terminal, window, jira, github, gitlab, or a future adapter. */
  provider: string;
  /** Immutable provider identity. A display label is never an identity. */
  id: string;
}
</storycode:copy>
<a id="f-59ed104225da-WorkspaceNode"></a>
### `WorkspaceNode`

<storycode:copy id="f-59ed104225da-WorkspaceNode">
export interface WorkspaceNode {
  /** Stable across label, state, and location changes; includes provider. */
  key: string;
  kind: WorkspaceNodeKind;
  /** `null` only for roots. */
  parent: string | null;
  label: string;
  state?: string;
  /** Direct unread events rooted at this node, not an aggregate of children. */
  unread: number;
  capabilities: readonly WorkspaceCapability[];
  source: WorkspaceNodeSource;
  freshness: WorkspaceFreshness;
}

</storycode:copy>
<a id="f-59ed104225da-ProjectedUnread"></a>
### `ProjectedUnread`

<storycode:copy id="f-59ed104225da-ProjectedUnread">
export interface ProjectedUnread {
  count: number;
  /** Direct sources contributing to the rendered count, for an honest tooltip. */
  sources: ReadonlyMap<string, number>;
}

</storycode:copy>
<a id="f-59ed104225da-workspaceNodeKey"></a>
### `workspaceNodeKey`

A stable key in the shared namespace. Neither label nor location belongs in it.

<storycode:copy id="f-59ed104225da-workspaceNodeKey">
/** A stable key in the shared namespace. Neither label nor location belongs in it. */
export function workspaceNodeKey(source: WorkspaceNodeSource): string {
  return `${encodeURIComponent(source.provider)}:${encodeURIComponent(source.id)}`;
}

</storycode:copy>
<a id="f-59ed104225da-projectUnread"></a>
### `projectUnread`

Put every direct unread count on the closest row a person can currently see.
Visibility is structural: a node is visible when it is a root, or its parent
is visible and expanded. Expansion only changes projection; it never mutates
the unread events. Invalid/orphaned nodes are omitted rather than leaking a
badge onto an unrelated root.

<storycode:copy id="f-59ed104225da-projectUnread">
/**
 * Put every direct unread count on the closest row a person can currently see.
 *
 * Visibility is structural: a node is visible when it is a root, or its parent
 * is visible and expanded. Expansion only changes projection; it never mutates
 * the unread events. Invalid/orphaned nodes are omitted rather than leaking a
 * badge onto an unrelated root.
 */
export function projectUnread(
  nodes: readonly WorkspaceNode[],
  expanded: ReadonlySet<string>,
): ReadonlyMap<string, ProjectedUnread> {
  const byKey = new Map(nodes.map((node) => [node.key, node] as const));
  const visibility = new Map<string, boolean>();

  const visible = (node: WorkspaceNode, visiting: Set<string>): boolean => {
    const known = visibility.get(node.key);
    if (known !== undefined) return known;
    if (node.parent === null) {
      visibility.set(node.key, true);
      return true;
    }
    if (visiting.has(node.key)) {
      visibility.set(node.key, false);
      return false;
    }
    const parent = byKey.get(node.parent);
    if (!parent) {
      visibility.set(node.key, false);
      return false;
    }
    visiting.add(node.key);
    const answer = visible(parent, visiting) && expanded.has(parent.key);
    visiting.delete(node.key);
    visibility.set(node.key, answer);
    return answer;
  };

  const out = new Map<string, { count: number; sources: Map<string, number> }>();
  for (const node of nodes) {
    if (node.unread <= 0) continue;
    let target: WorkspaceNode | undefined = node;
    const seen = new Set<string>();
    while (target && !visible(target, new Set())) {
      if (seen.has(target.key)) {
        target = undefined;
        break;
      }
      seen.add(target.key);
      target = target.parent === null ? undefined : byKey.get(target.parent);
    }
    if (!target) continue;
    const projected = out.get(target.key) ?? { count: 0, sources: new Map<string, number>() };
    projected.count += node.unread;
    projected.sources.set(
      node.source.provider,
      (projected.sources.get(node.source.provider) ?? 0) + node.unread,
    );
    out.set(target.key, projected);
  }
  return out;
}
</storycode:copy>

### Assemble `apps/web/src/lib/workspaceTree.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/workspaceTree.ts"><storycode:paste select="#f-59ed104225da-WorkspaceNodeKind"/><storycode:paste select="#f-59ed104225da-WorkspaceCapability"/><storycode:paste select="#f-59ed104225da-WorkspaceFreshness"/><storycode:paste select="#f-59ed104225da-WorkspaceNodeSource"/><storycode:paste select="#f-59ed104225da-WorkspaceNode"/><storycode:paste select="#f-59ed104225da-ProjectedUnread"/><storycode:paste select="#f-59ed104225da-workspaceNodeKey"/><storycode:paste select="#f-59ed104225da-projectUnread"/></storycode:file>

</storycode:doc>
