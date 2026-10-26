<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/indexedDocumentPath.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f389f96c1b6a"></a>
## `apps/web/src/views/indexedDocumentPath.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-f389f96c1b6a-indexedDocumentPath"></a>
### `indexedDocumentPath`

<storycode:copy id="f-f389f96c1b6a-indexedDocumentPath">
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

</storycode:copy>
<a id="f-f389f96c1b6a-openIndexedDocument"></a>
### `openIndexedDocument`

Open only registered identities, refreshing the index once for new sessions.

<storycode:copy id="f-f389f96c1b6a-openIndexedDocument">
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

</storycode:copy>
<a id="f-f389f96c1b6a-resolveDocumentPath"></a>
### `resolveDocumentPath`

Indexed identities take priority over the visible tree, which may omit sessions.

<storycode:copy id="f-f389f96c1b6a-resolveDocumentPath">
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
</storycode:copy>

### Assemble `apps/web/src/views/indexedDocumentPath.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/indexedDocumentPath.ts"><storycode:paste select="#f-f389f96c1b6a-indexedDocumentPath"/><storycode:paste select="#f-f389f96c1b6a-openIndexedDocument"/><storycode:paste select="#f-f389f96c1b6a-resolveDocumentPath"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-8e8f891f853a"></a>
## `apps/web/src/views/indexedDocumentPath.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-8e8f891f853a-session"></a>
### `session`

<storycode:copy id="f-8e8f891f853a-session">
import { expect, it, vi } from "vitest";
import { indexedDocumentPath, openIndexedDocument, resolveDocumentPath } from "./indexedDocumentPath";
const session = { id: "recorded-session", path: "sessions/cancellation.md", updated_at: "" };
</storycode:copy>
<a id="f-8e8f891f853a-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-8e8f891f853a-expression_statement">
it("resolves an indexed ignored session without a visible tree entry", () => {
  expect(indexedDocumentPath(session.path, [session])).toBe(session);
  expect(indexedDocumentPath("./sessions/cancellation.md", [session])).toBe(session);
  expect(indexedDocumentPath("/workspace/sessions/cancellation.md", [session])).toBe(session);
  expect(indexedDocumentPath("C:\\workspace\\sessions\\cancellation.md", [session])).toBe(session);
});
</storycode:copy>
<a id="f-8e8f891f853a-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-8e8f891f853a-expression_statement-2">
it("uses the complete indexed path rather than a shorter shared filename", () => {
  const root = { ...session, id: "root-doc", path: "cancellation.md" };
  expect(indexedDocumentPath("/workspace/sessions/cancellation.md", [root, session])).toBe(session);
  expect(indexedDocumentPath("cancellation.md", [session])).toBeUndefined();
});
</storycode:copy>
<a id="f-8e8f891f853a-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-8e8f891f853a-expression_statement-3">
it("does not admit unknown paths or guess between ambiguous index identities", () => {
  expect(indexedDocumentPath("sessions/missing.md", [session])).toBeUndefined();
  expect(indexedDocumentPath("../sessions/cancellation.md", [session])).toBeUndefined();
  expect(indexedDocumentPath("/workspace/sessions/cancellation.md", [session, { ...session, id: "other" }])).toBeUndefined();
});

</storycode:copy>
<a id="f-8e8f891f853a-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-8e8f891f853a-expression_statement-4">
it("prefers registered identities to the tree and preserves unknown paths", () => {
  expect(resolveDocumentPath(session.path, [session], [])).toEqual({path:session.path,id:session.id});
  expect(resolveDocumentPath("missing.md", [session], [])).toEqual({path:"missing.md"});
});
</storycode:copy>
<a id="f-8e8f891f853a-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-8e8f891f853a-expression_statement-5">
it("refreshes a newly recorded session once and opens only its registered identity", async () => {
  const actions={refresh:vi.fn().mockResolvedValue([session]),update:vi.fn(),open:vi.fn(),notice:vi.fn(),focusTree:vi.fn()};
  await openIndexedDocument(session.path,{path:session.path},"p",actions);
  expect(actions.refresh).toHaveBeenCalledExactlyOnceWith("p");
  expect(actions.update).toHaveBeenCalledWith([session]);
  expect(actions.open).toHaveBeenCalledExactlyOnceWith(session.id,session.path);
  await openIndexedDocument("missing.md",{path:"missing.md"},"p",actions);
  expect(actions.open).toHaveBeenCalledTimes(1);
  expect(actions.notice).toHaveBeenCalledWith(expect.stringContaining("not indexed"));
  expect(actions.focusTree).toHaveBeenCalledOnce();
});
</storycode:copy>
<a id="f-8e8f891f853a-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-8e8f891f853a-expression_statement-6">
it("does not refresh cached identities or admit paths after a failed refresh", async () => {
  const actions={refresh:vi.fn().mockRejectedValue(Error("offline")),update:vi.fn(),open:vi.fn(),notice:vi.fn(),focusTree:vi.fn()};
  await openIndexedDocument(session.path,{path:session.path,id:session.id},"p",actions);
  expect(actions.refresh).not.toHaveBeenCalled();
  expect(actions.open).toHaveBeenCalledOnce();
  await openIndexedDocument("missing.md",{path:"missing.md"},"p",actions);
  expect(actions.open).toHaveBeenCalledOnce();
  expect(actions.notice).toHaveBeenCalledWith(expect.stringContaining("offline"));
});
</storycode:copy>

### Assemble `apps/web/src/views/indexedDocumentPath.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/indexedDocumentPath.test.ts"><storycode:paste select="#f-8e8f891f853a-session"/><storycode:paste select="#f-8e8f891f853a-expression_statement"/><storycode:paste select="#f-8e8f891f853a-expression_statement-2"/><storycode:paste select="#f-8e8f891f853a-expression_statement-3"/><storycode:paste select="#f-8e8f891f853a-expression_statement-4"/><storycode:paste select="#f-8e8f891f853a-expression_statement-5"/><storycode:paste select="#f-8e8f891f853a-expression_statement-6"/></storycode:file>


</storycode:doc>
