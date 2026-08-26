import { describe, expect, it } from "vitest";

import { actionsFor, hasReplay } from "./railActions";

describe("which icons a card puts on the rail", () => {
  it("gives a cell its run and source icons, in that order", () => {
    // Order is the contract: the icon nearest the line is the one a cell
    // exists for, and a reader learns the column's shape once.
    expect(actionsFor("exec")).toEqual(["run", "source"]);
  });

  it("adds replay last, so appearing does not move the other two", () => {
    expect(actionsFor("exec", { replay: true })).toEqual(["run", "source", "replay"]);
  });

  it("gives a diagram only the way back to its source", () => {
    // A diagram has no verb of its own — nothing to run, nothing to replay.
    expect(actionsFor("diagram")).toEqual(["source"]);
  });

  it("takes the source icon off the cell that draws a picture", () => {
    // Two states, not three: the picture owns the toggle, the cell keeps Run.
    expect(actionsFor("exec", { source: false })).toEqual(["run"]);
    expect(actionsFor("exec", { source: false, replay: true })).toEqual(["run", "replay"]);
  });

  it("gives a picture only the way back to the code that drew it", () => {
    // Running belongs to the cell inside the file block, which has its own
    // Run icon; a second one here would run the same cell twice over.
    expect(actionsFor("picture")).toEqual(["source"]);
  });

  it("gives a prose fence only the converter", () => {
    expect(actionsFor("fence")).toEqual(["convert"]);
  });
});

describe("whether a cell has anything to replay", () => {
  const cell = {
    status: "ok",
    hasExpect: true,
    transcriptLength: 3,
    running: false,
  };

  it("offers replay when an expect block is standing in for the output", () => {
    expect(hasReplay(cell)).toBe(true);
  });

  it("offers nothing while the cell is running — the transcript is already live", () => {
    expect(hasReplay({ ...cell, running: true })).toBe(false);
  });

  it("offers nothing when the transcript is already the visible output", () => {
    // No expect block means nothing stands in for the transcript, so it is
    // simply shown and there is no second state to toggle to.
    expect(hasReplay({ ...cell, hasExpect: false })).toBe(false);
  });

  it("offers nothing when there is no transcript to reveal", () => {
    expect(hasReplay({ ...cell, transcriptLength: 0 })).toBe(false);
  });

  it("offers nothing for a cell the server has not run", () => {
    expect(hasReplay({ ...cell, status: undefined })).toBe(false);
  });

  it("covers a failed cell too — the diff is a summary, not what ran", () => {
    expect(hasReplay({ ...cell, status: "failed" })).toBe(true);
  });
});
