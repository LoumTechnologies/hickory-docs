import { Component, useEffect, useState, useSyncExternalStore } from "react";
import { createPortal } from "react-dom";
import { Buffer, Buffers } from "./buffers";
import { editorForBuffer, SIMPLE_EDITOR_ATTACHED } from "./Document";
import { EditorView } from "@codemirror/view";
import { DebugRailGroup, debugRailElement } from "../debug/DebugRail";
import type { RenderedSlot } from "../editor/rendered";
import type { Block, ExecBlock, DiagramBlock } from "../api/types";
import { request } from "../api/client";
import { openLocation, workspaceLinkTarget } from "../lib/revealLine";

const inlineUnavailable = "Inline rendering unavailable. Continue editing source or reload to retry.";

// The owners live above pane layout. Moving a cached editor cannot unmount a
// table's uncommitted cell editor or lose a rendered widget's own local state.
export function RichOwners({ buffers, recorded = false }: { buffers: Buffers; recorded?: boolean }) {
  const [, refresh] = useState(0);
  useEffect(() => { const update = () => refresh(value => value + 1); window.addEventListener(SIMPLE_EDITOR_ATTACHED, update); update(); return () => window.removeEventListener(SIMPLE_EDITOR_ATTACHED, update); }, []);
  return <>{[...buffers.values.values()].map(buffer => {
    const cached = editorForBuffer(buffer);
    return cached && /\.(md|markdown)$/i.test(buffer.file.path) ? <RichBoundary key={buffer.file.path} buffer={buffer} view={cached.view}><RichOwner buffer={buffer} view={cached.view} keepPatchSource={recorded && buffer.readonly} /></RichBoundary> : null;
  })}</>;
}
class RichBoundary extends Component<{ buffer: Buffer; view: EditorView; children: React.ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() { return { failed: true }; }
  componentDidCatch() {
    const cached = editorForBuffer(this.props.buffer);
    if (cached?.view !== this.props.view) return;
    cached.richAttached = false;
    cached.view.dispatch({ effects: [cached.prose.reconfigure([]), cached.view.scrollSnapshot()] });
    this.props.buffer.setInlineRenderNotice(inlineUnavailable);
  }
  render() { return this.state.failed ? null : this.props.children; }
}
function SlotPortal({ slot, children, css }: { slot: RenderedSlot; children: React.ReactNode; css: string }) {
  const [root] = useState(() => slot.el.shadowRoot ?? slot.el.attachShadow({ mode: "open" }));
  return createPortal(<><style>{css.replaceAll(":root", ":host") + `
:host {
  color-scheme: inherit; color: CanvasText;
  --bg: Canvas; --fg: CanvasText; --fg-muted: color-mix(in srgb, CanvasText 65%, Canvas); --muted: var(--fg-muted);
  --bg-raised: Canvas; --bg-sunken: color-mix(in srgb, CanvasText 5%, Canvas);
  --border: color-mix(in srgb, CanvasText 22%, Canvas);
  --accent: Highlight; --accent-2: light-dark(#176b73, #71d1d9);
  --ok: light-dark(#267341, #8cdaa2); --fail: light-dark(#b42330, #ff9696);
  --danger: var(--fail); --warn: light-dark(#855b0a, #ecc07c); --stale: var(--warn);
  --ok-bg: color-mix(in srgb, var(--ok) 10%, Canvas);
  --fail-bg: color-mix(in srgb, var(--fail) 10%, Canvas);
  --stale-bg: color-mix(in srgb, var(--warn) 10%, Canvas);
  --neutral-bg: var(--bg-sunken);
  --inline-terminal-bg: var(--bg-sunken);
  --inline-terminal-fg: CanvasText;
  --inline-terminal-heading: color-mix(in srgb, CanvasText 8%, Canvas);
  --inline-terminal-border: var(--border);
  --inline-terminal-scheme: light dark;
  --diff-del: var(--fail); --diff-ins: var(--ok);
  --select-bg: color-mix(in srgb, Highlight 22%, Canvas);
  --search-hit: color-mix(in srgb, var(--warn) 20%, Canvas);
  --search-hit-on: color-mix(in srgb, var(--warn) 40%, Canvas);
  --tok-keyword: light-dark(#7637a0, #d4a5ff);
  --tok-function: light-dark(#145e9c, #83c7ff);
  --tok-string: var(--ok); --tok-comment: GrayText;
  --tok-literal: var(--warn); --tok-type: var(--accent-2);
  --tok-property: var(--tok-function); --tok-tag: var(--tok-keyword);
  --tok-meta: GrayText; --tok-operator: GrayText;
  --tok-parameter: var(--tok-type); --tok-invalid: var(--fail);
}
`}</style>{children}</>, root);
}
function RichOwner({ buffer, view, keepPatchSource }: { buffer: Buffer; view: EditorView; keepPatchSource: boolean }) {
  useSyncExternalStore(buffer.subscribe, buffer.snapshot);
  const [slots, setSlots] = useState<RenderedSlot[]>([]);
  const [modules, setModules] = useState<Awaited<ReturnType<typeof load>> | null>(null);
  const [receiptRevision, refreshReceipts] = useState(0);
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
        EditorView.updateListener.of(update => { if (update.transactions.length) queueMicrotask(() => { if (!disposed) setSlots(registry.list()); }); }),
        libs.math.renderedMath(state => simpleMathSpans(libs, state)),
        libs.links.mdLinks({
          docPath: () => buffer.file.path,
          skip: state => [...libs.hick.verbatimRanges(libs.prose.structureOf(state).blocks),
            ...libs.markdown.fencedCodeRanges(state.doc.toString())].sort(([a], [b]) => a - b),
          onFollow: href => {
            const target = workspaceLinkTarget(href, buffer.file.path);
            if (!target || !view.dom.isConnected) return { status: "unavailable", reason: "Choose a link to a file in this workspace." };
            openLocation(target.path, target.line);
            return { status: "completed", value: { navigation_requested: true } };
          },
        }),
      ]) });
      const initial = libs.elements.renderableBlocks(libs.prose.structureOf(view.state), view.state.doc.toString()).filter(block =>
        block.name !== "math" && block.name !== "patch" && !libs.elements.slotKindOf(block)?.startsWith("session-")
        && !(buffer.dirty && view.state.selection.ranges.some(range => range.from >= block.from && range.to <= block.to)));
      view.dispatch({ effects: [libs.rendered.setRenderedBlocks.of(initial.map(block => block.from)), scroll] });
      buffer.setInlineRenderNotice("");
      setSlots(registry.list()); setModules(libs);
      };
      if (view.dom.isConnected) install();
      else {
        const attached = () => { if (view.dom.isConnected) { stopWaiting?.(); install(); } };
        window.addEventListener(SIMPLE_EDITOR_ATTACHED, attached);
        stopWaiting = () => window.removeEventListener(SIMPLE_EDITOR_ATTACHED, attached);
      }
    }).catch(() => {
      if (!disposed && editorForBuffer(buffer)?.view === view) buffer.setInlineRenderNotice(inlineUnavailable);
    });
    return () => { disposed = true; stopWaiting?.(); off?.(); const cached = editorForBuffer(buffer); if (cached?.view === view) { cached.richAttached = false; view.dispatch({ effects: cached.prose.reconfigure([]) }); } };
  }, [buffer, view, keepPatchSource, buffer.text.length > 200_000]);
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
      if (!cancelled && buffer.file.doc_id === id && buffer.text === source && !buffer.dirty && result.document === id && result.source_hash.replace(/^sha256:/, "") === expected) setBlocks({ source, blocks: result.blocks });
    }).catch(() => { /* Saved receipts are optional, never a reason to hide source. */ });
    return () => { cancelled = true; };
  }, [buffer, buffer.file.doc_id, buffer.base, buffer.dirty, receiptRevision]);
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
  const controls = modules.elements.renderableBlocks(modules.prose.structureOf(view.state), view.state.doc.toString()).filter(block =>
    block.name !== "math" && !(keepPatchSource && block.name === "patch") && !modules.elements.slotKindOf(block)?.startsWith("session-"));
  const math = simpleMathSpans(modules, view.state);
  const rail = debugRailElement(view);
  return <>{rail && math.map(span => createPortal(<DebugRailGroup key={`math:${span.from}`} view={view} at={span.from}>
    <button type="button" className="debug-icon" aria-label="Toggle equation source text" onClick={() => {
      const mode = view.state.field(modules.math.mathSourceModes).get(span.from);
      const touched = view.state.selection.ranges.some(range => view.state.doc.lineAt(range.from).number <= view.state.doc.lineAt(span.to).number && view.state.doc.lineAt(range.to).number >= view.state.doc.lineAt(span.from).number);
      view.dispatch({ effects: modules.math.setMathSource.of({ at: span.from, source: !(mode ?? touched) }) });
    }}>‹/›</button>
  </DebugRailGroup>, rail))}{rail && controls.map(block => createPortal(<DebugRailGroup key={block.from} view={view} at={block.from}>
    <button type="button" className="debug-icon" aria-label={modules.rendered.isRendered(view.state, block.from) ? "Edit source text" : "Render embedded content"} aria-pressed={!modules.rendered.isRendered(view.state, block.from)} onClick={() => {
      const effects = modules.rendered.isRendered(view.state, block.from) ? modules.rendered.showBlockSource.of(block.from) : modules.rendered.renderBlock.of(block.from);
      view.dispatch({ effects });
    }}>‹/›</button>
  </DebugRailGroup>, rail))}{slots.map(slot => {
    const context = { simpleTables: true, systemAppearance: true, view, path: buffer.file.path, docId: buffer.file.doc_id, execBlocks: saved.filter((block): block is ExecBlock => block.kind === "exec"),
      diagramBlocks: saved.filter((block): block is DiagramBlock => block.kind === "diagram"), runningCells: new Set<string>(), replaying: [],
      replaceBlockContent: (target: RenderedSlot, text: string, origin?: string) => modules.edit.replaceCapturedBlock(view, target, text, origin ?? "input.table"),
    };
    return <SlotPortal key={slot.key} slot={slot} css={modules.css.default + modules.katexCss.default + modules.graphCss.default + modules.terminalCss.default}>
      {slot.kind === "exec" && buffer.file.doc_id && <button type="button" disabled={buffer.dirty} title={buffer.dirty ? "Save the source before refreshing the recorded output." : undefined} onClick={() => refreshReceipts(value => value + 1)}>Refresh recorded output</button>}
      {modules.elements.elementViews[slot.kind].render(slot, context)}
    </SlotPortal>;
  })}</>;
}
function simpleMathSpans(libs: Awaited<ReturnType<typeof load>>, state: import("@codemirror/state").EditorState) {
  const { blocks } = libs.prose.structureOf(state);
  return [...libs.spans.mathSpans(state.doc.toString(), libs.hick.verbatimRanges(blocks)),
    ...blocks.filter(block => block.name === "math" && block.close).map(block => ({
      from: block.from, to: block.to, display: true, source: state.doc.sliceString(block.contentFrom, block.contentTo),
    }))].sort((a, b) => a.from - b.from);
}
async function load() {
  // Resolve the shared cyclic registry graph before importing its members in parallel.
  const parser = await import("../editor/hickLang");
  await parser.loadHickLang();
  const rendered = await import("../editor/rendered");
  const [prose, math, spans, hick, elements, edit, css, katexCss, graphCss, terminalCss, links, markdown] = await Promise.all([
    import("../editor/wysiwyg"), import("../editor/mathRender"), import("../lib/math"), import("../editor/hickDoc"),
    import("../elements"), import("../editor/replaceCapturedBlock"), import("../styles.css?inline"), import("katex/dist/katex.min.css?inline"), import("@xyflow/react/dist/style.css?inline"), import("@xterm/xterm/css/xterm.css?inline"), import("../editor/mdLinks"), import("../editor/markdownStyling"),
  ]);
  return { prose, rendered, math, spans, hick, elements, edit, css, katexCss, graphCss, terminalCss, links, markdown };
}
