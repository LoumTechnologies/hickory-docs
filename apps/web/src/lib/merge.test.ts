import { describe, expect, it } from "vitest";
import {
  conflictCount,
  mergeThreeWay,
  mergeTwoWay,
  mergedText,
  splitLines,
  type Resolution,
} from "./merge";

describe("three-way merge — what it can answer by itself", () => {
  const base = "one\ntwo\nthree\n";

  it("takes an edit only we made, without asking", () => {
    const regions = mergeThreeWay(base, "one\nTWO\nthree\n", base);
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe("one\nTWO\nthree\n");
  });

  it("takes an edit only they made, without asking", () => {
    const regions = mergeThreeWay(base, base, "one\ntwo\nTHREE\n");
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe("one\ntwo\nTHREE\n");
  });

  it("combines edits the two sides made in different places", () => {
    // The whole point over a two-way comparison: neither of these is a
    // question for the reader.
    const regions = mergeThreeWay(base, "ONE\ntwo\nthree\n", "one\ntwo\nTHREE\n");
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe("ONE\ntwo\nTHREE\n");
  });

  it("does not ask when both sides made the identical edit", () => {
    const same = "one\nTWO\nthree\n";
    const regions = mergeThreeWay(base, same, same);
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe(same);
  });

  it("takes a deletion only one side made", () => {
    const regions = mergeThreeWay(base, "one\nthree\n", base);
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe("one\nthree\n");
  });

  it("changes nothing when nobody changed anything", () => {
    const regions = mergeThreeWay(base, base, base);
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe(base);
  });
});

describe("three-way merge — what it has to ask about", () => {
  it("conflicts when both sides changed the same line differently", () => {
    const regions = mergeThreeWay("one\ntwo\nthree\n", "one\nOURS\nthree\n", "one\nTHEIRS\nthree\n");
    expect(conflictCount(regions)).toBe(1);
    const conflict = regions.find((r) => r.kind === "conflict");
    expect(conflict).toMatchObject({ base: "two\n", ours: "OURS\n", theirs: "THEIRS\n" });
  });

  it("honours the answer given for each conflict", () => {
    const regions = mergeThreeWay("one\ntwo\nthree\n", "one\nOURS\nthree\n", "one\nTHEIRS\nthree\n");
    const take = (choice: Resolution) => mergedText(regions, new Map([[0, choice]]));
    expect(take("ours")).toBe("one\nOURS\nthree\n");
    expect(take("theirs")).toBe("one\nTHEIRS\nthree\n");
    expect(take("base")).toBe("one\ntwo\nthree\n");
    expect(take("both")).toBe("one\nOURS\nTHEIRS\nthree\n");
  });

  it("keeps the reader's own text on screen while a conflict is unanswered", () => {
    // This text goes into the editor buffer while they are deciding. Their
    // typing vanishing mid-thought is the one unacceptable outcome.
    const regions = mergeThreeWay("one\ntwo\n", "one\nOURS\n", "one\nTHEIRS\n");
    expect(mergedText(regions)).toBe("one\nOURS\n");
  });

  it("groups a multi-line rewrite into one decision, not one per line", () => {
    const regions = mergeThreeWay("a\nb\nc\nd\n", "a\nX\nY\nd\n", "a\nP\nd\n");
    expect(conflictCount(regions)).toBe(1);
  });
});

describe("three-way merge — the laws that must always hold", () => {
  // These are diff3's own identities. They are worth asserting over a spread
  // of awkward shapes (empty files, no trailing newline, whole-file deletion)
  // because every one of them is a place an off-by-one in the walk shows up
  // as silently dropped text rather than as a crash.
  const shapes: [string, string, string][] = [
    ["one\ntwo\nthree\n", "one\nTWO\nthree\n", "one\ntwo\nTHREE\n"],
    ["", "added\n", "other\n"],
    ["a\nb\n", "", "a\nb\nc\n"],
    ["a\nb\nc\n", "a\nb\nc\nd\n", "z\na\nb\nc\n"],
    ["x\n", "y\n", "z\n"],
    ["no trailing newline", "no trailing newline!", "no trailing newline?"],
    ["a\nb\nc\nd\ne\n", "a\ne\n", "a\nb\nX\nd\ne\n"],
  ];

  it.each(shapes)("keeps OUR file when they changed nothing (%j)", (base, ours) => {
    expect(mergedText(mergeThreeWay(base, ours, base))).toBe(ours);
  });

  it.each(shapes)("keeps THEIR file when we changed nothing (%j)", (base, _ours, theirs) => {
    expect(mergedText(mergeThreeWay(base, base, theirs))).toBe(theirs);
  });

  it.each(shapes)("asks nothing when both sides agree (%j)", (base, ours) => {
    const regions = mergeThreeWay(base, ours, ours);
    expect(conflictCount(regions)).toBe(0);
    expect(mergedText(regions)).toBe(ours);
  });

  it.each(shapes)("never invents a line (%j)", (base, ours, theirs) => {
    // Every line of the result has to have come from somewhere.
    const known = new Set([...splitLines(base), ...splitLines(ours), ...splitLines(theirs)]);
    for (const line of splitLines(mergedText(mergeThreeWay(base, ours, theirs)))) {
      expect(known).toContain(line);
    }
  });
});

describe("two-way merge — honest about having no ancestor", () => {
  it("marks a differing run as a conflict rather than guessing", () => {
    // With no base there is no way to tell "they added it" from "we deleted
    // it", and a tool that guessed would throw away somebody's work.
    const regions = mergeTwoWay("one\ntwo\n", "one\ntwo\nthree\n");
    expect(conflictCount(regions)).toBe(1);
    expect(mergedText(regions, new Map([[0, "theirs"]]))).toBe("one\ntwo\nthree\n");
  });

  it("asks nothing about two identical files", () => {
    expect(conflictCount(mergeTwoWay("same\n", "same\n"))).toBe(0);
  });

  it("carries a base of empty string, so the UI can hide that button", () => {
    const conflict = mergeTwoWay("a\n", "b\n").find((r) => r.kind === "conflict");
    expect(conflict).toMatchObject({ base: "" });
  });
});
