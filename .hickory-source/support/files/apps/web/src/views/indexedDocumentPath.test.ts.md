<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/indexedDocumentPath.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `9485a86995e18c2bde32ce671c4c970416fda123cd985a953b9c805dbc1065b5`.

<book:entry path="apps/web/src/views/indexedDocumentPath.test.ts">
import { expect, it, vi } from "vitest";
import { indexedDocumentPath, openIndexedDocument, resolveDocumentPath } from "./indexedDocumentPath";
const session = { id: "recorded-session", path: "sessions/cancellation.md", updated_at: "" };
it("resolves an indexed ignored session without a visible tree entry", () => {
  expect(indexedDocumentPath(session.path, [session])).toBe(session);
  expect(indexedDocumentPath("./sessions/cancellation.md", [session])).toBe(session);
  expect(indexedDocumentPath("/workspace/sessions/cancellation.md", [session])).toBe(session);
  expect(indexedDocumentPath("C:\\workspace\\sessions\\cancellation.md", [session])).toBe(session);
});
it("uses the complete indexed path rather than a shorter shared filename", () => {
  const root = { ...session, id: "root-doc", path: "cancellation.md" };
  expect(indexedDocumentPath("/workspace/sessions/cancellation.md", [root, session])).toBe(session);
  expect(indexedDocumentPath("cancellation.md", [session])).toBeUndefined();
});
it("does not admit unknown paths or guess between ambiguous index identities", () => {
  expect(indexedDocumentPath("sessions/missing.md", [session])).toBeUndefined();
  expect(indexedDocumentPath("../sessions/cancellation.md", [session])).toBeUndefined();
  expect(indexedDocumentPath("/workspace/sessions/cancellation.md", [session, { ...session, id: "other" }])).toBeUndefined();
});

it("prefers registered identities to the tree and preserves unknown paths", () => {
  expect(resolveDocumentPath(session.path, [session], [])).toEqual({path:session.path,id:session.id});
  expect(resolveDocumentPath("missing.md", [session], [])).toEqual({path:"missing.md"});
});
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
it("does not refresh cached identities or admit paths after a failed refresh", async () => {
  const actions={refresh:vi.fn().mockRejectedValue(Error("offline")),update:vi.fn(),open:vi.fn(),notice:vi.fn(),focusTree:vi.fn()};
  await openIndexedDocument(session.path,{path:session.path,id:session.id},"p",actions);
  expect(actions.refresh).not.toHaveBeenCalled();
  expect(actions.open).toHaveBeenCalledOnce();
  await openIndexedDocument("missing.md",{path:"missing.md"},"p",actions);
  expect(actions.open).toHaveBeenCalledOnce();
  expect(actions.notice).toHaveBeenCalledWith(expect.stringContaining("offline"));
});
</book:entry>

</book:doc>
