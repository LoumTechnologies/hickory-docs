// The history lens: the repository read as a story.
//
// Protects docs/guarantees/lenses/the-history-lens-reads-the-repository-as-a-story.md

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { HistoryLens, OPEN_TAIL, floorIndex, storyOrder } from "./HistoryLens";
import { api } from "../api/client";
import type { GitCommit, GitLog } from "../api/types";

function commit(over: Partial<GitCommit> & { sha: string }): GitCommit {
  return {
    short: over.sha.slice(0, 7),
    parents: [],
    author: "Nate",
    email: "n@example.com",
    time: 1_700_000_000,
    subject: `Commit ${over.sha}`,
    body: "",
    refs: [],
    files: [{ path: "a.txt", status: "M", added: 1, removed: 0 }],
    added: 1,
    removed: 0,
    ...over,
  };
}

/** Newest first, as `git log` answers. */
const LOG: GitLog = {
  repository: true,
  commits: [
    commit({ sha: "c3333333", subject: "Change Main", draft: true }),
    commit({
      sha: "b2222222",
      subject: "Scaffold a web API",
      body: "Scaffolded.\n\nHick-Recipe: dotnet new webapi -o app\nHick-Image: sdk:9.0",
      recipe: { command: "dotnet new webapi -o app", image: "sdk:9.0" },
      files: [
        { path: "app/Program.cs", status: "A", added: 10, removed: 0 },
        { path: "app/app.csproj", status: "A", added: 5, removed: 0 },
      ],
      added: 15,
    }),
    commit({ sha: "a1111111", subject: "First", body: "The beginning." }),
  ],
  floor: {
    source: "upstream",
    sha: "b2222222",
    drafts: ["c3333333"],
    summary: "1 draft above origin/master.",
  },
};

function serve(log: GitLog = LOG) {
  vi.spyOn(api, "gitLog").mockResolvedValue(log);
  vi.spyOn(api, "gitStatus").mockResolvedValue({ repository: true, branch: "master", staged: 0, unstaged: 2, untracked: 0 });
  return vi.spyOn(api, "gitCommitDetail").mockResolvedValue({
    sha: "b2222222",
    diff: "diff --git a/app/Program.cs b/app/Program.cs\n+// scaffolded\n",
    files: ["app/Program.cs", "app/app.csproj"],
    edited_since: [{ path: "app/Program.cs", sha: "c3333333", short: "c333333", subject: "Change Main" }],
  });
}

beforeEach(() => vi.restoreAllMocks());
afterEach(() => cleanup());

describe("the story's shape", () => {
  it("reads oldest first", () => {
    expect(storyOrder(LOG.commits).map((c) => c.subject)).toEqual(["First", "Scaffold a web API", "Change Main"]);
  });

  it("puts the floor before the first draft", () => {
    expect(floorIndex(storyOrder(LOG.commits))).toBe(2);
    expect(floorIndex(storyOrder(LOG.commits).map((c) => ({ ...c, draft: false })))).toBeNull();
  });
});

describe("the history lens", () => {
  it("draws the commits as cards, oldest first, ending at the working tree", async () => {
    serve();
    render(<HistoryLens />);
    await waitFor(() => expect(screen.getByRole("article", { name: "First" })).toBeTruthy());
    const cards = screen.getAllByRole("article").map((a) => a.getAttribute("aria-label"));
    expect(cards).toEqual(["First", "Scaffold a web API", "Change Main", "Working tree"]);
    // It says what it is: a lens, on disk nowhere, read-only.
    expect(screen.getByRole("note").textContent).toMatch(/lens/);
    expect(screen.getByRole("note").textContent).toMatch(/cannot be saved/);
    // The tail is the working tree, with what is uncommitted.
    expect(screen.getByRole("article", { name: "Working tree" }).textContent).toMatch(/2 changes not yet committed/);
  });

  it("marks the publication floor between records and drafts", async () => {
    serve();
    render(<HistoryLens />);
    await waitFor(() => expect(screen.getByRole("separator", { name: "Publication floor" })).toBeTruthy());
    const floor = screen.getByRole("separator", { name: "Publication floor" });
    // The floor sits immediately before the first draft card.
    const next = floor.parentElement?.querySelector("article");
    expect(next?.getAttribute("aria-label")).toBe("Change Main");
    expect(screen.getByRole("article", { name: "Change Main" }).className).toContain("story-card--draft");
  });

  it("draws a recipe commit as a cell, unrecorded until replayed", async () => {
    serve();
    render(<HistoryLens />);
    const card = await screen.findByRole("article", { name: "Scaffold a web API" });
    expect(card.className).toContain("story-card--recipe");
    const cell = screen.getByRole("group", { name: "Recipe" });
    expect(cell.textContent).toContain("dotnet new webapi -o app");
    expect(cell.textContent).toContain("sdk:9.0");
    // A declared claim: nothing has replayed it, and the chip says so in
    // the words the spec allows.
    expect(cell.textContent).toContain("no evidence of drift");
    expect(cell.textContent).not.toMatch(/reproducible/);
    // The trailers are not repeated as prose.
    expect(cell.textContent).not.toContain("Hick-Recipe");
  });

  it("shows the diff as the card's output, and what was edited since", async () => {
    const detail = serve();
    render(<HistoryLens />);
    const card = await screen.findByRole("article", { name: "Scaffold a web API" });
    fireEvent.click(card.querySelector(".story-card__toggle")!);
    await waitFor(() => expect(detail).toHaveBeenCalledWith("b2222222"));
    await waitFor(() => expect(screen.getByRole("status").textContent).toMatch(/Edited since/));
    const edited = screen.getByRole("status").textContent ?? "";
    expect(edited).toContain("app/Program.cs");
    expect(edited).toContain("Change Main");
    expect(card.textContent).toContain("+// scaffolded");
  });

  it("folds the past and opens at the tail", async () => {
    const many: GitCommit[] = [];
    for (let i = 0; i < OPEN_TAIL + 5; i += 1) many.push(commit({ sha: `${i}`.padStart(8, "0"), subject: `Commit ${i}` }));
    serve({ repository: true, commits: [...many].reverse(), floor: null });
    render(<HistoryLens />);
    const fold = await screen.findByRole("button", { name: "5 earlier commits, folded" });
    expect(screen.queryByRole("article", { name: "Commit 0" })).toBeNull();
    expect(screen.getByRole("article", { name: `Commit ${OPEN_TAIL + 4}` })).toBeTruthy();
    fireEvent.click(fold);
    await waitFor(() => expect(screen.getByRole("article", { name: "Commit 0" })).toBeTruthy());
  });

  it("says so when the folder is not a repository", async () => {
    serve({ repository: false, commits: [], floor: null });
    render(<HistoryLens />);
    await waitFor(() => expect(screen.getByText(/not a git repository/)).toBeTruthy());
  });
});
