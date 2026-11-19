import { editorLanguageActions } from "../editor/languageActions";
import type { UiOutcome } from "../lib/uiActions";
import { cmKeyOf } from "../lib/keymap";
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
import { renderHoverMarkdown } from "./hoverMarkdown";
import type { DecorationSet, Tooltip } from "@codemirror/view";
import type { LspClient, LspDiagnostic, LspLocation } from "./client";
import type { LspPosition } from "./positions";
import { sameDocumentUri } from "./documentUri";
import { querySemanticLocations, semanticLocations, semanticQueries, type SemanticLocation, type SemanticQuery } from "../simple/Intelligence";

export interface LspNavigationTarget {
  uri: string;
  range: { start: LspPosition; end: LspPosition };
}

export interface RelatedLocationReading {
  title: string;
  locations: SemanticLocation[];
  originUri: string;
  current: () => boolean;
}

export type LocalSourceNavigation = (view: EditorView, departure: number) => void;

const relatedQueries = semanticQueries.filter(({kind}) =>
  !["definition", "references", "outline", "symbols"].includes(kind));

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
  /** Resolve a local construction byte into its selected complete endpoint. */
  targetAt?: (offset: number, view: EditorView) => { uri: string; position: LspPosition } | null;
  /** Open a definition/reference target. */
  onNavigate: (target: LspNavigationTarget, current?: () => boolean, view?: EditorView) => void;
  navigationIntent?: true | ((view: EditorView) => () => boolean);
  /** The host can retire navigation when the workspace or active tab changes. */
  captureNavigationContext?: () => () => boolean;
  /** Show a reference list (empty array means "none found"). */
  onReferences?: (locations: LspLocation[], from: LspNavigationTarget, current: () => boolean) => void;
  /** Only install Structure on editors whose text is in the request URI's coordinates. */
  chooseOutline?: (options: {label: string; value: number}[], current: () => boolean) => Promise<number | null>;
  onLocalNavigation?: LocalSourceNavigation;
  onRelatedLocations?: (reading: RelatedLocationReading) => void;
  /**
   * What this identifier currently HOLDS, when a debugger is paused.
   *
   * Resolves to null when nothing is paused, which is the usual case. The
   * value goes above the type in the tooltip: pointing at a symbol while
   * stopped is usually a question about its value, and the type is the
   * context for it.
   */
  runtimeValue?: (identifier: string) => Promise<string | null>;
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
              // Information and hints are marked too — a refused tool, a
              // command that exited 1 — but quietly: they are findable,
              // not counted (lib/problems.ts).
              class:
                d.severity === 1
                  ? "cm-lsp-error"
                  : d.severity === 2
                    ? "cm-lsp-warn"
                    : "cm-lsp-info",
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

export function hoverText(contents: unknown): string {
  if (contents == null) return "";
  if (typeof contents === "string") return contents;
  if (Array.isArray(contents)) return contents.map(hoverText).filter(Boolean).join("\n\n");
  const o = contents as { value?: unknown; language?: unknown };
  if (typeof o.value === "string") return o.value;
  return "";
}

// --- the extension bundle ---------------------------------------------------

/** The identifier under an offset, for asking a debugger about it. */
export function identifierAt(text: string, offset: number): string | null {
  const isWord = (ch: string) => /[A-Za-z0-9_$.]/.test(ch);
  if (offset >= text.length || !isWord(text[offset] ?? "")) return null;
  let start = offset;
  while (start > 0 && isWord(text[start - 1])) start--;
  let end = offset;
  while (end < text.length && isWord(text[end])) end++;
  const word = text.slice(start, end);
  // A bare number is not an identifier, and asking a debugger to evaluate
  // `42` wastes a round trip to be told it is 42.
  return /^[A-Za-z_$]/.test(word) ? word : null;
}

