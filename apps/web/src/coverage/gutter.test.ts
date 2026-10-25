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
