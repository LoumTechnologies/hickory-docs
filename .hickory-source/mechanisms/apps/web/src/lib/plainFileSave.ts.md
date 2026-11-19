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
  | { kind: "draft" }
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
  put: (content: string, baseHash: string, force: boolean, guard: string | null) => Promise<{ hash: string; save_guard?: string | null }>;
  /** The strict, independently acknowledged publication route for a detached draft. */
  publish?: (request: { content: string; baseHash: string; guard: string; id: string }, current: () => boolean) => Promise<{ hash: string; save_guard: string }>;
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
  load(content: string, hash: string, guard?: string | null): void;
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
  baseGuard(): string | null;
  /** Flush ordinary autosave for a dependent operation. A dirty detached draft refuses. */
  flushNow(): Promise<boolean>;
  /** Explicit Save / Save All may publish a detached draft with its captured guard. */
  saveNow(): Promise<boolean>;
  isDetached(): boolean;
  /** Restore the private publication mode before applying retained text. */
  restoreDetached(): void;
  /** Validate and prepare the plane switch without changing the saver. */
  prepareDetach(base: string, guard: string): { current: () => boolean; commit: () => void; rollback: () => void } | null;
  /** Throw away edits that have not reached the file and return its text. */
  discardPending(): string;
  /** Cancel the pending debounce (unmount). In-flight PUTs settle alone. */
  dispose(): void;
}

</storycode:copy>
<a id="f-07b4144df7a0-createPlainSaver"></a>
### `createPlainSaver`

The ordinary debounce and explicit Save share serialization. A prepared switch
checks the saved guard and rejects an executing or queued write. Commit cancels
the old debounce before the editor transaction can notify the saver. If another
editor refuses the group, rollback restores the old draft plane and its pending
autosave. A detached Save can retry the same retained request when the strict
receipt is uncertain, even if subsequent typing has moved the live draft.

<storycode:copy id="f-07b4144df7a0-createPlainSaver">
export function createPlainSaver(options: PlainSaverOptions): PlainSaver {
  const debounce = options.debounceMs ?? 600;
  // What the server holds, as far as a successful exchange has told us.
  let baseHash = "";
  let baseContent = "";
  let baseGuard: string | null = null;
  // The newest full buffer text; the debounce and the chain both read it at
  // fire time, so intermediate states are skipped rather than queued.
  let latest: string | null = null;
  let conflicted = false;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let detached = false, revision = 0, baseline = 0, queued = 0;
  let attempt: { content: string; baseHash: string; guard: string; id: string } | null = null;
  let state: PlainSaveState = { kind: "idle" };
  // Saves run strictly one after another, each against the hash the
  // previous exchange established.
  let chain: Promise<void> = Promise.resolve();

  const notify = (next: PlainSaveState) => { state = next; options.onState(next); };
  const clearTimer = () => { if (timer !== null) clearTimeout(timer); timer = null; };
  const dirty = () => latest !== null && latest !== baseContent;
  const schedule = () => { timer = setTimeout(() => { timer = null; void flush(); }, debounce); };

  const flush = (force = false, explicit = false): Promise<boolean> => {
    if (detached && (!explicit || force)) return Promise.resolve(!dirty() && !attempt && !conflicted);
    if (detached && queued) return Promise.resolve(false);
    const heldTarget = detached ? attempt?.content ?? latest : null;
    const originalBaseline = baseline;
    queued++;
    let succeeded = true;
    chain = chain.then(async () => {
      if (baseline !== originalBaseline || conflicted && !force) { succeeded = false; return; }
      const target = detached ? heldTarget : latest;
      if (target === null || target === baseContent && !attempt) return;
      notify({ kind: "saving" });
      try {
        let saved;
        if (detached) {
          if (!options.publish || !baseGuard || !baseHash) throw Error("Saved source authority is unavailable. Keep the draft and review the saved file before retrying.");
          attempt ??= { content: target, baseHash, guard: baseGuard, id: crypto.randomUUID().replaceAll("-", "") };
          saved = await options.publish(attempt, () => baseline === originalBaseline);
          if (baseline !== originalBaseline) { succeeded = false; return; }
          attempt = null;
        } else {
          saved = await options.put(target, baseHash, force, baseGuard);
          if (baseline !== originalBaseline) { succeeded = false; return; }
        }
        baseHash = saved.hash;
        baseContent = target;
        baseGuard = saved.save_guard ?? null;
        revision++;
        conflicted = false;
        notify(!dirty() ? { kind: "saved" } : { kind: detached ? "draft" : "saving" });
      } catch (e) {
        succeeded = false;
        if ((e as { status?: number }).status === 409) {
          conflicted = true;
          notify({ kind: "conflict" });
          return;
        }
        notify({
          kind: "error",
          message: e instanceof Error ? e.message : String(e),
        });
      }
    });
    return chain.then(() => succeeded && !conflicted && !dirty()).finally(() => { queued--; });
  };

  return {
    load(content: string, hash: string, guard?: string | null) {
      clearTimer();
      revision++;
      baseline++;
      baseHash = hash;
      baseContent = content;
      baseGuard = guard ?? null;
      latest = null;
      conflicted = false;
      attempt = null;
      notify({ kind: "idle" });
    },
    changed(next: string) {
      revision++;
      latest = next;
      notify(conflicted ? { kind: "conflict" } : next === baseContent && !attempt ? { kind: detached ? "idle" : "saved" } : { kind: detached ? "draft" : "saving" });
      clearTimer();
      if (!detached) schedule();
    },
    hasPendingEdits() {
      return dirty() || attempt !== null;
    },
    flushNow() {
      clearTimer();
      return flush();
    },
    saveNow() { clearTimer(); return flush(false, true); },
    isDetached() { return detached; },
    restoreDetached() {
      if (queued) throw Error("Wait for the current save before restoring this private draft.");
      clearTimer(); detached = true; revision++;
      notify(dirty() ? { kind: "draft" } : { kind: "idle" });
    },
    prepareDetach(base, guard) {
      if (!guard || !baseHash || guard !== baseGuard || base !== baseContent || conflicted || queued || attempt) return null;
      const captured = revision, previousText = latest, previousState = state, previousDetached = detached;
      const hadTimer = timer !== null;
      let committed = false;
      const current = () => !committed && revision === captured && !queued;
      return {
        current,
        commit() {
          if (!current()) throw Error("The plain file save plane changed. Review the draft again.");
          clearTimer(); detached = true; committed = true; revision++;
        },
        rollback() {
          if (!committed) return;
          committed = false; revision++; latest = previousText; detached = previousDetached;
          clearTimer();
          if (hadTimer) schedule();
          notify(previousState);
        },
      };
    },
    discardPending() {
      clearTimer(); revision++; baseline++;
      latest = null;
      conflicted = false;
      attempt = null;
      notify({ kind: "idle" });
      return baseContent;
    },
    baseContent() {
      return baseContent;
    },
    baseGuard() {
      return baseGuard;
    },
    resolve(how: "overwrite" | "reload") {
      if (!conflicted) return;
      if (how === "overwrite") {
        if (!detached) void flush(true);
      } else {
        // The caller is adopting the disk copy and will load() it; until
        // then nothing must fire against a hash known to be stale.
        conflicted = false;
        latest = null;
        attempt = null;
        revision++; baseline++;
        clearTimer();
      }
    },
    dispose() {
      clearTimer(); revision++;
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
