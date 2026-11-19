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
  /** Suspend writes while the engine owns a resource transition. */
  holdWrites(): ((keepPrivate: boolean) => void) | null;
  /** Restore the prior ordinary save mode after a confirmed clean restoration. */
  resumeOrdinaryWrites(): void;
  /** Restore the private publication mode before applying retained text. */
  restoreDetached(): void;
  /** Validate and prepare the plane switch without changing the saver. */
  prepareDetach(base: string, guard: string): { current: () => boolean; commit: () => void; rollback: () => void } | null;
  /** Throw away edits that have not reached the file and return its text. */
  discardPending(): string;
  /** Cancel the pending debounce (unmount). In-flight PUTs settle alone. */
  dispose(): void;
}

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
  let writeHeld = false;
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
    if (writeHeld) return Promise.resolve(false);
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
      notify(conflicted ? { kind: "conflict" } : next === baseContent && !attempt ? { kind: detached ? "idle" : "saved" } : { kind: detached || writeHeld ? "draft" : "saving" });
      clearTimer();
      if (!detached && !writeHeld) schedule();
    },
    hasPendingEdits() {
      return dirty() || attempt !== null;
    },
    flushNow() {
      clearTimer();
      return flush();
    },
    saveNow() { clearTimer(); return flush(false, true); },
    isDetached() { return detached || writeHeld; },
    holdWrites() {
      if (writeHeld || queued || conflicted || attempt) return null;
      writeHeld = true;
      clearTimer();
      let released = false;
      return keepPrivate => {
        if (released) return;
        released = true;
        writeHeld = false;
        if (keepPrivate && !detached) {
          detached = true;
          revision++;
          notify(dirty() ? {kind: "draft"} : {kind: "idle"});
        }
        if (!detached && dirty()) schedule();
      };
    },
    resumeOrdinaryWrites() {
      if (queued || dirty() || !baseGuard || attempt || conflicted) throw Error("The restored source has no clean saved baseline.");
      detached = false;
      revision++;
    },
    restoreDetached() {
      if (queued) throw Error("Wait for the current save before restoring this private draft.");
      clearTimer(); detached = true; revision++;
      notify(dirty() ? { kind: "draft" } : { kind: "idle" });
    },
    prepareDetach(base, guard) {
      if (!guard || !baseHash || guard !== baseGuard || base !== baseContent || writeHeld || conflicted || queued || attempt) return null;
      const captured = revision, previousText = latest, previousState = state, previousDetached = detached;
      const hadTimer = timer !== null;
      let committed = false;
      const current = () => !committed && revision === captured && !queued && !writeHeld;
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
