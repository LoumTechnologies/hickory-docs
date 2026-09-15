import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EditorView } from "@codemirror/view";
import { afterEach, describe, expect, it, vi } from "vitest";

import { fileRenameError, renameValues, TreeRenameEditor } from "./TreeRenameEditor";

afterEach(cleanup);

// Guarantee: docs/guarantees/authoring/the-workspace-tree-edits-like-an-editor.md
describe("the semantic rename column", () => {
  it("keeps one line per selected node and refuses paths", () => {
    expect(renameValues("a\nb", 2)).toEqual({ values: ["a", "b"] });
    expect(renameValues("a", 2).error).toContain("exactly 2 lines");
    expect(renameValues("a\n", 2).error).toContain("Line 2");
    expect(fileRenameError("elsewhere/a.md")).toContain("not a path");
    expect(fileRenameError("..")).toContain("not a file name");
  });

  it("applies every line independently and reports partial failure", async () => {
    const apply = vi.fn(async (_item: { key: string }, value: string) => {
      if (value === "taken.md") throw new Error("taken.md already exists");
    });
    render(
      <TreeRenameEditor
        items={[
          { key: "file:a", context: "src/a.md", value: "a.md" },
          { key: "file:b", context: "src/b.md", value: "b.md" },
        ]}
        apply={apply}
        validate={fileRenameError}
        onClose={() => {}}
      />,
    );
    const editor = EditorView.findFromDOM(document.querySelector(".cm-editor")!);
    if (!editor) throw new Error("rename editor did not mount");
    editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "renamed.md\ntaken.md" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply 2 renames" }));
    expect(await screen.findByText(/a\.md → renamed\.md/)).toBeTruthy();
    expect(screen.getByText(/b\.md → taken\.md/).closest("li")?.textContent).toContain("already exists");
    expect(apply).toHaveBeenCalledTimes(2);

    editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "ignored.md\nfixed.md" } });
    fireEvent.click(screen.getByRole("button", { name: "Retry failed renames" }));
    await screen.findByText(/b\.md → fixed\.md/);
    expect(apply).toHaveBeenCalledTimes(3);
    expect(apply.mock.calls.filter(([item]) => item.key === "file:a")).toHaveLength(1);
  });
});
