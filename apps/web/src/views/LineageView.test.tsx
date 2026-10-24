// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { LineageView } from "./LineageView";
vi.mock("../lineage/LineageColumns", () => ({ LineageColumns: () => <div>Captured lineage columns</div> }));
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
it("recovers a failed read and resets replay when the project changes", async () => {
  const docs = vi.spyOn(api, "projectDocs").mockRejectedValueOnce(Error("read refused")).mockResolvedValue([]);
  const structure = vi.spyOn(api, "structure").mockResolvedValue({ files: [], links: [] } as Awaited<ReturnType<typeof api.structure>>);
  const view = render(<LineageView projectId="first" />);
  await screen.findByText(/read refused/);
  fireEvent.click(screen.getByRole("button", { name: "Retry reading project lineage" }));
  await screen.findByText("No documents in this project yet.");
  expect(docs).toHaveBeenCalledTimes(2);
  await waitFor(() => expect(structure).toHaveBeenCalledTimes(1));
  view.rerender(<LineageView projectId="second" />);
  expect(screen.queryByText("No documents in this project yet.")).toBeNull();
  await screen.findByText("No documents in this project yet.");
  expect(docs).toHaveBeenLastCalledWith("second");
});
it("does not adopt a late read from the previous project", async () => {
  let reject!: (error: Error) => void;
  vi.spyOn(api, "projectDocs").mockImplementationOnce(() => new Promise((_resolve, fail) => { reject = fail; })).mockResolvedValue([]);
  vi.spyOn(api, "structure").mockResolvedValue({ files: [], links: [] } as Awaited<ReturnType<typeof api.structure>>);
  const view = render(<LineageView projectId="first" />);
  view.rerender(<LineageView projectId="second" />);
  await screen.findByText("No documents in this project yet.");
  await act(async () => { reject(Error("previous owner failure")); });
  expect(screen.queryByText(/previous owner failure/)).toBeNull();
});
it("keeps current structural navigation out of historical replay", async () => {
  vi.spyOn(api, "projectDocs").mockResolvedValue([{ id: "d", path: "note.md", updated_at: "" }]);
  vi.spyOn(api, "doc").mockResolvedValue({ id: "d", path: "note.md", source: "current", updated_at: "" });
  vi.spyOn(api, "outputs").mockResolvedValue({ files: [] } as Awaited<ReturnType<typeof api.outputs>>);
  vi.spyOn(api, "docHistory").mockResolvedValue({ repository: true, commits: [{ sha: "old", short: "old", time: 100, author: "T", subject: "Earlier", path: "note.md" }] });
  vi.spyOn(api, "docReplay").mockResolvedValue({ commit: "old", path: "note.md", grammar_boundary: false, source: "historical", outputs: [] });
  const structure = vi.spyOn(api, "structure").mockResolvedValue({ files: [], links: [] });
  render(<LineageView projectId="first" />);
  await screen.findByRole("slider");
  await waitFor(() => expect(structure).toHaveBeenCalledTimes(1));
  fireEvent.change(screen.getByRole("slider"), { target: { value: "0" } });
  await waitFor(() => expect(api.docReplay).toHaveBeenCalledWith("d", "old"));
  await screen.findByText("Earlier");
  expect(structure).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole("button", { name: "Show working-tree lineage" }));
  await waitFor(() => expect(structure).toHaveBeenCalledTimes(2));
});
it("shows finished stages while another output waits, then labels its deadline as incomplete", async () => {
  vi.useFakeTimers();
  try {
    vi.spyOn(api,"projectDocs").mockResolvedValue([{id:"first",path:"first.md",updated_at:""},{id:"slow",path:"slow.md",updated_at:""}]);
    vi.spyOn(api,"doc").mockImplementation(async id=>({id,path:id+".md",source:"# "+id,updated_at:""}));
    vi.spyOn(api,"outputs").mockImplementation(id=>id==="slow" ? new Promise(()=>{}) : Promise.resolve({files:[]}));
    vi.spyOn(api,"docHistory").mockResolvedValue({repository:false,commits:[]});
    vi.spyOn(api,"structure").mockResolvedValue({files:[],links:[]});
    await act(async()=>{render(<LineageView projectId="bounded" />);});
    expect(screen.getByText("Captured lineage columns")).toBeTruthy();
    expect(screen.getByText(/1 of 2 documents finished/)).toBeTruthy();
    await act(async()=>{await vi.advanceTimersByTimeAsync(10_000);});
    expect(screen.getByText(/slow.md: Read exceeded 10 seconds/)).toBeTruthy();
    expect(screen.getByRole("button",{name:"Retry reading project lineage"})).toBeTruthy();
    expect(screen.queryByText(/documents finished; reading/)).toBeNull();
  } finally { vi.useRealTimers(); }
});
