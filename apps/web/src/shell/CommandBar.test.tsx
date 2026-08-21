import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { CommandBar, modeHint, parseQuery } from "./CommandBar";

afterEach(() => {
  cleanup();
});

describe("one field, four questions", () => {
  it("reads a bare query as looking for a file", () => {
    expect(parseQuery("main.rs")).toEqual({ mode: "files", term: "main.rs" });
  });

  it("reads the prefixes every editor already taught people", () => {
    expect(parseQuery(">split right")).toEqual({ mode: "command", term: "split right" });
    expect(parseQuery("?invoice")).toEqual({ mode: "content", term: "invoice" });
    expect(parseQuery(":42")).toEqual({ mode: "line", term: "42" });
  });

  it("survives the space people leave after a prefix", () => {
    expect(parseQuery("  >  open  ")).toEqual({ mode: "command", term: "open" });
  });

  it("says what it will do, so the prefixes need no help page", () => {
    expect(modeHint("files")).toMatch(/type > for commands/i);
    expect(modeHint("command")).toBe("Commands");
    expect(modeHint("line")).toBe("Go to line");
  });
});

describe("the bar in use", () => {
  const item = (id: string, run = vi.fn()) => ({ id, label: id, detail: `/${id}`, run });

  it("asks for candidates in the mode that was typed", async () => {
    const candidates = vi.fn().mockReturnValue([]);
    render(<CommandBar candidates={candidates} />);
    fireEvent.change(screen.getByRole("textbox"), { target: { value: ">go" } });
    await waitFor(() => expect(candidates).toHaveBeenCalledWith("command", "go"));
  });

  it("runs the highlighted candidate on Enter", async () => {
    const run = vi.fn();
    render(<CommandBar candidates={() => [item("a", run), item("b")]} />);
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "a" } });
    await waitFor(() => expect(screen.getAllByRole("option")).toHaveLength(2));
    fireEvent.keyDown(input, { key: "Enter" });
    expect(run).toHaveBeenCalled();
  });

  it("moves the highlight with the arrows", async () => {
    const second = vi.fn();
    render(<CommandBar candidates={() => [item("a"), item("b", second)]} />);
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "x" } });
    await waitFor(() => expect(screen.getAllByRole("option")).toHaveLength(2));
    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(second).toHaveBeenCalled();
  });

  it("says plainly when nothing matches", async () => {
    render(<CommandBar candidates={() => []} />);
    fireEvent.change(screen.getByRole("textbox"), { target: { value: "zzz" } });
    await waitFor(() => expect(screen.getByText("Nothing matches.")).toBeTruthy());
  });

  it("closes on Escape without running anything", async () => {
    const run = vi.fn();
    render(<CommandBar candidates={() => [item("a", run)]} />);
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "a" } });
    await waitFor(() => expect(screen.getAllByRole("option")).toHaveLength(1));
    fireEvent.keyDown(input, { key: "Escape" });
    expect(screen.queryAllByRole("option")).toHaveLength(0);
    expect(run).not.toHaveBeenCalled();
  });

  it("opens on the chord every editor uses, and Shift starts a command", async () => {
    const candidates = vi.fn().mockReturnValue([]);
    render(<CommandBar candidates={candidates} />);
    fireEvent.keyDown(window, { key: "p", metaKey: true, shiftKey: true });
    await waitFor(() => expect(candidates).toHaveBeenCalledWith("command", ""));
  });
});
