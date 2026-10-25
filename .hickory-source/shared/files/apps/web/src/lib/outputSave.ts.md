<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/outputSave.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [An Edit Saved In A Woven File Lands In Its Document](../../../../../../guarantees/authoring/an-output-edit-lands-in-its-document/contract.md)
- [Saved means the displayed bytes reached the file](../../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md)
- [Unsaved Work Survives Closing The App](../../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md)

Captured SHA-256: `8f758f5833ecb52926857dfd701e3b3be9488916b256c3c127f422be91196604`.

<book:entry path="apps/web/src/lib/outputSave.ts">
// The debounced save loop behind an editable generated file.
//
// The pane reports the whole buffer; this turns it into byte edits against a
// BASELINE — the last content the server is known to hold — and posts them.
// The baseline is the whole correctness story: it advances only when a save
// succeeds (or a fresh load arrives), never on a render, because a diff
// computed against anything else re-sends edits the server already applied.
// That was a real bug: the woven markdown's document gained a duplicate of
// every earlier edit on each save, compounding until edits stopped mapping
// at all (see docs/guarantees/authoring/an-output-edit-lands-in-its-document.md).
//
// Saves are serialized: a new diff is computed only after the previous POST
// settles, against the baseline that POST left behind. Two in-flight saves
// diffed from the same baseline would race the server into the same
// duplication the baseline rule exists to prevent.

import { computeEdits, toByteEdits, type TextEdit } from "./diff";

export interface OutputSaverOptions {
  /** POST the edits; resolves on 2xx, rejects with the server's message. */
  post: (edits: TextEdit[]) => Promise<unknown>;
  /** Save failed (message) or succeeded after a failure (null). */
  onError: (message: string | null) => void;
  onState?: (state: "saving" | "saved" | "failed") => void;
  /** Pause that turns keystrokes into a save. Default 600ms. */
  debounceMs?: number;
}

export interface OutputSaver {
  /** The server's copy, freshly loaded — the first baseline. */
  load(content: string): void;
  /** The buffer changed; schedule a save of its full text. */
  changed(next: string): void;
  /**
   * True while the buffer holds edits the server has not confirmed —
   * debounce pending or POST in flight. The receiving side of a re-weave
   * checks this before adopting incoming content: while true, the incoming
   * text predates what the user typed here, and overwriting the buffer with
   * it would eat their keystrokes.
   */
  hasPendingEdits(): boolean;
  baselineContent(): string;
  flushNow(): Promise<boolean>;
  discardPending(expected?: string): Promise<string>;
  /** Cancel the pending debounce (unmount). In-flight POSTs settle alone. */
  dispose(): void;
}

export function createOutputSaver(options: OutputSaverOptions): OutputSaver {
  const debounce = options.debounceMs ?? 600;
  // What the server holds. Advanced by load() and by a successful save only.
  let baseline = "";
  // The newest full buffer text; the debounce and the chain both read it at
  // fire time, so intermediate states are skipped rather than queued.
  let latest: string | null = null;
  let timer: ReturnType<typeof setTimeout> | null = null;
  // Saves run strictly one after another; each diff is computed when its
  // turn comes, against whatever baseline the previous save established.
  let chain: Promise<void> = Promise.resolve();

  const flush = () => {
    chain = chain.then(async () => {
      const target = latest;
      if (target === null) return;
      const edits = toByteEdits(baseline, computeEdits(baseline, target));
      if (edits.length === 0) return;
      try {
        await options.post(edits);
        baseline = target;
        options.onError(null);
        options.onState?.(latest === baseline ? "saved" : "saving");
      } catch (e) {
        // The baseline stays: the server refused, so it still holds the old
        // bytes, and the next save's diff must carry this edit again.
        options.onError(e instanceof Error ? e.message : String(e));
        options.onState?.("failed");
      }
    });
  };

  return {
    load(content: string) {
      baseline = content;
      latest = null;
    },
    changed(next: string) {
      latest = next;
      options.onState?.(next === baseline ? "saved" : "saving");
      if (timer !== null) clearTimeout(timer);
      timer = setTimeout(flush, debounce);
    },
    hasPendingEdits() {
      // A successful save sets baseline = the text it sent, so a settled
      // saver reports false even though `latest` is still populated.
      return latest !== null && latest !== baseline;
    },
    baselineContent() { return baseline; },
    async flushNow() {
      if (timer !== null) clearTimeout(timer);
      timer = null;
      flush();
      await chain;
      return latest === null || latest === baseline;
    },
    async discardPending(expected = latest ?? baseline) {
      if ((latest ?? baseline) !== expected) throw Error("The generated buffer changed before discard; inspect its current bytes and retry.");
      if (timer !== null) clearTimeout(timer);
      timer = null;
      await chain;
      if ((latest ?? baseline) !== expected) throw Error("The generated buffer changed during discard; its newer draft remains pending.");
      latest = null;
      options.onError(null);
      options.onState?.("saved");
      return baseline;
    },
    dispose() {
      if (timer !== null) clearTimeout(timer);
      timer = null;
    },
  };
}
</book:entry>

</book:doc>
