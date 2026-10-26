<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/outputSave.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ff2a9a5b4231"></a>
## `apps/web/src/lib/outputSave.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Save a generated edit once and retain it when saving fails](../../../../../guarantees/authoring/an-output-edit-lands-in-its-document/contract.md), [Saved means the displayed bytes reached the file](../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md), [Close or recover a document without losing the wrong draft](../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md).

<a id="f-ff2a9a5b4231-OutputSaverOptions"></a>
### `OutputSaverOptions`

The debounced save loop behind an editable generated file.
The pane reports the whole buffer; this turns it into byte edits against a
BASELINE — the last content the server is known to hold — and posts them.
The baseline is the whole correctness story: it advances only when a save
succeeds (or a fresh load arrives), never on a render, because a diff
computed against anything else re-sends edits the server already applied.
That was a real bug: the woven markdown's document gained a duplicate of
every earlier edit on each save, compounding until edits stopped mapping
at all (see docs/guarantees/authoring/an-output-edit-lands-in-its-document.md).
Saves are serialized: a new diff is computed only after the previous POST
settles, against the baseline that POST left behind. Two in-flight saves
diffed from the same baseline would race the server into the same
duplication the baseline rule exists to prevent.

<storycode:copy id="f-ff2a9a5b4231-OutputSaverOptions">
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

</storycode:copy>
<a id="f-ff2a9a5b4231-OutputSaver"></a>
### `OutputSaver`

<storycode:copy id="f-ff2a9a5b4231-OutputSaver">
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

</storycode:copy>
<a id="f-ff2a9a5b4231-createOutputSaver"></a>
### Retain confirmed and current content

Keep the confirmed server baseline separate from the latest local buffer. Intermediate keystrokes may be coalesced, but a render must not advance the baseline or forget an edit that the server has not accepted.

<storycode:copy id="f-ff2a9a5b4231-createOutputSaver">
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

</storycode:copy>
<a id="f-ff2a9a5b4231-createOutputSaver--serialize-save-requests"></a>
### Serialize save requests

Chain requests so a diff is calculated only after the preceding request settles. Advance the baseline to the exact text sent on success. On failure retain the old baseline, report the error and let a retry include the unsaved change.

<storycode:copy id="f-ff2a9a5b4231-createOutputSaver--serialize-save-requests">
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

</storycode:copy>
<a id="f-ff2a9a5b4231-createOutputSaver--report-and-flush-pending-edits"></a>
### Report and flush pending edits

Loads establish a fresh baseline; changes replace the latest desired buffer and restart the debounce. Pending state compares that buffer with acknowledged content, so a later re-weave cannot overwrite keystrokes still waiting to save.

<storycode:copy id="f-ff2a9a5b4231-createOutputSaver--report-and-flush-pending-edits">
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
</storycode:copy>
<a id="f-ff2a9a5b4231-createOutputSaver--discard-only-the-inspected-buffer"></a>
### Discard only the inspected buffer

Check the expected buffer both before and after waiting for earlier saves. If it changed during the wait, refuse the discard rather than clearing a newer draft. Disposal cancels only the timer; already submitted requests settle through their existing chain.

<storycode:copy id="f-ff2a9a5b4231-createOutputSaver--discard-only-the-inspected-buffer">
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
</storycode:copy>


### Assemble `apps/web/src/lib/outputSave.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/outputSave.ts"><storycode:paste select="#f-ff2a9a5b4231-OutputSaverOptions"/><storycode:paste select="#f-ff2a9a5b4231-OutputSaver"/><storycode:paste select="#f-ff2a9a5b4231-createOutputSaver"/><storycode:paste select="#f-ff2a9a5b4231-createOutputSaver--serialize-save-requests"/><storycode:paste select="#f-ff2a9a5b4231-createOutputSaver--report-and-flush-pending-edits"/><storycode:paste select="#f-ff2a9a5b4231-createOutputSaver--discard-only-the-inspected-buffer"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-5dd719c6fb43"></a>
## `apps/web/src/lib/outputSave.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Save a generated edit once and retain it when saving fails](../../../../../guarantees/authoring/an-output-edit-lands-in-its-document/contract.md), [Saved means the displayed bytes reached the file](../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md), [Close or recover a document without losing the wrong draft](../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md).

<a id="f-5dd719c6fb43-LOADED"></a>
### `LOADED`

