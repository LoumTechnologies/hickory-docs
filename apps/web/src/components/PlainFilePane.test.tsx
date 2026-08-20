// Unsaved work coming back, and what happens when the file moved on while the
// app was closed.
//
// Protects docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
//
// Driven through the pane against a mocked API client, because the decision
// being tested — restore quietly, drop a stale draft, or open a merge — is
// made from what those two requests answer.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";

import { PlainFilePane } from "./PlainFilePane";
import { api } from "../api/client";
import type { WorkspaceDraft } from "../api/types";

const ON_DISK = "one\ntwo\nthree\n";

function serve({ content, drafts }: { content: string; drafts: WorkspaceDraft[] }) {
  vi.spyOn(api, "file").mockResolvedValue({
    path: "notes.md",
    language: "markdown",
    content,
    hash: `h:${content}`,
  });
  vi.spyOn(api, "drafts").mockResolvedValue({ drafts });
  vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  vi.spyOn(api, "saveDraft").mockResolvedValue({ ok: true });
  vi.spyOn(api, "saveFile").mockResolvedValue({ path: "notes.md", hash: "h:saved" });
  vi.spyOn(api, "files").mockResolvedValue({
    root: "notebook",
    tree: [{ name: "notes.md", path: "notes.md", dir: false }],
  });
}

const draft = (over: Partial<WorkspaceDraft> = {}): WorkspaceDraft => ({
  path: "notes.md",
  contents: "one\nMINE\nthree\n",
  base: ON_DISK,
  saved_at: 1,
  ...over,
});

beforeEach(() => {
  vi.restoreAllMocks();
});

afterEach(() => {
  cleanup();
});

describe("reopening a file that had unsaved changes", () => {
  it("puts the unsaved text back, without asking", () => {
    // The common case by a wide margin. A dialog here would train people to
    // dismiss dialogs, and the work is theirs — nothing was lost or decided.
    serve({ content: ON_DISK, drafts: [draft()] });
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => {
      expect(document.querySelector(".cm-content")?.textContent).toContain("MINE");
      expect(screen.queryByTestId("merge-view")).toBeNull();
    });
  });

  it("throws away a draft the file already agrees with", () => {
    // Somebody saved the same text from elsewhere; there is nothing unsaved.
    const discard = vi.fn().mockResolvedValue({ ok: true });
    serve({ content: ON_DISK, drafts: [draft({ contents: ON_DISK, base: "older\n" })] });
    vi.spyOn(api, "discardDraft").mockImplementation(discard);
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => expect(discard).toHaveBeenCalledWith("notes.md"));
  });

  it("opens a merge when the file moved on and the buffer did too", () => {
    serve({
      content: "one\nTHEIRS\nthree\n",
      drafts: [draft({ contents: "one\nMINE\nthree\n", base: ON_DISK })],
    });
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => {
      const merge = screen.getByTestId("merge-view");
      expect(merge.textContent).toMatch(/changed while you were away/i);
      expect(merge.textContent).toMatch(/still needs you/i);
    });
  });

  it("opens normally when there is no draft for this file", () => {
    serve({ content: ON_DISK, drafts: [draft({ path: "somewhere/else.md" })] });
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => {
      expect(document.querySelector(".cm-content")?.textContent).toContain("two");
      expect(screen.queryByTestId("merge-view")).toBeNull();
    });
  });

  it("opens normally when the draft store is unreachable", () => {
    // A machine with no data directory, or a read-only home. The file opens
    // as it is on disk, which is what would have happened anyway.
    serve({ content: ON_DISK, drafts: [] });
    vi.spyOn(api, "drafts").mockRejectedValue(new Error("no store"));
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => {
      expect(document.querySelector(".cm-content")?.textContent).toContain("two");
    });
  });
});

describe("a file that is already the output of a document", () => {
  it("is not offered a button to make it literate", () => {
    // It already is. Offering the verb tells the reader their document is not
    // what it plainly is.
    serve({ content: ON_DISK, drafts: [] });
    vi.spyOn(api, "files").mockResolvedValue({
      root: "notebook",
      tree: [{ name: "notes.md", path: "notes.md", dir: false, generated_by: "d1" }],
    });
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() => {
      expect(screen.queryByRole("button", { name: /make literate/i })).toBeNull();
      expect(screen.getByText(/already literate/i)).toBeTruthy();
    });
  });

  it("keeps the button for a file nobody writes", () => {
    serve({ content: ON_DISK, drafts: [] });
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() =>
      expect(screen.getByRole("button", { name: /make literate/i })).toBeTruthy(),
    );
  });

  it("keeps the button when the listing is unavailable", () => {
    // The adopt route's own refusal is the backstop; a failed listing must
    // not remove a verb that might be valid.
    serve({ content: ON_DISK, drafts: [] });
    vi.spyOn(api, "files").mockRejectedValue(new Error("no listing"));
    render(<PlainFilePane path="notes.md" />);
    return waitFor(() =>
      expect(screen.getByRole("button", { name: /make literate/i })).toBeTruthy(),
    );
  });
});
