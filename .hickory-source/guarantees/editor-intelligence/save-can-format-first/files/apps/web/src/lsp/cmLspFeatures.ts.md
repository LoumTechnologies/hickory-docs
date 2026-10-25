<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lsp/cmLspFeatures.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Every Shortcut Is A Setting](../../../../../../every-shortcut-is-a-setting/contract.md)
- [Save Can Format First, With The File's Own Formatter](../../../../../contract.md)

Captured SHA-256: `b0958cf2c62c9a347ac74570aed1f19700edf5920c54977d95f1bd8d5aa59cbe`.

<book:entry path="apps/web/src/lsp/cmLspFeatures.ts">
import { formatOptionFallback } from "./formatOptionFallback";
import { sameDocumentUri } from "./documentUri";
import { codeActionController } from "./codeActionController";
import { editorLanguageActions } from "../editor/languageActions";
import { prepareEditorChanges } from "../editor/textInput";
import type { UiOutcome } from "../lib/uiActions";
// The rest of the language server, in the notebook.
//
// `cmLsp.ts` covers what the editor had first — diagnostics, hover, goto,
// references. This is everything else the meta-LSP forwards: colouring,
// inlay hints, highlight-on-cursor, signature help, folding, symbols, code
// actions and rename.
//
// LAYOUT RULE (inherited from cmLsp.ts, and the reason this file is careful):
// nothing here may change a line's HEIGHT. Split view draws ribbons between
// document lines and generated output, and geometry computed against a line
// that grew is geometry that points at the wrong place. So semantic tokens
// are mark decorations, inlay hints are inline widgets, and neither is a
// block. Folding is the one exception, and it is the user asking for it.
//
// Every feature degrades to nothing. A server that cannot colour returns no
// tokens and the editor looks exactly as it did before — which is the whole
// bargain of a meta-LSP over many children, since one document's Python and
// Rust blocks routinely have different capabilities.

import { bindingOf, cmKeyOf } from "../lib/keymap";
import { foldService } from "@codemirror/language";
import { Facet, RangeSetBuilder, StateEffect, StateField } from "@codemirror/state";
import type { Extension, Text } from "@codemirror/state";
import { Decoration, EditorView, ViewPlugin, WidgetType, keymap } from "@codemirror/view";
import type { DecorationSet, ViewUpdate } from "@codemirror/view";
import type { CodeAction, InlayHint, LspClient, WorkspaceEdit } from "./client";
import type { LspPosition, LspRange } from "./positions";
import { decodeSemanticTokens, tokenClass } from "./semanticTokens";
import type { SemanticLegend } from "./semanticTokens";

/** Offsets, so the decoration layers never touch LSP coordinates. */
export interface Span {
  from: number;
  to: number;
  class: string;
}

// --- semantic tokens (colouring) -------------------------------------------

export const setSemanticTokens = StateEffect.define<Span[]>();

export const semanticTokenField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const effect of tr.effects) {
      if (!effect.is(setSemanticTokens)) continue;
      deco = buildMarks(effect.value, tr.state.doc.length);
    }
    return deco;
  },
  provide: (field) => EditorView.decorations.from(field),
});

// --- document highlight (the other uses of what the cursor is on) ----------

export const setHighlights = StateEffect.define<Span[]>();

export const highlightField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const effect of tr.effects) {
      if (!effect.is(setHighlights)) continue;
      deco = buildMarks(effect.value, tr.state.doc.length);
    }
    return deco;
  },
  provide: (field) => EditorView.decorations.from(field),
});

/**
 * Build mark decorations, sorted and clamped.
 *
 * `RangeSetBuilder` requires ascending order and throws otherwise, and a
 * server's tokens can arrive after an edit that shortened the document — so
 * both the sort and the clamp are load-bearing, not defensive habit.
 */
