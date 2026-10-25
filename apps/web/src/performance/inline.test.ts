import { afterEach, expect, it, vi } from "vitest";
import { waitFor } from "@testing-library/react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { performanceInline } from "./inline";
import { selectPerformance } from "./selection";
import { selectedObservations } from "../debug/observations";
import type { PerformanceRun } from "./types";
vi.mock("../lib/reviews", () => ({ sha256: async (text: string) => text }));
let view: EditorView | undefined;
afterEach(() => { view?.destroy(); view = undefined; selectPerformance(null); document.body.replaceChildren(); });
// Guarantee: docs/guarantees/execution/profiles-keep-their-measured-scope.md
it("expands measured hotspots into shared agent context and clears them after edits", async () => {
  const source = "# App\nfunction hot() {}\n";
  const run = { id: "sampled-run", source_changed: false, source_state: "current", sources: { "app.md": source }, labels: ["manual"], identity: { collector: "V8", version: "1", source_hashes: { "app.md": source } }, mappings: {}, report: { limitations: ["sampling"], threads: [{ name: "main", unit: "seconds", measured_weight: 0.1, aggregation: "sampled", interval_start: 0, interval_end: 0.1, rows: [{ frame: { name: "hot", file: "app.md", line: 2 }, self_weight: 0.02, inclusive_weight: 0.1, samples: 20, total_calls: null, primitive_calls: null }] }] } } as unknown as PerformanceRun;
  const host = document.createElement("div"); document.body.append(host);
  view = new EditorView({ parent: host, state: EditorState.create({ doc: source, extensions: [performanceInline(() => "app.md")] }) }); selectPerformance(run);
  await waitFor(() => expect(host.querySelector(".performance-inline summary")?.textContent).toContain("hot"));
  expect(selectedObservations("app.md", source)).toHaveLength(0);
  const details = host.querySelector("details")!; details.open = true; details.dispatchEvent(new Event("toggle"));
  expect(selectedObservations("app.md", source)[0].reading.annotations[0].payload).toMatchObject({ row: { total_calls: null, samples: 20 }, unit: "seconds" });
  expect(host.querySelector("table")?.textContent).toContain("unavailable / unavailable");
  view.dispatch({ changes: { from: 0, to: 1, insert: "changed" } });
  await waitFor(() => expect(host.querySelector(".performance-inline")).toBeNull());
  expect(selectedObservations("app.md", source)).toHaveLength(0);
});
