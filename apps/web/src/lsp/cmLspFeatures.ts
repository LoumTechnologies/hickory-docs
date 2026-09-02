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

import { foldService } from "@codemirror/language";
import { Facet, RangeSetBuilder, StateEffect, StateField } from "@codemirror/state";
import type { Extension } from "@codemirror/state";
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
  const fromChanges = edit.changes?.[uri] ?? [];
  const fromDocuments = (edit.documentChanges ?? [])
    .filter((change) => change.textDocument?.uri === uri)
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
  const fields = [semanticTokenField, highlightField, inlayHintField];
  if (!client) return fields;

  return [
    ...fields,
    refreshPlugin(opts),
    signatureHelpTooltip(opts),
    foldFromServer(opts),
    keymap.of([
      { key: "F2", run: (view) => renameAt(view, opts) },
      { key: "Mod-.", run: (view) => codeActionAt(view, opts) },
      {
        key: "Shift-Alt-f",
        run: (view) => {
          void formatDocument(view, opts);
          return true;
        },
      },
    ]),
    formatter.of((view) => formatDocument(view, opts)),
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
        const data = await client.semanticTokens(uri).catch(() => null);
        if (!data) return;
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
        const head = this.view.state.selection.main.head;
        const position = positionAt(head, this.view);
        if (!position) {
          this.view.dispatch({ effects: setHighlights.of([]) });
          return;
        }
        const ranges = await client.documentHighlight(uri, position).catch(() => []);
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
function signatureHelpTooltip(opts: LspFeatureOptions): Extension {
  return EditorView.updateListener.of((update: ViewUpdate) => {
    if (!update.docChanged) return;
    const { client, uri, positionAt } = opts;
    if (!client) return;
    let triggered = false;
    update.changes.iterChanges((_fromA, _toA, _fromB, _toB, inserted) => {
      const text = inserted.toString();
      if (text === "(" || text === "," || text === ")") triggered = true;
    });
    if (!triggered) return;
    const view = update.view;
    const position = positionAt(view.state.selection.main.head, view);
    if (!position) return;
    void client
      .signatureHelp(uri, position)
      .then((help) => showSignature(view, help ? signatureLabel(help) : ""))
      .catch(() => showSignature(view, ""));
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

function showSignature(view: EditorView, label: string) {
  const existing = view.dom.querySelector(".cm-lsp-signature");
  if (!label) {
    existing?.remove();
    return;
  }
  const element = (existing as HTMLElement | null) ?? document.createElement("div");
  element.className = "cm-lsp-signature";
  element.textContent = label;
  if (!existing) view.dom.appendChild(element);
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
  const fetch = (view: EditorView) => {
    const { client, uri, offsetAt } = opts;
    if (!client) return;
    void client
      .foldingRanges(uri)
      .then((found) => {
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
        ranges = [];
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
          if (this.timer) clearTimeout(this.timer);
          this.timer = setTimeout(() => fetch(this.view), REFRESH_DELAY_MS);
        }
        destroy() {
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
  void (async () => {
    const { client, uri, positionAt, offsetAt, onRename, onMessage } = opts;
    if (!client || !onRename) return;
    const head = view.state.selection.main.head;
    const position = positionAt(head, view);
    if (!position) return;

    // prepareRename first: a server that says "not here" is saying the
    // symbol is not renameable, and asking the user for a new name before
    // finding that out wastes their answer.
    const range = await client.prepareRename(uri, position).catch(() => null);
    if (!range) {
      onMessage?.("Nothing renameable at the cursor.");
      return;
    }
    const from = offsetAt(range.start, view);
    const to = offsetAt(range.end, view);
    const current = from !== null && to !== null ? view.state.doc.sliceString(from, to) : "";
    const next = await onRename(current);
    if (!next || next === current) return;

    const edit = await client.rename(uri, position, next).catch(() => null);
    applyEdit(view, edit, opts);
  })();
  return true;
}

function codeActionAt(view: EditorView, opts: LspFeatureOptions): boolean {
  void (async () => {
    const { client, uri, positionAt, onCodeActions, onMessage } = opts;
    if (!client || !onCodeActions) return;
    const selection = view.state.selection.main;
    const start = positionAt(selection.from, view);
    const end = positionAt(selection.to, view) ?? start;
    if (!start || !end) return;
    const actions = await client.codeActions(uri, { start, end }).catch(() => []);
    if (actions.length === 0) {
      onMessage?.("No code actions here.");
      return;
    }
    const chosen = await onCodeActions(actions);
    if (chosen?.edit) applyEdit(view, chosen.edit, opts);
    else if (chosen?.command) onMessage?.(`"${chosen.title}" needs a command this editor cannot run yet.`);
  })();
  return true;
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
export function formatView(view: EditorView): Promise<boolean> {
  const format = view.state.facet(formatter)[0];
  return format ? format(view) : Promise.resolve(false);
}

/**
 * Ask the server for the document's formatting and apply it, back to front.
 *
 * Back to front for the reason `applyEdit` gives below. Marked as a user
 * event so a pane that saves only what a person did saves this too: the
 * person asked for it.
 */
export async function formatDocument(view: EditorView, opts: LspFeatureOptions): Promise<boolean> {
  const { client, uri, offsetAt, onMessage } = opts;
  if (!client) return false;
  const edits = await client.formatting(uri).catch(() => []);
  if (edits.length === 0) {
    onMessage?.("Nothing to format — either the file is already formatted or no formatter answers for it.");
    return false;
  }
  const changes = edits
    .map((edit) => {
      const from = offsetAt(edit.range.start, view);
      const to = offsetAt(edit.range.end, view);
      if (from === null || to === null) return null;
      return { from, to, insert: edit.newText };
    })
    .filter((change): change is { from: number; to: number; insert: string } => change !== null)
    .sort((a, b) => b.from - a.from);
  if (changes.length === 0) return false;
  view.dispatch({ changes, userEvent: "input.format" });
  return true;
}

/**
 * Apply a workspace edit to this document, back to front.
 *
 * Back to front because every edit's range is in the coordinates of the
 * document as the server saw it: applying the first one shifts everything
 * after it, and a forwards pass writes each subsequent edit at the wrong
 * offset — which on a rename means corrupting the identifier it was
 * supposed to change.
 */
function applyEdit(view: EditorView, edit: WorkspaceEdit | null, opts: LspFeatureOptions) {
  const { uri, offsetAt, onMessage } = opts;
  const edits = editsForUri(edit, uri);
  if (edits.length === 0) {
    onMessage?.("The server made no changes.");
    return;
  }
  const changes = edits
    .map((item) => {
      const from = offsetAt(item.range.start, view);
      const to = offsetAt(item.range.end, view);
      if (from === null || to === null) return null;
      return { from, to, insert: item.newText };
    })
    .filter((change): change is { from: number; to: number; insert: string } => change !== null)
    .sort((a, b) => b.from - a.from);
  if (changes.length > 0) view.dispatch({ changes });

  const elsewhere = urisInEdit(edit).filter((other) => other !== uri);
  if (elsewhere.length > 0) {
    onMessage?.(
      `Also changes ${elsewhere.length} other document${elsewhere.length === 1 ? "" : "s"}: ${elsewhere.join(", ")}`,
    );
  }
}
