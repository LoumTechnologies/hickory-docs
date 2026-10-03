import { describe, expect, it } from "vitest";
import { applyEdits, computeEdits, toByteEdits } from "./diff";
import { byteLength } from "./offsets";

function roundTrip(oldText: string, newText: string) {
  const edits = computeEdits(oldText, newText);
  expect(applyEdits(oldText, edits)).toBe(newText);
  return edits;
}

describe("computeEdits", () => {
  it("returns no edits for identical text", () => {
    expect(computeEdits("a\nb\n", "a\nb\n")).toEqual([]);
  });

  it("produces one tight edit for a single-line change", () => {
    const edits = roundTrip("a\nhello world\nb\n", "a\nhello there\nb\n");
    expect(edits).toHaveLength(1);
    expect(edits[0]).toEqual({ start: 8, end: 13, text: "there" });
  });

  it("produces separate edits for changes in distant lines", () => {
    const oldText = "one\ntwo\nthree\nfour\nfive\n";
    const newText = "ONE\ntwo\nthree\nfour\nFIVE\n";
    const edits = roundTrip(oldText, newText);
    expect(edits).toHaveLength(2);
    expect(edits[0].text).toBe("ONE");
    expect(edits[1].text).toBe("FIVE");
  });

  it("handles pure insertion and pure deletion", () => {
    const ins = roundTrip("a\nc\n", "a\nb\nc\n");
    expect(ins).toHaveLength(1);
    expect(ins[0].start).toBe(ins[0].end);

    const del = roundTrip("a\nb\nc\n", "a\nc\n");
    expect(del).toHaveLength(1);
    expect(del[0].text).toBe("");
  });

  it("handles append at end without trailing newline", () => {
    roundTrip("a\nb", "a\nb\nc");
    roundTrip("", "content\n");
    roundTrip("content\n", "");
  });

  it("keeps unchanged interior lines out of the edits", () => {
    const oldText = "h1\nsame\nsame\nold\nsame\n";
    const newText = "h1\nsame\nsame\nnew\nsame\n";
    const edits = roundTrip(oldText, newText);
    expect(edits).toHaveLength(1);
    expect(oldText.slice(edits[0].start, edits[0].end)).toBe("old");
  });

  it("toByteEdits converts char offsets to UTF-8 byte offsets", () => {
    const oldText = "naïve\nline\n"; // ï is 2 bytes in UTF-8
    const newText = "naïve\nLine\n";
    const edits = computeEdits(oldText, newText);
    expect(edits).toHaveLength(1);
    const bytes = toByteEdits(oldText, edits);
    // char offset of "l" is 6; byte offset is 7 (ï = 2 bytes).
    expect(edits[0].start).toBe(6);
    expect(bytes[0].start).toBe(7);
    expect(byteLength(oldText)).toBe(oldText.length + 1);
  });

  it("survives emoji (surrogate pairs) in the buffer", () => {
    const oldText = "x 🎉 y\n";
    const newText = "x 🎉 z\n";
    const edits = roundTrip(oldText, newText);
    const bytes = toByteEdits(oldText, edits);
    expect(bytes[0].start).toBe(byteLength("x 🎉 "));
  });
});

// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
import { unifiedDiff } from "./diff";
it("shows a final-newline-only edit and an added empty line", () => {
  expect(unifiedDiff("note.md", "note", "note\n")).toContain("-note\n\\ No newline at end of file\n+note");
  expect(unifiedDiff("note.md", "", "\n")).toContain("@@ -0,0 +1,1 @@\n+\n");
});
