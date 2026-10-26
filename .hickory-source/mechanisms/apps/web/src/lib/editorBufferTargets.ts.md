<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/editorBufferTargets.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-17f526449e3c"></a>
## `apps/web/src/lib/editorBufferTargets.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [New Document Creates Another Unnamed Buffer](../../../../../guarantees/authoring/new-document-is-an-act/contract.md).

<a id="f-17f526449e3c-BUFFER_PREFIX"></a>
### `BUFFER_PREFIX`

<storycode:copy id="f-17f526449e3c-BUFFER_PREFIX">
import { rememberReadingState, restoreReadingState } from "./contextProjection";
// Explicit editor identities for provenance that has no filesystem path.
import type { EditorView } from "@codemirror/view";
import type { Dispatch, SetStateAction } from "react";
import { activate, panes, type Layout } from "../shell/layout";
import { lensSources, registerLens } from "./lensSources";
import { claimReveal, onRevealLine, revealLine } from "./revealLine";

export const BUFFER_PREFIX = "hickory-buffer://";
</storycode:copy>
<a id="f-17f526449e3c-instance"></a>
### `instance`

Tab ids restart when a window reloads; old receipts must not bind to a new tab.

<storycode:copy id="f-17f526449e3c-instance">
// Tab ids restart when a window reloads; old receipts must not bind to a new tab.
const instance = Array.from(crypto.getRandomValues(new Uint8Array(16)), byte => byte.toString(16).padStart(2, "0")).join("");
</storycode:copy>
<a id="f-17f526449e3c-bufferTarget"></a>
### `bufferTarget`

<storycode:copy id="f-17f526449e3c-bufferTarget">
export const bufferTarget = (id: string) => `${BUFFER_PREFIX}${instance}/${id}`;
</storycode:copy>
<a id="f-17f526449e3c-registerBufferEditor"></a>
### `registerBufferEditor`

<storycode:copy id="f-17f526449e3c-registerBufferEditor">
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

</storycode:copy>
<a id="f-17f526449e3c-openBufferTarget"></a>
### `openBufferTarget`

Consume buffer locations before any filesystem-opening path.

<storycode:copy id="f-17f526449e3c-openBufferTarget">
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

</storycode:copy>
<a id="f-17f526449e3c-bufferEditor"></a>
### `bufferEditor`

<storycode:copy id="f-17f526449e3c-bufferEditor">
export function bufferEditor(id: string): EditorView | null {
  return lensSources().find(lens => lens.path === bufferTarget(id) && lens.view.dom.isConnected)?.view ?? null;
}
</storycode:copy>

### Assemble `apps/web/src/lib/editorBufferTargets.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/editorBufferTargets.ts"><storycode:paste select="#f-17f526449e3c-BUFFER_PREFIX"/><storycode:paste select="#f-17f526449e3c-instance"/><storycode:paste select="#f-17f526449e3c-bufferTarget"/><storycode:paste select="#f-17f526449e3c-registerBufferEditor"/><storycode:paste select="#f-17f526449e3c-openBufferTarget"/><storycode:paste select="#f-17f526449e3c-bufferEditor"/></storycode:file>

</storycode:doc>
