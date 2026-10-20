// Putting a chosen element into the buffer.
//
// The decision about what bytes to write lives in lib/insertCatalog.ts; this
// is the half that needs a live CodeMirror view — reading what surrounds the
// caret, dispatching the change, and leaving the selection somewhere useful.

import type { EditorView } from "@codemirror/view";

import { buildInsertion, type FieldValues, type InsertElement } from "../lib/insertCatalog";

/**
 * How much of the buffer either side of the caret the padding rules read.
 *
 * They only ever count newlines immediately adjacent to the insertion point,
 * so a window is exactly as good as the whole document and does not copy a
 * megabyte of text to insert twelve characters. It must stay big enough that
 * the window is only ever empty at the true start or end of the buffer,
 * which is what "the document starts here, no blank line needed" is read
 * from.
 */
const WINDOW = 64;

/**
 * Insert `element` at the view's caret, replacing its selection.
 *
 * The transaction carries a `userEvent` on purpose. DocumentEditor keeps
 * everything WITHOUT one out of the undo history — that is how a
 * collaborator's edit and the room's first sync stay un-undoable — so an
 * insert dispatched without one could never be taken back with Ctrl+Z.
 */
export function insertElement(
  view: EditorView,
  element: InsertElement,
  values: FieldValues,
  /** The body as the panel last showed it. Omitted, the selection (or the
   * catalogue's starter text) is used. */
  body?: string,
  userEvent = "input.insertElement",
): void {
  const { state } = view;
  const range = state.selection.main;
  const insertion = buildInsertion(
    element,
    values,
    {
      before: state.doc.sliceString(Math.max(0, range.from - WINDOW), range.from),
      after: state.doc.sliceString(range.to, Math.min(state.doc.length, range.to + WINDOW)),
      selected: state.doc.sliceString(range.from, range.to),
    },
    body,
  );

  view.dispatch({
    changes: { from: range.from, to: range.to, insert: insertion.text },
    selection: {
      anchor: range.from + insertion.selectFrom,
      head: range.from + insertion.selectTo,
    },
    scrollIntoView: true,
    userEvent,
  });
  // The panel had the focus; typing over the placeholder is the next thing
  // anybody does, so the buffer takes it back.
  view.focus();
}

/** The dialog owns this exact editor/source/range even after another pane gains focus. */
export function captureInsertion(view: EditorView) {
  return { view, doc: view.state.doc, selection: { anchor: view.state.selection.main.anchor, head: view.state.selection.main.head } };
}
export function insertionCurrent(capture: ReturnType<typeof captureInsertion>): boolean {
  return capture.view.dom.isConnected && !capture.view.state.readOnly && capture.view.state.doc === capture.doc;
}
export function insertCapturedElement(capture: ReturnType<typeof captureInsertion>, element: InsertElement, values: FieldValues, body: string, agent = false): boolean {
  if (!insertionCurrent(capture)) return false;
  capture.view.dispatch({ selection: capture.selection });
  insertElement(capture.view, element, values, body, agent ? "input.uiAgentInsert" : "input.insertElement");
  return true;
}
