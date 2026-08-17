// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { weaveOutputs } from "../lib/weave";
import { LineageColumns } from "./LineageColumns";
import { buildModel } from "./build";

afterEach(cleanup);

const DOC = `# Bisect

Prose that belongs to the document, long enough to fold.

<hick:copy id="search">def bisect_left(xs, target):
    lo, hi = 0, len(xs)
    return lo
</hick:copy>

<hick:file path="search/bisect.py" language="python">
<hick:paste select="#search"/>
</hick:file>
`;

function model() {
  return buildModel([
    { path: "notes/bisect.hick", source: DOC, outputs: weaveOutputs(DOC, "notes/bisect.hick") },
  ]);
}

describe("the lineage browser", () => {
  it("names the column for its stage, not for the file it has open", () => {
    render(<LineageColumns model={model()} />);
    expect(screen.getByText("bisect")).toBeTruthy();
    expect(screen.getByText(/notes\/bisect\.hick · \d+\/\d+ lines/)).toBeTruthy();
  });

  it("shows the stage's files and opens one in place", () => {
    render(<LineageColumns model={model()} />);
    const column = document.querySelector('[data-column="0"]')!;
    // The document is open by default; its generated file is listed too.
    expect(within(column as HTMLElement).getByText("bisect.py")).toBeTruthy();
    // Open row carries the RELATIVE PATH — a name alone is not a location.
    expect(within(column as HTMLElement).getByText("notes/bisect.hick")).toBeTruthy();
  });

  it("opens a generated file when its row is clicked", () => {
    render(<LineageColumns model={model()} />);
    const column = document.querySelector('[data-column="0"]') as HTMLElement;
    fireEvent.click(within(column).getByText("bisect.py"));
    expect(within(column).getByText("search/bisect.py")).toBeTruthy();
    expect(within(column).getByText(/search\/bisect\.py · \d+\/\d+ lines/)).toBeTruthy();
  });

  it("folds from the gutter and offers four ways to move the hole back", () => {
    render(<LineageColumns model={model()} />);
    const column = document.querySelector('[data-column="0"]') as HTMLElement;
    const gutters = column.querySelectorAll(".lin-gutter");
    expect(gutters.length).toBeGreaterThan(4);

    fireEvent.pointerDown(gutters[2]);
    fireEvent.pointerEnter(gutters[4]);
    fireEvent.pointerUp(window);

    const hole = column.querySelector(".hole");
    expect(hole).toBeTruthy();
    // Two edges, each moving both ways.
    expect(hole!.querySelectorAll(".hole-btn")).toHaveLength(4);
    // A hole at the top of a file cannot grow upward, and says so by being
    // disabled rather than by disappearing.
    const first = hole!.querySelectorAll(".hole-btn")[0] as HTMLButtonElement;
    if (hole!.getAttribute("data-from") === "0") expect(first.disabled).toBe(true);
  });

  it("reveals a hole's lines from either edge", () => {
    render(<LineageColumns model={model()} />);
    const column = document.querySelector('[data-column="0"]') as HTMLElement;
    const gutters = column.querySelectorAll(".lin-gutter");
    const before = column.querySelectorAll(".lin-row").length;

    fireEvent.pointerDown(gutters[3]);
    fireEvent.pointerEnter(gutters[6]);
    fireEvent.pointerUp(window);
    const folded = column.querySelectorAll(".lin-row").length;
    expect(folded).toBeLessThan(before);

    const hole = column.querySelector(".hole")!;
    const showAll = hole.querySelector(".hole-all") as HTMLButtonElement | null;
    fireEvent.click(showAll ?? (hole.querySelectorAll(".hole-btn")[1] as HTMLButtonElement));
    expect(column.querySelectorAll(".lin-row").length).toBeGreaterThan(folded);
  });

  it("folds a stage to its best matches, and counts every match", () => {
    render(<LineageColumns model={model()} />);
    const column = document.querySelector('[data-column="0"]') as HTMLElement;
    const all = column.querySelectorAll(".lin-row").length;

    // A rare term narrows the file. A common one legitimately does not — the
    // search is ranked, not a substring filter, so `bisect` really does occur
    // nearly everywhere in a document about bisection.
    fireEvent.change(within(column).getByLabelText(/Search notes\/bisect\.hick/), {
      target: { value: "lo, hi" },
    });
    expect(column.querySelectorAll(".lin-row").length).toBeLessThan(all);

    fireEvent.change(screen.getByLabelText("Search every stage"), {
      target: { value: "bisect_left" },
    });
    expect(screen.getByText(/\d+ in \d+ files?/)).toBeTruthy();
  });

  it("finds an identifier written in the other naming convention", () => {
    // The reason for ranked search over substring: nothing in this document
    // contains the literal string `bisectLeft`.
    render(<LineageColumns model={model()} />);
    fireEvent.change(screen.getByLabelText("Search every stage"), {
      target: { value: "bisectLeft" },
    });
    const count = screen.getByText(/\d+ in \d+ files?/).textContent ?? "";
    expect(Number(count.split(" ")[0])).toBeGreaterThan(0);
  });

  it("selecting a fragment marks what it feeds", () => {
    render(<LineageColumns model={model()} />);
    const column = document.querySelector('[data-column="0"]') as HTMLElement;
    const target = [...column.querySelectorAll(".lin-text")].find((el) =>
      el.textContent?.includes("def bisect_left"),
    )!;
    fireEvent.click(target);
    expect(column.querySelector('[data-rel="self"]')).toBeTruthy();
  });

  it("never closes the file you are reading to show what it links to", () => {
    // Both ends of a paste can live in one stage — a fragment and the file it
    // weaves — and a stage has one column. Answering "what does this feed?"
    // by closing the document is answering by destroying the question.
    render(<LineageColumns model={model()} />);
    const column = document.querySelector('[data-column="0"]') as HTMLElement;
    const fragment = [...column.querySelectorAll(".lin-text")].find((el) =>
      el.textContent?.includes("def bisect_left"),
    )!;
    fireEvent.click(fragment);

    // Still the document, and the clicked block is marked as the selection.
    expect(within(column).getByText("notes/bisect.hick")).toBeTruthy();
    expect(column.querySelectorAll('[data-rel="self"]').length).toBeGreaterThan(0);
  });

  it("lets a link kind be switched off", () => {
    render(<LineageColumns model={model()} />);
    const paste = document.querySelector<HTMLElement>('[data-tip^="Computed."]')!;
    expect(paste.getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(paste);
    expect(paste.getAttribute("aria-pressed")).toBe("false");
  });
});
