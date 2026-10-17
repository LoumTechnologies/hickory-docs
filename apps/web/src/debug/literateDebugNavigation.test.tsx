import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { IDLE_SESSION } from "./useDebugger";
import { ensurePlainDebugHost, PlainDebugHosts, resetPlainDebugHosts } from "./plainDebugHosts";
import { clearPreferredDebugSurface, preferDebugSurface, registerDebugSurface } from "./preferredDebugLocation";
import { OPEN_LOCATION_EVENT } from "../lib/revealLine";

const state = vi.hoisted(() => ({ debug: {} }));
vi.mock("./useDebugger", async importOriginal => ({
  ...await importOriginal<typeof import("./useDebugger")>(),
  useWorkspaceDebugger: () => state.debug,
}));
afterEach(() => { cleanup(); resetPlainDebugHosts(); clearPreferredDebugSurface("main.py"); });

// Guarantee: docs/guarantees/debugging/stepping-prefers-the-literate-view.md
it("lets the launching literate surface consume step navigation before opening a plain file", () => {
  const frame = { id: 1, name: "helper", source: "helper.py", source_line: 2, line: null, in_document: false };
  state.debug = { ...IDLE_SESSION, status: "paused", frames: [frame] };
  const reveal = vi.fn(() => true), open = vi.fn();
  const unregister = registerDebugSurface("view", reveal);
  preferDebugSurface("main.py", "view");
  window.addEventListener(OPEN_LOCATION_EVENT, open);
  ensurePlainDebugHost("main.py");
  const mounted = render(<PlainDebugHosts />);
  expect(reveal).toHaveBeenCalledWith(frame);
  expect(open).not.toHaveBeenCalled();
  reveal.mockReturnValue(false);
  act(() => { state.debug = { ...state.debug, frames: [{ ...frame, source_line: 3 }] }; });
  mounted.rerender(<PlainDebugHosts />);
  expect(open).toHaveBeenCalledOnce();
  window.removeEventListener(OPEN_LOCATION_EVENT, open);
  unregister();
});