function buildMarks(spans: Span[], docLength: number): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  const sorted = [...spans].sort((a, b) => a.from - b.from || a.to - b.to);
  for (const span of sorted) {
    const from = Math.max(0, Math.min(span.from, docLength));
    const to = Math.max(0, Math.min(span.to, docLength));
    if (to <= from) continue;
    builder.add(from, to, Decoration.mark({ class: span.class }));
  }
  return builder.finish();
}

// --- inlay hints ------------------------------------------------------------

class InlayWidget extends WidgetType {
  constructor(private readonly text: string) {
    super();
  }
  // Without this, CodeMirror redraws every hint on every update and the
  // cursor flickers as the DOM under it is replaced.
  eq(other: InlayWidget) {
    return other.text === this.text;
  }
  toDOM() {
    const span = document.createElement("span");
    span.className = "cm-lsp-inlay";
    span.textContent = this.text;
    // Hints are not text: they must never be selected, copied, or counted
    // as part of the document the user is editing.
    span.setAttribute("aria-hidden", "true");
    return span;
  }
  ignoreEvent() {
    return false;
  }
  reading() { return this.text; }
}

export const setInlayHints = StateEffect.define<{ at: number; text: string }[]>();

export const inlayHintField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const effect of tr.effects) {
      if (!effect.is(setInlayHints)) continue;
      const builder = new RangeSetBuilder<Decoration>();
      const max = tr.state.doc.length;
      const sorted = [...effect.value].sort((a, b) => a.at - b.at);
      for (const hint of sorted) {
        const at = Math.max(0, Math.min(hint.at, max));
        builder.add(at, at, Decoration.widget({ widget: new InlayWidget(hint.text), side: 1 }));
      }
      deco = builder.finish();
    }
    return deco;
  },
  provide: (field) => EditorView.decorations.from(field),
});

/** An inlay hint's label, which is either a string or labelled parts. */
export function inlayText(hint: InlayHint): string {
  if (typeof hint.label === "string") return hint.label;
  return hint.label.map((part) => part.value).join("");
}

// --- turning a workspace edit into editor changes ---------------------------

/**
 * The edits a workspace edit makes to ONE document.
 *
 * A rename can touch several files; the notebook only holds this one, and an
 * edit to a document that is not open cannot be applied here. Returning just
 * this document's edits — and letting the caller report the rest — beats
 * applying half a rename silently.
 */
export function editsForUri(
  edit: WorkspaceEdit | null,
  uri: string,
): { range: LspRange; newText: string }[] {
  if (!edit) return [];
  const fromChanges = Object.entries(edit.changes ?? {}).filter(([target]) => sameDocumentUri(target, uri)).flatMap(([, edits]) => edits);
  const fromDocuments = (edit.documentChanges ?? [])
    .filter((change) => !!change.textDocument?.uri && sameDocumentUri(change.textDocument.uri, uri))
    .flatMap((change) => change.edits ?? []);
  return [...fromChanges, ...fromDocuments];
}

/** Every URI a workspace edit touches, for telling the user what else moved. */
export function urisInEdit(edit: WorkspaceEdit | null): string[] {
  if (!edit) return [];
  const uris = new Set<string>(Object.keys(edit.changes ?? {}));
  for (const change of edit.documentChanges ?? []) {
    if (change.textDocument?.uri) uris.add(change.textDocument.uri);
  }
  return [...uris];
}

// --- the extension bundle ---------------------------------------------------

export interface LspFeatureOptions {
  client: LspClient | null;
  uri: string;
  /** CodeMirror offset → document position, or null for synthetic text. */
  positionAt: (offset: number, view: EditorView) => LspPosition | null;
  /** Document position → CodeMirror offset, or null when out of this view. */
  offsetAt: (position: LspPosition, view: EditorView) => number | null;
  /** Exact writable range, for a projection that can rearrange source fragments. */
  rangeAt?: (range: LspRange, view: EditorView) => {from:number;to:number} | null;
  /**
   * Inlay hints add inline text. Off where line geometry is measured (Split
   * view's ribbons), on in the full-width editor.
   */
  inlayHints?: boolean;
  /** Ask the user for a new name; null cancels. */
  onRename?: (current: string) => Promise<string | null> | string | null;
  /** Offer a list of code actions; the chosen one is applied. */
  onCodeActions?: (actions: CodeAction[]) => Promise<CodeAction | null> | CodeAction | null;
  /** Report something the user needs to know (a refused rename, say). */
  onMessage?: (message: string) => void;
}

