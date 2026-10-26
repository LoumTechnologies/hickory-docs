<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/mdPaste.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-722af83950b4"></a>
## `apps/web/src/editor/mdPaste.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An Image Dropped In Becomes A File And A Reference](../../../../../guarantees/authoring/an-image-dropped-in-becomes-a-file-and-a-reference/contract.md), [Pasting A URL Over Words Makes A Link](../../../../../guarantees/authoring/pasting-a-url-over-words-makes-a-link/contract.md).

<a id="f-722af83950b4-Pending"></a>
### `Pending`

Two things a paste can be that are not "insert this text", and one thing a
drop can be.
- **A URL over a selection is a link being applied.** Select three words,
paste an address, and every editor people already use turns it into a
link rather than replacing the words with the address. Doing anything
else here would be the surprising choice.
- **An image on the clipboard is a picture being put in the note.** So is
an image file dragged in from a file manager. Both are written into the
folder beside the document (`POST /api/asset`) and referenced with
ordinary markdown, because a note that carries a picture as a base64
blob is a note nobody can read in a diff and nothing else can open.
Everything this writes is markdown a reader would have typed. That is the
whole point: the woven `.md` needs no special support for either of these,
because there is nothing special in the document to support.
## The pending position, and why it is a StateField
Writing the file is a round trip, and the buffer is a CRDT that a
collaborator (or the up-loop, or the agent) may edit while it is in flight.
A position captured before the await and used after it would put the image
wherever that offset landed, which is not where the pointer was. So the
position is held in a StateField and mapped through every change until the
bytes are on disk — the same discipline any deferred insertion needs.

<storycode:copy id="f-722af83950b4-Pending">
// Two things a paste can be that are not "insert this text", and one thing a
// drop can be.
//
//  - **A URL over a selection is a link being applied.** Select three words,
//    paste an address, and every editor people already use turns it into a
//    link rather than replacing the words with the address. Doing anything
//    else here would be the surprising choice.
//  - **An image on the clipboard is a picture being put in the note.** So is
//    an image file dragged in from a file manager. Both are written into the
//    folder beside the document (`POST /api/asset`) and referenced with
//    ordinary markdown, because a note that carries a picture as a base64
//    blob is a note nobody can read in a diff and nothing else can open.
//
// Everything this writes is markdown a reader would have typed. That is the
// whole point: the woven `.md` needs no special support for either of these,
// because there is nothing special in the document to support.
//
// ## The pending position, and why it is a StateField
//
// Writing the file is a round trip, and the buffer is a CRDT that a
// collaborator (or the up-loop, or the agent) may edit while it is in flight.
// A position captured before the await and used after it would put the image
// wherever that offset landed, which is not where the pointer was. So the
// position is held in a StateField and mapped through every change until the
// bytes are on disk — the same discipline any deferred insertion needs.

