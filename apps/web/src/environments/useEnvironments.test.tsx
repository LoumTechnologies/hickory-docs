// Protects docs/guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { environmentApi, type EnvironmentFinding } from "../api/environments";
import { useEnvironments } from "./useEnvironments";
import { onShowTerminal } from "../lib/revealLine";

vi.mock("../api/environments", () => ({ environmentApi: { inspect: vi.fn(), act: vi.fn(), choose: vi.fn() } }));
const finding: EnvironmentFinding = {
  project: "/project", manager: "uv", target: "host", profile: "manager defaults", revision: "one",
  state: "environment-missing", message: "Missing dependencies", details: "", actions: [], install_url: null,
  interpreter: null, manager_choices: [],
};
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(environmentApi.inspect).mockResolvedValue({ findings: [finding], running: {} });
});
afterEach(cleanup);

it("dismisses only the observed evidence revision and clears a successful recheck", async () => {
  const { result } = renderHook(useEnvironments);
  await waitFor(() => expect(result.current.notices).toHaveLength(1));
  act(() => result.current.dismiss(finding));
  await act(() => result.current.refresh(true));
  expect(result.current.notices).toHaveLength(0);
  vi.mocked(environmentApi.inspect).mockResolvedValue({ findings: [{ ...finding, revision: "two" }], running: {} });
  await act(() => result.current.refresh(true));
  expect(result.current.notices).toHaveLength(1);
  vi.mocked(environmentApi.inspect).mockResolvedValue({ findings: [{ ...finding, state: "ready", revision: "three" }], running: {} });
  await act(() => result.current.refresh(true));
  expect(result.current.notices).toHaveLength(0);
});

it("opens the returned terminal and reports failed repairs without hiding the finding", async () => {
  const show = vi.fn(); const unsubscribe = onShowTerminal(show);
  const { result } = renderHook(useEnvironments);
  await waitFor(() => expect(result.current.notices).toHaveLength(1));
  vi.mocked(environmentApi.act).mockResolvedValue({ id: "session", title: "uv: sync" } as never);
  await act(() => result.current.act(finding, "sync"));
  expect(show).toHaveBeenCalledWith({ id: "session", title: "uv: sync" });
  vi.mocked(environmentApi.act).mockRejectedValue(new Error("The environment changed. Recheck."));
  await act(() => result.current.act(finding, "sync"));
  expect(result.current.error).toBe("The environment changed. Recheck.");
  expect(result.current.notices).toHaveLength(1);
  unsubscribe();
});