const REFRESH_DELAY_MS = 250;

export function lspFeatures(opts: LspFeatureOptions): Extension[] {
  const { client } = opts;
  const fields = [semanticTokenField, highlightField, inlayHintField, signatureField, signatureRenderer, annotationReading(opts.uri)];
  if (!client) return fields;
  const codeActions = codeActionController(opts, applyEdit);

  return [
    ...codeActions.extensions,
    ...fields,
    refreshPlugin(opts),
    signatureHelpTooltip(opts),
    foldFromServer(opts),
    // Bound through the keymap (lib/keymap.ts), read when the editor is
    // built: a rebinding in Settings reaches editors opened after it.
    keymap.of(
      [
        { key: cmKeyOf("editor.rename"), run: (view: EditorView) => renameAt(view, opts) },
        { key: cmKeyOf("editor.codeAction"), run: (view: EditorView) => codeActions.human(view) },
        {
          key: cmKeyOf("editor.format"),
          run: (view: EditorView) => {
            void formatView(view);
            return true;
          },
        },
      ].filter((binding): binding is { key: string; run: (view: EditorView) => boolean } => binding.key !== null),
    ),
    keymap.of(formatOptionFallback(bindingOf("editor.format"), view => {
      void formatView(view);
      return true;
    })),
    formatter.of((view) => formatDocument(view, opts)),
    editorLanguageActions.of({
      id: `${opts.uri}:format`, label: `Format editor source (${opts.uri.slice(0, 200)})`, parameters: {},
      enabled: view => !view.state.readOnly && view.state.facet(EditorView.editable) && (!opts.rangeAt || !!opts.positionAt(view.state.selection.main.head, view)),
      execute: async (view, _args, signal) => {
        const applied = await formatDocument(view, opts, signal);
        return { status: "completed", value: { source_changed: applied, saved: false, verified: false, scope: "Actual shared formatter result; false means no changes applied, not verified formatting" } };
      },
    }),
    ...(opts.onRename ? [editorLanguageActions.of({
      id: `${opts.uri}:rename`, label: `Rename editor symbol (${opts.uri.slice(0, 200)})`, parameters: { name: { type: "string", maxLength: 4096 } },
      enabled: (view: EditorView) => !view.state.readOnly && view.state.facet(EditorView.editable) && !!opts.positionAt(view.state.selection.main.head, view),
      execute: (view, args, signal) => renameSymbol(view, opts, async () => args.name as string, signal),
    })] : []),
    EditorView.theme({
      ".cm-lsp-inlay": {
        opacity: "0.6",
        fontStyle: "italic",
        // Hints must not shift the code around them vertically.
        fontSize: "0.9em",
        padding: "0 0.15em",
      },
      ".cm-lsp-highlight": { background: "var(--lsp-highlight)" },
    }),
  ];
}

/**
 * Re-ask the server after the document settles.
 *
 * Debounced because these are whole-document requests: asking on every
 * keystroke would queue work the next keystroke invalidates, and the tokens
 * that eventually arrived would be for text nobody is looking at any more.
 */
