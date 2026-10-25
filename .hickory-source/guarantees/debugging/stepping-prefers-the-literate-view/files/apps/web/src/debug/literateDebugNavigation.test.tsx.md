<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/literateDebugNavigation.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Stepping prefers the launching literate view](../../../../../contract.md)

Captured SHA-256: `f6cceab8984c9ade1e03300f8209c14b498a3ab0d834552a16c22b49d17606a7`.

<book:entry path="apps/web/src/debug/literateDebugNavigation.test.tsx">
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { retainCapturedContext } from "../lib/capturedContexts";
import { IDLE_SESSION } from "./useDebugger";
import { ensurePlainDebugHost, PlainDebugHosts, resetPlainDebugHosts } from "./plainDebugHosts";
import { clearPreferredDebugSurface, preferDebugSurface, registerDebugSurface } from "./preferredDebugLocation";
import { OPEN_LOCATION_EVENT } from "../lib/revealLine";

vi.mock("../lib/capturedContexts", () => ({retainCapturedContext:vi.fn(() => vi.fn())}));

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

it("retains a captured project continuously across running and paused states", () => {
  const release = vi.fn();
  vi.mocked(retainCapturedContext).mockClear().mockReturnValue(release);
  state.debug = {...IDLE_SESSION, status:"starting"};
  ensurePlainDebugHost("hick-project:///test/app.py");
  const mounted = render(<PlainDebugHosts/>);
  for (const status of ["running","paused","running","paused"]) {
    act(() => {state.debug = {...IDLE_SESSION,status};});
    mounted.rerender(<PlainDebugHosts/>);
    expect(release).not.toHaveBeenCalled();
  }
  expect(retainCapturedContext).toHaveBeenCalledOnce();
  act(() => {state.debug = {...IDLE_SESSION,status:"finished"};});
  mounted.rerender(<PlainDebugHosts/>);
  expect(release).toHaveBeenCalledOnce();
});
</book:entry>

</book:doc>
