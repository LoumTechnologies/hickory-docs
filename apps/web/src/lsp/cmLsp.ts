// CodeMirror bindings for the LSP bridge: hover, go-to-definition, find
// references, and diagnostics.
//
// LAYOUT RULE (the one that matters here): diagnostics are drawn as MARK
// decorations only — a squiggly underline via background-image, never a border
// or a block widget. Anything that changes a line's height would have to be a
// StateField and would still perturb the ribbon geometry in Split view, so
// this deliberately stays background-only.
//
// Navigation is coordinate-agnostic: callers give `positionAt`, which converts
// a CodeMirror offset into a position in the DOCUMENT's coordinate space. The
// document editor passes an identity mapping; the output editor maps through
// provenance first, so Cmd-clicking a symbol in generated code asks the same
// question about the prose that produced it.

import { RangeSetBuilder, StateEffect, StateField } from "@codemirror/state";
import type { Extension, Text } from "@codemirror/state";
import { Decoration, EditorView, hoverTooltip, keymap } from "@codemirror/view";
import type { DecorationSet, Tooltip } from "@codemirror/view";
import type { LspClient, LspDiagnostic, LspLocation } from "./client";
import type { LspPosition } from "./positions";

export interface LspNavigationTarget {
  uri: string;
  range: { start: LspPosition; end: LspPosition };
}

export interface CmLspOptions {
  client: LspClient | null;
  /** Document URI every request is asked against (`hick:///<doc-path>`). */
  uri: string;
  /**
   * Map a CodeMirror offset in THIS editor to a position in the document's
   * coordinate space, or null when this offset has no source origin (a
   * synthetic byte the weaver produced).
   */
  positionAt: (offset: number, view: EditorView) => LspPosition | null;
  /** Open a definition/reference target. */
  onNavigate: (target: LspNavigationTarget) => void;
  /** Show a reference list (empty array means "none found"). */
  onReferences?: (locations: LspLocation[], from: LspNavigationTarget) => void;
}

// --- diagnostics ------------------------------------------------------------

export const setLspDiagnostics = StateEffect.define<{ from: number; to: number; severity: number }[]>();

export const lspDiagnosticField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const e of tr.effects) {
      if (e.is(setLspDiagnostics)) {
        const builder = new RangeSetBuilder<Decoration>();
        const sorted = [...e.value].sort((a, b) => a.from - b.from || a.to - b.to);
        const max = tr.state.doc.length;
        for (const d of sorted) {
          const from = Math.min(d.from, max);
          const to = Math.min(d.to, max);
          if (to <= from) continue;
          builder.add(
            from,
            to,
            Decoration.mark({ class: d.severity === 1 ? "cm-lsp-error" : "cm-lsp-warn" }),
          );
        }
        deco = builder.finish();
      }
    }
    return deco;
  },
  provide: (f) => EditorView.decorations.from(f),
});

/** Convert LSP diagnostics into editor offsets using a position mapper. */
export function diagnosticRanges(
  diagnostics: LspDiagnostic[],
  offsetAt: (pos: LspPosition) => number | null,
): { from: number; to: number; severity: number }[] {
  const out: { from: number; to: number; severity: number }[] = [];
  for (const d of diagnostics) {
    const from = offsetAt(d.range.start);
    const to = offsetAt(d.range.end);
    if (from === null || to === null) continue;
    out.push({ from, to: Math.max(to, from + 1), severity: d.severity ?? 1 });
  }
  return out;
}

// --- hover ------------------------------------------------------------------

function hoverText(contents: unknown): string {
  if (contents == null) return "";
  if (typeof contents === "string") return contents;
  if (Array.isArray(contents)) return contents.map(hoverText).filter(Boolean).join("\n\n");
  const o = contents as { value?: unknown; language?: unknown };
  if (typeof o.value === "string") return o.value;
  return "";
}

// --- the extension bundle ---------------------------------------------------

export function lspSupport(opts: CmLspOptions): Extension[] {
  const { client, uri, positionAt, onNavigate, onReferences } = opts;
  if (!client) return [lspDiagnosticField];

  const requestAt = async (
    offset: number,
    view: EditorView,
    what: "definition" | "references",
  ): Promise<void> => {
    const position = positionAt(offset, view);
    if (!position) return;
    const from: LspNavigationTarget = { uri, range: { start: position, end: position } };
    if (what === "definition") {
      const locations = await client.definition(uri, position).catch(() => []);
      if (locations.length > 0) onNavigate(locations[0]);
      return;
    }
    const locations = await client.references(uri, position).catch(() => []);
    onReferences?.(locations, from);
  };

  return [
    lspDiagnosticField,
    hoverTooltip(async (view, pos): Promise<Tooltip | null> => {
      const position = positionAt(pos, view);
      if (!position) return null;
      const result = await client.hover(uri, position).catch(() => null);
      const text = hoverText(result?.contents);
      if (!text.trim()) return null;
      return {
        pos,
        above: true,
        create: () => {
          const dom = document.createElement("div");
          dom.className = "cm-lsp-hover";
          dom.textContent = text;
          return { dom };
        },
      };
    }),
    keymap.of([
      {
        key: "F12",
        run: (view) => {
          void requestAt(view.state.selection.main.head, view, "definition");
          return true;
        },
      },
      {
        key: "Shift-F12",
        run: (view) => {
          void requestAt(view.state.selection.main.head, view, "references");
          return true;
        },
      },
      {
        key: "Alt-F12",
        run: (view) => {
          void requestAt(view.state.selection.main.head, view, "references");
          return true;
        },
      },
    ]),
    EditorView.domEventHandlers({
      mousedown(event, view) {
        if (!(event.metaKey || event.ctrlKey) || event.button !== 0) return false;
        const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
        if (pos === null) return false;
        event.preventDefault();
        void requestAt(pos, view, event.shiftKey ? "references" : "definition");
        return true;
      },
    }),
    // Ctrl/Cmd held: show the symbol under the pointer as a link, so the
    // affordance is discoverable instead of being folklore.
    EditorView.theme({
      "&.cm-lsp-linking .cm-content": { cursor: "pointer" },
    }),
  ];
}

// --- coordinate helpers -----------------------------------------------------

/**
 * CodeMirror offset → LSP position. Both count UTF-16 code units within a
 * line, so this is a line lookup and a subtraction — no re-encoding.
 */
export function offsetToPosition(doc: Text, offset: number): LspPosition {
  const clamped = Math.max(0, Math.min(offset, doc.length));
  const line = doc.lineAt(clamped);
  return { line: line.number - 1, character: clamped - line.from };
}

/** LSP position → CodeMirror offset, clamped into the document. */
export function positionToOffset(doc: Text, pos: LspPosition): number {
  const lineNo = Math.max(1, Math.min(pos.line + 1, doc.lines));
  const line = doc.line(lineNo);
  return Math.min(line.from + Math.max(0, pos.character), line.to);
}
