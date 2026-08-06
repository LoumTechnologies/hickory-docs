// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { CellPanel } from "./CellPanel";
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
    const { container } = render(
      <CellPanel
        block={cell({ status: "ok", expect: { match: "exact", body: "apple,3\nbanana,5\n" } })}
      />,
    );
    // The expect body in the source is the single visible copy of the output.
    expect(container.querySelector(".transcript")).toBeNull();
    expect(screen.getByText("✓ output verified")).toBeTruthy();
    // No separate "expects …" summary chip anymore.
    expect(container.textContent).not.toContain("expects exact");
    fireEvent.click(screen.getByRole("button", { name: "Replay" }));
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
    expect(screen.getByRole("button", { name: "Replay" })).toBeTruthy();
  });

  it("cell without expect keeps its transcript visible — that IS the output", () => {
    const { container } = render(<CellPanel block={cell({ status: "ok" })} />);
    expect(container.querySelector(".transcript")).toBeTruthy();
    // Single-command transcript: the `$ cmd` echo is suppressed (the command
    // is the source text directly above the panel).
    expect(container.querySelector(".terminal")!.textContent).not.toContain("sort fruit.csv");
    expect(container.querySelector(".terminal")!.textContent).toContain("apple,3");
  });

  it("multi-command transcripts keep their `$` prompts so outputs stay attributable", () => {
    const { container } = render(
      <CellPanel
        block={cell({
          status: "ok",
          transcript: [
            { t: 0, kind: "cmd", data: "printf 'a\\n' > f" },
            { t: 100, kind: "cmd", data: "cat f" },
            { t: 200, kind: "out", data: "a\n" },
            { t: 250, kind: "exit", code: 0 },
          ],
        })}
      />,
    );
    expect(container.querySelector(".terminal")!.textContent).toContain("$ cat f");
  });
});
