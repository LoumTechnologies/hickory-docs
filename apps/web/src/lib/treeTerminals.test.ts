import { describe, expect, it } from "vitest";
import {
  hiddenSummary,
  rankSessions,
  urgentCount,
} from "./treeTerminals";

const s = (title: string, state: string) => ({ title, state });

// Guarantee: docs/guarantees/terminal/a-session-appears-where-it-is-working.md
describe("which terminal node comes first", () => {
  it("ranks by urgency, not by when it started", () => {
    // Sorting by start time would make the one thing worth acting on wander
    // as other sessions come and go.
    const order = rankSessions([
      s("idle-one", "idle"),
      s("build", "working"),
      s("prompt", "needs-you"),
      s("broken", "failed"),
    ]).map((x) => x.title);
    expect(order).toEqual(["prompt", "broken", "build", "idle-one"]);
  });

  it("is stable while nothing changes state", () => {
    const order = rankSessions([s("zeta", "working"), s("alpha", "working")]);
    expect(order.map((x) => x.title)).toEqual(["alpha", "zeta"]);
  });

  it("sorts a state it does not know LAST", () => {
    // A build we cannot interpret must not outrank a question somebody is
    // waiting on.
    const order = rankSessions([s("mystery", "sideways"), s("prompt", "needs-you")]);
    expect(order.map((x) => x.title)).toEqual(["prompt", "mystery"]);
  });
});

describe("which terminal nodes need attention", () => {
  it("counts the urgent ones", () => {
    expect(urgentCount([s("a", "needs-you"), s("b", "working"), s("c", "failed")])).toBe(2);
  });
});

describe("what a folded directory says", () => {
  it("says nothing when nothing is running", () => {
    expect(hiddenSummary(0, 0)).toBe("");
  });

  it("counts, and gets the plural right", () => {
    expect(hiddenSummary(1, 0)).toBe("1 terminal running in here");
    expect(hiddenSummary(3, 0)).toBe("3 terminals running in here");
  });

  it("says when one of them is waiting on you", () => {
    expect(hiddenSummary(3, 1)).toBe("3 terminals running in here, 1 needing you");
  });
});
