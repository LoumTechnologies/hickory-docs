<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain the editor instance while moving a document

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

CodeMirror owns typing, selection, scroll and undo. Cache the editor by buffer identity and move its existing DOM between hosts. Buffer notifications update the status, not React-controlled document text.

<ui:copy id="simple-1a4a59fc95-0">
import { useLayoutEffect, useRef, useSyncExternalStore } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, drawSelection } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { Buffer } from "./buffers";
const editors = new WeakMap<Buffer, EditorView>();
export function Document({ buffer }: { buffer: Buffer }) {
  const host = useRef<HTMLDivElement>(null);
  useSyncExternalStore(buffer.subscribe, buffer.snapshot);
  useLayoutEffect(() => {
    let editor = editors.get(buffer);
    if (!editor) {
      editor = new EditorView({ state: EditorState.create({ doc: buffer.text, extensions: [
        history(), drawSelection(), EditorView.lineWrapping, keymap.of([...defaultKeymap, ...historyKeymap]),
        EditorState.readOnly.of(buffer.readonly), EditorView.editable.of(!buffer.readonly),
        EditorView.contentAttributes.of({ "aria-label": buffer.file.path }),
        EditorView.updateListener.of(update => { if (update.docChanged) buffer.edit(update.state.doc.toString()); }),
        EditorView.theme({ "&": { height: "100%" }, ".cm-scroller": { overflow: "auto", fontFamily: "ui-monospace, monospace", fontSize: "13px" } }),
      ] }) });
      editors.set(buffer, editor);
    }
    host.current?.append(editor.dom);
    return () => { editor.dom.remove(); };
  }, [buffer]);
  return <div className="document-body">
    <div ref={host} className="editor-host" />
    {buffer.error && <div role="alert" className="notice">{buffer.error}</div>}
  </div>;
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/Document.tsx"><ui:paste select="#simple-1a4a59fc95-0"/></ui:file>

</ui:doc>
