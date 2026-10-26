<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/plainFileSave.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-07b4144df7a0"></a>
## `apps/web/src/lib/plainFileSave.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Plain File Opens, Edits, And Never Silently Loses A Byte](../../../../../guarantees/authoring/a-plain-file-opens-and-saves/contract.md), [Saved means the displayed bytes reached the file](../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md).

<a id="f-07b4144df7a0-PlainSaveState"></a>
### `PlainSaveState`

The debounced save loop behind a plain file.
Sibling of outputSave.ts, with whole-file PUTs instead of byte edits and a
hash where that saver has a text baseline. The correctness story is the
same: the base hash advances only when a save succeeds (or a fresh load
arrives), and saves are strictly serialized — two in-flight PUTs carrying
the same base hash would race each other into a 409 neither earned.
A 409 is not an error to retry: it means the file changed on disk under
the buffer, and only a person can say whose bytes win. The saver parks in
"conflict" and waits for resolve() — overwrite (force) or reload (adopt
the disk copy) — instead of re-sending on the next keystroke.

<storycode:copy id="f-07b4144df7a0-PlainSaveState">
// The debounced save loop behind a plain file.
//
// Sibling of outputSave.ts, with whole-file PUTs instead of byte edits and a
// hash where that saver has a text baseline. The correctness story is the
// same: the base hash advances only when a save succeeds (or a fresh load
// arrives), and saves are strictly serialized — two in-flight PUTs carrying
// the same base hash would race each other into a 409 neither earned.
//
// A 409 is not an error to retry: it means the file changed on disk under
// the buffer, and only a person can say whose bytes win. The saver parks in
// "conflict" and waits for resolve() — overwrite (force) or reload (adopt
// the disk copy) — instead of re-sending on the next keystroke.

export type PlainSaveState =
  | { kind: "idle" }
  | { kind: "saving" }
  | { kind: "saved" }
  /** The disk moved underneath the buffer; resolve() is the only way on. */
  | { kind: "conflict" }
  | { kind: "error"; message: string };

</storycode:copy>
<a id="f-07b4144df7a0-PlainSaverOptions"></a>
### `PlainSaverOptions`

<storycode:copy id="f-07b4144df7a0-PlainSaverOptions">
export interface PlainSaverOptions {
  /** PUT the whole content under `baseHash`; resolves to the new hash. A
   * rejection whose `status` is 409 is the disk-moved conflict. */
  put: (content: string, baseHash: string, force: boolean) => Promise<{ hash: string }>;
  onState: (state: PlainSaveState) => void;
  /** Pause that turns keystrokes into a save. Default 600ms. */
  debounceMs?: number;
}

</storycode:copy>
<a id="f-07b4144df7a0-PlainSaver"></a>
### `PlainSaver`

<storycode:copy id="f-07b4144df7a0-PlainSaver">
export interface PlainSaver {
  /** The server's copy, freshly loaded — content is only needed to tell "no
   * edits yet" from "edits pending"; the hash is what saves ride on. */
  load(content: string, hash: string): void;
  /** The buffer changed; schedule a save of its full text. */
  changed(next: string): void;
  /** True while the buffer holds text the server has not confirmed. The
   * external-reload path checks this before adopting disk content — while
   * true, the disk copy predates what was typed here. */
  hasPendingEdits(): boolean;
  /** In conflict: "overwrite" forces the buffer onto disk; "reload" means
   * the caller adopted the disk copy and will load() it. */
  resolve(how: "overwrite" | "reload"): void;
  /** The last content a successful exchange established — what the file held
   * when this editing session began, as far as we ever knew.
   *
   * This is the COMMON ANCESTOR of the buffer and whatever the disk says now,
   * which is what turns a conflict from "reload or overwrite, pick one" into
   * a three-way merge. It is also what a draft records, so the same choice is
   * available after a restart. */
  baseContent(): string;
  /** Write whatever is pending NOW, skipping the debounce — what File >
   * Save All asks of every pane holding a file. A buffer with nothing
   * pending, or one parked on a conflict, does nothing: forcing a conflicted
   * save from a menu item would overwrite somebody's work without asking. */
  flushNow(): Promise<boolean>;
  /** Throw away edits that have not reached the file and return its text. */
  discardPending(): string;
  /** Cancel the pending debounce (unmount). In-flight PUTs settle alone. */
  dispose(): void;
}

</storycode:copy>
<a id="f-07b4144df7a0-createPlainSaver"></a>
### `createPlainSaver`