import { StateEffect, StateField } from "@codemirror/state";
import type { EditorState, Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { imageMarkdown, linkOverSelection } from "../lib/mdLinks";

/** Where an in-flight image will land, and which upload it belongs to. */
interface Pending {
  id: number;
  pos: number;
}

</storycode:copy>
<a id="f-722af83950b4-addPending"></a>
### `addPending`

<storycode:copy id="f-722af83950b4-addPending">
const addPending = StateEffect.define<Pending>();
</storycode:copy>
<a id="f-722af83950b4-dropPending"></a>
### `dropPending`

<storycode:copy id="f-722af83950b4-dropPending">
const dropPending = StateEffect.define<number>();

</storycode:copy>
<a id="f-722af83950b4-pendingField"></a>
### `pendingField`

<storycode:copy id="f-722af83950b4-pendingField">
const pendingField = StateField.define<readonly Pending[]>({
  create: () => [],
  update(value, tr) {
    let next = value;
    if (tr.docChanged) {
      next = next.map((p) => ({ id: p.id, pos: tr.changes.mapPos(p.pos, 1) }));
    }
    for (const effect of tr.effects) {
      if (effect.is(addPending)) next = [...next, effect.value];
      else if (effect.is(dropPending)) next = next.filter((p) => p.id !== effect.value);
    }
    return next;
  },
});

</storycode:copy>
<a id="f-722af83950b4-pendingPos"></a>
### `pendingPos`

Where the upload started, as it stands now, or null if it was forgotten.

<storycode:copy id="f-722af83950b4-pendingPos">
/** Where the upload started, as it stands now, or null if it was forgotten. */
function pendingPos(state: EditorState, id: number): number | null {
  const found = state.field(pendingField, false)?.find((p) => p.id === id);
  return found ? Math.min(found.pos, state.doc.length) : null;
}

</storycode:copy>
<a id="f-722af83950b4-SavedAsset"></a>
### `SavedAsset`

What an image upload has to answer with: the destination to write into the
markdown, relative to the document.

<storycode:copy id="f-722af83950b4-SavedAsset">
/** What an image upload has to answer with: the destination to write into the
 * markdown, relative to the document. */
export interface SavedAsset {
  relative: string;
}

</storycode:copy>
<a id="f-722af83950b4-MdPasteConfig"></a>
### `MdPasteConfig`

<storycode:copy id="f-722af83950b4-MdPasteConfig">
export interface MdPasteConfig {
  /** The document being edited, root-relative — it decides which `assets/`
   * directory the image lands in. Read at drop time rather than captured,
   * because an untitled buffer gets a path when it is first saved. */
  docPath: () => string | null;
  /** Write the bytes somewhere and answer where. The seam the tests use, and
   * the one place this module knows about the server at all. */
  upload: (file: File, docPath: string | null) => Promise<SavedAsset>;
  /** Whether this range is prose, so a URL pasted over a shell command is
   * still a URL and not a markdown link inside a script. Absent means all of
   * it is prose, which is what a plain markdown buffer is. */
  isProse?: (state: EditorState, from: number, to: number) => boolean;
  /** Say why nothing happened. A failed write that reports nothing is worse
   * than one that refuses out loud. */
  onError?: (message: string) => void;
}

</storycode:copy>
<a id="f-722af83950b4-imageFilesOf"></a>
### `imageFilesOf`

Image files in a clipboard or drag payload, in the order they arrived.

<storycode:copy id="f-722af83950b4-imageFilesOf">
/** Image files in a clipboard or drag payload, in the order they arrived. */
export function imageFilesOf(data: DataTransfer | null): File[] {
  if (!data) return [];
  const files: File[] = [];
  for (const item of Array.from(data.files ?? [])) {
    if (item.type.startsWith("image/")) files.push(item);
  }
  if (files.length > 0) return files;
  // A screenshot on the clipboard is an *item*, not a file entry, in some
  // browsers; `getAsFile` is the only way to reach it.
  for (const item of Array.from(data.items ?? [])) {
    if (item.kind !== "file" || !item.type.startsWith("image/")) continue;
    const file = item.getAsFile();
    if (file) files.push(file);
  }
  return files;
}

</storycode:copy>
<a id="f-722af83950b4-pastedImageName"></a>
### `pastedImageName`

A name for a clipboard image, which has none.
Not a timestamp: two screenshots pasted in the same second would collide,
and the server's own de-duplication already handles the case that matters
(the same bytes twice). A short random tail is enough to keep them apart
and short enough to read in a file listing.

<storycode:copy id="f-722af83950b4-pastedImageName">
/**
 * A name for a clipboard image, which has none.
 *
 * Not a timestamp: two screenshots pasted in the same second would collide,
 * and the server's own de-duplication already handles the case that matters
 * (the same bytes twice). A short random tail is enough to keep them apart
 * and short enough to read in a file listing.
 */
export function pastedImageName(file: File, suffix: string): string {
  if (file.name && file.name !== "image.png") return file.name;
  const ext = (file.type.split("/")[1] || "png").replace(/[^a-z0-9]/gi, "");
  return `pasted-${suffix}.${ext}`;
}

</storycode:copy>
<a id="f-722af83950b4-altFor"></a>
### `altFor`

The alt text for an image inserted from a file: its own name, which is the
only thing here that knows anything about the picture. A caption the reader
writes over is better than an empty one they have to notice.

<storycode:copy id="f-722af83950b4-altFor">
/** The alt text for an image inserted from a file: its own name, which is the
 * only thing here that knows anything about the picture. A caption the reader
 * writes over is better than an empty one they have to notice. */
export function altFor(name: string): string {
  const base = name.replace(/\.[^.]+$/, "");
  return base.replace(/[-_]+/g, " ").trim();
}

</storycode:copy>
<a id="f-722af83950b4-base64Of"></a>
### `base64Of`

A file's bytes as standard base64, for the JSON body `POST /api/asset`
takes.
`FileReader` rather than `arrayBuffer()` + a hand-rolled encoder: the
`data:` URL it produces is already base64 and already correct for every
encoding edge (the server strips the prefix), and building the string in JS
one 8-bit character at a time overflows the argument limit on a megabyte
image.

<storycode:copy id="f-722af83950b4-base64Of">
/**
 * A file's bytes as standard base64, for the JSON body `POST /api/asset`
 * takes.
 *
 * `FileReader` rather than `arrayBuffer()` + a hand-rolled encoder: the
 * `data:` URL it produces is already base64 and already correct for every
 * encoding edge (the server strips the prefix), and building the string in JS
 * one 8-bit character at a time overflows the argument limit on a megabyte
 * image.
 */
export function base64Of(file: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () =>
      reject(new Error("the image could not be read from the clipboard or the drag"));
    reader.onload = () => resolve(String(reader.result ?? ""));
    reader.readAsDataURL(file);
  });
}

