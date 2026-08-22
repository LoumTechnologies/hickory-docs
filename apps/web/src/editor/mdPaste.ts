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

const addPending = StateEffect.define<Pending>();
const dropPending = StateEffect.define<number>();

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

/** Where the upload started, as it stands now, or null if it was forgotten. */
function pendingPos(state: EditorState, id: number): number | null {
  const found = state.field(pendingField, false)?.find((p) => p.id === id);
  return found ? Math.min(found.pos, state.doc.length) : null;
}

/** What an image upload has to answer with: the destination to write into the
 * markdown, relative to the document. */
export interface SavedAsset {
  relative: string;
}

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

/** The alt text for an image inserted from a file: its own name, which is the
 * only thing here that knows anything about the picture. A caption the reader
 * writes over is better than an empty one they have to notice. */
export function altFor(name: string): string {
  const base = name.replace(/\.[^.]+$/, "");
  return base.replace(/[-_]+/g, " ").trim();
}

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

let nextId = 1;

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
