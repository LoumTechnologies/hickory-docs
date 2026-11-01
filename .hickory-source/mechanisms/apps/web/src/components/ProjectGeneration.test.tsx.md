<gencode:doc xmlns:gencode="http://www.hickorydocs.com/1.0">
# Exercise explicit generation in the dialog

For: maintainers changing safe document opening and explicit generation.

Preview must not publish, and mounting the control must not even read the filesystem. Only a second explicit action sends publication with the captured revision.

<gencode:copy id="generation-dialog-preview-test">import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { request } from "../api/client";
import { ProjectGenerationDialog } from "./ProjectGeneration";
vi.mock("../api/client", () => ({ request: vi.fn() }));
afterEach(() => { cleanup(); vi.mocked(request).mockReset(); });
const captured = { root: "/chosen/project", entry: "project.md", revision: "captured-revision", outputs: ["src/app.py"] };
it("opening reads nothing; preview discloses the context and only confirmation generates", async () => {
  vi.mocked(request).mockResolvedValueOnce(captured).mockResolvedValueOnce({ written: ["src/app.py"] });
  render(<ProjectGenerationDialog onClose={() => {}} />);
  expect(request).not.toHaveBeenCalled();
  expect(screen.queryByRole("button", { name: "Generate 1 file" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Preview generation" }));
  await screen.findByText("/chosen/project");
  expect(request).toHaveBeenLastCalledWith("POST", "/api/generation/preview", { root: ".", entry: "project.md" });
  fireEvent.click(screen.getByRole("button", { name: "Generate 1 file" }));
  await screen.findByText("Generated 1 file.");
  expect(request).toHaveBeenLastCalledWith("POST", "/api/generation/generate", { root: ".", entry: "project.md", revision: "captured-revision" });
});
</gencode:copy>

Changing context retires the old plan. A stale-source refusal discards it and does not retry a potentially mutating request.

<gencode:copy id="generation-dialog-refusal-test">it("changing the target invalidates the preview and a stale-plan refusal never retries publication", async () => {
  vi.mocked(request).mockResolvedValueOnce(captured).mockResolvedValueOnce(captured).mockRejectedValueOnce(new Error("Project sources changed; preview generation again"));
  render(<ProjectGenerationDialog onClose={() => {}} />);
  fireEvent.click(screen.getByRole("button", { name: "Preview generation" }));
  await screen.findByRole("button", { name: "Generate 1 file" });
  fireEvent.change(screen.getByLabelText(/Entry document/), { target: { value: "other.md" } });
  expect(screen.queryByRole("button", { name: "Generate 1 file" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Preview generation" }));
  fireEvent.click(await screen.findByRole("button", { name: "Generate 1 file" }));
  await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("sources changed"));
  expect(request).toHaveBeenCalledTimes(3);
  expect(screen.queryByRole("button", { name: "Generate 1 file" })).toBeNull();
});
</gencode:copy>

<gencode:file path="apps/web/src/components/ProjectGeneration.test.tsx"><gencode:paste select="#generation-dialog-preview-test"/><gencode:paste select="#generation-dialog-refusal-test"/></gencode:file>
</gencode:doc>