function refreshPlugin(opts: LspFeatureOptions) {
  return ViewPlugin.fromClass(
    class {
      private alive = true;
      private timer: ReturnType<typeof setTimeout> | null = null;
      private cursorTimer: ReturnType<typeof setTimeout> | null = null;

      constructor(private view: EditorView) {
        this.schedule();
      }

      update(update: ViewUpdate) {
        if (update.docChanged) this.schedule();
        else if (update.selectionSet) this.scheduleCursor();
      }

      schedule() {
        if (this.timer) clearTimeout(this.timer);
        this.timer = setTimeout(() => void this.refresh(), REFRESH_DELAY_MS);
      }

      scheduleCursor() {
        if (this.cursorTimer) clearTimeout(this.cursorTimer);
        this.cursorTimer = setTimeout(() => void this.highlight(), REFRESH_DELAY_MS);
      }

      async refresh() {
        await Promise.all([this.colour(), this.hints()]);
      }

      async colour() {
        const { client, uri, offsetAt } = opts;
        if (!client) return;
        const legend = semanticLegend(client);
        if (!legend) return;
        const source = this.view.state.doc;
        const data = await client.semanticTokens(uri).catch(() => null);
        if (!data || !this.alive || this.view.state.doc !== source) return;
        const spans: Span[] = [];
        for (const token of decodeSemanticTokens(data, legend)) {
          const from = offsetAt({ line: token.line, character: token.start }, this.view);
          const to = offsetAt(
            { line: token.line, character: token.start + token.length },
            this.view,
          );
          if (from === null || to === null) continue;
          spans.push({ from, to, class: tokenClass(token) });
        }
        this.view.dispatch({ effects: setSemanticTokens.of(spans) });
      }

      async hints() {
        const { client, uri, offsetAt, inlayHints } = opts;
        if (!client || !inlayHints) return;
        const doc = this.view.state.doc;
        const range: LspRange = {
          start: { line: 0, character: 0 },
          end: { line: Math.max(0, doc.lines - 1), character: 0 },
        };
        const hints = await client.inlayHints(uri, range).catch(() => []);
        if (!this.alive || this.view.state.doc !== doc) return;
        const placed: { at: number; text: string }[] = [];
        for (const hint of hints) {
          const at = offsetAt(hint.position, this.view);
          if (at === null) continue;
          placed.push({ at, text: inlayText(hint) });
        }
        this.view.dispatch({ effects: setInlayHints.of(placed) });
      }

      async highlight() {
        const { client, uri, positionAt, offsetAt } = opts;
        if (!client) return;
        const original = this.view.state;
        const head = original.selection.main.head;
        const position = positionAt(head, this.view);
        if (!position) {
          this.view.dispatch({ effects: setHighlights.of([]) });
          return;
        }
        const ranges = await client.documentHighlight(uri, position).catch(() => []);
        if (!this.alive || this.view.state.doc !== original.doc || !this.view.state.selection.eq(original.selection)) return;
        const spans: Span[] = [];
        for (const range of ranges) {
          const from = offsetAt(range.start, this.view);
          const to = offsetAt(range.end, this.view);
          if (from === null || to === null) continue;
          spans.push({ from, to, class: "cm-lsp-highlight" });
        }
        this.view.dispatch({ effects: setHighlights.of(spans) });
      }

      destroy() {
        this.alive = false;
        if (this.timer) clearTimeout(this.timer);
        if (this.cursorTimer) clearTimeout(this.cursorTimer);
      }
    },
  );
}

// --- signature help ---------------------------------------------------------

/**
 * The signature of the call the cursor is inside, shown while typing
 * arguments.
 *
 * Triggered by the characters that open and separate an argument list rather
 * than by every keystroke: those are the moments the answer changes, and
 * asking otherwise is a request per character for an unchanged reply.
 */
export const setSignatureHelp = StateEffect.define<{ label: string; source: Text; at: number } | null>();
export const signatureField = StateField.define<{ label: string; source: Text; at: number } | null>({
  create: () => null,
  update(value, transaction) {
    if (transaction.docChanged || transaction.selection) value = null;
    for (const effect of transaction.effects) if (effect.is(setSignatureHelp)) value = effect.value && effect.value.source === transaction.state.doc ? effect.value : null;
    return value;
  },
});

