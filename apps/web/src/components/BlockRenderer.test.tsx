// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Block } from "../api/types";
import { BlockRenderer } from "./BlockRenderer";

afterEach(cleanup);

const blocks: Block[] = [
  { kind: "prose", html: "<h1>Title</h1><p>Some <code>prose</code>.</p>", span: [0, 30] },
  {
    kind: "exec",
    id: "cell-1",
    container: "shell",
    image: "debian:12",
    command: "hickory --version",
    span: [30, 90],
    status: "ok",
    transcript: [
      { t: 0, kind: "cmd", data: "hickory --version" },
      { t: 100, kind: "out", data: "hickory 0.4.2\n" },
      { t: 150, kind: "exit", code: 0 },
    ],
    expect: { match: "regex-lines", body: "hickory \\d+\\.\\d+\\.\\d+" },
  },
  { kind: "file", path: "src/main.rs", language: "rust", body: "fn main() {}", span: [90, 140] },
  { kind: "session-assistant", body: "I ran the cell.", span: [140, 160] },
];

describe("BlockRenderer", () => {
  it("renders prose as HTML", () => {
    render(<BlockRenderer blocks={blocks} />);
    expect(screen.getByRole("heading", { name: "Title" })).toBeTruthy();
  });

  it("renders an exec cell with command, status chip and Run button", () => {
    render(<BlockRenderer blocks={blocks} />);
    const cell = screen.getByTestId("exec-cell-1");
    expect(cell.textContent).toContain("hickory --version");
    expect(cell.textContent).toContain("shell");
    expect(cell.querySelector(".chip-ok")?.textContent).toBe("ok");
    expect(screen.getByRole("button", { name: "Run" })).toBeTruthy();
  });

  it("invokes onRunCell with the exec block id", () => {
    const onRun = vi.fn();
    render(<BlockRenderer blocks={blocks} onRunCell={onRun} />);
    fireEvent.click(screen.getByRole("button", { name: "Run" }));
    expect(onRun).toHaveBeenCalledWith("cell-1");
  });

  it("shows the completed transcript inside the terminal", () => {
    render(<BlockRenderer blocks={blocks} />);
    expect(screen.getByTestId("terminal").textContent).toContain("hickory 0.4.2");
  });

  it("marks a never-run cell accordingly", () => {
    const neverRun: Block[] = [
      { kind: "exec", id: "c2", container: "shell", command: "true", span: [0, 10] },
    ];
    render(<BlockRenderer blocks={neverRun} />);
    expect(screen.getByTestId("exec-c2").querySelector(".chip")?.textContent).toBe("never run");
  });

  it("renders file blocks and session blocks", () => {
    render(<BlockRenderer blocks={blocks} />);
    expect(screen.getByText("src/main.rs")).toBeTruthy();
    expect(screen.getByText("I ran the cell.")).toBeTruthy();
    expect(screen.getByText("assistant")).toBeTruthy();
  });

  it("renders an SVG figure when a cell's stdout is an SVG document", () => {
    const svgBlocks: Block[] = [
      {
        kind: "exec",
        id: "fig",
        container: "py",
        command: "python plot.py",
        span: [0, 10],
        status: "ok",
        transcript: [
          { t: 0, kind: "cmd", data: "python plot.py" },
          { t: 50, kind: "out", data: '<svg viewBox="0 0 10 10"><circle cx="5" cy="5" r="4"/></svg>' },
          { t: 60, kind: "exit", code: 0 },
        ],
      },
    ];
    const { container } = render(<BlockRenderer blocks={svgBlocks} />);
    expect(container.querySelector(".cell-figure svg")).toBeTruthy();
  });
});
