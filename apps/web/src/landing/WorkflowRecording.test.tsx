import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
// @ts-expect-error Test runtime has Node crypto; browser code does not import it.
import { webcrypto } from "node:crypto";
import { WorkflowRecording } from "./WorkflowRecording";
import { sha256 } from "../lib/reviews";
afterEach(() => { cleanup(); vi.unstubAllGlobals(); });
async function setup(recording = false, wrongRevision = false) {
  vi.stubGlobal("crypto", webcrypto);
  const source = "# Public requirement\n", hash = await sha256(source);
  const snapshot = { format: "hickory-viewer-snapshot", version: 1, path: "examples/ai-engineer-deadline.md", source, source_hash: hash, captured_at_ms: 1, execution: "Test fixture", cells: [], verification: { status: "failed", revision: "a".repeat(40), source_hash: hash, commands: [], requirements: [], origin: "Test fixture", scope: "Test fixture", dependencies: {}, assets: {}, environment: {} }, ...(recording ? { demonstration: { capture_kind: "native-provider", captured_revision: "b".repeat(40), recording_asset: "unselected.webm" } } : {}) };
  const fetcher = vi.fn().mockResolvedValueOnce(new Response(JSON.stringify({ version: 1, documents: [{ path: snapshot.path, snapshot: "selected/snapshot.json", source_hash: hash, revision: (wrongRevision ? "c" : "a").repeat(40) }] }), { status: 200 })).mockResolvedValueOnce(new Response(JSON.stringify(snapshot), { status: 200 }));
  vi.stubGlobal("fetch", fetcher); return fetcher;
}
// Guarantee: docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md
it("does not manufacture a recording or turn a published failure into a pass", async () => {
  const fetcher = await setup(); render(<WorkflowRecording />);
  expect(fetcher).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Load reviewed workflow recording" }));
  await screen.findByText(/A real native\/provider recording has not been published/);
  expect(screen.getByText(/result failed/)).toBeTruthy();
  expect(document.querySelector("video")).toBeNull(); expect(fetcher).toHaveBeenCalledTimes(2);
});
it("rejects media absent from the reviewed manifest before downloading it", async () => {
  const fetcher = await setup(true); render(<WorkflowRecording />);
  fireEvent.click(screen.getByRole("button", { name: "Load reviewed workflow recording" }));
  await screen.findByText(/Invalid reviewed demonstration assets/);
  expect(document.querySelector("video")).toBeNull(); expect(fetcher).toHaveBeenCalledTimes(2);
});
it("rejects mismatched publication revisions", async () => {
  const fetcher = await setup(false, true); render(<WorkflowRecording />);
  fireEvent.click(screen.getByRole("button", { name: "Load reviewed workflow recording" }));
  await screen.findByText(/Publication index and snapshot revisions differ/);
  expect(document.querySelector("video")).toBeNull(); expect(fetcher).toHaveBeenCalledTimes(2);
});