</storycode:copy>
<a id="f-722af83950b4-nextId"></a>
### `nextId`

<storycode:copy id="f-722af83950b4-nextId">
let nextId = 1;

</storycode:copy>
<a id="f-722af83950b4-mdPaste"></a>
### `mdPaste`

Paste and drop, for a buffer whose prose is markdown.
Returns an extension; everything it needs from the app arrives in `config`
so this file imports no API client and no React.

<storycode:copy id="f-722af83950b4-mdPaste">
/**
 * Paste and drop, for a buffer whose prose is markdown.
 *
 * Returns an extension; everything it needs from the app arrives in `config`
 * so this file imports no API client and no React.
 */
export function mdPaste(config: MdPasteConfig): Extension {
  const insertImages = (view: EditorView, files: File[], at: number) => {
    const docPath = config.docPath();
    for (const file of files) {
      const id = nextId++;
      view.dispatch({ effects: addPending.of({ id, pos: at }) });
      const named = new File(
        [file],
        pastedImageName(file, id.toString(36)),
        { type: file.type },
      );
      config
        .upload(named, docPath)
        .then((saved) => {
          const pos = pendingPos(view.state, id);
          if (pos === null) return;
          const markdown = imageMarkdown(saved.relative, altFor(named.name));
          view.dispatch({
            changes: { from: pos, insert: markdown },
            selection: { anchor: pos + markdown.length },
            effects: dropPending.of(id),
            // The user's own act, not a remote one: it belongs in the undo
            // history beside the drop that started it.
            userEvent: "input.paste",
          });
        })
        .catch((error: unknown) => {
          view.dispatch({ effects: dropPending.of(id) });
          config.onError?.(
            error instanceof Error ? error.message : String(error),
          );
        });
    }
  };

  return [
    pendingField,
    EditorView.domEventHandlers({
      paste(event, view) {
        const images = imageFilesOf(event.clipboardData);
        if (images.length > 0) {
          event.preventDefault();
          insertImages(view, images, view.state.selection.main.from);
          return true;
        }
        const text = event.clipboardData?.getData("text/plain") ?? "";
        const { from, to } = view.state.selection.main;
        if (from === to) return false;
        if (config.isProse && !config.isProse(view.state, from, to)) return false;
        const link = linkOverSelection(view.state.sliceDoc(from, to), text);
        if (link === null) return false;
        event.preventDefault();
        view.dispatch({
          changes: { from, to, insert: link },
          // The caret lands after the whole link rather than selecting it:
          // the next thing anyone types is the next word of the sentence.
          selection: { anchor: from + link.length },
          userEvent: "input.paste",
        });
        return true;
      },
      drop(event, view) {
        const images = imageFilesOf(event.dataTransfer);
        if (images.length === 0) return false;
        event.preventDefault();
        // Where the pointer was, not where the caret was: a drag has its own
        // target and the caret is wherever it was left.
        const at =
          view.posAtCoords({ x: event.clientX, y: event.clientY }) ??
          view.state.selection.main.from;
        insertImages(view, images, at);
        return true;
      },
      dragover(event) {
        // Without this the browser refuses the drop and opens the image in
        // the window instead, replacing the app.
        if (imageFilesOf(event.dataTransfer).length === 0) return false;
        event.preventDefault();
        if (event.dataTransfer) event.dataTransfer.dropEffect = "copy";
        return true;
      },
    }),
  ];
}
</storycode:copy>

