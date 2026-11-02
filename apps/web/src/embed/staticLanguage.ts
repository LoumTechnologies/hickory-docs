import type { Extension } from "@codemirror/state";
import { EditorView, ViewPlugin, Decoration } from "@codemirror/view";
import { foldService } from "@codemirror/language";
import { LspClient } from "../lsp/client";
import type { LspLocation, LspDiagnostic, ServerCapabilities, InlayHint, FoldingRange, DocumentSymbol, SignatureHelp, HoverResult } from "../lsp/client";
import type { JsonRpcMessage, LspChannel } from "../lsp/channel";
import { lspSupport, setLspDiagnostics, diagnosticRanges } from "../lsp/cmLsp";
import { semanticTokenField, setSemanticTokens, inlayHintField, setInlayHints, inlayText, highlightField, setHighlights } from "../lsp/cmLspFeatures";
import { decodeSemanticTokens, tokenClass } from "../lsp/semanticTokens";
import { utf16ToPosition } from "../lsp/positions";
import type { LspPosition, LspRange } from "../lsp/positions";
import { sha256 } from "../lib/reviews";

function sourceCoordinates(source: string) {
  const starts = [0], ends: number[] = [], removed: number[] = [];
  for (const match of source.matchAll(/\r\n?|\n/g)) {
    ends.push(match.index); starts.push(match.index + match[0].length);
    if (match[0].length === 2) removed.push(match.index);
  }
  ends.push(source.length);
  const view = (offset: number) => {
    let left = 0, right = removed.length;
    while (left < right) { const middle = (left + right) >>> 1; if (removed[middle] < offset) left = middle + 1; else right = middle; }
    return offset-left;
  };
  const raw = (position: LspPosition) => !Number.isInteger(position.line) || !Number.isInteger(position.character) || position.line < 0 || position.character < 0 || position.line >= starts.length
    ? source.length : Math.min(starts[position.line] + position.character, ends[position.line]);
  return { raw, view, normalized: source.replace(/\r\n?/g, "\n") };
}

type Answer = { state: "captured"; result: unknown } | { state: "unavailable"; code?: number };
interface Occurrence { from: number; to: number; position: LspPosition; [method: string]: unknown }
export interface StaticLanguage {
  version: 1; source_hash: string; state: "captured" | "unavailable";
  reason?: string | null;
  spans: { from: number; to: number; class: string }[];
  language: { capabilities: ServerCapabilities; queries: Record<string, Answer>; occurrences: Occurrence[]; diagnostics: LspDiagnostic[]; shared?: Record<string, { definition: Answer; references: Answer }> } | null;
}

/** Bind analysis to the verified document, then refuse malformed paint ranges. */
export async function decodeStaticLanguage(value: unknown, source: string, expected?: string): Promise<StaticLanguage | null> {
  if (!expected && value === undefined) return null; // Earlier publications remain readable.
  const encoded = typeof value === "string" ? value : JSON.stringify(value);
  if (!expected || await sha256(encoded) !== expected) throw Error("Published language data does not match its hash");
  let data = (typeof value === "string" ? JSON.parse(value) : value) as StaticLanguage;
  const packed = (data as StaticLanguage & { packed_spans?: { classes: string[]; data: number[] } })?.packed_spans;
  if (packed !== undefined) {
    if (!packed || !Array.isArray(packed.classes) || packed.classes.some(name => typeof name !== "string" || !/^(tok-[\w-]+)( tok-[\w-]+)*$/.test(name)) || !Array.isArray(packed.data) || packed.data.length % 3 || !packed.data.every(Number.isInteger) || !Array.isArray(data.spans) || data.spans.length) throw Error("Invalid packed language spans");
    const spans = [];
    for (let i = 0; i < packed.data.length; i += 3) {
      const [from, length, index] = packed.data.slice(i, i + 3);
      if (index < 0 || index >= packed.classes.length) throw Error("Invalid packed token class");
      spans.push({ from, to: from + length, class: packed.classes[index] });
    }
    data = { version: data.version, source_hash: data.source_hash, state: data.state, reason: data.reason, language: data.language, spans };
  }
  const span = (item: { from: number; to: number }) => Number.isInteger(item.from) && Number.isInteger(item.to) && item.from >= 0 && item.to > item.from && item.to <= source.length;
  if (!data || data.version !== 1 || data.source_hash !== await sha256(source) || !["captured", "unavailable"].includes(data.state) || (data.reason != null && typeof data.reason !== "string") || !Array.isArray(data.spans) || data.spans.some(item => !span(item) || !/^(tok-[\w-]+)( tok-[\w-]+)*$/.test(item.class))) throw Error("Invalid static language data");
  if (data.language && (!Array.isArray(data.language.occurrences) || data.language.occurrences.some(item => !span(item)) || !Array.isArray(data.language.diagnostics) || !data.language.queries || !data.language.capabilities)) throw Error("Invalid static language capture");
  return data;
}

