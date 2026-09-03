// The history lens: the repository read as a story.
//
// Protects docs/guarantees/lenses/the-history-lens-reads-the-repository-as-a-story.md

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";

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
      recipe: { command: "dotnet new webapi -o app", image: "sdk:9.0", output: "4b825dc", output_path: "app", output_matches: true },
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
  vi.spyOn(api, "gitStage").mockResolvedValue({ ok: true } as never);
  vi.spyOn(api, "gitCommit").mockResolvedValue({ sha: "d4444444", short: "d444444" } as never);
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
    // git checked the tree against the trailer: this one is the scaffold
    // exactly, so it can be upgraded.
    expect(cell.textContent).toContain("matches its recorded output");
    expect(cell.textContent).toContain("upgradeable");
  });

  it("says when a recipe commit was edited before it was committed", async () => {
    // The visual answer to "is this upgradeable?": a tree that is not the
    // trailer's is a scaffold with edits fused into it, and replay cannot
    // separate them.
    const edited = {
      ...LOG,
      commits: LOG.commits.map((c) =>
        c.recipe ? { ...c, recipe: { ...c.recipe, output_matches: false } } : c,
      ),
    };
    serve(edited);
    render(<HistoryLens />);
    const cell = await screen.findByRole("group", { name: "Recipe" });
    expect(cell.textContent).toContain("edited before it was committed");
    expect(cell.textContent).toContain("not upgradeable");
    expect(cell.textContent).not.toContain("matches its recorded output");
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

// Protects docs/guarantees/lenses/a-recipe-commit-can-be-replayed.md,
// docs/guarantees/lenses/the-tail-of-the-story-is-the-next-commit.md and
// docs/guarantees/lenses/the-past-is-edited-by-rebase-above-the-floor.md
describe("the story's verbs", () => {
  it("replays a recipe whose tree matches, and says what came of it", async () => {
    serve();
    const replay = vi.spyOn(api, "gitReplay").mockResolvedValue({
      of: "b2222222",
      sha: "e5555555",
      short: "e555555",
      same: false,
      moved: "rebase",
      head: "f6666666",
      said: [],
    });
    render(<HistoryLens />);
    const cell = await screen.findByRole("group", { name: "Recipe" });
    fireEvent.click(within(cell).getByRole("button", { name: "Replay" }));
    await waitFor(() => expect(replay).toHaveBeenCalledWith("b2222222"));
    await waitFor(() => expect(screen.getByRole("alert").textContent).toMatch(/Replayed b222222 as e555555/));
    expect(screen.getByRole("alert").textContent).toMatch(/differs/);
    expect(screen.getByRole("alert").textContent).toMatch(/rebase/);
  });

  it("offers no replay on a commit edited before it was committed, and shows replay evidence apart", async () => {
    const log = {
      ...LOG,
      commits: LOG.commits.map((c) =>
        c.recipe
          ? { ...c, recipe: { ...c.recipe, output_matches: false, replay_of: "a1111111", replay_same: true } }
          : c,
      ),
    };
    serve(log);
    render(<HistoryLens />);
    const cell = await screen.findByRole("group", { name: "Recipe" });
    expect(within(cell).queryByRole("button", { name: "Replay" })).toBeNull();
    // Evidence, worded as evidence: a replay happened and matched.
    expect(cell.textContent).toContain("replayed · same as a111111");
    expect(cell.textContent).not.toContain("no evidence of drift");
  });

  it("gives drafts reword, move and drop, and records nothing", async () => {
    serve();
    const reword = vi.spyOn(api, "gitReword").mockResolvedValue({ head: "x" });
    const drop = vi.spyOn(api, "gitDrop").mockResolvedValue({ head: "x" });
    render(<HistoryLens />);
    const draft = await screen.findByRole("article", { name: "Change Main" });
    const record = screen.getByRole("article", { name: "First" });
    expect(within(record).queryByRole("group", { name: "Edit this draft" })).toBeNull();
    const edit = within(draft).getByRole("group", { name: "Edit this draft" });
    // The only draft is both first and last: nowhere to move.
    expect(within(edit).getByRole("button", { name: "Move earlier" })).toHaveProperty("disabled", true);
    expect(within(edit).getByRole("button", { name: "Move later" })).toHaveProperty("disabled", true);

    fireEvent.click(within(edit).getByRole("button", { name: "Reword" }));
    const textarea = within(draft).getByRole("textbox", { name: "New message" }) as HTMLTextAreaElement;
    expect(textarea.value).toBe("Change Main");
    fireEvent.change(textarea, { target: { value: "Change Main, said better" } });
    fireEvent.click(within(draft).getByRole("button", { name: "Save" }));
    await waitFor(() => expect(reword).toHaveBeenCalledWith("c3333333", "Change Main, said better"));

    fireEvent.click(within(screen.getByRole("article", { name: "Change Main" })).getByRole("button", { name: "Drop" }));
    await waitFor(() => expect(drop).toHaveBeenCalledWith("c3333333"));
  });

  it("shows git's words when a verb is refused, and re-reads the story", async () => {
    serve();
    vi.spyOn(api, "gitDrop").mockRejectedValue(new Error("the working tree has uncommitted changes"));
    render(<HistoryLens />);
    const draft = await screen.findByRole("article", { name: "Change Main" });
    const calls = vi.mocked(api.gitLog).mock.calls.length;
    fireEvent.click(within(draft).getByRole("button", { name: "Drop" }));
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("uncommitted changes"));
    await waitFor(() => expect(vi.mocked(api.gitLog).mock.calls.length).toBeGreaterThan(calls));
  });

  it("commits the working tree from the tail, and runs a command as a recipe", async () => {
    serve();
    const recipe = vi.spyOn(api, "gitRecipe").mockResolvedValue({ sha: "e5555555", short: "e555555", output_tree: "t", said: [] });
    render(<HistoryLens />);
    const tail = await screen.findByRole("article", { name: "Working tree" });
    const message = within(tail).getByRole("textbox", { name: "What happened" });
    expect(within(tail).getByRole("button", { name: "Commit" })).toHaveProperty("disabled", true);
    fireEvent.change(message, { target: { value: "Fix the thing" } });
    fireEvent.click(within(tail).getByRole("button", { name: "Commit" }));
    await waitFor(() => expect(api.gitCommit).toHaveBeenCalledWith("Fix the thing"));
    expect(api.gitStage).toHaveBeenCalledWith({ all: true });

    fireEvent.change(within(tail).getByRole("textbox", { name: "Command" }), { target: { value: "dotnet new webapi -o app" } });
    fireEvent.change(within(tail).getByRole("textbox", { name: "Output folder" }), { target: { value: "app" } });
    fireEvent.click(within(tail).getByRole("button", { name: "Run and commit" }));
    await waitFor(() => expect(recipe).toHaveBeenCalledWith("dotnet new webapi -o app", "app"));
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("committed app/ as e555555"));
  });
});

