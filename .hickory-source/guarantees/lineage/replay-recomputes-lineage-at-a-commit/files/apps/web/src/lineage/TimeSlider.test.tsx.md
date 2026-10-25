<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lineage/TimeSlider.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Replay Recomputes Exact Lineage At Any Commit, And Names Its One Limit](../../../../../contract.md)

Captured SHA-256: `d8e40c566532905ae78d7355c0760285260ab4b3b28c4a93356e919293acf69d`.

<book:entry path="apps/web/src/lineage/TimeSlider.test.tsx">
// docs/guarantees/lineage/replay-recomputes-lineage-at-a-commit.md
import { describe, expect, it, vi, beforeEach } from "vitest";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";

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

it("shares native and model replay selection and retires replaced project handles", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const change = vi.fn();
  const viewFor = (owner: string, at: string | null, stops = commits) => <UiActionContext.Provider value={registry}><TimeSlider owner={owner} commits={stops} at={at} onChange={change} /></UiActionContext.Provider>;
  const view = render(viewFor("first", null));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const run = async (handle: string, args: Record<string, number> = {}) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle, arguments: args, request_id: crypto.randomUUID() }); });
    return result;
  };
  const pick = () => actions().find(action => action.label === "Select lineage replay stop")!.handle;
  expect(await run(pick(), { index: 0 })).toMatchObject({ status: "result", outcome: { value: { selection_requested: true, replay_completed: false } } });
  expect(change).toHaveBeenLastCalledWith("old");
  fireEvent.change(screen.getByRole("slider"), { target: { value: "1" } });
  expect(change).toHaveBeenLastCalledWith("new");
  const stale = pick();
  view.rerender(viewFor("second", "gone"));
  expect(screen.queryByText("As it stands")).toBeNull();
  expect(screen.getByText(/Selected revision is unavailable/)).toBeTruthy();
  change.mockClear();
  expect(await run(stale, { index: 0 })).not.toMatchObject({ status: "result" });
  expect(change).not.toHaveBeenCalled();
  view.rerender(viewFor("second", "gone", []));
  expect(actions().some(action => action.label === "Select lineage replay stop")).toBe(false);
  expect(await run(actions().find(action => action.label === "Show working-tree lineage")!.handle)).toMatchObject({ status: "result", outcome: { value: { at: null, read_completed: false } } });
  expect(change).toHaveBeenLastCalledWith(null);
  fireEvent.click(screen.getByRole("button", { name: "Show working-tree lineage" }));
  expect(change).toHaveBeenCalledTimes(2);
});
</book:entry>

</book:doc>
