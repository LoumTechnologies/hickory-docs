<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/attribution.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Authorship display accepts non-Git providers](../../../../../contract.md)

Captured SHA-256: `5bb1639b170f8eaa739c44a4ecf3934b1725e9ff9ad7c8bc1fe85d741f708764`.

<book:entry path="apps/web/src/lib/attribution.test.ts">
import { afterEach, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { authorColor, attributionProvider, installAttributionProvider } from "./attribution";
import { blameGutter, authorshipField, setBlame, setAuthorshipShown } from "../editor/blameGutter";
afterEach(() => localStorage.clear());
// Guarantee: docs/guarantees/lineage/authorship-display-accepts-non-git-providers.md
it("accepts a non-Git provider and restores Git on removal", async () => {
  const remove = installAttributionProvider({ id: "receipts", label: "Editor receipts", lines: async () => [] });
  expect(attributionProvider().id).toBe("receipts");
  expect(await attributionProvider().lines("note.md")).toEqual([]);
  remove(); expect(attributionProvider().id).toBe("git");
});
it("clears attribution when the attributed text changes", () => {
  let state = EditorState.create({ doc: "agent words", extensions: blameGutter() });
  state = state.update({ effects: [setAuthorshipShown.of(true), setBlame.of([{ line: 1, commit: "abc", author: "Sam", email: "sam@example.com", time: 0, summary: "", uncommitted: false, origin: "agent" }])] }).state;
  expect(state.field(authorshipField).marks.size).toBe(1);
  state = state.update({ changes: { from: 0, to: 5, insert: "human" } }).state;
  expect(state.field(authorshipField).marks.size).toBe(0);
});
it("keeps credited-author colors stable", () => {
  expect(authorColor("sam@example.com")).toBe(authorColor("sam@example.com"));
  expect(authorColor("sam@example.com")).toBeGreaterThanOrEqual(0);
});
</book:entry>

</book:doc>
