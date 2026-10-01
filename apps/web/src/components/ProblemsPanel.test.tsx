// Protects docs/guarantees/editor-intelligence/a-problem-count-opens-a-list.md

import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ProblemsPanel, problemRows } from "./ProblemsPanel";

afterEach(cleanup);

const at = (line: number, severity: number, message: string) => ({
  range: { start: { line, character: 0 }, end: { line, character: 4 } },
  severity,
  message,
});

describe("the rows behind the count", () => {
  it("puts the worst first, then orders by where they are", () => {
    const rows = problemRows([
      { docId: "b", path: "b.md", diagnostics: [at(9, 2, "a warning"), at(2, 1, "later error")] },
      { docId: "a", path: "a.md", diagnostics: [at(4, 1, "earlier error")] },
    ]);
    expect(rows.map((r) => r.diagnostic.message)).toEqual([
      "earlier error", // error, a.md
      "later error", // error, b.md
      "a warning", // warning, last whatever its line
    ]);
  });

  it("leaves out information and hints, the way the count does", () => {
    // A list that reaches four hundred rows is a list nobody opens, and the
    // number in the status bar would stop matching what opening it shows.
    const rows = problemRows([
      {
        docId: "a",
        path: "a.md",
        diagnostics: [at(1, 3, "consider this"), at(2, 4, "a hint"), at(3, 2, "a warning")],
      },
    ]);
    expect(rows.map((r) => r.diagnostic.message)).toEqual(["a warning"]);
  });
});

describe("the panel", () => {
  it("lists each problem with where it is, and reports the pick", () => {
    const onPick = vi.fn();
    const rows = problemRows([
      { docId: "d1", path: "notes/a.md", diagnostics: [at(11, 1, "undefined name `foo`")] },
    ]);
    render(<ProblemsPanel rows={rows} onPick={onPick} onClose={() => {}} />);

    expect(screen.getByText("1 problem")).toBeTruthy();
    expect(screen.getByText("undefined name `foo`")).toBeTruthy();
    // One-based, like every editor's gutter and every compiler's output.
    expect(screen.getByText("notes/a.md:12")).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: /undefined name/ }));
    expect(onPick).toHaveBeenCalledWith(rows[0]);
  });

  it("says nothing is wrong rather than showing an empty box", () => {
    render(<ProblemsPanel rows={[]} onPick={() => {}} onClose={() => {}} />);
    expect(screen.getByText("Nothing is wrong right now")).toBeTruthy();
  });
});
