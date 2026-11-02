import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EditorView } from "@codemirror/view";
import { foldedRanges } from "@codemirror/language";
import { SourceFileTree } from "./SourceFileTree";

const paths = ["project.md", "src/deep/chapter.md", "src/other.md", "docs/guide.md"];
afterEach(() => { cleanup(); sessionStorage.clear(); });
function view() { return EditorView.findFromDOM(document.querySelector(".source-tree .cm-editor")!)!; }

it("uses read-only file text, real links and keyboard activation without a backend", () => {
  const onOpen = vi.fn(), fetcher = vi.spyOn(globalThis, "fetch");
  try {
    render(<SourceFileTree paths={paths} current="project.md" identity="test" onOpen={onOpen} />);
    const editor = view();
    expect(editor.state.readOnly).toBe(true);
    expect(editor.contentDOM.getAttribute("contenteditable")).toBe("false");
    expect(editor.state.doc.toString()).toBe("docs/\n  guide.md\nsrc/\n  deep/\n    chapter.md\n  other.md\nproject.md");
    expect(screen.getByRole("link", { name: "project.md" }).getAttribute("aria-current")).toBe("page");
    fireEvent.keyDown(editor.contentDOM, { key: "Enter" });
    expect(onOpen).toHaveBeenCalledWith("project.md");
    expect(fetcher).not.toHaveBeenCalled();
  } finally { fetcher.mockRestore(); }
});

it("remembers expansion across documents and reveals the selected chapter", () => {
  const first = render(<SourceFileTree paths={paths} current="project.md" identity="test" onOpen={() => {}} />);
  fireEvent.click(screen.getByText("docs/", { exact: false }));
  expect(screen.getByRole("link", { name: "guide.md" })).toBeTruthy();
  first.unmount();
  render(<SourceFileTree paths={paths} current="src/deep/chapter.md" identity="test" onOpen={() => {}} />);
  expect(screen.getByRole("link", { name: "guide.md" })).toBeTruthy();
  expect(screen.getByRole("link", { name: "chapter.md" }).getAttribute("aria-current")).toBe("page");
  expect(foldedRanges(view().state).size).toBe(0);
});

it("still opens the current path when saved sidebar state is invalid", () => {
  sessionStorage.setItem("hickory.source.files:test", "invalid");
  render(<SourceFileTree paths={paths} current="src/deep/chapter.md" identity="test" onOpen={() => {}} />);
  expect(screen.getByRole("link", { name: "chapter.md" }).getAttribute("href")).toBe("?path=src%2Fdeep%2Fchapter.md");
});
