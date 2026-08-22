import { describe, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { altFor, imageFilesOf, mdPaste, pastedImageName } from "./mdPaste";

/** A DataTransfer stand-in: jsdom has no constructor for one, and only the
 * two collections this module reads are needed. */
function transfer(files: File[]): DataTransfer {
  return {
    files,
    items: files.map((file) => ({ kind: "file", type: file.type, getAsFile: () => file })),
    getData: () => "",
  } as unknown as DataTransfer;
}

function textTransfer(text: string): DataTransfer {
  return {
    files: [],
    items: [],
    getData: (kind: string) => (kind === "text/plain" ? text : ""),
  } as unknown as DataTransfer;
}

function editorWith(
  doc: string,
  upload = vi.fn(async () => ({ relative: "assets/shot.png" })),
) {
  const view = new EditorView({
    state: EditorState.create({
      doc,
      extensions: [mdPaste({ docPath: () => "notes/a.hick", upload })],
    }),
    parent: document.body,
  });
  return { view, upload };
}

/** Fire a paste at the editor the way the browser would.
 *
 * `defaultPrevented` is not the signal for "this module stayed out of it":
 * CodeMirror handles every paste itself and prevents the default regardless.
 * What separates the two cases is what ends up in the buffer. */
function paste(view: EditorView, data: DataTransfer) {
  const event = new Event("paste", { bubbles: true, cancelable: true }) as ClipboardEvent;
  Object.defineProperty(event, "clipboardData", { value: data });
  view.contentDOM.dispatchEvent(event);
  return event;
}

describe("pasting a URL over a selection", () => {
  it("makes a markdown link out of the selected words", () => {
    const { view } = editorWith("read the spec today");
    view.dispatch({ selection: { anchor: 5, head: 13 } });
    paste(view, textTransfer("https://example.com/spec"));
    expect(view.state.doc.toString()).toBe(
      "read [the spec](https://example.com/spec) today",
    );
    view.destroy();
  });

  it("leaves an ordinary paste alone, so the browser still handles it", () => {
    const { view } = editorWith("read the spec today");
    view.dispatch({ selection: { anchor: 5, head: 13 } });
    paste(view, textTransfer("the summary"));
    expect(view.state.doc.toString()).toBe("read the summary today");
    view.destroy();
  });

  it("does nothing special with no selection: you get the URL you copied", () => {
    const { view } = editorWith("read this");
    view.dispatch({ selection: { anchor: 9 } });
    paste(view, textTransfer("https://example.com"));
    expect(view.state.doc.toString()).toBe("read thishttps://example.com");
    view.destroy();
  });

  it("refuses inside a range the caller says is not prose", () => {
    const upload = vi.fn(async () => ({ relative: "x.png" }));
    const view = new EditorView({
      state: EditorState.create({
        doc: "curl a url here",
        extensions: [
          mdPaste({ docPath: () => null, upload, isProse: () => false }),
        ],
      }),
      parent: document.body,
    });
    view.dispatch({ selection: { anchor: 5, head: 10 } });
    paste(view, textTransfer("https://example.com"));
    expect(view.state.doc.toString()).toBe("curl https://example.com here");
    view.destroy();
  });
});

describe("pasting an image", () => {
  it("writes the file and references it in markdown", async () => {
    const upload = vi.fn(async () => ({ relative: "assets/shot.png" }));
    const { view } = editorWith("here: ", upload);
    view.dispatch({ selection: { anchor: 6 } });
    const file = new File(["bytes"], "shot.png", { type: "image/png" });
    expect(paste(view, transfer([file])).defaultPrevented).toBe(true);
    await vi.waitFor(() =>
      expect(view.state.doc.toString()).toBe("here: ![shot](assets/shot.png)"),
    );
    expect(upload).toHaveBeenCalledWith(expect.any(File), "notes/a.hick");
    view.destroy();
  });

  it("lands where the drop was even when the buffer moved underneath it", async () => {
    // The write is a round trip and the buffer is a CRDT: a position captured
    // before the await and used after it would put the picture wherever that
    // offset happened to land.
    let release: (v: { relative: string }) => void = () => undefined;
    const upload = vi.fn(
      () => new Promise<{ relative: string }>((resolve) => (release = resolve)),
    );
    const { view } = editorWith("end", upload);
    view.dispatch({ selection: { anchor: 3 } });
    paste(view, transfer([new File(["b"], "a.png", { type: "image/png" })]));
    // A collaborator inserts ahead of the pending position.
    view.dispatch({ changes: { from: 0, insert: "start " } });
    release({ relative: "assets/a.png" });
    await vi.waitFor(() =>
      expect(view.state.doc.toString()).toBe("start end![a](assets/a.png)"),
    );
    view.destroy();
  });

  it("says why nothing appeared when the write fails", async () => {
    const onError = vi.fn();
    const view = new EditorView({
      state: EditorState.create({
        doc: "",
        extensions: [
          mdPaste({
            docPath: () => null,
            upload: async () => {
              throw new Error("the open folder is read-only");
            },
            onError,
          }),
        ],
      }),
      parent: document.body,
    });
    paste(view, transfer([new File(["b"], "a.png", { type: "image/png" })]));
    await vi.waitFor(() =>
      expect(onError).toHaveBeenCalledWith("the open folder is read-only"),
    );
    expect(view.state.doc.toString()).toBe("");
    view.destroy();
  });
});

describe("what arrives on a clipboard", () => {
  it("takes image files and ignores everything else", () => {
    const png = new File(["b"], "a.png", { type: "image/png" });
    const text = new File(["b"], "a.txt", { type: "text/plain" });
    expect(imageFilesOf(transfer([png, text]))).toEqual([png]);
    expect(imageFilesOf(null)).toEqual([]);
  });

  it("names a screenshot, which arrives with no name of its own", () => {
    const shot = new File(["b"], "image.png", { type: "image/png" });
    expect(pastedImageName(shot, "7f")).toBe("pasted-7f.png");
    const named = new File(["b"], "chart.png", { type: "image/png" });
    expect(pastedImageName(named, "7f")).toBe("chart.png");
  });

  it("reads a caption out of the file name", () => {
    expect(altFor("quarterly-revenue.png")).toBe("quarterly revenue");
  });
});