<storycode:copy id="f-07b4144df7a0-createPlainSaver">
export function createPlainSaver(options: PlainSaverOptions): PlainSaver {
  const debounce = options.debounceMs ?? 600;
  // What the server holds, as far as a successful exchange has told us.
  let baseHash = "";
  let baseContent = "";
  // The newest full buffer text; the debounce and the chain both read it at
  // fire time, so intermediate states are skipped rather than queued.
  let latest: string | null = null;
  let conflicted = false;
  let timer: ReturnType<typeof setTimeout> | null = null;
  // Saves run strictly one after another, each against the hash the
  // previous exchange established.
  let chain: Promise<void> = Promise.resolve();

  const flush = (force = false): Promise<boolean> => {
    let succeeded = true;
    chain = chain.then(async () => {
      if (conflicted && !force) return;
      const target = latest;
      if (target === null || target === baseContent) return;
      options.onState({ kind: "saving" });
      try {
        const saved = await options.put(target, baseHash, force);
        baseHash = saved.hash;
        baseContent = target;
        conflicted = false;
        options.onState(latest === target ? { kind: "saved" } : { kind: "saving" });
      } catch (e) {
        succeeded = false;
        if ((e as { status?: number }).status === 409) {
          conflicted = true;
          options.onState({ kind: "conflict" });
          return;
        }
        options.onState({
          kind: "error",
          message: e instanceof Error ? e.message : String(e),
        });
      }
    });
    return chain.then(() => succeeded && !conflicted && (latest === null || latest === baseContent));
  };

  return {
    load(content: string, hash: string) {
      baseHash = hash;
      baseContent = content;
      latest = null;
      conflicted = false;
      options.onState({ kind: "idle" });
    },
    changed(next: string) {
      latest = next;
      options.onState(conflicted ? { kind: "conflict" } : next === baseContent ? { kind: "saved" } : { kind: "saving" });
      if (timer !== null) clearTimeout(timer);
      timer = setTimeout(() => flush(), debounce);
    },
    hasPendingEdits() {
      return latest !== null && latest !== baseContent;
    },
    flushNow() {
      if (timer !== null) {
        clearTimeout(timer);
        timer = null;
      }
      return flush();
    },
    discardPending() {
      if (timer !== null) clearTimeout(timer);
      timer = null;
      latest = null;
      conflicted = false;
      options.onState({ kind: "idle" });
      return baseContent;
    },
    baseContent() {
      return baseContent;
    },
    resolve(how: "overwrite" | "reload") {
      if (!conflicted) return;
      if (how === "overwrite") {
        flush(true);
      } else {
        // The caller is adopting the disk copy and will load() it; until
        // then nothing must fire against a hash known to be stale.
        conflicted = false;
        latest = null;
        if (timer !== null) clearTimeout(timer);
        timer = null;
      }
    },
    dispose() {
      if (timer !== null) clearTimeout(timer);
      timer = null;
    },
  };
}
</storycode:copy>

### Assemble `apps/web/src/lib/plainFileSave.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/plainFileSave.ts"><storycode:paste select="#f-07b4144df7a0-PlainSaveState"/><storycode:paste select="#f-07b4144df7a0-PlainSaverOptions"/><storycode:paste select="#f-07b4144df7a0-PlainSaver"/><storycode:paste select="#f-07b4144df7a0-createPlainSaver"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-94afcad10b27"></a>
## `apps/web/src/lib/plainFileSave.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Plain File Opens, Edits, And Never Silently Loses A Byte](../../../../../guarantees/authoring/a-plain-file-opens-and-saves/contract.md), [Conflict actions name the write](../../../../../guarantees/authoring/conflict-actions-name-the-write/contract.md), [Saved means the displayed bytes reached the file](../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md).

<a id="f-94afcad10b27-server"></a>
### `server`

The plain-file save loop: whole-file PUTs ride the hash of the content
they were based on, so a file rewritten on disk underneath the buffer
becomes a parked conflict — never a silent overwrite, never a retry loop.
Protects docs/guarantees/authoring/a-plain-file-opens-and-saves.md.

<storycode:copy id="f-94afcad10b27-server">
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

</storycode:copy>
<a id="f-94afcad10b27-saver"></a>
### `saver`

<storycode:copy id="f-94afcad10b27-saver">
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

</storycode:copy>
<a id="f-94afcad10b27-settle"></a>
### `settle`

<storycode:copy id="f-94afcad10b27-settle">
const settle = () => new Promise((resolve) => setTimeout(resolve, 15));

</storycode:copy>
<a id="f-94afcad10b27-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-94afcad10b27-expression_statement">
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

</storycode:copy>
<a id="f-94afcad10b27-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-94afcad10b27-expression_statement-2">
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
</storycode:copy>

### Assemble `apps/web/src/lib/plainFileSave.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/plainFileSave.test.ts"><storycode:paste select="#f-94afcad10b27-server"/><storycode:paste select="#f-94afcad10b27-saver"/><storycode:paste select="#f-94afcad10b27-settle"/><storycode:paste select="#f-94afcad10b27-expression_statement"/><storycode:paste select="#f-94afcad10b27-expression_statement-2"/></storycode:file>


</storycode:doc>
