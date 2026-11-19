// The plain-file save loop: whole-file PUTs ride the hash of the content
// they were based on, so a file rewritten on disk underneath the buffer
// becomes a parked conflict — never a silent overwrite, never a retry loop.
// Protects docs/guarantees/authoring/a-plain-file-opens-and-saves.md.

import { describe, expect, it, vi } from "vitest";

import { createPlainSaver, type PlainSaveState } from "./plainFileSave";

/** A model server: content + hash, occasionally rewritten "externally". */
function server(initial: string) {
  let content = initial;
  let hash = `h:${initial}`;
  return {
    externalWrite(next: string) {
      content = next;
      hash = `h:${next}`;
    },
    get content() {
      return content;
    },
    get hash() {
      return hash;
    },
    put: async (next: string, baseHash: string, force: boolean) => {
      if (!force && baseHash !== hash) {
        throw Object.assign(new Error("changed on disk"), { status: 409 });
      }
      content = next;
      hash = `h:${next}`;
      return { hash };
    },
  };
}

function saver(remote: ReturnType<typeof server>) {
  const states: PlainSaveState[] = [];
  const s = createPlainSaver({
    put: remote.put,
    onState: (st) => states.push(st),
    debounceMs: 1,
  });
  s.load(remote.content, remote.hash);
  return { s, states };
}

const settle = () => new Promise((resolve) => setTimeout(resolve, 15));

describe("createPlainSaver", () => {
  it("holds detached edits and retries the exact explicit save after an uncertain reply", async () => {
    const put = vi.fn(async () => ({hash: "legacy"}));
    const publish = vi.fn<NonNullable<Parameters<typeof createPlainSaver>[0]["publish"]>>()
      .mockRejectedValueOnce(Error("reply lost"))
      .mockResolvedValueOnce({hash: "after hash", save_guard: "second guard"})
      .mockResolvedValueOnce({hash: "typed hash", save_guard: "third guard"});
    const s = createPlainSaver({put, publish, onState: () => {}, debounceMs: 1});
    s.load("saved", "original hash", "original guard");
    s.changed("previous edit");
    const plane = s.prepareDetach("saved", "original guard")!;
    expect(plane.current()).toBe(true);
    plane.commit();
    s.changed("proposed");
    s.changed("previous edit"); // Native Undo.
    expect(await s.flushNow()).toBe(false);
    await settle();
    expect(put).not.toHaveBeenCalled();
    expect(publish).not.toHaveBeenCalled();
    expect(await s.saveNow()).toBe(false);
    s.changed("newer typed draft");
    expect(await s.saveNow()).toBe(false);
    expect(publish.mock.calls[1]?.[0]).toEqual(publish.mock.calls[0]?.[0]);
    expect(publish.mock.calls[0]?.[0]).toMatchObject({content: "previous edit", baseHash: "original hash", guard: "original guard"});
    expect(s.hasPendingEdits()).toBe(true);
    expect(await s.saveNow()).toBe(true);
    expect(publish.mock.calls[2]?.[0]).toMatchObject({content: "newer typed draft", baseHash: "after hash", guard: "second guard"});
    expect(put).not.toHaveBeenCalled();
  });

  it("restores the ordinary pending debounce when a prepared group aborts", async () => {
    const remote = server("saved");
    const {s} = saver(remote);
    s.load(remote.content, remote.hash, "guard");
    s.changed("earlier typing");
    const plane = s.prepareDetach("saved", "guard")!;
    plane.commit();
    s.changed("proposed");
    plane.rollback();
    expect(s.isDetached()).toBe(false);
    await settle();
    expect(remote.content).toBe("earlier typing");
  });

  it("replaces and clears the physical guard with the saved baseline", async () => {
    const put = async (content: string) => ({hash: content, save_guard: content === "confirmed" ? "new guard" : null});
    const s = createPlainSaver({put, onState: () => {}});
    s.load("original", "first hash", "original guard");
    s.changed("confirmed");
    expect([s.baseContent(), s.baseGuard()]).toEqual(["original", "original guard"]);
    await s.flushNow();
    expect([s.baseContent(), s.baseGuard()]).toEqual(["confirmed", "new guard"]);
    s.changed("no guard");
    await s.flushNow();
    expect([s.baseContent(), s.baseGuard()]).toEqual(["no guard", null]);
  });
  it("saves the newest text after the pause, and advances the hash it rides on", async () => {
    const remote = server("one\n");
    const { s, states } = saver(remote);
    s.changed("two\n");
    s.changed("three\n"); // intermediate state skipped, not queued
    await settle();
    expect(remote.content).toBe("three\n");
    expect(states.at(-1)).toEqual({ kind: "saved" });
    // The next save rides the NEW hash: it must not 409 against itself.
    s.changed("four\n");
    await settle();
    expect(remote.content).toBe("four\n");
    expect(s.hasPendingEdits()).toBe(false);
  });

  it("parks on 409 instead of overwriting or retrying, until a person resolves", async () => {
    const remote = server("theirs v1\n");
    const { s, states } = saver(remote);
    remote.externalWrite("theirs v2\n"); // git checkout under the buffer
    s.changed("mine\n");
    await settle();
    expect(remote.content).toBe("theirs v2\n"); // nothing overwritten
    expect(states.at(-1)).toEqual({ kind: "conflict" });
    s.changed("mine, more typing\n"); // keystrokes during the banner
    await settle();
    expect(remote.content).toBe("theirs v2\n"); // still parked, no retry
    s.resolve("overwrite");
    await settle();
    expect(remote.content).toBe("mine, more typing\n");
    expect(states.at(-1)).toEqual({ kind: "saved" });
  });

  it("resolve('reload') stands down; load() of the disk copy re-arms it", async () => {
    const remote = server("theirs v1\n");
    const { s } = saver(remote);
    remote.externalWrite("theirs v2\n");
    s.changed("mine\n");
    await settle();
    s.resolve("reload");
    s.load(remote.content, remote.hash); // the pane adopted the disk copy
    expect(s.hasPendingEdits()).toBe(false);
    s.changed("mine again\n");
    await settle();
    expect(remote.content).toBe("mine again\n");
  });

  it("a non-409 failure reports, keeps the hash, and the next save re-sends", async () => {
    let fail = true;
    const remote = server("start\n");
    const flaky = {
      ...remote,
      put: async (next: string, baseHash: string, force: boolean) => {
        if (fail) throw new Error("disk full");
        return remote.put(next, baseHash, force);
      },
    };
    const states: PlainSaveState[] = [];
    const s = createPlainSaver({ put: flaky.put, onState: (st) => states.push(st), debounceMs: 1 });
    s.load(remote.content, remote.hash);
    s.changed("edited\n");
    await settle();
    expect(states.at(-1)).toEqual({ kind: "error", message: "disk full" });
    expect(s.hasPendingEdits()).toBe(true);
    fail = false;
    s.changed("edited\n");
    await settle();
    expect(remote.content).toBe("edited\n");
  });
});

// Guarantee: docs/guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file.md
it("does not report Save succeeded while a plain file is parked on a conflict", async () => {
  const remote = server("base\n");
  const { s, states } = saver(remote);
  remote.externalWrite("external\n");
  s.changed("buffer\n");
  await settle();
  expect(await s.flushNow()).toBe(false);
  s.changed("newer buffer\n");
  expect(states.at(-1)).toEqual({ kind: "conflict" });
  expect(remote.content).toBe("external\n");
});
