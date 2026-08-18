// The plain-file save loop: whole-file PUTs ride the hash of the content
// they were based on, so a file rewritten on disk underneath the buffer
// becomes a parked conflict — never a silent overwrite, never a retry loop.
// Protects docs/guarantees/authoring/a-plain-file-opens-and-saves.md.

import { describe, expect, it } from "vitest";

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
