import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { SearchPanel } from "./SearchPanel";
import { searchWorkspace } from "../lib/workspaceSearch";

afterEach(cleanup);

it("finds and navigates an open file without a folder", async () => {
  const navigate = vi.fn();
  const close = vi.fn();
  const search = (query: string, limit: number) => searchWorkspace(query, limit, [
    { path: "/outside/open.txt", content: "heading\nunsaved invoice" },
  ]);
  render(<SearchPanel search={search} folderOpen={false} onNavigate={navigate} onClose={close} />);
  fireEvent.change(screen.getByRole("searchbox"), { target: { value: "invoice" } });
  const result = await screen.findByRole("option");
  expect(result.textContent).toContain("unsaved invoice");
  expect(screen.getByPlaceholderText("Search open files")).toBeTruthy();
  fireEvent.click(result);
  expect(navigate).toHaveBeenCalledWith(expect.objectContaining({ path: "/outside/open.txt", start_line: 2 }));
  expect(close).toHaveBeenCalledOnce();
});

it("includes both folder results and open files", async () => {
  const search = (query: string, limit: number) => searchWorkspace(query, limit, [
    { path: "/outside/open.txt", content: "invoice" },
  ], async () => ({ semantic: false, hits: [
    { path: "closed.md", start_line: 1, end_line: 1, score: 1, snippet: "folder invoice" },
  ] }));
  render(<SearchPanel search={search} folderOpen onNavigate={vi.fn()} onClose={vi.fn()} />);
  fireEvent.change(screen.getByRole("searchbox"), { target: { value: "invoice" } });
  await waitFor(() => expect(screen.getAllByRole("option")).toHaveLength(2));
  expect(screen.getByPlaceholderText("Search open files and this folder")).toBeTruthy();
});
