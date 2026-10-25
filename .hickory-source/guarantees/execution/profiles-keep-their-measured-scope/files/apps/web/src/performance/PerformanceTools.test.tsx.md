<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/performance/PerformanceTools.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Profiles keep their measured scope](../../../../../contract.md)

Captured SHA-256: `ae0a53cb120c9ce9c2749a32f16faf04534150c20e43c563b8fbc2cdf2103b3d`.

<book:entry path="apps/web/src/performance/PerformanceTools.test.tsx">
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { PerformancePanel } from "./PerformanceTools";
import { request } from "../api/client";
vi.mock("../api/client", () => ({ request: vi.fn() }));
vi.mock("../lib/revealLine", () => ({ openLocation: vi.fn() }));
afterEach(cleanup);
const run = { id: "1".repeat(32), mode: "manual-browser", status: "completed", source_state: "historical", source_changed: false, started_ms: 1, labels: [], sources: {}, mappings: {}, identity: { collector: "V8", version: "1", scope: ".", source_hashes: {} }, report: { limitations: ["sampling"], raw: "{}", format: "chrome-cpu", threads: [{ name: "main", unit: "seconds", measured_weight: 0.2, interval_start: 0, interval_end: 0.2, aggregation: "sampled", rows: [{ frame: { name: "hot", file: null, line: null }, self_weight: 0.1, inclusive_weight: 0.2, samples: 12, total_calls: null, primitive_calls: null }] }] }, output: "" };
// Guarantee: docs/guarantees/execution/profiles-keep-their-measured-scope.md
it("keeps sampled calls unavailable and unresolved native frames visible", async () => {
  vi.mocked(request).mockImplementation(async (_method, _path, body) => ((body as { action: string }).action === "list" ? { runs: [run] } : run) as never);
  render(<PerformancePanel close={() => {}} />);
  fireEvent.click(await screen.findByRole("button", { name: /manual-browser · completed/ }));
  expect(await screen.findByText("V8 · completed · historical source")).toBeTruthy();
  expect((screen.getByRole("button", { name: /hot · native\/unresolved/ }) as HTMLButtonElement).disabled).toBe(true);
  expect(screen.getByText("unavailable / unavailable")).toBeTruthy();
});
it("starts real manual profiling through the shared endpoint", async () => {
  vi.mocked(request).mockReset(); vi.mocked(request).mockImplementation(async (_method, _path, body) => ((body as { action: string }).action === "list" ? { runs: [] } : run) as never);
  render(<PerformancePanel close={() => {}} />);
  fireEvent.change(screen.getByLabelText("Page URL contains"), { target: { value: "localhost:3000" } });
  fireEvent.click(screen.getByRole("button", { name: "Start browser CPU profile" }));
  await waitFor(() => expect(request).toHaveBeenCalledWith("POST", "/api/performance", expect.objectContaining({ action: "browser-start", selector: "localhost:3000", url: "http://127.0.0.1:9222" })));
});
</book:entry>

</book:doc>
