import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";

import { StatusBar } from "./StatusBar";

afterEach(() => {
  cleanup();
});

const bar = (over: Partial<React.ComponentProps<typeof StatusBar>> = {}) =>
  render(
    <StatusBar
      problems={{ errors: 0, warnings: 0 }}
      needsAttention={0}
      path={null}
      caret={null}
      zoom={1}
      {...over}
    />,
  );

describe("what the status bar always says", () => {
  it("gives the counts a name, because the digits alone are ambiguous", () => {
    bar({ problems: { errors: 2, warnings: 1 } });
    expect(screen.getByRole("button", { name: "2 errors, 1 warning" })).toBeTruthy();
  });

  it("says so plainly when nothing is wrong", () => {
    bar();
    expect(screen.getByRole("button", { name: "No problems" })).toBeTruthy();
  });

  it("goes to the next problem when the counts are pressed", () => {
    const onProblems = vi.fn();
    bar({ problems: { errors: 1, warnings: 0 }, onProblems });
    fireEvent.click(screen.getByRole("button", { name: "1 error" }));
    expect(onProblems).toHaveBeenCalled();
  });
});

describe("what it leaves out", () => {
  it("spends no slot on a constant 100% zoom", () => {
    // A status bar that always says "100%" has given a permanent row to
    // something nobody reads.
    bar({ zoom: 1 });
    expect(screen.queryByText("100%")).toBeNull();
    cleanup();
    bar({ zoom: 1.5 });
    expect(screen.getByText("150%")).toBeTruthy();
  });

  it("says nothing about terminals when none are waiting", () => {
    bar({ needsAttention: 0 });
    expect(screen.queryByText(/waiting/)).toBeNull();
  });

  it("shows the waiting count when there is one, and walks the queue", () => {
    const onAttention = vi.fn();
    bar({ needsAttention: 2, onAttention });
    const button = screen.getByRole("button", { name: "2 terminals needing you" });
    fireEvent.click(button);
    expect(onAttention).toHaveBeenCalled();
  });

  it("shows no caret position when nothing is focused", () => {
    bar({ caret: null });
    expect(screen.queryByText(/^Ln /)).toBeNull();
  });
});

describe("what it says about the view", () => {
  it("shows the caret and the path of the focused file", () => {
    bar({ caret: { line: 42, column: 7 }, path: "src/main.rs" });
    expect(screen.getByText("Ln 42, Col 7")).toBeTruthy();
    expect(screen.getByText("src/main.rs")).toBeTruthy();
  });
});
