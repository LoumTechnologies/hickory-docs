// docs/guarantees/collaboration/the-merged-view-is-a-lens.md
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { api, installMockHandler } from "../api/client";
import { MergedView } from "./MergedView";

beforeEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

afterEach(() => installMockHandler(null as never));

const worktrees = (n = 2) =>
  vi.spyOn(api, "worktrees").mockResolvedValue({
    repository: n > 0,
    worktrees: [
      { path: "/repo", name: "repo", branch: "master", current: true },
      { path: "/side", name: "side-tree", branch: "side", current: false },
    ].slice(0, n),
  });

const merged = (over: Partial<Parameters<typeof api.merged> extends never ? never : any> = {}) =>
  vi.spyOn(api, "merged").mockResolvedValue({
    repository: true,
    path: "shared.txt",
    sources: [
      { path: "/repo", name: "repo", branch: "master", current: true },
      { path: "/side", name: "side-tree", branch: "side", current: false },
    ],
    regions: [
      { kind: "shared", text: "head\n" },
      {
        kind: "variant",
        by_source: { repo: "master only\n", "side-tree": "side only\n" },
      },
      { kind: "shared", text: "tail\n" },
    ],
    missing: [],
    shared_lines: 2,
    variants: 1,
    read_only: true,
    ...over,
  });

describe("the merged view", () => {
  it("shows agreed regions once and differences per source", async () => {
    worktrees();
    merged();
    render(<MergedView path="shared.txt" />);
    await waitFor(() => expect(screen.getByText(/head/)).toBeTruthy());
    // The agreed lines appear ONCE, not once per source.
    expect(screen.getAllByText(/^head$/m)).toHaveLength(1);
    expect(screen.getByText(/master only/)).toBeTruthy();
    expect(screen.getByText(/side only/)).toBeTruthy();
  });

  it("says it is a lens rather than a document you could save", async () => {
    // Getting this wrong reverses the product's central claim.
    worktrees();
    merged();
    render(<MergedView path="shared.txt" />);
    await waitFor(() => expect(screen.getByText(/lens · read-only/)).toBeTruthy());
  });

  it("scopes agreement to the capture and says later writes recheck target bytes", async () => {
    worktrees();
    merged();
    render(<MergedView path="shared.txt" />);
    await waitFor(() =>
      expect(screen.getByText(/agreed by construction for this capture/)).toBeTruthy(),
    );
    expect(screen.getByText(/recheck each target’s captured bytes/)).toBeTruthy();
  });

  it("names a worktree that does not have the file rather than dropping it", async () => {
    worktrees();
    merged({ missing: ["side-tree"] });
    render(<MergedView path="only-here.txt" />);
    await waitFor(() =>
      expect(screen.getByText(/does not\s+have this file at all/)).toBeTruthy(),
    );
    expect(screen.getByText(/an answer, not an omission/)).toBeTruthy();
  });

  it("says what a one-worktree repository would show, rather than an empty pane", async () => {
    worktrees(1);
    merged({ sources: [], regions: [], shared_lines: 0, variants: 0 });
    render(<MergedView path="shared.txt" />);
    await waitFor(() =>
      expect(screen.getByText(/would just\s+be the file/)).toBeTruthy(),
    );
    expect(screen.getByText(/git worktree add/)).toBeTruthy();
  });

  it("says plainly when a folder is not a repository", async () => {
    vi.spyOn(api, "worktrees").mockResolvedValue({ repository: false, worktrees: [] });
    vi.spyOn(api, "merged").mockResolvedValue({
      repository: false,
      sources: [],
      regions: [],
      missing: [],
    });
    render(<MergedView path="shared.txt" />);
    await waitFor(() =>
      expect(screen.getByText(/not a git repository with worktrees/)).toBeTruthy(),
    );
  });
});

it("routes a native-opened region through real agent draft edits, host callback replies and native guarded undo", async () => {
  worktrees();
  const original = { repository: true, path: "shared.txt", sources: [
    { path: "/repo", name: "repo", current: true }, { path: "/side", name: "side-tree", current: false },
  ], regions: [{ kind: "shared" as const, text: "old" }], missing: [], snapshot: { repo: "old", "side-tree": "old" } };
  const captured = vi.spyOn(api, "merged").mockResolvedValue(original);
  const posts: { path: string; body: unknown }[] = [];
  const undo = [{ source: "repo", text: "old", expected: "edited" }];
  let refuseWrite = false;
  installMockHandler(async (_method, path, body) => {
    posts.push({ path, body });
    if (path === "/api/merged/write") {
      if (refuseWrite) throw Error("fixture transport uncertainty");
      captured.mockResolvedValueOnce({ ...original, snapshot: { repo: "edited", "side-tree": "old" }, regions: [{ kind: "variant", by_source: { repo: "edited", "side-tree": "old" } }] });
      return { targets: [{ source: "repo", ok: true }], undo };
    }
    if (path === "/api/merged/undo") return { targets: [{ source: "repo", ok: true }] };
    throw Error(`Unexpected fixture request ${path}`);
  });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><MergedView path="shared.txt" /></UiActionContext.Provider>);
  await screen.findByLabelText("Open repo for writing");
  fireEvent.click(screen.getByLabelText("Open repo for writing"));
  await waitFor(() => expect((screen.getByLabelText("Open repo for writing") as HTMLInputElement).checked).toBe(true));
  fireEvent.click(screen.getByRole("button", { name: "Edit shared region" }));
  await screen.findByRole("region", { name: "Worktree region edit" });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label && action.enabled)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  await run("Replace comparison draft range", { from: 0, to: 3, text: "edited" });
  expect(await run("Apply worktree region draft")).toMatchObject({ status: "result", outcome: { value: { path: "shared.txt", target: "repo", route: "shared", response: { targets: [{ source: "repo", ok: true }] } } } });
  expect(posts[0]).toEqual({ path: "/api/merged/write", body: { path: "shared.txt", targets: ["repo", "side-tree"], writable: ["repo"], expected: original.snapshot, region: 0, text: "edited", route: "shared", source: "repo" } });
  expect(screen.queryByRole("region", { name: "Worktree region edit" })).toBeNull();
  fireEvent.click(await screen.findByRole("button", { name: "Undo repo" }));
  await waitFor(() => expect(posts[1]).toEqual({ path: "/api/merged/undo", body: { path: "shared.txt", undo } }));
  await waitFor(() => expect(screen.queryByRole("button", { name: "Undo repo" })).toBeNull());
  fireEvent.click(screen.getByRole("button", { name: "Edit shared region" }));
  await screen.findByRole("region", { name: "Worktree region edit" });
  await run("Replace comparison draft range", { from: 0, to: 3, text: "retry draft" });
  refuseWrite = true;
  expect(await run("Apply worktree region draft")).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  expect((screen.getByRole("button", { name: "Apply region to selected targets" }) as HTMLButtonElement).disabled).toBe(true);
  expect(actions().some(action => action.label === "Replace comparison draft range" && action.enabled)).toBe(false);
  expect(screen.getByText(/from an earlier request/)).toBeTruthy();
  expect(screen.getByText(/Latest write request failed/)).toBeTruthy();
});