### Assemble `apps/web/src/editor/mdPaste.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/mdPaste.ts"><storycode:paste select="#f-722af83950b4-Pending"/><storycode:paste select="#f-722af83950b4-addPending"/><storycode:paste select="#f-722af83950b4-dropPending"/><storycode:paste select="#f-722af83950b4-pendingField"/><storycode:paste select="#f-722af83950b4-pendingPos"/><storycode:paste select="#f-722af83950b4-SavedAsset"/><storycode:paste select="#f-722af83950b4-MdPasteConfig"/><storycode:paste select="#f-722af83950b4-imageFilesOf"/><storycode:paste select="#f-722af83950b4-pastedImageName"/><storycode:paste select="#f-722af83950b4-altFor"/><storycode:paste select="#f-722af83950b4-base64Of"/><storycode:paste select="#f-722af83950b4-nextId"/><storycode:paste select="#f-722af83950b4-mdPaste"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-c4094026435c"></a>
## `apps/web/src/editor/mdPaste.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An Image Dropped In Becomes A File And A Reference](../../../../../guarantees/authoring/an-image-dropped-in-becomes-a-file-and-a-reference/contract.md), [Pasting A URL Over Words Makes A Link](../../../../../guarantees/authoring/pasting-a-url-over-words-makes-a-link/contract.md).

<a id="f-c4094026435c-transfer"></a>
### `transfer`

<storycode:copy id="f-c4094026435c-transfer">
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

</storycode:copy>
<a id="f-c4094026435c-textTransfer"></a>
### `textTransfer`

<storycode:copy id="f-c4094026435c-textTransfer">
function textTransfer(text: string): DataTransfer {
  return {
    files: [],
    items: [],
    getData: (kind: string) => (kind === "text/plain" ? text : ""),
  } as unknown as DataTransfer;
}

</storycode:copy>
<a id="f-c4094026435c-editorWith"></a>
### `editorWith`

<storycode:copy id="f-c4094026435c-editorWith">
function editorWith(
  doc: string,
  upload = vi.fn(async () => ({ relative: "assets/shot.png" })),
) {
  const view = new EditorView({
    state: EditorState.create({
      doc,
      extensions: [mdPaste({ docPath: () => "notes/a.md", upload })],
    }),
    parent: document.body,
  });
  return { view, upload };
}

</storycode:copy>
<a id="f-c4094026435c-paste"></a>
### `paste`

Fire a paste at the editor the way the browser would.
`defaultPrevented` is not the signal for "this module stayed out of it":
CodeMirror handles every paste itself and prevents the default regardless.
What separates the two cases is what ends up in the buffer.

<storycode:copy id="f-c4094026435c-paste">
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

</storycode:copy>
<a id="f-c4094026435c-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c4094026435c-expression_statement">
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

</storycode:copy>
<a id="f-c4094026435c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c4094026435c-expression_statement-2">
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
    expect(upload).toHaveBeenCalledWith(expect.any(File), "notes/a.md");
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

</storycode:copy>
<a id="f-c4094026435c-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-c4094026435c-expression_statement-3">
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
</storycode:copy>

### Assemble `apps/web/src/editor/mdPaste.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/mdPaste.test.ts"><storycode:paste select="#f-c4094026435c-transfer"/><storycode:paste select="#f-c4094026435c-textTransfer"/><storycode:paste select="#f-c4094026435c-editorWith"/><storycode:paste select="#f-c4094026435c-paste"/><storycode:paste select="#f-c4094026435c-expression_statement"/><storycode:paste select="#f-c4094026435c-expression_statement-2"/><storycode:paste select="#f-c4094026435c-expression_statement-3"/></storycode:file>


</storycode:doc>
