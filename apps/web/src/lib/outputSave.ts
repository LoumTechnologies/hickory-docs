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
      } catch (e) {
        // The baseline stays: the server refused, so it still holds the old
        // bytes, and the next save's diff must carry this edit again.
        options.onError(e instanceof Error ? e.message : String(e));
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
      if (timer !== null) clearTimeout(timer);
      timer = setTimeout(flush, debounce);
    },
    hasPendingEdits() {
      // A successful save sets baseline = the text it sent, so a settled
      // saver reports false even though `latest` is still populated.
      return latest !== null && latest !== baseline;
    },
    dispose() {
      if (timer !== null) clearTimeout(timer);
      timer = null;
    },
  };
}
