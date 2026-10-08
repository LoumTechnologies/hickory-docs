// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { representations, type Representation } from "../api/representations";
import { viewBuffers } from "../lib/viewBuffers";
import { RepresentationPane } from "./RepresentationPane";

vi.mock("../editor/DocumentEditor", () => ({ DocumentEditor: ({ docId, initialSource, onChange, readOnly }: {
  docId: string; initialSource: string; onChange?: (source: string) => void; readOnly?: boolean;
}) => <textarea aria-label={docId} defaultValue={initialSource} readOnly={readOnly} onChange={event => onChange?.(event.target.value)} /> }));
vi.mock("./RepresentationTools", () => ({ RepresentationTools: () => null }));
vi.mock("../components/ChatDock", () => ({ ChatDock: () => null }));

const record = (source: string, revision = "r1"): Representation => ({ id: "v", backing: { kind: "files", paths: ["code.py"] },
  source, revision, files: [{ path: "code.py", content: source, hash: revision, provenance: [] }], explanation_stale: false });
afterEach(() => { cleanup(); viewBuffers.ids().forEach(id => viewBuffers.forget(id)); vi.restoreAllMocks(); });
async function mount() {
  vi.spyOn(representations, "get").mockResolvedValue(record("old"));
  vi.spyOn(representations, "preview").mockResolvedValue(record("new").files);
  const mounted = render(<RepresentationPane id="v" bufferId="tab-v" />);
  const editor = await screen.findByLabelText("lens:v");
  return { ...mounted, editor };
}

// Guarantees: docs/guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file.md;
// docs/guarantees/lenses/a-view-declares-where-edits-land.md.
describe("saving a working representation", () => {
  it("keeps later typing unsaved when an earlier save completes", async () => {
    let complete!: (record: Representation) => void;
    const edit = vi.spyOn(representations, "edit").mockImplementation(() => new Promise(done => { complete = done; }));
    const { editor } = await mount();
    expect(screen.getByLabelText("About this view").textContent).toContain("code.py");
    fireEvent.change(editor, { target: { value: "first edit" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(edit).toHaveBeenCalledWith("v", "r1", "first edit");
    fireEvent.change(editor, { target: { value: "newer edit" } });
    await act(async () => complete(record("first edit", "r2")));
    expect((editor as HTMLTextAreaElement).value).toBe("newer edit");
    expect(screen.getByText("Unsaved changes")).toBeTruthy();
    expect(viewBuffers.get("tab-v")?.dirty).toBe(true);
    edit.mockResolvedValue(record("newer edit", "r3"));
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(viewBuffers.get("tab-v")?.dirty).toBe(false));
    expect(edit).toHaveBeenLastCalledWith("v", "r2", "newer edit");
  });

  it("can save an inactive tab through the workspace registry", async () => {
    const edit = vi.spyOn(representations, "edit").mockResolvedValue(record("edited", "r2"));
    const { editor, unmount } = await mount();
    fireEvent.change(editor, { target: { value: "edited" } });
    unmount();
    expect(viewBuffers.get("tab-v")?.dirty).toBe(true);
    await act(async () => expect(await viewBuffers.get("tab-v")!.save()).toBe(true));
    expect(edit).toHaveBeenCalledWith("v", "r1", "edited");
    expect(viewBuffers.get("tab-v")?.dirty).toBe(false);
  });
});
