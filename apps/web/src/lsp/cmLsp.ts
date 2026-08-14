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

/** One diagnostic, in editor offsets, with the text a person needs to read. */
export interface DiagnosticSpan {
  from: number;
  to: number;
  severity: number;
  /** What the server said. Carried through so hovering the squiggle can
      show it — an underline with no message tells you where the problem is
      and nothing about what it is. */
  message?: string;
  /** Which server said it (`typescript`, `basedpyright`), so a document with
      several languages does not leave you guessing. */
  source?: string;
}

export const setLspDiagnostics = StateEffect.define<DiagnosticSpan[]>();

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
            // The message rides along on the decoration's spec, which is how
            // the hover below finds it without a second data structure to
            // keep in step with this one.
            Decoration.mark({
              class: d.severity === 1 ? "cm-lsp-error" : "cm-lsp-warn",
              message: d.message ?? "",
              source: d.source ?? "",
              severity: d.severity,
            }),
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
): DiagnosticSpan[] {
  const out: DiagnosticSpan[] = [];
  for (const d of diagnostics) {
    const from = offsetAt(d.range.start);
    const to = offsetAt(d.range.end);
    if (from === null || to === null) continue;
    out.push({
      from,
      // A zero-width diagnostic (an insertion point) still has to be
      // hoverable, so it is widened to one character.
      to: Math.max(to, from + 1),
      severity: d.severity ?? 1,
      message: d.message,
      source: d.source,
    });
  }
  return out;
}

/** The diagnostics under `pos`, outermost first, for the hover. */
export function diagnosticsAt(
  state: { field: (f: typeof lspDiagnosticField) => DecorationSet },
  pos: number,
): { message: string; source: string; severity: number }[] {
  const found: { message: string; source: string; severity: number }[] = [];
  state.field(lspDiagnosticField).between(pos, pos, (_from, _to, value) => {
    const spec = value.spec as { message?: string; source?: string; severity?: number };
    if (spec.message) {
      found.push({
        message: spec.message,
        source: spec.source ?? "",
        severity: spec.severity ?? 1,
      });
    }
  });
  return found;
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
      // The diagnostic comes FIRST, and comes from local state rather than a
      // request: if you are pointing at a squiggle, the thing you want is the
      // reason for the squiggle. Hover info is what a symbol IS; a diagnostic
      // is what is wrong with it, and an underline that cannot tell you which
      // is only half a report.
      const problems = diagnosticsAt(view.state, pos);
      const position = positionAt(pos, view);
      const result = position ? await client.hover(uri, position).catch(() => null) : null;
      const text = hoverText(result?.contents);
      if (problems.length === 0 && !text.trim()) return null;

      return {
        pos,
        above: true,
        create: () => {
          const dom = document.createElement("div");
          dom.className = "cm-lsp-hover";
          for (const problem of problems) {
            const line = document.createElement("div");
            line.className =
              problem.severity === 1 ? "cm-lsp-hover-error" : "cm-lsp-hover-warn";
            line.textContent = problem.source
              ? `${problem.message}  (${problem.source})`
              : problem.message;
            dom.appendChild(line);
          }
          if (text.trim()) {
            const info = document.createElement("div");
            // Separated from the problem above it, so two different kinds of
            // statement do not read as one paragraph.
            info.className = problems.length > 0 ? "cm-lsp-hover-info" : "";
            info.textContent = text;
            dom.appendChild(info);
          }
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
