import { Component, useEffect, useState, useSyncExternalStore } from "react";
import { createPortal } from "react-dom";
import { Buffer, Buffers } from "./buffers";
import { editorForBuffer, SIMPLE_EDITOR_ATTACHED } from "./Document";
import type { EditorView } from "@codemirror/view";
import type { RenderedSlot } from "../editor/rendered";
import type { Block, ExecBlock, DiagramBlock } from "../api/types";
import { request } from "../api/client";

// The owners live above pane layout. Moving a cached editor cannot unmount a
// table's uncommitted cell editor or lose a rendered widget's own local state.
export function RichOwners({ buffers }: { buffers: Buffers }) {
  const [, refresh] = useState(0);
  useEffect(() => { const update = () => refresh(value => value + 1); window.addEventListener(SIMPLE_EDITOR_ATTACHED, update); update(); return () => window.removeEventListener(SIMPLE_EDITOR_ATTACHED, update); }, []);
  return <>{[...buffers.values.values()].map(buffer => {
    const cached = editorForBuffer(buffer);
    return cached && /\.(md|markdown)$/i.test(buffer.file.path) ? <RichBoundary key={buffer.file.path} buffer={buffer} view={cached.view}><RichOwner buffer={buffer} view={cached.view} /></RichBoundary> : null;
  })}</>;
}
class RichBoundary extends Component<{ buffer: Buffer; view: EditorView; children: React.ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() { return { failed: true }; }
  componentDidCatch() {
    const cached = editorForBuffer(this.props.buffer);
    if (cached?.view === this.props.view) { cached.richAttached = false; cached.view.dispatch({ effects: cached.prose.reconfigure([]) }); }
    this.props.buffer.error = "Inline rendering unavailable. Continue editing source or reload to retry."; this.props.buffer.publish();
  }
  render() { return this.state.failed ? null : this.props.children; }
}
function SlotPortal({ slot, children, css }: { slot: RenderedSlot; children: React.ReactNode; css: string }) {
  const [root] = useState(() => slot.el.shadowRoot ?? slot.el.attachShadow({ mode: "open" }));
  return createPortal(<><style>{css.replaceAll(":root", ":host")}</style>{children}</>, root);
}
function RichOwner({ buffer, view }: { buffer: Buffer; view: EditorView }) {
  useSyncExternalStore(buffer.subscribe, buffer.snapshot);
  const [slots, setSlots] = useState<RenderedSlot[]>([]);
  const [modules, setModules] = useState<Awaited<ReturnType<typeof load>> | null>(null);
  const [blocks, setBlocks] = useState<{ source: string; blocks: Block[] }>({ source: "", blocks: [] });
  useEffect(() => {
    if (buffer.text.length > 200_000) return;
    let disposed = false, off: (() => void) | undefined, stopWaiting: (() => void) | undefined;
    void load().then(libs => {
      const install = () => {
      if (disposed || !view.dom.isConnected) return;
      const cached = editorForBuffer(buffer); if (!cached || cached.view !== view) return;
      const registry = new libs.rendered.RenderedRegistry();
      off = registry.subscribe(() => setSlots(registry.list()));
      cached.richAttached = true;
      const scroll = view.scrollSnapshot();
      view.dispatch({ effects: cached.prose.reconfigure([
        libs.prose.wysiwyg(new libs.prose.EnvRegistry()), libs.rendered.renderedBlocks(registry),
        libs.math.renderedMath(state => libs.spans.mathSpans(state.doc.toString(), libs.hick.verbatimRanges(libs.prose.structureOf(state).blocks))),
      ]) });
      const initial = libs.elements.renderableBlocks(libs.prose.structureOf(view.state), view.state.doc.toString()).filter(block => !libs.elements.slotKindOf(block)?.startsWith("session-") && !(buffer.dirty && view.state.selection.ranges.some(range => range.from >= block.from && range.to <= block.to)));
      view.dispatch({ effects: [libs.rendered.setRenderedBlocks.of(initial.map(block => block.from)), scroll] });
      setSlots(registry.list()); setModules(libs);
      };
      if (view.dom.isConnected) install();
      else {
        const attached = () => { if (view.dom.isConnected) { stopWaiting?.(); install(); } };
        window.addEventListener(SIMPLE_EDITOR_ATTACHED, attached);
        stopWaiting = () => window.removeEventListener(SIMPLE_EDITOR_ATTACHED, attached);
      }
    }).catch(error => { buffer.error = "Inline rendering unavailable: " + (error instanceof Error ? error.message : String(error)); buffer.publish(); });
    return () => { disposed = true; stopWaiting?.(); off?.(); const cached = editorForBuffer(buffer); if (cached?.view === view) { cached.richAttached = false; view.dispatch({ effects: cached.prose.reconfigure([]) }); } };
  }, [buffer, view, buffer.text.length > 200_000]);
  useEffect(() => {
    if (typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(() => view.requestMeasure()); slots.forEach(slot => observer.observe(slot.el)); return () => observer.disconnect();
  }, [view, slots]);
  useEffect(() => {
    if (!buffer.file.doc_id || buffer.dirty) return;
    const source = buffer.text, id = buffer.file.doc_id; let cancelled = false;
    void crypto.subtle.digest("SHA-256", new TextEncoder().encode(source)).then(async bytes => {
      const expected = [...new Uint8Array(bytes)].map(byte => byte.toString(16).padStart(2, "0")).join("");
      const result = await request<{ document: string; source_hash: string; blocks: Block[] }>("GET", `/api/docs/${id}/render`);
      if (!cancelled && buffer.text === source && !buffer.dirty && result.document === id && result.source_hash.replace(/^sha256:/, "") === expected) setBlocks({ source, blocks: result.blocks });
    }).catch(() => { /* Saved receipts are optional, never a reason to hide source. */ });
    return () => { cancelled = true; };
  }, [buffer, buffer.base, buffer.dirty]);
  if (!modules) return null;
  const saved: Block[] = [];
  if (blocks.source === buffer.text && !buffer.dirty) {
    const bytes = new TextEncoder().encode(blocks.source), decoder = new TextDecoder("utf-8", { fatal: true });
    for (const block of blocks.blocks) {
      if (!Array.isArray(block.span) || block.span[0] < 0 || block.span[1] < block.span[0] || block.span[1] > bytes.length) continue;
      try { saved.push({ ...block, span: [decoder.decode(bytes.slice(0, block.span[0])).length, decoder.decode(bytes.slice(0, block.span[1])).length] }); }
      catch { /* Invalid UTF-8 boundaries cannot identify a source widget. */ }
    }
  }
  const tables = modules.elements.renderableBlocks(modules.prose.structureOf(view.state), view.state.doc.toString()).filter(block => modules.elements.slotKindOf(block) === "table");
  const sourceTables = tables.filter(block => !slots.some(slot => slot.at === block.from));
  return <>{sourceTables.length > 0 && view.dom.parentElement && createPortal(<button type="button" className="simple-render-tables" onClick={() => {
    view.dispatch({ effects: modules.rendered.setRenderedBlocks.of([...new Set([...slots.map(slot => slot.at), ...tables.map(block => block.from)])]) });
  }}>Edit tables as grids</button>, view.dom.parentElement)}{slots.map(slot => {
    const captured = view.state.doc;
    const context = { simpleTables: true, view, path: buffer.file.path, docId: buffer.file.doc_id, execBlocks: saved.filter((block): block is ExecBlock => block.kind === "exec"),
      diagramBlocks: saved.filter((block): block is DiagramBlock => block.kind === "diagram"), runningCells: new Set<string>(), replaying: [],
      replaceBlockContent: (target: RenderedSlot, text: string, origin?: string) => modules.edit.replaceCapturedBlock(view, target, text, origin ?? "input.table"),
    };
    return <SlotPortal key={slot.key} slot={slot} css={modules.css.default + modules.katexCss.default + modules.graphCss.default + modules.terminalCss.default}>
      {!slot.outputOnly && <button type="button" style={{ border: "1px solid currentColor", borderRadius: 4, padding: "3px 8px", cursor: "pointer" }} onClick={() => {
        if (view.state.doc !== captured) return;
        view.dispatch({ effects: modules.rendered.showBlockSource.of(slot.at), selection: { anchor: Math.min(slot.at, view.state.doc.length) }, scrollIntoView: true }); view.focus();
      }}>Edit source text</button>}
      {modules.elements.elementViews[slot.kind].render(slot, context)}
    </SlotPortal>;
  })}</>;
}
async function load() {
  // Resolve the shared cyclic registry graph before importing its members in parallel.
  const parser = await import("../editor/hickLang");
  await parser.loadHickLang();
  const rendered = await import("../editor/rendered");
  const [prose, math, spans, hick, elements, edit, css, katexCss, graphCss, terminalCss] = await Promise.all([
    import("../editor/wysiwyg"), import("../editor/mathRender"), import("../lib/math"), import("../editor/hickDoc"),
    import("../elements"), import("../editor/replaceCapturedBlock"), import("../styles.css?inline"), import("katex/dist/katex.min.css?inline"), import("@xyflow/react/dist/style.css?inline"), import("@xterm/xterm/css/xterm.css?inline"),
  ]);
  return { prose, rendered, math, spans, hick, elements, edit, css, katexCss, graphCss, terminalCss };
}
