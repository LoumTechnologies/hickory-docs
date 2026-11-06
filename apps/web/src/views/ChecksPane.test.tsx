// Contract: .hickory-source/mechanisms/optional-checks.md
import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { ChecksPane } from "./ChecksPane";
vi.mock("../api/client", () => ({ api: { checks: vi.fn() } }));
afterEach(() => { cleanup(); vi.useRealTimers(); vi.resetAllMocks(); });
it("keeps results visibly stale through provider failure", async () => {
  vi.useFakeTimers();
  vi.mocked(api.checks).mockResolvedValueOnce({ availability: "ready", message: null, runs: [
    { id: "r1", parent: null, name: "just check", status: "passed", revision: "abcdef", created_ms: 1, code: 0 },
  ] }).mockResolvedValue({ availability: "unavailable", message: "Start local-ci service", runs: [] });
  render(<ChecksPane />);
  await act(async () => {});
  expect(screen.getByText("passed (0)")).toBeTruthy();
  await act(async () => { await vi.advanceTimersByTimeAsync(3000); });
  expect(screen.getByText(/may be stale/)).toBeTruthy();
  expect(screen.getByText("passed (0)")).toBeTruthy();
  expect(screen.getByText("Start local-ci service")).toBeTruthy();
});
it("shows disabled setup and retires late requests", async () => {
  vi.mocked(api.checks).mockResolvedValue({ availability: "disabled", message: "Use just checks-configure", runs: [] });
  const view = render(<ChecksPane />);
  expect(await screen.findByText("Use just checks-configure")).toBeTruthy();
  view.unmount();
  let finish!: (value: Awaited<ReturnType<typeof api.checks>>) => void;
  vi.mocked(api.checks).mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  const late = render(<ChecksPane />);
  late.unmount();
  await act(async () => { finish({ availability: "ready", message: null, runs: [] }); });
  expect(screen.queryByText("No recent checks for this folder.")).toBeNull();
});
