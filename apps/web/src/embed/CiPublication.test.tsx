import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { CiPublicationView, decodeCiPublication, type CiPublication } from "./CiPublication";
vi.mock("./DocumentEmbed", () => ({ DocumentEmbed: ({ source, readOnly, onChange }: { source: string; readOnly: boolean; onChange: (change: { source: string }) => void }) => <textarea aria-label="Document" value={source} readOnly={readOnly} onChange={event => onChange({ source: event.target.value })} /> }));
vi.mock("../lib/reviews", () => ({ sha256: vi.fn(async (source: string) => source === "original" ? "a".repeat(64) : "b".repeat(64)) }));
afterEach(cleanup);
const publication: CiPublication = { format: "hickory-ci-publication", version: 1, title: "Local run", captured_at: "2026-10-26T00:00:00Z", base_revision: "a".repeat(40), tested_revision: null, scope: "Historical only", checks: [
  { id: "first", owner: "behavior/contract.md", source: "original", source_hash: "a".repeat(64), status: "failed", argv: ["python3", "test.py"], needs: [], assertion: "exit-zero", exit_code: 1, elapsed_seconds: 0.1, logs: { stderr: "actual failure" } },
  { id: "second", owner: "another/contract.md", source: "original", source_hash: "a".repeat(64), status: "passed", argv: ["python3"], needs: [], assertion: "exit-zero", exit_code: 0, elapsed_seconds: 0.1, logs: {} },
] };
it("rejects tampered source and duplicate check identities", async () => {
  await expect(decodeCiPublication(publication)).resolves.toBe(publication);
  await expect(decodeCiPublication({ ...publication, checks: [{ ...publication.checks[0], source: "changed" }] })).rejects.toThrow("hash");
  await expect(decodeCiPublication({ ...publication, checks: [publication.checks[0], publication.checks[0]] })).rejects.toThrow("Invalid");
});
it("keeps historical results separate from a disposable draft", () => {
  render(<CiPublicationView publication={publication} />);
  const document = screen.getByLabelText("Document") as HTMLTextAreaElement;
  expect(document.readOnly).toBe(true);
  expect(screen.getByText("actual failure")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Try editing a local draft" }));
  const draft = screen.getByLabelText("Document") as HTMLTextAreaElement;
  expect(draft.readOnly).toBe(false);
  fireEvent.change(draft, { target: { value: "my draft" } });
  expect(screen.getByRole("status").textContent).toContain("have not been tested");
  fireEvent.click(screen.getByRole("button", { name: /passed.*second/ }));
  expect((screen.getByLabelText("Document") as HTMLTextAreaElement).value).toBe("original");
  expect((screen.getByLabelText("Document") as HTMLTextAreaElement).readOnly).toBe(true);
});