The generated-file save loop: diffs go to the server against the content
the server actually holds.
Reproduces a real bug: the pane reset its baseline to the FIRST-load weave
on every render, so the second save re-sent the first edit and the woven
markdown's document gained " Truly nobody." twice (observed live against
`.dev/project/debugging.md`). Protects the app-pane half of

<storycode:copy id="f-5dd719c6fb43-LOADED">
// The generated-file save loop: diffs go to the server against the content
// the server actually holds.
//
// Reproduces a real bug: the pane reset its baseline to the FIRST-load weave
// on every render, so the second save re-sent the first edit and the woven
// markdown's document gained " Truly nobody." twice (observed live against
// `.dev/project/debugging.md`). Protects the app-pane half of
// docs/guarantees/authoring/an-output-edit-lands-in-its-document.md.

import { describe, expect, it } from "vitest";
import { applyEdits, type TextEdit } from "./diff";
import { createOutputSaver } from "./outputSave";

const LOADED = "# Title\n\nanybody stepping at all.\n\nMore prose.\n";

</storycode:copy>
<a id="f-5dd719c6fb43-saver"></a>
### `saver`

<storycode:copy id="f-5dd719c6fb43-saver">
function saver(post: (edits: TextEdit[]) => Promise<unknown>) {
  const errors: (string | null)[] = [];
  const s = createOutputSaver({
    post,
    onError: (m) => errors.push(m),
    debounceMs: 1,
  });
  s.load(LOADED);
  return { s, errors };
}

</storycode:copy>
<a id="f-5dd719c6fb43-settle"></a>
### `settle`

<storycode:copy id="f-5dd719c6fb43-settle">
const settle = () => new Promise((resolve) => setTimeout(resolve, 10));

</storycode:copy>
<a id="f-5dd719c6fb43-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-5dd719c6fb43-expression_statement">
describe("createOutputSaver", () => {
  it("does not re-send an already-applied edit on the next save", async () => {
    // A model server: applies each byte-edit batch to its own copy.
    let server = LOADED;
    const batches: TextEdit[][] = [];
    const { s } = saver(async (edits) => {
      batches.push(edits);
      server = applyEdits(server, edits); // LOADED is ASCII: bytes == chars
    });

    const once = LOADED.replace("at all.", "at all. Truly nobody.");
    s.changed(once);
    await settle();

    const twice = once.replace("Truly nobody.", "Truly nobody. And again.");
    s.changed(twice);
    await settle();

    expect(batches).toHaveLength(2);
    // The bug: the second batch was diff(first load, buffer), so the server —
    // which had already applied the first edit — gained it a second time:
    // "at all. Truly nobody. And again. Truly nobody."
    expect(server).toBe(twice);
    expect(server.match(/Truly nobody\./g)).toHaveLength(1);
  });

  it("keeps the failed edit in the next diff, and clears the error on success", async () => {
    let server = LOADED;
    let refuse = true;
    const { s, errors } = saver(async (edits) => {
      if (refuse) throw new Error("edit overlaps a synthetic range");
      server = applyEdits(server, edits);
    });

    const once = LOADED.replace("More prose.", "More prose. Kept.");
    s.changed(once);
    await settle();
    expect(errors).toEqual(["edit overlaps a synthetic range"]);
    expect(server).toBe(LOADED);

    // The server refused, so its bytes are unchanged; the next save must
    // carry the first edit again — the baseline did not advance.
    refuse = false;
    const twice = once.replace("# Title", "# Title!");
    s.changed(twice);
    await settle();
    expect(server).toBe(twice);
    expect(errors).toEqual(["edit overlaps a synthetic range", null]);
  });

  it("serializes overlapping saves so each diff sees the previous result", async () => {
    let server = LOADED;
    let release: () => void = () => {};
    const gate = new Promise<void>((resolve) => {
      release = resolve;
    });
    let first = true;
    const batches: TextEdit[][] = [];
    const { s } = saver(async (edits) => {
      batches.push(edits);
      if (first) {
        first = false;
        await gate; // the first POST hangs; typing continues meanwhile
      }
      server = applyEdits(server, edits);
    });

    const once = LOADED.replace("at all.", "at all. One.");
    s.changed(once);
    await settle();

    const twice = once.replace("One.", "One. Two.");
    s.changed(twice);
    await settle();

    release();
    await settle();

    expect(server).toBe(twice);
    expect(server.match(/One\./g)).toHaveLength(1);
  });

  it("a reload replaces the baseline", async () => {
    let server = LOADED;
    const { s } = saver(async (edits) => {
      server = applyEdits(server, edits);
    });

    const rewoven = "fresh weave\n";
    s.load(rewoven);
    server = rewoven;
    s.changed("fresh weave, edited\n");
    await settle();
    expect(server).toBe("fresh weave, edited\n");
  });

  // hasPendingEdits is the guard that keeps a re-weave arriving over the wire
  // from overwriting keystrokes the server has not confirmed yet.
  it("reports pending edits from the keystroke until the save is confirmed", async () => {
    let resolvePost: () => void = () => undefined;
    const { s } = saver(
      () =>
        new Promise<void>((resolve) => {
          resolvePost = resolve;
        }),
    );

    expect(s.hasPendingEdits()).toBe(false); // freshly loaded

    const edited = LOADED.replace("at all.", "at all. Pending.");
    s.changed(edited);
    expect(s.hasPendingEdits()).toBe(true); // debounce pending

    await settle(); // debounce fired; POST is in flight
    expect(s.hasPendingEdits()).toBe(true);

    resolvePost();
    await settle();
    expect(s.hasPendingEdits()).toBe(false); // confirmed: baseline caught up
  });

  it("keeps reporting pending edits after a refused save — the diff must go again", async () => {
    const { s } = saver(async () => {
      throw new Error("edit overlaps a synthetic range");
    });
    s.changed(LOADED.replace("at all.", "at all. Refused."));
    await settle();
    expect(s.hasPendingEdits()).toBe(true);
  });
});

