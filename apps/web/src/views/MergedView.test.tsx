// docs/guarantees/collaboration/the-merged-view-is-a-lens.md
import { describe, expect, it, vi, beforeEach } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";

import { api } from "../api/client";
import { MergedView } from "./MergedView";

beforeEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

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

  it("says a shared region is agreed by construction", async () => {
    worktrees();
    merged();
    render(<MergedView path="shared.txt" />);
    await waitFor(() =>
      expect(screen.getByText(/it never diverged, so it\s+cannot conflict later/)).toBeTruthy(),
    );
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
