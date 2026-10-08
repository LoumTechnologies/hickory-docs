// Guarantee: docs/guarantees/agent/assumptions-remain-auditable-after-the-work.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { AssumptionAudit, VerificationStanding, type AuditRecord } from "./AssumptionAudit";
import { request } from "../api/client";
vi.mock("../api/client", () => ({ request: vi.fn() }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
const records: AuditRecord[] = [
  { record_kind: "audit", id: "earlier", actor: "model-declared", turn: "abandoned", session_line: 2, entry: { statement: "Assumed dollars", reason: "Choose units", evidence: [], affects: ["totals.csv"], alternatives: ["euros"], falsifier: "Header labels euros", exclusions: [] } },
  { record_kind: "audit", id: "correction", actor: "model-declared", turn: "current", parent: "base", session_line: 8, entry: { statement: "Amounts are euros", reason: "Read header", supersedes: "earlier", evidence: ["input.csv"], affects: [], alternatives: [], falsifier: "Header corrected", exclusions: [] } },
  { record_kind: "read-snapshot", path: "input.csv", session_line: 4, first_line: 1, last_line: 1, content: "Historical euros\n", sha256: "old", current_state: "changed" },
  { record_kind: "verification", state: "failed", actor: "harness", session_line: 9, revision: "abc", scope: "Document expectations" },
];
it("keeps assumptions from abandoned branches and corrections visible without certifying completeness", () => {
  render(<AssumptionAudit records={records} path="sessions/demo.md" />);
  expect(screen.getByText("Assumed dollars")).toBeTruthy();
  expect(screen.getByText("Amounts are euros")).toBeTruthy();
  expect(screen.getByText(/completeness is not established/)).toBeTruthy();
  expect(screen.getByText(/parent base/)).toBeTruthy();
  fireEvent.change(screen.getByLabelText("Search assumption audit"), { target: { value: "dollars" } });
  expect(screen.queryByText("Amounts are euros")).toBeNull();
});
it("shows historical bytes and changed standing, then shows failed execution separately", () => {
  render(<AssumptionAudit records={records} path="sessions/demo.md" />);
  fireEvent.change(screen.getByLabelText("Audit records"), { target: { value: "read-snapshot" } });
  expect(screen.getByText("Historical euros")).toBeTruthy();
  expect(screen.getByText(/current source changed/)).toBeTruthy();
  fireEvent.change(screen.getByLabelText("Audit records"), { target: { value: "verification" } });
  expect(screen.getByText(/failed · current sources unchecked · revision abc/)).toBeTruthy();
});
it("appends a user correction referencing the original instead of changing it", async () => {
  vi.mocked(request).mockResolvedValue({ recorded: "new entry" });
  render(<AssumptionAudit records={records} path="sessions/demo.md" />);
  fireEvent.change(screen.getByLabelText("Statement"), { target: { value: "Corrected units" } });
  fireEvent.change(screen.getByLabelText("Reason"), { target: { value: "Inspected the invoice" } });
  fireEvent.change(screen.getByLabelText("Supersedes entry id (optional)"), { target: { value: "earlier" } });
  fireEvent.click(screen.getByRole("button", { name: "Record audit note", hidden: true }));
  await waitFor(() => expect(request).toHaveBeenCalledWith("POST", "/api/sessions/audit", { path: "sessions/demo.md", entry: { statement: "Corrected units", reason: "Inspected the invoice", supersedes: "earlier" } }));
  expect(screen.getByText("Assumed dollars")).toBeTruthy();
});

it("keeps a historical pass visibly stale and excludes another branch's pass", () => {
  const checks: AuditRecord[] = [{ record_kind: "verification", turn: "selected", state: "passed", current_state: "changed", revision: "old", session_line: 5 }, { record_kind: "verification", turn: "abandoned", state: "passed", current_state: "unchanged", revision: "other", session_line: 7 }];
  render(<VerificationStanding records={checks} turn="selected" running={false} />);
  expect(screen.getByText("historical pass · current sources changed")).toBeTruthy();
  expect(screen.queryByText("other")).toBeNull();
});

it("shows recording gaps even while filtering declarations", () => {
  render(<AssumptionAudit records={[{ record_kind: "audit-gap", state: "unreadable", session_line: 7 }]} path="sessions/demo.md" />);
  expect(screen.getByRole("alert", { hidden: true }).textContent).toContain("Coverage is incomplete");
});