// Preserve the existing bottom-pinned human signature surface. Its renderer
// owns its node; model readings use the field, never inspect this DOM.
const signatureRenderer = ViewPlugin.fromClass(class {
  private element: HTMLElement | null = null;
  constructor(private view: EditorView) { this.render(); }
  update() { this.render(); }
  private render() {
    const value = this.view.state.field(signatureField, false);
    if (!value?.label) { this.element?.remove(); this.element = null; return; }
    if (!this.element) { this.element = document.createElement("div"); this.element.className = "cm-lsp-signature"; this.view.dom.appendChild(this.element); }
    this.element.textContent = value.label;
  }
  destroy() { this.element?.remove(); }
});


function signatureHelpTooltip(opts: LspFeatureOptions): Extension {
  return ViewPlugin.fromClass(class {
    private alive = true;
    update(update: ViewUpdate) {
      if (!update.docChanged) return;
      const { client, uri, positionAt } = opts;
      if (!client) return;
      let triggered = false;
      update.changes.iterChanges((_fromA, _toA, _fromB, _toB, inserted) => {
        if (["(", ",", ")"].includes(inserted.toString())) triggered = true;
      });
      if (!triggered) return;
      const view = update.view, original = view.state;
      const position = positionAt(original.selection.main.head, view);
      if (!position) return;
      void client.signatureHelp(uri, position).catch(() => null).then(help => {
        if (!this.alive || !view.dom.isConnected || view.state.doc !== original.doc || !view.state.selection.eq(original.selection) || view.state.field(signatureField, false) === undefined) return;
        const label = help ? signatureLabel(help) : "";
        view.dispatch({ effects: setSignatureHelp.of(label ? { label, source: original.doc, at: original.selection.main.head } : null) });
      });
    }
    destroy() { this.alive = false; }
  });
}

function annotationReading(uri: string): Extension {
  return editorLanguageActions.of({
    id: `${uri}:annotations`, label: `Read editor language annotations (${uri.slice(0, 200)})`, enabled: () => true,
    parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } },
    execute: async (view, args) => {
      const marks = (field: typeof semanticTokenField) => {
        const spans: { from: number; to: number; class: string | null; text: string | null }[] = [];
        view.state.field(field, false)?.between(0, view.state.doc.length, (from, to, decoration) => { spans.push({ from, to, class: decoration.spec.class ?? null, text: decoration.spec.widget instanceof InlayWidget ? decoration.spec.widget.reading() : null }); });
        return spans;
      };
      const signature = view.state.field(signatureField, false);
      const bytes = new TextEncoder().encode(JSON.stringify({ semantic_marks: marks(semanticTokenField), highlights: marks(highlightField), inlay_hints: marks(inlayHintField), signature: signature && { label: signature.label, at: signature.at }, coordinate_unit: "UTF-16 code units", scope: "Actual rendered annotation state; positions may be mapped across edits until the provider refreshes. Not independent type or execution verification." }));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside current annotation reading" };
      return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null, encoding: "JSON UTF-8" } };
    },
  });
}

/** The active signature's label, with the active parameter marked in text. */
export function signatureLabel(help: {
  signatures: { label: string; activeParameter?: number }[];
  activeSignature?: number;
}): string {
  const signature = help.signatures?.[help.activeSignature ?? 0];
  return signature?.label ?? "";
}

// --- folding ----------------------------------------------------------------

/**
 * Fold where the server says a construct is, not where the indentation
 * suggests it might be.
 *
 * CodeMirror asks for one line at a time and wants an answer synchronously,
 * so the ranges are fetched in the background and answered from the last set
 * received. A fold that appears a moment after the document settles is
 * better than one computed from a grammar the document does not have.
 */
