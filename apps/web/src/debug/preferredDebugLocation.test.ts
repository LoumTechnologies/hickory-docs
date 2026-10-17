import { expect, it, vi } from "vitest";
import { clearPreferredDebugSurface, preferDebugSurface, registerDebugSurface, revealPreferredDebugLocation, representedFrameOffset } from "./preferredDebugLocation";
import { type RepresentedFile } from "../lsp/representationMapping";
import type { Frame } from "./client";

// Guarantee: docs/guarantees/debugging/stepping-prefers-the-literate-view.md
it("keeps a callee represented in the launching view and falls back for absent lines", () => {
  const source = "# Helper\nreturn 2\n";
  const file: RepresentedFile = { path: "helper.py", hash: "", content: "return 2\n", provenance: [
    { start: 0, end: 9, origin: { kind: "literal", doc_path: "view", span: [9, 18] } },
  ] };
  const reveal = vi.fn();
  const unregister = registerDebugSurface("view", frame => {
    const offset = representedFrameOffset(source, [file], frame);
    if (offset === null) return false;
    reveal(offset);
    return true;
  });
  const frame: Frame = { id: 1, name: "helper", source: "helper.py", source_line: 0, line: null, in_document: false };
  preferDebugSurface("main.py", "view");
  expect(revealPreferredDebugLocation("main.py", frame)).toBe(true);
  expect(reveal).toHaveBeenCalledWith(9);
  expect(revealPreferredDebugLocation("main.py", { ...frame, source: "library.py" })).toBe(false);
  expect(revealPreferredDebugLocation("main.py", { ...frame, source_line: 10 })).toBe(false);
  expect(revealPreferredDebugLocation("other.py", frame)).toBe(false);
  clearPreferredDebugSurface("main.py");
  expect(revealPreferredDebugLocation("main.py", frame)).toBe(false);
  preferDebugSurface("main.py", "view");
  unregister();
  expect(revealPreferredDebugLocation("main.py", frame)).toBe(false);
  clearPreferredDebugSurface("main.py");
});
