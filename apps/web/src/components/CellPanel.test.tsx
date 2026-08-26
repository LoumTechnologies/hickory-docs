// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { CellPanel } from "./CellPanel";
import { hasReplay } from "../lib/railActions";
import type { ExecBlock, TranscriptEvent } from "../api/types";

afterEach(cleanup);

const transcript: TranscriptEvent[] = [
  { t: 0, kind: "cmd", data: "sort fruit.csv" },
  { t: 300, kind: "out", data: "apple,3\nbanana,5\n" },
  { t: 350, kind: "exit", code: 0 },
];

function cell(overrides: Partial<ExecBlock>): ExecBlock {
  return {
    kind: "exec",
    id: "c1",
    container: "shell",
    command: "sort fruit.csv",
    span: [0, 10],
    transcript,
    ...overrides,
  };
}

describe("CellPanel de-duplication", () => {
  it("verified cell (ok + expect): no transcript by default, ✓ affordance, Replay reveals it", () => {
    const { container, rerender } = render(
      <CellPanel
        block={cell({ status: "ok", expect: { match: "exact", body: "apple,3\nbanana,5\n" } })}
      />,
    );
    // The expect body in the source is the single visible copy of the output.
    expect(container.querySelector(".transcript")).toBeNull();
    expect(screen.getByText("✓ output verified")).toBeTruthy();
    // No separate "expects …" summary chip anymore.
    expect(container.textContent).not.toContain("expects exact");
    // Nothing in the panel is clickable now — the Replay verb is an icon on
    // the action rail, and the panel only reflects the state it is handed.
    expect(container.querySelector("button")).toBeNull();
    rerender(
      <CellPanel
        block={cell({ status: "ok", expect: { match: "exact", body: "apple,3\nbanana,5\n" } })}
        replay
      />,
    );
    expect(container.querySelector(".transcript")).toBeTruthy();
  });

  it("failed cell with expect: compact line diff of actual vs expected", () => {
    const { container } = render(
      <CellPanel
        block={cell({
          status: "failed",
          transcript: [
            { t: 0, kind: "cmd", data: "sort fruit.csv" },
            { t: 300, kind: "out", data: "apple,3\ncherry,7\n" },
            { t: 350, kind: "exit", code: 1 },
          ],
          expect: { match: "exact", body: "apple,3\nbanana,5\n" },
        })}
      />,
    );
    const diff = screen.getByTestId("cell-diff");
    expect(diff.textContent).toContain("- banana,5");
    expect(diff.textContent).toContain("+ cherry,7");
    // Transcript stays collapsed behind Replay (the diff is the second copy).
    expect(container.querySelector(".transcript")).toBeNull();
    // The Replay verb lives on the rail; `hasReplay` is what puts it there,
    // and it agrees with the panel about there being something to reveal.
    expect(
      hasReplay({ status: "failed", hasExpect: true, transcriptLength: 3, running: false }),
    ).toBe(true);
  });

  it("cell without expect keeps its transcript visible — that IS the output", () => {
    const { container } = render(<CellPanel block={cell({ status: "ok" })} />);
    expect(container.querySelector(".transcript")).toBeTruthy();
    // What the panel shows is the watching binding of the terminal, not a
    // `<pre>` — so what it *draws* is asserted in
    // `terminal/WatchingTerminal.test.tsx` against a real emulator, and what
    // it is *handed* in `lib/watchStream.test.ts`. All this panel decides is
    // whether it appears at all.
    expect(container.querySelector("[data-testid='watch-terminal']")).toBeTruthy();
  });
});
