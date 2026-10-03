import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";

import { GitPane, sides, when } from "./GitPane";
import { bisects } from "../api/representations";
import { api } from "../api/client";
import type { GitChanges, GitCommit, GitLog } from "../api/types";

beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(bisects, "list").mockResolvedValue({sessions:[]});
  // The working tree is read beside the log; the history tests are about the
  // graph, so they see a repository with nothing to commit.
  vi.spyOn(api, "gitChanges").mockResolvedValue({
    repository: true,
    branch: "master",
    upstream: null,
    ahead: 0,
    behind: 0,
    files: [],
  });
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

const serve = (log: Partial<GitLog>) =>
  vi.spyOn(api, "gitLog").mockResolvedValue({
    repository: true,
    commits: [commit()],
    floor: null,
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
    fireEvent.click(screen.getByRole("button", { expanded: false, name: /Add the thing/ }));
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
    fireEvent.click(screen.getByRole("button", { expanded: false, name: /Add the thing/ }));
    const row = screen.getByText("new.rs").closest("button")!;
    expect(within(row).getByText(/old\.rs/)).toBeTruthy();
    expect(within(row).getByText("R")).toBeTruthy();
  });

  it("says 'binary' rather than showing zeroes for a binary file", async () => {
    // Zero would claim the change touched nothing.
    serve({ commits: [commit({ files: [{ path: "logo.png", status: "M" }] })] });
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { expanded: false, name: /Add the thing/ }));
    expect(screen.getByText("binary")).toBeTruthy();
  });

  it("explains an empty merge rather than showing nothing", async () => {
    serve({ commits: [commit({ parents: ["a", "b"], files: [], added: 0, removed: 0 })] });
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { expanded: false, name: /Add the thing/ }));
    expect(screen.getByText(/changes are in the commits it joins/i)).toBeTruthy();
  });

  it("opens a file when its row is clicked", async () => {
    const onOpenFile = vi.fn();
    serve({});
    render(<GitPane onOpenFile={onOpenFile} />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { expanded: false, name: /Add the thing/ }));
    fireEvent.click(screen.getByText("src/main.rs").closest("button")!);
    expect(onOpenFile).toHaveBeenCalledWith("src/main.rs");
  });

  it("closes again", async () => {
    serve({});
    render(<GitPane />);
    await waitFor(() => expect(screen.getByText("Add the thing")).toBeTruthy());
    const summary = screen.getByRole("button", { expanded: false, name: /Add the thing/ });
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

// docs/guarantees/collaboration/the-publication-floor-is-computed-and-shown.md
describe("the publication floor", () => {
  it("marks the commits above it as drafts and says why", () => {
    serve({
      commits: [
        commit({ sha: "aaa", short: "aaa", subject: "still a draft", draft: true }),
        commit({ sha: "bbb", short: "bbb", subject: "published", parents: ["ccc"] }),
      ],
      floor: {
        published_ref: "origin/master",
        sha: "bbb",
        source: "upstream",
        drafts: ["aaa"],
        summary:
          "1 commit(s) above the floor at origin/master: still drafts, because nobody else can be holding them yet. Everything below is a record.",
      },
    });
    render(<GitPane />);
    return waitFor(() => {
      expect(screen.getByText(/1 draft$/)).toBeTruthy();
      expect(screen.getByText(/nobody else can be holding them/)).toBeTruthy();
      // One badge, on the one commit above the floor.
      expect(screen.getAllByText("draft")).toHaveLength(1);
    });
  });

  it("says nothing where the repository has published nothing to compare against", () => {
    // A folder with no remote has published nothing; a floor bar claiming a
    // line exists would be an invented fact.
    serve({ floor: null });
    render(<GitPane />);
    return waitFor(() => expect(screen.queryByText(/above the floor/)).toBeNull());
  });
});

// Protects docs/guarantees/collaboration/the-git-pane-does-the-daily-loop.md
describe("the working tree", () => {
  const serveChanges = (over: Partial<GitChanges> = {}) =>
    vi.spyOn(api, "gitChanges").mockResolvedValue({
      repository: true,
      branch: "feature",
      upstream: "origin/feature",
      ahead: 2,
      behind: 0,
      files: [
        { path: "src/main.rs", index: "M", tree: " " },
        { path: "README.md", index: " ", tree: "M" },
        { path: "notes.md", index: "?", tree: "?" },
      ],
      ...over,
    });

  it("splits git's two columns into the two lists a person stages between", () => {
    const { staged, unstaged } = sides([
      { path: "both.rs", index: "M", tree: "M" },
      { path: "new.md", index: "?", tree: "?" },
      { path: "moved.rs", from: "old.rs", index: "R", tree: " " },
    ]);
    expect(staged.map((f) => f.path)).toEqual(["both.rs", "moved.rs"]);
    expect(unstaged.map((f) => f.path)).toEqual(["both.rs", "new.md"]);
    // An untracked file reads as an addition, not as a question mark.
    expect(unstaged[1].status).toBe("A");
    expect(staged[1].from).toBe("old.rs");
  });

  it("shows the branch, how far ahead it is, and the files on each side", async () => {
    serve({});
    serveChanges();
    render(<GitPane />);
    const changes = await screen.findByRole("region", { name: "Working tree" });
    expect(within(changes).getByText("feature")).toBeTruthy();
    expect(within(changes).getByText("↑2")).toBeTruthy();
    const staged = within(changes).getByRole("region", { name: "Staged files" });
    expect(within(staged).getByText("src/main.rs")).toBeTruthy();
    const unstaged = within(changes).getByRole("region", { name: "Unstaged files" });
    expect(within(unstaged).getByText("README.md")).toBeTruthy();
    expect(within(unstaged).getByText("notes.md")).toBeTruthy();
  });

  it("stages a file with one click and re-reads the repository", async () => {
    serve({});
    const changes = serveChanges();
    const stage = vi.spyOn(api, "gitStage").mockResolvedValue({ ok: true });
    render(<GitPane />);
    const unstaged = await screen.findByRole("region", { name: "Unstaged files" });
    const row = within(unstaged).getByText("README.md").closest("li")!;
    fireEvent.click(within(row).getByRole("button", { name: "Stage" }));
    await waitFor(() => expect(stage).toHaveBeenCalledWith({ paths: ["README.md"] }));
    await waitFor(() => expect(changes.mock.calls.length).toBeGreaterThan(1));
  });

  it("shows a file's diff when its row is clicked", async () => {
    serve({});
    serveChanges();
    vi.spyOn(api, "gitDiff").mockResolvedValue({
      path: "README.md",
      diff: "@@ -1 +1,2 @@\n one\n+two\n",
      binary: false,
    });
    render(<GitPane />);
    const unstaged = await screen.findByRole("region", { name: "Unstaged files" });
    fireEvent.click(within(unstaged).getByText("README.md"));
    await waitFor(() => expect(screen.getByRole("region", { name: "Diff of README.md" })).toBeTruthy());
    expect(document.querySelectorAll(".diff-line--add")).toHaveLength(1);
  });

  it("commits what is staged with the message typed, and nothing without one", async () => {
    serve({});
    serveChanges();
    const commit = vi.spyOn(api, "gitCommit").mockResolvedValue({ sha: "s", short: "s", subject: "Do it" });
    render(<GitPane />);
    await screen.findByRole("region", { name: "Working tree" });
    const button = screen.getByRole("button", { name: /^Commit/ });
    expect((button as HTMLButtonElement).disabled).toBe(true);
    fireEvent.change(screen.getByLabelText("Commit message"), { target: { value: "Do it" } });
    expect((button as HTMLButtonElement).disabled).toBe(false);
    fireEvent.click(button);
    await waitFor(() => expect(commit).toHaveBeenCalledWith("Do it", false));
    await waitFor(() => expect(screen.getByRole("status").textContent).toMatch(/Committed/));
  });

  it("asks twice before discarding, and shows git's own words when it refuses", async () => {
    serve({});
    serveChanges();
    const discard = vi
      .spyOn(api, "gitDiscard")
      .mockRejectedValue(new Error("git checkout refused: pathspec 'README.md' did not match"));
    render(<GitPane />);
    const unstaged = await screen.findByRole("region", { name: "Unstaged files" });
    const row = within(unstaged).getByText("README.md").closest("li")!;
    const button = within(row).getByRole("button", { name: "Discard" });
    fireEvent.click(button);
    expect(discard).not.toHaveBeenCalled();
    expect(within(row).getByRole("button", { name: "Discard?" })).toBeTruthy();
    fireEvent.click(within(row).getByRole("button", { name: "Discard?" }));
    await waitFor(() => expect(discard).toHaveBeenCalledWith(["README.md"]));
    await waitFor(() => expect(screen.getByRole("alert").textContent).toMatch(/pathspec/));
  });

  it("switches branches from the branch menu", async () => {
    serve({});
    serveChanges();
    vi.spyOn(api, "gitBranches").mockResolvedValue({
      branches: [
        { name: "feature", upstream: "origin/feature", current: true },
        { name: "master", upstream: "origin/master", current: false },
      ],
    });
    const checkout = vi.spyOn(api, "gitCheckout").mockResolvedValue({ ok: true, branch: "master" });
    render(<GitPane />);
    await screen.findByRole("region", { name: "Working tree" });
    fireEvent.click(screen.getByRole("button", { name: /feature/ }));
    const menu = await screen.findByRole("group", { name: "Branches" });
    fireEvent.click(within(menu).getByRole("button", { name: /master/ }));
    await waitFor(() => expect(checkout).toHaveBeenCalledWith("master"));
  });

  it("offers no verbs for a folder that is not a repository", async () => {
    serve({ repository: false, commits: [] });
    vi.spyOn(api, "gitChanges").mockResolvedValue({ repository: false, files: [] });
    render(<GitPane />);
    await screen.findByText(/not a git repository/i);
    expect(screen.queryByRole("region", { name: "Working tree" })).toBeNull();
  });
});
