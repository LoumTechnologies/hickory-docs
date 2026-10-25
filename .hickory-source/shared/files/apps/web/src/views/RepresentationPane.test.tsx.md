<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/RepresentationPane.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [Saved means the displayed bytes reached the file](../../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md)
- [A view declares where edits land](../../../../../../guarantees/lenses/a-view-declares-where-edits-land/contract.md)

Captured SHA-256: `529a986d2b7393d47cca475acc5654306b5b858ca033a0712c72a425118ad9d1`.

<book:entry path="apps/web/src/views/RepresentationPane.test.tsx">
// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { representations, type Representation } from "../api/representations";
import { viewBuffers } from "../lib/viewBuffers";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { RepresentationPane } from "./RepresentationPane";

vi.mock("../editor/DocumentEditor", () => ({ DocumentEditor: ({ docId, initialSource, onChange, readOnly }: {
  docId: string; initialSource: string; onChange?: (source: string) => void; readOnly?: boolean;
}) => <textarea aria-label={docId} defaultValue={initialSource} readOnly={readOnly} onChange={event => onChange?.(event.target.value)} /> }));
vi.mock("./RepresentationTools", () => ({ RepresentationTools: () => null }));
vi.mock("../components/ChatDock", () => ({ ChatDock: () => null }));

const record = (source: string, revision = "r1"): Representation => ({ id: "v", backing: { kind: "files", paths: ["code.py"] },
  source, revision, files: [{ path: "code.py", content: source, hash: revision, provenance: [] }], explanation_stale: false });
afterEach(() => { act(() => { viewBuffers.ids().forEach(id => viewBuffers.get(id)?.discard()); }); cleanup(); viewBuffers.ids().forEach(id => viewBuffers.forget(id)); vi.restoreAllMocks(); });
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

it("keeps revision selection in a closed menu until requested", async () => {
  await mount();
  const menu = screen.getByText("Compare revisions…").closest("details")!;
  expect(menu.open).toBe(false);
  fireEvent.click(screen.getByText("Compare revisions…"));
  expect(menu.open).toBe(true);
  expect((screen.getByLabelText("Comparison target") as HTMLInputElement).placeholder).toBe("Current code (editable)");
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("keeps later typing when a source refresh completes", async () => {
  let complete!: (value: Representation) => void;
  const refresh = vi.spyOn(representations, "refresh").mockImplementation(() => new Promise(done => { complete = done; }));
  const { editor } = await mount();
  fireEvent.click(screen.getByRole("button", { name: "Refresh from source" }));
  expect(refresh).toHaveBeenCalledWith("v");
  fireEvent.change(editor, { target: { value: "typed during refresh" } });
  await act(async () => complete(record("disk changed", "r2")));
  expect((editor as HTMLTextAreaElement).value).toBe("typed during refresh");
  expect(screen.getByText(/Source changed while refreshing/)).toBeTruthy();
  expect(viewBuffers.get("tab-v")?.dirty).toBe(true);
});

it("keeps later typing when a revision comparison completes", async () => {
  let complete!: (value: Awaited<ReturnType<typeof representations.compare>>) => void;
  vi.spyOn(representations, "compare").mockImplementation(() => new Promise(done => { complete = done; }));
  const { editor } = await mount();
  fireEvent.click(screen.getByText("Compare revisions…"));
  fireEvent.click(screen.getByRole("button", { name: "Show diff" }));
  fireEvent.change(editor, { target: { value: "typed during comparison" } });
  await act(async () => complete({ base: "HEAD", target: "past", source: "base", target_source: "historical", editable: false, revision: "r2" }));
  expect((editor as HTMLTextAreaElement).value).toBe("typed during comparison");
  expect((editor as HTMLTextAreaElement).readOnly).toBe(false);
  expect(screen.getByText(/Source changed while comparing/)).toBeTruthy();
});

it("honors a read-only comparison reply and returns to loaded current code", async () => {
  vi.spyOn(representations, "compare").mockResolvedValue({ base: "resolved", target: null, source: "base", target_source: "old", editable: false, revision: "r2" });
  const { editor } = await mount();
  fireEvent.click(screen.getByText("Compare revisions…"));
  fireEvent.click(screen.getByRole("button", { name: "Show diff" }));
  await waitFor(() => expect((editor as HTMLTextAreaElement).readOnly).toBe(true));
  fireEvent.click(screen.getByRole("button", { name: "Return to current code" }));
  await waitFor(() => expect((editor as HTMLTextAreaElement).readOnly).toBe(false));
  expect(screen.getByText("Saved")).toBeTruthy();
});

// Guarantee: docs/guarantees/lenses/a-view-declares-where-edits-land.md
it("exposes the opening failure before a literate view record/editor is available", async () => {
  vi.spyOn(representations, "get").mockRejectedValue(new Error("Literate view is unavailable"));
  const registry = new UiActions({ workspace: "p", window: "literate-loading" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><RepresentationPane id="v" bufferId="failed-view" /></UiActionContext.Provider>);
  await screen.findByText("Literate view is unavailable");
  const action = registry.groups().groups.flatMap(group => registry.observe(group, 64).actions).find(action => action.label === "Read literate view load state bytes")!;
  let reply: any;
  await act(async () => { reply = await registry.act(registry.scope, { id: "acp:status", kind: "agent" }, { handle: action.handle, arguments: { offset: 0, limit: 8192 }, request_id: "failure" }); });
  expect(JSON.parse(new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes)))).toMatchObject({ id: "v", loaded: false, notice: "Literate view is unavailable", source_read: false });
  expect(screen.queryByRole("textbox")).toBeNull();
});
</book:entry>

</book:doc>
