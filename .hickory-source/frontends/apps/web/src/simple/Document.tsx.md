<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain the editor instance while moving a document

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

CodeMirror owns typing, selection, scroll and undo. Cache the editor by buffer identity and move its existing DOM between hosts. Buffer notifications update the status, not React-controlled document text.

The minimap reserves forty units beside the text rather than overlaying readable lines. Its navigation changes scroll, never saved bytes. Language controls float over the editor and do not reflow it. Browser capabilities use the existing LSP client and its snapshot guards; native keyword and word-list previews deliberately do not claim semantic results. Native styling does not replace text, preserving the platform undo and selection model. Preview coloring skips very large files instead of delaying readable text indefinitely.

<ui:copy id="simple-1a4a59fc95-0">
import { useEffect, useLayoutEffect, useRef, useState, useSyncExternalStore } from "react";
import { Compartment, EditorState } from "@codemirror/state";
import { EditorView, keymap, drawSelection } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { completionKeymap } from "@codemirror/autocomplete";
import { useWorkspaceLsp } from "../lsp/useLsp";
import { lspSupport } from "../lsp/cmLsp";
import { lspFeatures } from "../lsp/cmLspFeatures";
import { completions } from "../lsp/completion";
import { positionToUtf16, utf16ToPosition } from "../lsp/positions";
import { sameDocumentUri } from "../lsp/documentUri";
import { Buffer } from "./buffers";
const editors = new WeakMap<Buffer, { view: EditorView; language: Compartment }>();
export function Document({ buffer }: { buffer: Buffer }) {
  const host = useRef<HTMLDivElement>(null);
  useSyncExternalStore(buffer.subscribe, buffer.snapshot);
  const session = useWorkspaceLsp(buffer.file.path, buffer.text);
  const [capabilities, refreshCapabilities] = useState(0);
  useEffect(() => session.client?.onServerCapabilities(() => refreshCapabilities(value => value + 1)), [session.client]);
  useLayoutEffect(() => {
    let cached = editors.get(buffer);
    if (!cached) {
      const language = new Compartment();
      const view = new EditorView({ state: EditorState.create({ doc: buffer.text, extensions: [
        history(), drawSelection(), EditorView.lineWrapping, language.of([]),
        keymap.of([...completionKeymap, ...defaultKeymap, ...historyKeymap]),
        EditorState.readOnly.of(buffer.readonly), EditorView.editable.of(!buffer.readonly),
        EditorView.contentAttributes.of({ "aria-label": buffer.file.path }),
        EditorView.updateListener.of(update => { if (update.docChanged) buffer.edit(update.state.doc.toString()); }),
        EditorView.theme({ "&": { height: "100%" }, ".cm-scroller": { overflow: "auto", fontFamily: "ui-monospace, monospace", fontSize: "13px" } }),
      ] }) });
      cached = { view, language }; editors.set(buffer, cached);
    }
    host.current?.append(cached.view.dom);
    return () => { cached.view.dom.remove(); };
  }, [buffer]);
  useEffect(() => {
    const cached = editors.get(buffer);
    if (!cached || !session.client || !session.ready) return;
    const { view, language } = cached;
    const positionAt = (offset: number, editor: EditorView) => utf16ToPosition(editor.state.doc.toString(), offset);
    const offsetAt = (position: { line: number; character: number }, editor: EditorView) => positionToUtf16(editor.state.doc.toString(), position);
    view.dispatch({ effects: language.reconfigure([
      ...lspSupport({ client: session.client, uri: session.uri, positionAt,
        onNavigate: target => { if (sameDocumentUri(target.uri, session.uri)) {
          const anchor = offsetAt(target.range.start, view);
          view.dispatch({ selection: { anchor }, effects: EditorView.scrollIntoView(anchor, { y: "center" }) });
        } } }),
      ...lspFeatures({ client: session.client, uri: session.uri, positionAt, offsetAt, inlayHints: true }),
      ...(!buffer.readonly ? [completions({ lsp: { client: session.client, uri: session.uri,
        positionAt: (offset, state) => utf16ToPosition(state.doc.toString(), offset) } })] : []),
    ]) });
    return () => { view.dispatch({ effects: language.reconfigure([]) }); };
  }, [buffer, session.client, session.uri, session.ready, capabilities]);
  return <div className="document-body">
    <div className="editor-row"><div ref={host} className="editor-host" /><Minimap buffer={buffer} /></div>
    {buffer.error && <div role="alert" className="notice">{buffer.error}</div>}
  </div>;
}
</ui:copy>

## Navigate without editing

Draw at most one density mark per two screen pixels and repaint on resize, text changes and scroll. The viewport rectangle comes from the existing editor layout. Pointer navigation and Home/End only dispatch a scroll effect, leaving selection and undo untouched.

<ui:copy id="simple-document-minimap">
function Minimap({ buffer }: { buffer: Buffer }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const view = editors.get(buffer)!.view, element = canvas.current!;
    const draw = () => {
      const height = element.clientHeight, width = 40;
      element.width = width * devicePixelRatio; element.height = height * devicePixelRatio;
      const context = element.getContext("2d"); if (!context || !height) return;
      context.scale(devicePixelRatio, devicePixelRatio);
      const doc = view.state.doc, rows = Math.min(doc.lines, Math.ceil(height / 2));
      context.fillStyle = getComputedStyle(element).color;
      for (let row = 0; row < rows; row++) {
        const line = doc.line(1 + Math.floor(row * doc.lines / rows));
        context.globalAlpha = .5;
        context.fillRect(2, row * height / rows, Math.min(36, line.length * .65), 1);
      }
      const first = view.lineBlockAtHeight(view.scrollDOM.scrollTop).from;
      const last = view.lineBlockAtHeight(Math.min(view.contentHeight, view.scrollDOM.scrollTop + view.scrollDOM.clientHeight)).to;
      context.globalAlpha = .18;
      context.fillRect(0, (doc.lineAt(first).number - 1) / doc.lines * height, width,
        Math.max(4, (doc.lineAt(last).number - doc.lineAt(first).number + 1) / doc.lines * height));
    };
    const observer = new ResizeObserver(draw); observer.observe(element);
    view.scrollDOM.addEventListener("scroll", draw); draw();
    return () => { observer.disconnect(); view.scrollDOM.removeEventListener("scroll", draw); };
  }, [buffer, buffer.text]);
  const jump = (fraction: number) => {
    const view = editors.get(buffer)!.view;
    const line = view.state.doc.line(Math.max(1, Math.min(view.state.doc.lines, 1 + Math.floor(fraction * view.state.doc.lines))));
    view.dispatch({ effects: EditorView.scrollIntoView(line.from, { y: "center" }) });
  };
  return <canvas ref={canvas} className="minimap" role="navigation" aria-label="Document minimap" tabIndex={0}
    onPointerDown={event => { event.currentTarget.setPointerCapture(event.pointerId); jump((event.clientY - event.currentTarget.getBoundingClientRect().top) / event.currentTarget.clientHeight); }}
    onPointerMove={event => { if (event.buttons) jump((event.clientY - event.currentTarget.getBoundingClientRect().top) / event.currentTarget.clientHeight); }}
    onKeyDown={event => { if (event.key === "Home" || event.key === "End") { event.preventDefault(); jump(event.key === "Home" ? 0 : 1); } }} />;
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/Document.tsx"><ui:paste select="#simple-1a4a59fc95-0"/><ui:paste select="#simple-document-minimap"/></ui:file>

</ui:doc>
