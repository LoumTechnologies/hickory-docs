import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";

import { GitPane, when } from "./GitPane";
import { api } from "../api/client";
import type { GitCommit } from "../api/types";

beforeEach(() => {
  vi.restoreAllMocks();
});
afterEach(() => {
  cleanup();
});

const commit = (over: Partial<GitCommit> = {}): GitCommit => ({
  sha: "abc123def",
  short: "abc123d",
  parents: ["parent"],
  author: "Ada Lovelace",
  email: "ada@example.com",
  time: Math.floor(Date.now() / 1000) - 86_400,
  subject: "Add the thing",
  body: "",
  refs: [],
  files: [{ path: "src/main.rs", status: "M", added: 12, removed: 3 }],
  added: 12,
  removed: 3,
  ...over,
});

const serve = (log: Partial<{ repository: boolean; commits: GitCommit[] }>) =>
  vi.spyOn(api, "gitLog").mockResolvedValue({
    repository: true,
    commits: [commit()],
    ...log,
  });

describe("a folder with no repository", () => {
  it("says so plainly, because notes folders often are not one", () => {
    serve({ repository: false, commits: [] });
    render(<GitPane />);
    return waitFor(() =>
      expect(screen.getByText(/not a git repository/i)).toBeTruthy(),
    );
  });

  it("says what to run if you want one", () => {
    serve({ repository: false, commits: [] });
    render(<GitPane />);
    return waitFor(() => expect(screen.getByText("git init")).toBeTruthy());
  });
});

describe("what a commit's node says without being opened", () => {
  it("carries the subject, the author, the hash and the churn", async () => {
    serve({});
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    expect(screen.getByText("Ada Lovelace")).toBeTruthy();
    expect(screen.getByText("abc123d")).toBeTruthy();
    expect(screen.getByText("+12")).toBeTruthy();
    expect(screen.getByText("−3")).toBeTruthy();
  });

  it("shows the branch and tag names pointing at it", async () => {
    serve({ commits: [commit({ refs: ["master", "v1.0"] })] });
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("master")).toBeTruthy());
    expect(screen.getByText("v1.0")).toBeTruthy();
  });

  it("draws a node per commit in the graph", async () => {
    serve({ commits: [commit({ sha: "a", parents: ["b"] }), commit({ sha: "b", parents: [] })] });
    render(<GitPane />);
    await waitFor(() =>
      expect(document.querySelectorAll(".git-node")).toHaveLength(2),
    );
  });

  it("marks a merge differently from an ordinary commit", async () => {
    serve({ commits: [commit({ parents: ["a", "b"] })] });
    render(<GitPane />);
    await waitFor(() =>
      expect(document.querySelector(".git-node--merge")).toBeTruthy(),
    );
  });
});

describe("expanding a commit", () => {
  it("shows its files with no second request", async () => {
    // The files came with the log — one git invocation for everything — so
    // there is no spinner and no moment where the row is open and empty.
    const gitLog = serve({});
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { expanded: false }));
    expect(screen.getByText("src/main.rs")).toBeTruthy();
    expect(gitLog).toHaveBeenCalledTimes(1);
  });

  it("shows a rename as one change with both names", async () => {
    serve({
      commits: [
        commit({ files: [{ path: "new.rs", from: "old.rs", status: "R", added: 1, removed: 1 }] }),
      ],
    });
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { expanded: false }));
    const row = screen.getByText("new.rs").closest("button")!;
    expect(within(row).getByText(/old\.rs/)).toBeTruthy();
    expect(within(row).getByText("R")).toBeTruthy();
  });

  it("says 'binary' rather than showing zeroes for a binary file", async () => {
    // Zero would claim the change touched nothing.
    serve({ commits: [commit({ files: [{ path: "logo.png", status: "M" }] })] });
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { expanded: false }));
    expect(screen.getByText("binary")).toBeTruthy();
  });

  it("explains an empty merge rather than showing nothing", async () => {
    serve({ commits: [commit({ parents: ["a", "b"], files: [], added: 0, removed: 0 })] });
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { expanded: false }));
    expect(screen.getByText(/changes are in the commits it joins/i)).toBeTruthy();
  });

  it("opens a file when its row is clicked", async () => {
    const onOpenFile = vi.fn();
    serve({});
    render(<GitPane onOpenFile={onOpenFile} />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { expanded: false }));
    fireEvent.click(screen.getByText("src/main.rs").closest("button")!);
    expect(onOpenFile).toHaveBeenCalledWith("src/main.rs");
  });

  it("closes again", async () => {
    serve({});
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    const summary = screen.getByRole("button", { expanded: false });
    fireEvent.click(summary);
    expect(screen.queryByText("src/main.rs")).toBeTruthy();
    fireEvent.click(summary);
    expect(screen.queryByText("src/main.rs")).toBeNull();
  });
});

describe("how a commit's age reads", () => {
  const now = Date.UTC(2026, 7, 21);
  const at = (days: number) => Math.floor((now - days * 86_400_000) / 1000);

  it("is relative while relative means something", () => {
    expect(when(at(0), now)).toBe("today");
    expect(when(at(1), now)).toBe("yesterday");
    expect(when(at(5), now)).toBe("5d ago");
    expect(when(at(60), now)).toBe("2mo ago");
  });

  it("becomes a year past that", () => {
    expect(when(at(500), now)).toBe("2025");
  });
});
