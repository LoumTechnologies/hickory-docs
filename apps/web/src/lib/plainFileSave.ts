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

export interface PlainSaverOptions {
  /** PUT the whole content under `baseHash`; resolves to the new hash. A
   * rejection whose `status` is 409 is the disk-moved conflict. */
  put: (content: string, baseHash: string, force: boolean) => Promise<{ hash: string }>;
  onState: (state: PlainSaveState) => void;
  /** Pause that turns keystrokes into a save. Default 600ms. */
  debounceMs?: number;
}

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
  flushNow(): void;
  /** Cancel the pending debounce (unmount). In-flight PUTs settle alone. */
  dispose(): void;
}

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

  const flush = (force = false) => {
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
      flush();
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
