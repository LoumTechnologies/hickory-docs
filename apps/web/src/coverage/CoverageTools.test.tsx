import { cleanup, render, screen, fireEvent, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { CoveragePanel } from "./CoverageTools";
import { coverageCall } from "./client";
vi.mock("./client", () => ({ coverageCall: vi.fn() }));
vi.mock("../lib/revealLine", () => ({ openLocation: vi.fn() }));
const run = { id: "a".repeat(32), mode: "manual", status: "completed", source_state: "historical", started_ms: 1, labels: ["adversarial"], project: ".", language: "typescript", identity: { collector: "istanbul-browser", version: "v1", source_hashes: {} }, tests: [], output: "", evidence: null, metadata: { groups: {}, manual: {}, tests: {} }, mappings: {}, notes: "", manual_verdict: null, argv: [], checkpoints: [] };
afterEach(cleanup);
beforeEach(() => { vi.mocked(coverageCall).mockReset(); vi.mocked(coverageCall).mockImplementation(async body => (body.action === "list" ? { runs: [run] } : run) as never); });
// Guarantee: docs/guarantees/verification/coverage-retains-its-measured-scope.md
it("keeps missing coverage and historical source distinct from a failed manual verdict", async () => {
  render(<CoveragePanel close={() => {}} />);
  fireEvent.click(await screen.findByRole("button", { name: /manual · completed/ }));
  expect(await screen.findByText("Coverage unavailable. This is not zero coverage.")).toBeTruthy();
  expect(screen.getByText("manual · completed · historical source")).toBeTruthy();
  expect(screen.getByText("Recorded verdict: none")).toBeTruthy();
  fireEvent.change(screen.getByLabelText("Manual workflow verdict"), { target: { value: "failed" } });
  fireEvent.change(screen.getByLabelText("Observation / prerequisites"), { target: { value: "Wrong rejection message" } });
  fireEvent.click(screen.getByRole("button", { name: "Record manual verdict" }));
  await waitFor(() => expect(coverageCall).toHaveBeenCalledWith(expect.objectContaining({ action: "annotate", verdict: "failed", notes: "Wrong rejection message" })));
});
it("offers human and AI browser collection through the same service", async () => {
  render(<CoveragePanel close={() => {}} />);
  fireEvent.click(screen.getByText("Human or AI manual testing"));
  fireEvent.change(screen.getByLabelText("Page URL contains"), { target: { value: "localhost:3000" } });
  fireEvent.click(screen.getByRole("button", { name: "Start browser coverage" }));
  await waitFor(() => expect(coverageCall).toHaveBeenCalledWith(expect.objectContaining({ action: "manual-start", url: "http://127.0.0.1:9222", selector: "localhost:3000" })));
});
