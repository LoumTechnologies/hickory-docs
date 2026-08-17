// The generated-file save loop: diffs go to the server against the content
// the server actually holds.
//
// Reproduces a real bug: the pane reset its baseline to the FIRST-load weave
// on every render, so the second save re-sent the first edit and the woven
// markdown's document gained " Truly nobody." twice (observed live against
// `.dev/project/debugging.hick`). Protects the app-pane half of
// docs/guarantees/authoring/an-output-edit-lands-in-its-document.md.

import { describe, expect, it } from "vitest";
import { applyEdits, type TextEdit } from "./diff";
import { createOutputSaver } from "./outputSave";

const LOADED = "# Title\n\nanybody stepping at all.\n\nMore prose.\n";

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

const settle = () => new Promise((resolve) => setTimeout(resolve, 10));

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
