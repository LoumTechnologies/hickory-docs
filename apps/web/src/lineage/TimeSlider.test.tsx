// docs/guarantees/lineage/replay-recomputes-lineage-at-a-commit.md
import { describe, expect, it, vi, beforeEach } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";

import { TimeSlider, indexOf, positions } from "./TimeSlider";
import type { ReplayCommit } from "../api/types";

const commit = (over: Partial<ReplayCommit>): ReplayCommit => ({
  sha: "aaa",
  short: "aaa",
  time: 1_700_000_000,
  author: "T",
  subject: "a change",
  path: "note.md",
  ...over,
});

// git reports newest first; a person reads time oldest → newest.
const commits = [
  commit({ sha: "new", short: "new", subject: "newer", time: 200 }),
  commit({ sha: "old", short: "old", subject: "older", time: 100 }),
];

beforeEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("the stops", () => {
  it("run oldest to newest and end at the working tree", () => {
    // The last position is NOT a commit and must not pretend to be one.
    const stops = positions(commits);
    expect(stops.map((c) => c?.sha ?? null)).toEqual(["old", "new", null]);
  });

  it("puts an unknown selection at the working tree rather than at nothing", () => {
    expect(indexOf(commits, null)).toBe(2);
    expect(indexOf(commits, "old")).toBe(0);
    expect(indexOf(commits, "gone")).toBe(2);
  });
});

describe("the slider", () => {
  it("names the commit it is showing, and says the last stop is not one", () => {
    render(<TimeSlider commits={commits} at={null} onChange={() => {}} />);
    expect(screen.getByText(/As it stands/)).toBeTruthy();
    expect(screen.getByText(/the working tree, which is not a commit/)).toBeTruthy();
  });

  it("moves to the commit at the position picked", () => {
    const onChange = vi.fn();
    render(<TimeSlider commits={commits} at={null} onChange={onChange} />);
    fireEvent.change(screen.getByRole("slider"), { target: { value: "0" } });
    expect(onChange).toHaveBeenCalledWith("old");
  });

  it("states the grammar boundary rather than showing an error", () => {
    render(
      <TimeSlider
        commits={commits}
        at="old"
        onChange={() => {}}
        boundary="replay stops here: Replay works back to the last grammar change"
      />,
    );
    expect(
      screen.getByText(/Replay works back to the last grammar change/),
    ).toBeTruthy();
  });

  it("says plainly that replay reads git and remembers nothing of its own", () => {
    render(<TimeSlider commits={[]} at={null} onChange={() => {}} />);
    expect(screen.getByText(/nowhere to slide to/)).toBeTruthy();
  });

  it("never claims to relate one version's spans to another's", () => {
    render(<TimeSlider commits={commits} at={null} onChange={() => {}} />);
    expect(screen.getByText(/relating one version/)).toBeTruthy();
  });
});
