import { describe, expect, it } from "vitest";

import { INITIAL_GIT, gitReducer, pushState, shaOf } from "./git";

describe("the demo's git strip", () => {
  it("refuses to commit or push nothing", () => {
    expect(gitReducer(INITIAL_GIT, { type: "commit", message: "x", author: "you" })).toBe(
      INITIAL_GIT,
    );
    expect(gitReducer(INITIAL_GIT, { type: "push" })).toBe(INITIAL_GIT);
    expect(pushState(INITIAL_GIT).enabled).toBe(false);
  });

  it("commits an edit, then has exactly one commit to push", () => {
    const dirty = gitReducer(INITIAL_GIT, { type: "edited" });
    expect(dirty.dirty).toBe(true);

    const committed = gitReducer(dirty, { type: "commit", message: "Raise the limit", author: "you" });
    expect(committed.dirty).toBe(false);
    expect(committed.commits).toHaveLength(2);
    expect(pushState(committed)).toEqual({ label: "Push (1)", enabled: true });

    const pushed = gitReducer(committed, { type: "push" });
    expect(pushState(pushed).enabled).toBe(false);
  });

  it("does not let a pull swallow work you had not committed", () => {
    // The pulled text necessarily changes the document, which would otherwise
    // read as "clean" and quietly lose the visitor's uncommitted edit.
    const dirty = gitReducer(INITIAL_GIT, { type: "edited" });
    const pulled = gitReducer(dirty, {
      type: "pull",
      message: "Add the enterprise row",
      author: "octocat",
      dirtyBefore: true,
    });
    expect(pulled.dirty).toBe(true);
    expect(pulled.commits).toHaveLength(2);
  });

  it("treats a pulled commit as already on the remote", () => {
    const pulled = gitReducer(INITIAL_GIT, {
      type: "pull",
      message: "Add the enterprise row",
      author: "octocat",
      dirtyBefore: false,
    });
    expect(pushState(pulled).enabled).toBe(false);
  });

  it("keeps a local commit unpushed across a pull", () => {
    const local = gitReducer(gitReducer(INITIAL_GIT, { type: "edited" }), {
      type: "commit",
      message: "Mine",
      author: "you",
    });
    const pulled = gitReducer(local, {
      type: "pull",
      message: "Theirs",
      author: "octocat",
      dirtyBefore: false,
    });
    expect(pushState(pulled)).toEqual({ label: "Push (2)", enabled: true });
  });

  it("derives stable short hashes rather than random ones", () => {
    // A demo whose commit ids change on every render cannot be screenshotted
    // or pointed at in a conversation.
    expect(shaOf("Raise the limit", 1)).toBe(shaOf("Raise the limit", 1));
    expect(shaOf("Raise the limit", 1)).not.toBe(shaOf("Raise the limit", 2));
    expect(shaOf("Raise the limit", 1)).toMatch(/^[0-9a-f]{7}$/);
  });
});