function result<T>(answer: Answer | undefined, fallback: T): T {
  return answer?.state === "captured" && answer.result != null ? answer.result as T : fallback;
}

/** A finite, local transport. No URL, fetch, WebSocket or mutation methods. */
export function staticLanguageClient(data: StaticLanguage, source: string, path = "project.md"): LspClient {
  const coordinates = sourceCoordinates(source);
  const listeners = new Set<(message: JsonRpcMessage) => void>();
  const channel: LspChannel = {
    handleFrame: () => false,
    onMessage: callback => { listeners.add(callback); return () => listeners.delete(callback); },
    send(message) {
      if (!("method" in message) || !("id" in message) || message.id === null) return;
      const params = message.params as { position?: LspPosition; textDocument?: { uri?: string } } | undefined;
      const method = message.method.replace(/^textDocument\//, "");
      const allowed = ["hover", "definition", "references", "documentHighlight", "signatureHelp", "documentSymbol", "semanticTokens/full", "foldingRange", "inlayHint"].includes(method) && params?.textDocument?.uri === `hick:///${path}`;
      let answer = allowed ? data.language?.queries[method] : undefined;
      if (allowed && params?.position) {
        const offset = coordinates.raw(params.position);
        const occurrence = data.language?.occurrences.find(item => offset >= item.from && offset < item.to);
        answer = occurrence?.[method] as Answer | undefined;
        if (occurrence && Array.isArray(occurrence.symbolIds) && ["definition", "references", "documentHighlight"].includes(method)) {
          const key = method === "definition" ? "definition" : "references";
          const targets = occurrence.symbolIds.flatMap(id => typeof id === "string" ? result<LspLocation[]>(data.language?.shared?.[id]?.[key], []) : []);
          const unique = [...new Map(targets.map(target => [JSON.stringify(target), target])).values()];
          answer = { state: "captured", result: method === "documentHighlight" ? unique.filter(target => target.uri === `hick:///${path}`).map(target => ({ range: target.range, kind: "kind" in target && target.kind === 3 ? 3 : 2 })) : unique };
        }
        if (method === "hover") {
          const help = result<SignatureHelp | null>(occurrence?.signatureHelp as Answer | undefined, null);
          if (help?.signatures.length) {
            const hover = result<HoverResult | null>(answer, null);
            answer = { state: "captured", result: { contents: [...(Array.isArray(hover?.contents) ? hover.contents : [hover?.contents ?? ""]), ...help.signatures.map(signature => ({ value: signature.label }))] } };
          }
        }
      }
      const response: JsonRpcMessage = answer?.state === "captured"
        ? { jsonrpc: "2.0", id: message.id, result: answer.result }
        : { jsonrpc: "2.0", id: message.id, error: { code: -32601, message: "This answer was not captured in the published revision" } };
      queueMicrotask(() => { for (const callback of listeners) callback(response); });
    },
  };
  return new LspClient(channel);
}

/** Only read-only layers are installed; the live editing feature bundle is absent. */
export function staticLanguageExtensions(data: StaticLanguage, source: string, path: string,
  navigate: (target: LspLocation) => void, references: (targets: LspLocation[]) => void): Extension[] {
  const client = staticLanguageClient(data, source, path);
  const coordinates = sourceCoordinates(source);
  const offset = (position: LspPosition) => coordinates.view(coordinates.raw(position));
  const language = data.language;
  const marks = Decoration.set(data.spans.flatMap(span => {
    const from = coordinates.view(span.from), to = coordinates.view(span.to);
    return to > from ? [Decoration.mark({ class: span.class }).range(from, to)] : [];
  }), true);
  const tokens = result<{ data: number[] } | null>(language?.queries["semanticTokens/full"], null);
  const legend = language?.capabilities.semanticTokensProvider?.legend;
  const semantic = tokens && legend?.tokenTypes ? decodeSemanticTokens(tokens.data, { tokenTypes: legend.tokenTypes, tokenModifiers: legend.tokenModifiers ?? [] }).map(token => ({ from: offset({ line: token.line, character: token.start }), to: offset({ line: token.line, character: token.start + token.length }), class: tokenClass(token) })) : [];
  const hints = result<InlayHint[]>(language?.queries.inlayHint, []);
  const folds = result<FoldingRange[]>(language?.queries.foldingRange, []);
  return [
    EditorView.decorations.of(marks), semanticTokenField, inlayHintField, highlightField,
    EditorView.theme({ ".cm-lsp-inlay": { opacity: "0.6", fontStyle: "italic", fontSize: "0.9em", padding: "0 0.15em" }, ".cm-lsp-highlight": { background: "var(--lsp-highlight, #6988a440)" } }),
    ...lspSupport({ client, uri: `hick:///${path}`, positionAt: at => utf16ToPosition(coordinates.normalized, at), onNavigate: navigate, onReferences: references }),
    foldService.of((_state, from, to) => {
      const fold = folds.find(item => offset({ line: item.startLine, character: item.startCharacter ?? 0 }) >= from && offset({ line: item.startLine, character: item.startCharacter ?? 0 }) <= to);
      if (!fold) return null;
      const end = offset({ line: fold.endLine, character: fold.endCharacter ?? source.split("\n")[fold.endLine]?.length ?? 0 });
      return end > to ? { from: to, to: end } : null;
    }),
    ViewPlugin.fromClass(class {
      constructor(view: EditorView) {
        queueMicrotask(() => { if (view.dom.isConnected) view.dispatch({ effects: [
          setSemanticTokens.of(semantic), setInlayHints.of(hints.map(hint => ({ at: offset(hint.position), text: inlayText(hint) }))),
          setLspDiagnostics.of(diagnosticRanges(language?.diagnostics ?? [], offset)),
        ] }); });
      }
      destroy() { client.dispose(); }
    }),
    EditorView.updateListener.of(update => {
      if (!update.selectionSet) return;
      const at = update.state.selection.main.head;
      const occurrence = language?.occurrences.find(item => at >= item.from && at < item.to);
      const highlights = result<{ range: LspRange; kind?: number }[]>(occurrence?.documentHighlight as Answer | undefined, []);
      queueMicrotask(() => { if (update.view.dom.isConnected) update.view.dispatch({ effects: setHighlights.of(highlights.map(item => ({ from: offset(item.range.start), to: offset(item.range.end), class: "cm-lsp-highlight" }))) }); });
    }),
  ];
}

export function staticSymbols(data: StaticLanguage | null): DocumentSymbol[] {
  const flatten = (symbols: DocumentSymbol[]): DocumentSymbol[] => symbols.flatMap(symbol => [symbol, ...flatten(symbol.children ?? [])]);
  return flatten(result<DocumentSymbol[]>(data?.language?.queries.documentSymbol, []));
}