function foldFromServer(opts: LspFeatureOptions): Extension {
  let ranges: FoldSpan[] = [];
  let generation = 0;
  const fetch = (view: EditorView) => {
    const { client, uri, offsetAt } = opts;
    if (!client) return;
    const source = view.state.doc, requested = ++generation;
    void client
      .foldingRanges(uri)
      .then((found) => {
        if (requested !== generation || view.state.doc !== source) return;
        ranges = found
          .map((range) => {
            // Fold from the END of the first line: the header stays visible,
            // which is the whole point of folding rather than hiding.
            const from = offsetAt({ line: range.startLine, character: Number.MAX_SAFE_INTEGER }, view);
            const to = offsetAt(
              { line: range.endLine, character: range.endCharacter ?? Number.MAX_SAFE_INTEGER },
              view,
            );
            return from !== null && to !== null && to > from ? { from, to } : null;
          })
          .filter((span): span is FoldSpan => span !== null);
      })
      .catch(() => {
        if (requested === generation && view.state.doc === source) ranges = [];
      });
  };

  return [
    ViewPlugin.fromClass(
      class {
        private timer: ReturnType<typeof setTimeout> | null = null;
        constructor(private view: EditorView) {
          fetch(view);
        }
        update(update: ViewUpdate) {
          if (!update.docChanged) return;
          generation++; ranges = [];
          if (this.timer) clearTimeout(this.timer);
          this.timer = setTimeout(() => fetch(this.view), REFRESH_DELAY_MS);
        }
        destroy() {
          generation++; ranges = [];
          if (this.timer) clearTimeout(this.timer);
        }
      },
    ),
    foldServiceFor(() => ranges),
  ];
}

interface FoldSpan {
  from: number;
  to: number;
}

/** CodeMirror's fold service, answered from whatever the server last sent. */
function foldServiceFor(current: () => FoldSpan[]): Extension {
  return foldService.of((state, lineStart, lineEnd) => {
    for (const span of current()) {
      if (span.from >= lineStart && span.from <= lineEnd && span.to <= state.doc.length) {
        return { from: span.from, to: span.to };
      }
    }
    return null;
  });
}

/** The legend the bridge announced, or null if the server cannot colour. */
export function semanticLegend(client: LspClient): SemanticLegend | null {
  const legend = client.serverCapabilities?.semanticTokensProvider?.legend;
  if (!legend?.tokenTypes) return null;
  return {
    tokenTypes: legend.tokenTypes,
    tokenModifiers: legend.tokenModifiers ?? [],
  };
}

function renameAt(view: EditorView, opts: LspFeatureOptions): boolean {
  if (!opts.client || !opts.onRename || view.state.readOnly || !view.state.facet(EditorView.editable) || !opts.positionAt(view.state.selection.main.head, view)) return false;
  void renameSymbol(view, opts, opts.onRename);
  return true;
}

export async function renameSymbol(view: EditorView, opts: LspFeatureOptions, askName: NonNullable<LspFeatureOptions["onRename"]>, signal?: AbortSignal): Promise<UiOutcome> {
  const original = view.state.doc;
  const { client, uri, positionAt, offsetAt, onMessage } = opts;
  const current = () => view.state.doc === original && !view.state.readOnly && view.state.facet(EditorView.editable) && !signal?.aborted;
  const refuse = (reason: string): UiOutcome => { onMessage?.(reason); return { status: "unavailable", reason }; };
  const position = positionAt(view.state.selection.main.head, view);
  if (!client || !current() || !position) return refuse("Rename needs a writable mapped source position.");
  try {
    const range = await client.prepareRename(uri, position);
    if (!current()) return refuse("The buffer changed. Request rename again.");
    if (!range) return refuse("Nothing renameable at the cursor.");
    const from = offsetAt(range.start, view), to = offsetAt(range.end, view);
    if (from === null || to === null || !Number.isInteger(from) || !Number.isInteger(to) || from < 0 || to < from || to > original.length || (opts.rangeAt && !opts.rangeAt(range, view))) return refuse("Rename crosses reading fragments. Rename in the source file instead.");
    const previous = original.sliceString(from, to);
    const next = await askName(previous);
    if (!current()) return refuse("The buffer changed. Request rename again.");
    if (!next || next === previous) return { status: "completed", value: { source_changed: false, cancelled_or_unchanged: true } };
    const edit = await client.rename(uri, position, next);
    if (!current()) return refuse("The buffer changed. Request rename again.");
    const changed = applyEdit(view, edit, opts);
    return { status: "completed", value: { source_changed: changed, other_documents: urisInEdit(edit).filter(other => !sameDocumentUri(other, uri)), scope: "Current editor only; other document edits are reported, not applied", saved: false, verified: false } };
  } catch (e) { return refuse(e instanceof Error ? e.message : String(e)); }
}

