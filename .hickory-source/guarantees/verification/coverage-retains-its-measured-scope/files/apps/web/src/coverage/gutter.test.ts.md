<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/coverage/gutter.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Coverage retains its measured scope](../../../../../contract.md)

Captured SHA-256: `7fbbbe939e42f3df734893334dc1a33715de7585adc99e6e1de8a5822048676b`.

<book:entry path="apps/web/src/coverage/gutter.test.ts">
import { afterEach, expect, it, vi } from "vitest";
import { waitFor } from "@testing-library/react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { coverageGutter } from "./gutter";
import { selectCoverage } from "./selection";
import type { CoverageRun } from "./types";
vi.mock("../lib/reviews", () => ({ sha256: async (text: string) => text }));
let view: EditorView | undefined;
afterEach(() => { view?.destroy(); view = undefined; selectCoverage(null); document.body.replaceChildren(); });
// Guarantee: docs/guarantees/verification/coverage-retains-its-measured-scope.md
it("draws measured code only for matching bytes and removes marks on buffer edits", async () => {
  const source = "accept()\nreject()\n";
  const run = { source_changed_during_run: false, source_state: "current", id: "run", labels: ["happy-path"], identity: { source_hashes: { "app.ts": source } }, evidence: { attribution: "group", report: { files: { "app.ts": { lines: { 1: 1, 2: 0 }, line_counts: true } } } }, mappings: {} } as unknown as CoverageRun;
  const host = document.createElement("div"); document.body.append(host);
  view = new EditorView({ parent: host, state: EditorState.create({ doc: source, extensions: [coverageGutter(() => "app.ts")] }) });
  selectCoverage(run);
  await waitFor(() => expect(host.querySelectorAll(".cm-coverage-hit")).toHaveLength(1));
  expect(host.querySelectorAll(".cm-coverage-miss")).toHaveLength(1);
  expect(host.querySelector(".cm-coverage-gutter")?.textContent).toContain("○");
  view.dispatch({ changes: { from: 0, to: 6, insert: "changed" } });
  await waitFor(() => expect(host.querySelectorAll(".cm-coverage-hit, .cm-coverage-miss")).toHaveLength(0));
});
it("allows buffer edits when no coverage is selected without nested editor updates", async () => {
  selectCoverage(null);
  const host = document.createElement("div"); document.body.append(host);
  view = new EditorView({ parent: host, state: EditorState.create({ doc: "original", extensions: [coverageGutter(() => "app.ts")] }) });
  view.dispatch({ changes: { from: 0, to: 8, insert: "edited" } });
  await new Promise(resolve => setTimeout(resolve, 0));
  expect(view.state.doc.toString()).toBe("edited");
  expect(host.querySelectorAll(".cm-coverage-hit, .cm-coverage-miss")).toHaveLength(0);
});
</book:entry>

</book:doc>
