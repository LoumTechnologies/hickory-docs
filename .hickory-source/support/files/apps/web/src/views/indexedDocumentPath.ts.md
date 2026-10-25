<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/indexedDocumentPath.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `9f0dd33f7b0fc6e0cf27d05bca5af31372d319b29939024e4964962ddfc7b1f0`.

<book:entry path="apps/web/src/views/indexedDocumentPath.ts">
import { nodeForAbsolutePath } from "../lib/openPath";
import { samePath } from "../lib/paths";
import type { FileNode } from "../api/types";
import type { DocSummary } from "../api/types";

/** Resolve only documents the host already indexed, including ignored sessions.
 * This never opens/registers a new filesystem path or changes tree visibility. */
export function indexedDocumentPath(reported: string, documents: readonly DocSummary[]): DocSummary | undefined {
  const normalize = (path: string) => path.replace(/\\/g, "/").replace(/^\.\//, "");
  const path = normalize(reported);
  const exact = documents.filter(doc => normalize(doc.path) === path);
  if (exact.length) return exact.length === 1 ? exact[0] : undefined;
  // A relative name is explicit, not a request to guess between folders.
  if (!path.startsWith("/") && !/^[A-Za-z]:\//.test(path)) return undefined;
  const matches = documents.filter(doc => path.endsWith(`/${normalize(doc.path)}`));
  const longest = Math.max(0, ...matches.map(doc => normalize(doc.path).length));
  const best = matches.filter(doc => normalize(doc.path).length === longest);
  return best.length === 1 ? best[0] : undefined;
}

/** Open only registered identities, refreshing the index once for new sessions. */
export async function openIndexedDocument(
  path: string,
  cached: { path: string; id?: string },
  projectId: string | null | undefined,
  actions: {
    refresh: (projectId: string) => Promise<DocSummary[]>;
    update: (documents: DocSummary[]) => void;
    open: (id: string, path: string) => void;
    notice: (message: string) => void;
    focusTree: () => void;
  },
): Promise<void> {
  if (cached.id) { actions.open(cached.id, cached.path); return; }
  if (!projectId) { actions.focusTree(); return; }
  try {
    const documents = await actions.refresh(projectId);
    actions.update(documents);
    const indexed = indexedDocumentPath(path, documents);
    if (indexed) actions.open(indexed.id, indexed.path);
    else {
      actions.notice(`This document is not indexed in the open folder: ${path}`);
      actions.focusTree();
    }
  } catch (error) {
    actions.notice(`Could not open recorded document: ${String(error)}`);
  }
}

/** Indexed identities take priority over the visible tree, which may omit sessions. */
export function resolveDocumentPath(reported: string, documents: readonly DocSummary[], roots: readonly { tree: FileNode[] }[]): { path: string; id?: string } {
  const indexed = indexedDocumentPath(reported, documents);
  if (indexed) return { path: indexed.path, id: indexed.id };
  const relative = (nodes: readonly FileNode[]): FileNode | null => {
    for (const node of nodes) {
      if (!node.dir && samePath(node.path, reported)) return node;
      if (node.children) {
        const found = relative(node.children);
        if (found) return found;
      }
    }
    return null;
  };
  for (const root of roots) {
    const node = nodeForAbsolutePath(root.tree, reported) ?? relative(root.tree);
    if (node) return { path: node.path, id: node.doc_id };
  }
  return { path: reported };
}
</book:entry>

</book:doc>