// --- formatting -------------------------------------------------------------

/**
 * How to format the buffer this extension is in, for a caller outside the
 * editor — the workspace's Save, when format-on-save is on. A facet rather
 * than an exported function because the caller has the view and not the
 * options the view was built with.
 */
export const formatter = Facet.define<(view: EditorView) => Promise<boolean>>();

/** Format `view` through whatever formatter its extensions provide. Resolves
 * false when there is none, or the server had nothing to say. */
export async function formatView(view: EditorView): Promise<boolean> {
  for (const format of view.state.facet(formatter)) if (await format(view)) return true;
  return false;
}

/**
 * Ask the server for formatting and validate the entire original-coordinate
 * batch before one native editor transaction. Mark the real input event so
 * existing editor save/provenance policies process the result.
 */
export async function formatDocument(view: EditorView, opts: LspFeatureOptions, signal?: AbortSignal): Promise<boolean> {
  const { client, uri, onMessage } = opts;
  if (!client || view.state.readOnly || !view.state.facet(EditorView.editable) || signal?.aborted || (opts.rangeAt && !opts.positionAt(view.state.selection.main.head, view))) return false;
  const original = view.state.doc;
  const edits = await client.formatting(uri).catch(() => []);
  if (view.state.doc !== original || view.state.readOnly || !view.state.facet(EditorView.editable) || signal?.aborted) { onMessage?.("The buffer changed or is read-only. Request formatting again."); return false; }
  if (opts.rangeAt && edits.some(edit=>!opts.rangeAt!(edit.range,view))) { onMessage?.("Formatting crosses reading fragments. Format the source file instead."); return false; }
  if (edits.length === 0) {
    onMessage?.("Nothing to format — either the file is already formatted or no formatter answers for it.");
    return false;
  }
  return applyLanguageBatch(view, edits, opts, "input.format");
}

/** Validate all mapped edits before dispatching one source transaction. */
function applyLanguageBatch(view: EditorView, edits: { range: LspRange; newText: string }[], opts: LspFeatureOptions, event: string): boolean {
  if (view.state.readOnly || !view.state.facet(EditorView.editable)) { opts.onMessage?.("The editor is read-only."); return false; }
  const changes: { from: number; to: number; insert: string }[] = [];
  for (const edit of edits) {
    const mapped = opts.rangeAt?.(edit.range, view);
    const from = mapped?.from ?? opts.offsetAt(edit.range.start, view), to = mapped?.to ?? opts.offsetAt(edit.range.end, view);
    if ((opts.rangeAt && !mapped) || from === null || to === null) { opts.onMessage?.("The edit crosses reading fragments. Apply it in the source file instead."); return false; }
    changes.push({ from, to, insert: edit.newText });
  }
  changes.sort((a, b) => a.from - b.from || a.to - b.to);
  const source = view.state.doc.toString(), prepared = prepareEditorChanges(source, changes);
  if ("reason" in prepared) { opts.onMessage?.(prepared.reason); return false; }
  if (prepared.expected === source) return false;
  view.dispatch({ changes, userEvent: event });
  return view.state.doc.toString() !== source;
}

function applyEdit(view: EditorView, edit: WorkspaceEdit | null, opts: LspFeatureOptions): boolean {
  const edits = editsForUri(edit, opts.uri);
  if (!edits.length) { opts.onMessage?.("The server made no changes."); return false; }
  const changed = applyLanguageBatch(view, edits, opts, "input.lsp");
  const elsewhere = urisInEdit(edit).filter(other => !sameDocumentUri(other, opts.uri));
  if (elsewhere.length) opts.onMessage?.(`Also changes ${elsewhere.length} other document${elsewhere.length === 1 ? "" : "s"}: ${elsewhere.join(", ")}`);
  return changed;
}
</book:entry>

</book:doc>
