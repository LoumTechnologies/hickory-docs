import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { CommandBar, modeHint, parseQuery, openCommandBar } from "./CommandBar";

afterEach(() => {
  cleanup();
});

describe("one field, four questions", () => {
  it("reads a bare query as a search of file contents", () => {
    expect(parseQuery("Write")).toEqual({ mode: "content", term: "Write" });
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
    expect(modeHint("content")).toMatch(/search open files and folder/i);
    expect(modeHint("command")).toBe("Commands");
    expect(modeHint("line")).toBe("Go to line");
  });
});

describe("the bar in use", () => {
  const item = (id: string, run = vi.fn()) => ({ id, label: id, detail: `/${id}`, run });

  it("never runs an old result while a new query is being resolved", async () => {
    const oldRun = vi.fn(), newRun = vi.fn();
    let finish!: (items: ReturnType<typeof item>[]) => void;
    const candidates = (_mode: string, term: string) => term === "old" ? [item("old", oldRun)]
      : new Promise<ReturnType<typeof item>[]>(resolve => { finish = resolve; });
    render(<CommandBar candidates={candidates} />);
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "old" } });
    await screen.findByRole("option", { name: /old/ });
    fireEvent.change(input, { target: { value: "new" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(oldRun).not.toHaveBeenCalled();
    expect(screen.queryAllByRole("option")).toHaveLength(0);
    expect(screen.getByText("Searching…")).toBeTruthy();
    await waitFor(() => expect(finish).toBeTypeOf("function"));
    finish([item("new", newRun)]);
    await screen.findByRole("option", { name: /new/ });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(newRun).toHaveBeenCalledOnce();
  });

  it("connects keyboard selection to the result announced to assistive technology", async () => {
    render(<CommandBar candidates={() => [item("a"), item("b")]} />);
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "x" } });
    await screen.findByRole("option", { name: /a/ });
    fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(input.getAttribute("aria-activedescendant")).toBe(screen.getByRole("option", { name: /b/ }).id);
    expect(input.getAttribute("aria-controls")).toBe(screen.getByRole("listbox").id);
  });

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
    fireEvent.keyDown(window, { key: "p", ctrlKey: true, shiftKey: true });
    await waitFor(() => expect(candidates).toHaveBeenCalledWith("command", ""));
  });
});

// Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
it("reveals the overlay on demand and restores editor focus on dismissal", async () => {
  const candidates = vi.fn().mockReturnValue([]);
  render(<><textarea aria-label="Source" defaultValue="keep my text" /><CommandBar candidates={candidates} overlay /></>);
  const source = screen.getByRole("textbox", { name: "Source" }) as HTMLTextAreaElement;
  source.focus(); source.setSelectionRange(2, 6);
  expect(screen.queryByRole("search")).toBeNull();
  fireEvent.keyDown(window, { key: "p", ctrlKey: true });
  const palette = screen.getByRole("textbox", { name: "Search open files and folder" });
  expect(document.activeElement).toBe(palette);
  fireEvent.keyDown(palette, { key: "Escape" });
  expect(screen.queryByRole("search")).toBeNull();
  expect(document.activeElement).toBe(source);
  expect([source.selectionStart, source.selectionEnd, source.value]).toEqual([2, 6, "keep my text"]);
  openCommandBar(true);
  await waitFor(() => expect(screen.getByRole("textbox", { name: "Commands" })).toBeTruthy());
});

// Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
it("can reopen immediately after Escape without a stale blur closing it", async () => {
  render(<CommandBar candidates={() => []} overlay />);
  fireEvent.keyDown(window, { key: "p", ctrlKey: true });
  const input = screen.getByRole("textbox");
  fireEvent.keyDown(input, { key: "Escape" });
  openCommandBar();
  await waitFor(() => expect(document.activeElement).toBe(input));
  fireEvent.change(input, { target: { value: ">Settings" } });
  await waitFor(() => expect(screen.getByRole("textbox", { name: "Commands" })).toBe(input));
  await new Promise(resolve => setTimeout(resolve, 150));
  expect(screen.getByRole("search")).toBeTruthy();
});
