<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/editorBufferTargets.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [New Document Creates Another Unnamed Buffer](../../../../../../guarantees/authoring/new-document-is-an-act/contract.md)

Captured SHA-256: `1cfa37003957fba4ec223d623f872917fab17a900588a90e86729451ef991644`.

<book:entry path="apps/web/src/lib/editorBufferTargets.ts">
import { rememberReadingState, restoreReadingState } from "./contextProjection";
// Explicit editor identities for provenance that has no filesystem path.
import type { EditorView } from "@codemirror/view";
import type { Dispatch, SetStateAction } from "react";
import { activate, panes, type Layout } from "../shell/layout";
import { lensSources, registerLens } from "./lensSources";
import { claimReveal, onRevealLine, revealLine } from "./revealLine";

export const BUFFER_PREFIX = "hickory-buffer://";
// Tab ids restart when a window reloads; old receipts must not bind to a new tab.
const instance = Array.from(crypto.getRandomValues(new Uint8Array(16)), byte => byte.toString(16).padStart(2, "0")).join("");
export const bufferTarget = (id: string) => `${BUFFER_PREFIX}${instance}/${id}`;
export function registerBufferEditor(id: string, view: EditorView): () => void {
  const path = bufferTarget(id);
  restoreReadingState(path, view);
  const unregister = registerLens({ path, view, source: view.state.doc.toString(), links: [] });
  const reveal = (target: string) => {
    if (target !== path) return;
    const line = claimReveal(path);
    if (line === null) return;
    const position = view.state.doc.line(Math.min(line, view.state.doc.lines)).from;
    view.dispatch({ selection: { anchor: position }, scrollIntoView: true });
    view.focus();
  };
  const unlisten = onRevealLine(reveal);
  reveal(path);
  return () => { rememberReadingState(path, view); unregister(); unlisten(); };
}

/** Consume buffer locations before any filesystem-opening path. */
export function openBufferTarget(path: string, line: number, layout: Layout,
  setLayout: Dispatch<SetStateAction<Layout>>, notice: (message: string) => void): boolean {
  if (!path.startsWith(BUFFER_PREFIX)) return false;
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex(tab => bufferTarget(tab.id) === path && tab.kind === "untitled");
    if (index < 0) continue;
    revealLine(path, line);
    setLayout(current => activate(current, pane.id, index));
    return true;
  }
  notice("The cited unsaved editor is no longer open. Its supplied text remains in the session context.");
  return true;
}

export function bufferEditor(id: string): EditorView | null {
  return lensSources().find(lens => lens.path === bufferTarget(id) && lens.view.dom.isConnected)?.view ?? null;
}
</book:entry>

</book:doc>