export function lspSupport(opts: CmLspOptions): Extension[] {
  const { client, uri, positionAt, targetAt, onNavigate, onReferences, runtimeValue } = opts;
  let navigationRevision = 0;
  const target = (offset: number, view: EditorView) => targetAt ? targetAt(offset, view) : (() => { const position = positionAt(offset, view); return position ? {uri,position} : null; })();
  if (!client) return [lspDiagnosticField];
  const outlineKey = opts.chooseOutline ? cmKeyOf("editor.structure") : null;

  const captureIntent = (view: EditorView, signal?: AbortSignal) => {
    const original = view.state.doc, revision = ++navigationRevision;
    const intent = opts.navigationIntent === true
      ? () => navigationRevision === revision
      : opts.navigationIntent?.(view);
    const contextCurrent = opts.captureNavigationContext?.();
    return {
      original,
      current: () => !signal?.aborted && view.state.doc === original && view.dom.isConnected
        && (!intent || intent()) && (!contextCurrent || contextCurrent()),
    };
  };
  const requestAt = async (
    offset: number,
    view: EditorView,
    what: "definition" | "references",
    signal?: AbortSignal,
  ): Promise<UiOutcome> => {
    const location = target(offset, view);
    if (!location) return { status: "unavailable", reason: "Position has no mapped language source" };
    const {uri,position} = location;
    const from: LspNavigationTarget = { uri, range: { start: position, end: position } };
    const {current} = captureIntent(view, signal);
    try {
      const locations = await (what === "definition" ? client.definition(uri, position) : client.references(uri, position));
      if (!current()) return { status: "unavailable", reason: "Navigation source changed, editor closed or request cancelled" };
      if (what === "definition" && locations.length) {
        if (opts.navigationIntent) onNavigate(locations[0], current, view);
        else onNavigate(locations[0]);
      }
      if (what === "references") onReferences?.(locations, from, current);
      return { status: "completed", value: { kind: what, total: locations.length, navigation_requested: what === "definition" && locations.length > 0, references_requested: what === "references" && !!onReferences, scope: "Actual language-provider response and shared navigation callback; navigation is not an execution witness" } };
    } catch (e) { return { status: "unavailable", reason: e instanceof Error ? e.message : String(e) }; }
  };
  const requestOutline = async (view: EditorView, signal?: AbortSignal): Promise<UiOutcome> => {
    const {chooseOutline} = opts;
    if (!chooseOutline) return {status: "unavailable", reason: "File Structure is unavailable for this editor"};
    const {original, current} = captureIntent(view, signal);
    try {
      const symbols = semanticLocations(await client.documentSymbols(uri), uri, true);
      if (!current()) return {status: "unavailable", reason: "The source or foreground editor changed"};
      const options = symbols.flatMap(({name, uri: targetUri, range: {start: {line, character}}}) => {
        if (!name || !sameDocumentUri(uri, targetUri) || line >= original.lines) return [];
        const row = original.line(line + 1);
        if (character > row.length || /[\uD800-\uDBFF][\uDC00-\uDFFF]/.test(row.text.slice(character - 1, character + 1))) return [];
        return [{label: `${name}  ${line + 1}:${character + 1}`, value: row.from + character}];
      });
      void chooseOutline(options, current).then(offset => {
        if (offset === null || !current()) return;
        const departure = view.state.selection.main.head;
        view.dispatch({selection: {anchor: offset}, effects: EditorView.scrollIntoView(offset, {y: "center"})});
        if (view.state.doc === original && view.state.selection.main.head === offset) {
          opts.onLocalNavigation?.(view, departure);
        }
        view.focus();
      });
      return {status: "completed", value: {total: options.length, structure_requested: true}};
    } catch (error) {
      return {status: "unavailable", reason: error instanceof Error ? error.message : String(error)};
    }
  };
  const requestRelated = async (view: EditorView, kind: SemanticQuery, title: string,
    signal?: AbortSignal, paletteCurrent?: () => boolean): Promise<UiOutcome> => {
    const subject = target(view.state.selection.main.head, view);
    if (!subject) return {status: "unavailable", reason: "Position has no mapped language source"};
    if (paletteCurrent && !paletteCurrent()) return {status: "unavailable", reason: "A newer command superseded this request"};
    const captured = captureIntent(view, signal);
    const current = () => captured.current() && (!paletteCurrent || paletteCurrent());
    try {
      const locations = await querySemanticLocations(client, subject.uri, subject.position, kind, undefined, current);
      if (!current()) return {status: "unavailable", reason: "The source or foreground editor changed"};
      opts.onRelatedLocations?.({title, locations, originUri: subject.uri, current});
      return {status: "completed", value: {total: locations.length, locations_requested: true}};
    } catch (error) {
      return {status: "unavailable", reason: error instanceof Error ? error.message : String(error)};
    }
  };
  let hoverSnapshot: { source: Text; id: string; bytes: Uint8Array } | undefined;
  const hoverAt = async (view: EditorView, pos: number) => {
    const original = view.state.doc;
    const problems = diagnosticsAt(view.state, pos);
    const location = target(pos, view);
    const identifier = identifierAt(original.toString(), pos);
    const [result, value] = await Promise.all([
      location ? client.hover(location.uri, location.position).catch(() => null) : Promise.resolve(null),
      identifier && runtimeValue ? runtimeValue(identifier).catch(() => null) : Promise.resolve(null),
    ]);
    if (view.state.doc !== original || !view.dom.isConnected) return null;
    const found = { problems, identifier, value, text: hoverText(result?.contents) };
    hoverSnapshot = { source: original, id: crypto.randomUUID(), bytes: new TextEncoder().encode(JSON.stringify(found)) };
    return { ...found, snapshot: hoverSnapshot };
  };

  return [
    lspDiagnosticField,
    ...(opts.onRelatedLocations ? relatedQueries.map(({kind, label}) => editorLanguageActions.of({
      id: `${uri}:related:${kind}`, label: `${label} at editor cursor (${uri.slice(0, 200)})`, parameters: {},
      enabled: view => !!target(view.state.selection.main.head, view),
      execute: (view, _args, signal) => requestRelated(view, kind, label, signal),
      command: {id: kind, label, run: (view, current) => { void requestRelated(view, kind, label, undefined, current); }},
    })) : []),
    ...(opts.chooseOutline ? [editorLanguageActions.of({
      id: `${uri}:outline`, label: `File structure (${uri.slice(0, 200)})`, parameters: {},
      enabled: () => true, execute: (view, _args, signal) => requestOutline(view, signal),
    })] : []),
    ...(["definition", "references"] as const).map(what => editorLanguageActions.of({
      id: `${uri}:${what}`, label: `${what === "definition" ? "Go to definition" : "Find references"} at editor cursor (${uri.slice(0, 200)})`, parameters: {},
      enabled: view => !!target(view.state.selection.main.head, view),
      execute: (view, _args, signal) => requestAt(view.state.selection.main.head, view, what, signal),
    })),
    editorLanguageActions.of({
      id: `${uri}:hover`, label: `Inspect editor hover (${uri.slice(0, 200)})`,
      parameters: { position: { type: "number", minimum: 0, maximum: 100000000, integer: true } },
      enabled: () => true,
      execute: async (view, args, signal) => {
        const pos = args.position as number;
        if (pos > view.state.doc.length) return { status: "unavailable", reason: "Hover position outside current editor source" };
        const found = await hoverAt(view, pos);
        if (!found || signal.aborted || found.snapshot.source !== view.state.doc) return { status: "unavailable", reason: "Hover source changed, editor closed or request cancelled" };
        return { status: "completed", value: { snapshot: found.snapshot.id, total_bytes: found.snapshot.bytes.length, scope: "Read captured diagnostic, language hover and debugger identifier value through Read editor hover snapshot bytes" } };
      },
    }),
    editorLanguageActions.of({
      id: `${uri}:hover-read`, label: `Read editor hover snapshot bytes (${uri.slice(0, 200)})`,
      parameters: { snapshot: { type: "string", maxLength: 128 }, offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } },
      revision: () => hoverSnapshot?.id,
      enabled: view => !!hoverSnapshot && hoverSnapshot.source === view.state.doc,
      execute: async (view, args) => {
        const snapshot = hoverSnapshot;
        if (!snapshot || snapshot.id !== args.snapshot || snapshot.source !== view.state.doc) return { status: "unavailable", reason: "Hover snapshot changed; inspect the current hover again" };
        const { bytes } = snapshot;
        const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
        if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside captured hover response" };
        return { status: "completed", value: { snapshot: snapshot.id, bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null, scope: "Immutable human-handler hover capture; runtime value is captured evidence, not a current execution verdict" } };
      },
    }),
    hoverTooltip(async (view, pos): Promise<Tooltip | null> => {
      const found = await hoverAt(view, pos);
      if (!found) return null;
      const { problems, identifier, value, text } = found;
      if (problems.length === 0 && !text.trim() && !value) return null;

      return {
        pos,
        above: true,
        create: () => {
          const dom = document.createElement("div");
          dom.className = "cm-lsp-hover";
          for (const problem of problems) {
            const line = document.createElement("div");
            line.className =
              problem.severity === 1
                ? "cm-lsp-hover-error"
                : problem.severity === 2
                  ? "cm-lsp-hover-warn"
                  : "cm-lsp-hover-note";
            line.textContent = problem.source
              ? `${problem.message}  (${problem.source})`
              : problem.message;
            dom.appendChild(line);
          }
          if (value && identifier) {
            // The runtime value, above the type: while stopped, "what does
            // this hold" is the question being asked.
            const runtime = document.createElement("div");
            runtime.className = "cm-lsp-hover-value";
            runtime.textContent = `${identifier} = ${value}`;
            dom.appendChild(runtime);
          }
          if (text.trim()) {
            const info = document.createElement("div");
            // Separated from the problem above it, so two different kinds of
            // statement do not read as one paragraph.
            info.className = problems.length > 0 || value ? "cm-lsp-hover-info" : "";
            // Rendered, not dumped: a server answers in Markdown, and setting
            // it as text showed the backticks, the word `rust`, and a row of
            // hyphens. Never `innerHTML` — the string is another program's.
            info.appendChild(renderHoverMarkdown(text));
            dom.appendChild(info);
          }
          return { dom };
        },
      };
    }),
    ...(opts.navigationIntent === true ? [EditorView.updateListener.of(update => {
      if (update.docChanged || update.selectionSet) navigationRevision++;
    })] : []),
    keymap.of([
      ...(outlineKey ? [{key: outlineKey, run: (view: EditorView) => { void requestOutline(view); return true; }}] : []),
      ...([ ["editor.definition", "definition"], ["editor.references", "references"] ] as const).flatMap(([action,what]) => {
        const key = cmKeyOf(action);
        return key ? [{key,run:(view:EditorView)=>{void requestAt(view.state.selection.main.head,view,what);return true;}}] : [];
      }),
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