</storycode:copy>
<a id="f-5dd719c6fb43-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-5dd719c6fb43-expression_statement-2">
// Guarantee: docs/guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file.md
it("Save fails while a generated edit is refused, then retries the same pending bytes", async () => {
  let refuse = true;
  let remote = LOADED;
  const { s } = saver(async edits => {
    if (refuse) throw new Error("cannot reverse this edit");
    remote = applyEdits(remote, edits);
  });
  s.changed("changed\n");
  expect(await s.flushNow()).toBe(false);
  expect(remote).toBe(LOADED);
  expect(s.hasPendingEdits()).toBe(true);
  refuse = false;
  expect(await s.flushNow()).toBe(true);
  expect(remote).toBe("changed\n");
  expect(s.hasPendingEdits()).toBe(false);
});
</storycode:copy>
<a id="f-5dd719c6fb43-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-5dd719c6fb43-expression_statement-3">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("does not discard generated edits typed while an earlier save is settling", async () => {
  let finish!: () => void, calls = 0, remote = "base";
  const saver = createOutputSaver({ debounceMs: 60000, onError: () => {}, post: async edits => {
    if (++calls === 1) await new Promise<void>(resolve => { finish = resolve; });
    remote = applyEdits(remote, edits);
  } });
  saver.load("base"); saver.changed("first save");
  const saving = saver.flushNow(); await Promise.resolve();
  const discarding = saver.discardPending("first save");
  const refused = expect(discarding).rejects.toThrow("newer draft remains pending");
  saver.changed("newer bytes"); finish();
  expect(await saving).toBe(false); await refused;
  expect(saver.hasPendingEdits()).toBe(true); expect(remote).toBe("first save");
  expect(await saver.flushNow()).toBe(true); expect(remote).toBe("newer bytes"); saver.dispose();
});
</storycode:copy>
<a id="f-5dd719c6fb43-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-5dd719c6fb43-expression_statement-4">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("refuses an already-stale expected generated discard without clearing its pending edit", async () => {
  let remote = "base";
  const saver = createOutputSaver({ debounceMs: 60000, onError: () => {}, post: async edits => { remote = applyEdits(remote, edits); } });
  saver.load("base"); saver.changed("new draft");
  await expect(saver.discardPending("stale draft")).rejects.toThrow("changed before discard");
  expect(saver.hasPendingEdits()).toBe(true);
  expect(await saver.flushNow()).toBe(true); expect(remote).toBe("new draft"); saver.dispose();
});
</storycode:copy>

### Assemble `apps/web/src/lib/outputSave.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/outputSave.test.ts"><storycode:paste select="#f-5dd719c6fb43-LOADED"/><storycode:paste select="#f-5dd719c6fb43-saver"/><storycode:paste select="#f-5dd719c6fb43-settle"/><storycode:paste select="#f-5dd719c6fb43-expression_statement"/><storycode:paste select="#f-5dd719c6fb43-expression_statement-2"/><storycode:paste select="#f-5dd719c6fb43-expression_statement-3"/><storycode:paste select="#f-5dd719c6fb43-expression_statement-4"/></storycode:file>


</storycode:doc>
