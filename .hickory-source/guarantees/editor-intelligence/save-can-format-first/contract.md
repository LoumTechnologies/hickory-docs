<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Save Can Format First, With The File's Own Formatter

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a file or document with a language server behind it, when the person
presses Shift+Alt+F, then the buffer is formatted by whatever formatter that
language server offers — rustfmt through rust-analyzer, the Python server's
own, prettier through the TypeScript server — and the edits are applied as
one user edit; and given the per-user setting **Format on save** is on, when
the person chooses Save or Save All, then the focused buffer is formatted
first and the formatted text is what is saved. A `hick:file` block in a
document is formatted in the block's own indentation. The setting is off by
default.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/save-can-format-first.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-c6800936a2be"></a>
## `apps/web/src/lib/formatOnSave.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Save Can Format First, With The File's Own Formatter](contract.md).

<a id="f-c6800936a2be-enabled"></a>
### `enabled`

Whether Save runs the formatter first.
A per-user setting the server persists (ui.json, beside the window title)
so the desktop app and `hick up` agree. Cached here because the moment it
is read — a Save keystroke — is not a moment to wait on a request; the
workspace loads it once at mount and the Settings page updates it on
toggle.

<storycode:copy id="f-c6800936a2be-enabled">
// Whether Save runs the formatter first.
//
// A per-user setting the server persists (ui.json, beside the window title)
// so the desktop app and `hick up` agree. Cached here because the moment it
// is read — a Save keystroke — is not a moment to wait on a request; the
// workspace loads it once at mount and the Settings page updates it on
// toggle.

import { api } from "../api/client";

let enabled = false;

</storycode:copy>
<a id="f-c6800936a2be-formatOnSave"></a>
### `formatOnSave`

<storycode:copy id="f-c6800936a2be-formatOnSave">
/** Whether Save formats first. */
export function formatOnSave(): boolean {
  return enabled;
}

</storycode:copy>
<a id="f-c6800936a2be-setFormatOnSave"></a>
### `setFormatOnSave`

<storycode:copy id="f-c6800936a2be-setFormatOnSave">
export function setFormatOnSave(on: boolean): void {
  enabled = on;
}

</storycode:copy>
<a id="f-c6800936a2be-loadFormatOnSave"></a>
### `loadFormatOnSave`

Read the setting from the server. A failure leaves it off, which is the
default, rather than surfacing as an error a Save could hit.

<storycode:copy id="f-c6800936a2be-loadFormatOnSave">
/** Read the setting from the server. A failure leaves it off, which is the
 * default, rather than surfacing as an error a Save could hit. */
export async function loadFormatOnSave(): Promise<void> {
  try {
    const ui = await api.settingsUi();
    enabled = ui.format_on_save === true;
  } catch {
    // Off: the honest default when nothing can be read.
  }
}
</storycode:copy>

### Assemble `apps/web/src/lib/formatOnSave.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/formatOnSave.ts"><storycode:paste select="#f-c6800936a2be-enabled"/><storycode:paste select="#f-c6800936a2be-formatOnSave"/><storycode:paste select="#f-c6800936a2be-setFormatOnSave"/><storycode:paste select="#f-c6800936a2be-loadFormatOnSave"/></storycode:file>

<a id="output-6bc953112fd0"></a>
## `apps/web/src/lsp/client.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Save Can Format First, With The File's Own Formatter](contract.md).

<a id="f-6bc953112fd0-LspLocation"></a>
### `LspLocation`

Typed JSON-RPC client for the LSP bridge (api.md v0.3).
Sits on an `LspChannel` (channel.ts). Per the contract the server owns the
LSP lifecycle (`initialize` is handled server-side), so a client starts
straight at `didOpen` with URI `hick:///<doc-path>` and LSP positions over
the same source text the editor holds. Definition/reference targets may
come back as `hick:///<doc-path>` or, for untranslatable generated-output
locations, `hick-output:///<output-path>`.

<storycode:copy id="f-6bc953112fd0-LspLocation">
// Typed JSON-RPC client for the LSP bridge (api.md v0.3).
//
// Sits on an `LspChannel` (channel.ts). Per the contract the server owns the
// LSP lifecycle (`initialize` is handled server-side), so a client starts
// straight at `didOpen` with URI `hick:///<doc-path>` and LSP positions over
// the same source text the editor holds. Definition/reference targets may
// come back as `hick:///<doc-path>` or, for untranslatable generated-output
// locations, `hick-output:///<output-path>`.

import type { JsonRpcMessage, LspChannel } from "./channel";
import type { LspPosition, LspRange } from "./positions";

export interface LspLocation {
  uri: string;
  range: LspRange;
}

</storycode:copy>
<a id="f-6bc953112fd0-LspDiagnostic"></a>
### `LspDiagnostic`

<storycode:copy id="f-6bc953112fd0-LspDiagnostic">
export interface LspDiagnostic {
  range: LspRange;
  severity?: number;
  source?: string;
  message: string;
}

</storycode:copy>
<a id="f-6bc953112fd0-PublishDiagnosticsParams"></a>
### `PublishDiagnosticsParams`

<storycode:copy id="f-6bc953112fd0-PublishDiagnosticsParams">
export interface PublishDiagnosticsParams {
  uri: string;
  diagnostics: LspDiagnostic[];
}

</storycode:copy>
<a id="f-6bc953112fd0-HoverResult"></a>
### `HoverResult`

<storycode:copy id="f-6bc953112fd0-HoverResult">
export interface HoverResult {
  contents: unknown;
  range?: LspRange;
}

</storycode:copy>
<a id="f-6bc953112fd0-CompletionItem"></a>
### `CompletionItem`

<storycode:copy id="f-6bc953112fd0-CompletionItem">
export interface CompletionItem {
  label: string;
  kind?: number;
  detail?: string;
  insertText?: string;
  [key: string]: unknown;
}

</storycode:copy>
<a id="f-6bc953112fd0-SignatureHelp"></a>
### `SignatureHelp`

<storycode:copy id="f-6bc953112fd0-SignatureHelp">
export interface SignatureHelp {
  signatures: {
    label: string;
    documentation?: unknown;
    parameters?: { label: string | [number, number]; documentation?: unknown }[];
    activeParameter?: number;
  }[];
  activeSignature?: number;
  activeParameter?: number;
}

</storycode:copy>
<a id="f-6bc953112fd0-DocumentSymbol"></a>
### `DocumentSymbol`

<storycode:copy id="f-6bc953112fd0-DocumentSymbol">
export interface DocumentSymbol {
  name: string;
  detail?: string;
  kind: number;
  range: LspRange;
  selectionRange: LspRange;
  children?: DocumentSymbol[];
  /** The flat `SymbolInformation` shape some servers still return. */
  location?: LspLocation;
}

</storycode:copy>
<a id="f-6bc953112fd0-InlayHint"></a>
### `InlayHint`

<storycode:copy id="f-6bc953112fd0-InlayHint">
export interface InlayHint {
  position: LspPosition;
  label: string | { value: string }[];
  kind?: number;
  paddingLeft?: boolean;
  paddingRight?: boolean;
}

</storycode:copy>
<a id="f-6bc953112fd0-FoldingRange"></a>
### `FoldingRange`

<storycode:copy id="f-6bc953112fd0-FoldingRange">
export interface FoldingRange {
  startLine: number;
  endLine: number;
  startCharacter?: number;
  endCharacter?: number;
  kind?: string;
}

</storycode:copy>
<a id="f-6bc953112fd0-CodeAction"></a>
### `CodeAction`

<storycode:copy id="f-6bc953112fd0-CodeAction">
export interface CodeAction {
  title: string;
  kind?: string;
  edit?: WorkspaceEdit;
  command?: { title: string; command: string; arguments?: unknown[] };
}

</storycode:copy>
<a id="f-6bc953112fd0-TextEdit"></a>
### `TextEdit`

<storycode:copy id="f-6bc953112fd0-TextEdit">
/** One replacement in one document. */
export interface TextEdit {
  range: LspRange;
  newText: string;
}

</storycode:copy>
<a id="f-6bc953112fd0-WorkspaceEdit"></a>
### `WorkspaceEdit`

<storycode:copy id="f-6bc953112fd0-WorkspaceEdit">
export interface WorkspaceEdit {
  changes?: Record<string, { range: LspRange; newText: string }[]>;
  documentChanges?: {
    textDocument?: { uri: string; version?: number | null };
    edits?: { range: LspRange; newText: string }[];
  }[];
}

</storycode:copy>
<a id="f-6bc953112fd0-ServerCapabilities"></a>
### `ServerCapabilities`

Only the parts of the server's capabilities the notebook acts on.

<storycode:copy id="f-6bc953112fd0-ServerCapabilities">
/** Only the parts of the server's capabilities the notebook acts on. */
export interface ServerCapabilities {
  semanticTokensProvider?: {
    legend?: { tokenTypes?: string[]; tokenModifiers?: string[] };
  };
  renameProvider?: unknown;
  inlayHintProvider?: unknown;
  foldingRangeProvider?: unknown;
  documentSymbolProvider?: unknown;
  signatureHelpProvider?: unknown;
  codeActionProvider?: unknown;
  [key: string]: unknown;
}

</storycode:copy>
<a id="f-6bc953112fd0-Pending"></a>
### `Pending`

<storycode:copy id="f-6bc953112fd0-Pending">
type Pending = {
  resolve: (value: unknown) => void;
  reject: (err: Error) => void;
  timer: ReturnType<typeof setTimeout>;
};

</storycode:copy>
<a id="f-6bc953112fd0-LspRequestError"></a>
### `LspRequestError`

<storycode:copy id="f-6bc953112fd0-LspRequestError">
export class LspRequestError extends Error {
  constructor(
    public readonly code: number,
    message: string,
  ) {
    super(message);
    this.name = "LspRequestError";
  }
}

</storycode:copy>
<a id="f-6bc953112fd0-normalizeLocations"></a>
### `normalizeLocations`

Normalize Location | Location[] | LocationLink[] | null to LspLocation[].

<storycode:copy id="f-6bc953112fd0-normalizeLocations">
/** Normalize Location | Location[] | LocationLink[] | null to LspLocation[]. */
export function normalizeLocations(result: unknown): LspLocation[] {
  if (result == null) return [];
  const items = Array.isArray(result) ? result : [result];
  const out: LspLocation[] = [];
  for (const item of items) {
    const o = item as Record<string, unknown>;
    if (typeof o.uri === "string" && o.range) {
      out.push({ uri: o.uri, range: o.range as LspRange });
    } else if (typeof o.targetUri === "string" && o.targetRange) {
      out.push({ uri: o.targetUri, range: o.targetRange as LspRange });
    }
  }
  return out;
}

</storycode:copy>
<a id="f-6bc953112fd0-LspClient"></a>
### `LspClient`

<storycode:copy id="f-6bc953112fd0-LspClient">
export class LspClient {
  private nextId = 1;
  private pending = new Map<number, Pending>();
  private diagnosticsListeners = new Set<(params: PublishDiagnosticsParams) => void>();
  private capabilityListeners = new Set<(capabilities: ServerCapabilities) => void>();
  private capabilities: ServerCapabilities | null = null;
  private unsubscribe: () => void;
  private disposed = false;
  private documentSync = new Map<string, () => void>();

  /** Explicit edits must see pending source before asking the provider. */
  registerDocumentSync(uri: string, flush: () => void): () => void {
    this.documentSync.set(uri, flush);
    return () => { if (this.documentSync.get(uri) === flush) this.documentSync.delete(uri); };
  }

  constructor(
    private channel: LspChannel,
    private requestTimeoutMs = 30_000,
  ) {
    this.unsubscribe = channel.onMessage((msg) => this.handleMessage(msg));
  }

  private handleMessage(msg: JsonRpcMessage) {
    if ("id" in msg && msg.id !== null && !("method" in msg)) {
      const id = typeof msg.id === "number" ? msg.id : Number.NaN;
      const pending = this.pending.get(id);
      if (!pending) return;
      this.pending.delete(id);
      clearTimeout(pending.timer);
      if ("error" in msg && msg.error) {
        pending.reject(new LspRequestError(msg.error.code, msg.error.message));
      } else {
        pending.resolve("result" in msg ? msg.result : null);
      }
      return;
    }
    if ("method" in msg && msg.method === "textDocument/publishDiagnostics") {
      const params = (msg as { params?: PublishDiagnosticsParams }).params;
      if (params) for (const cb of this.diagnosticsListeners) cb(params);
      return;
    }
    if ("method" in msg && msg.method === "hick/serverCapabilities") {
      const params = (msg as { params?: { capabilities?: ServerCapabilities } }).params;
      this.capabilities = params?.capabilities ?? null;
      if (this.capabilities) {
        for (const cb of this.capabilityListeners) cb(this.capabilities);
      }
    }
  }

  /** Send a raw request; resolves with the JSON-RPC result. */
  request(method: string, params?: unknown): Promise<unknown> {
    if (this.disposed) return Promise.reject(new Error("LspClient disposed"));
    if (["textDocument/prepareRename", "textDocument/rename", "textDocument/codeAction", "textDocument/formatting"].includes(method)) {
      const uri = (params as { textDocument?: { uri?: string } } | undefined)?.textDocument?.uri;
      if (uri) this.documentSync.get(uri)?.();
    }
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`LSP request ${method} timed out`));
      }, this.requestTimeoutMs);
      this.pending.set(id, { resolve, reject, timer });
      this.channel.send({ jsonrpc: "2.0", id, method, params });
    });
  }

  /** Send a raw notification (no response). */
  notify(method: string, params?: unknown) {
    if (this.disposed) return;
    this.channel.send({ jsonrpc: "2.0", method, params });
  }

  // -- document lifecycle ---------------------------------------------------

  didOpen(uri: string, text: string, version = 1, languageId = "hick") {
    this.notify("textDocument/didOpen", {
      textDocument: { uri, languageId, version, text },
    });
  }

  /** Full-text sync, matching the bridge's TextDocumentSyncKind.FULL. */
  didChange(uri: string, text: string, version: number) {
    this.notify("textDocument/didChange", {
      textDocument: { uri, version },
      contentChanges: [{ text }],
    });
  }

  didClose(uri: string) {
    this.notify("textDocument/didClose", { textDocument: { uri } });
  }

  // -- language features ----------------------------------------------------

  async hover(uri: string, position: LspPosition): Promise<HoverResult | null> {
    const result = await this.request("textDocument/hover", {
      textDocument: { uri },
      position,
    });
    return (result as HoverResult) ?? null;
  }

  async definition(uri: string, position: LspPosition): Promise<LspLocation[]> {
    const result = await this.request("textDocument/definition", {
      textDocument: { uri },
      position,
    });
    return normalizeLocations(result);
  }

  async references(
    uri: string,
    position: LspPosition,
    includeDeclaration = true,
  ): Promise<LspLocation[]> {
    const result = await this.request("textDocument/references", {
      textDocument: { uri },
      position,
      context: { includeDeclaration },
    });
    return normalizeLocations(result);
  }

  async completion(uri: string, position: LspPosition): Promise<CompletionItem[]> {
    const result = await this.request("textDocument/completion", {
      textDocument: { uri },
      position,
    });
    if (result == null) return [];
    if (Array.isArray(result)) return result as CompletionItem[];
    const list = result as { items?: CompletionItem[] };
    return list.items ?? [];
  }

  /** Documentation and the full text edit arrive here, not in the list. */
  async resolveCompletion(item: CompletionItem): Promise<CompletionItem> {
    const result = await this.request("completionItem/resolve", item);
    return (result as CompletionItem) ?? item;
  }

  async signatureHelp(uri: string, position: LspPosition): Promise<SignatureHelp | null> {
    const result = await this.request("textDocument/signatureHelp", {
      textDocument: { uri },
      position,
    });
    return (result as SignatureHelp) ?? null;
  }

  /** The raw token array, still delta-encoded — see `decodeSemanticTokens`. */
  async semanticTokens(uri: string): Promise<number[] | null> {
    const result = await this.request("textDocument/semanticTokens/full", {
      textDocument: { uri },
    });
    const data = (result as { data?: number[] } | null)?.data;
    return Array.isArray(data) ? data : null;
  }

  async documentHighlight(uri: string, position: LspPosition): Promise<LspRange[]> {
    const result = await this.request("textDocument/documentHighlight", {
      textDocument: { uri },
      position,
    });
    if (!Array.isArray(result)) return [];
    return (result as { range: LspRange }[]).map((item) => item.range).filter(Boolean);
  }

  async documentSymbols(uri: string): Promise<DocumentSymbol[]> {
    const result = await this.request("textDocument/documentSymbol", {
      textDocument: { uri },
    });
    return Array.isArray(result) ? (result as DocumentSymbol[]) : [];
  }

  async inlayHints(uri: string, range: LspRange): Promise<InlayHint[]> {
    const result = await this.request("textDocument/inlayHint", {
      textDocument: { uri },
      range,
    });
    return Array.isArray(result) ? (result as InlayHint[]) : [];
  }

  async foldingRanges(uri: string): Promise<FoldingRange[]> {
    const result = await this.request("textDocument/foldingRange", {
      textDocument: { uri },
    });
    return Array.isArray(result) ? (result as FoldingRange[]) : [];
  }

  async codeActions(uri: string, range: LspRange, diagnostics: LspDiagnostic[] = []) {
    const result = await this.request("textDocument/codeAction", {
      textDocument: { uri },
      range,
      context: { diagnostics },
    });
    return Array.isArray(result) ? (result as CodeAction[]) : [];
  }

  /** Null means "not renameable here", which the editor should say plainly. */
  async prepareRename(uri: string, position: LspPosition): Promise<LspRange | null> {
    const result = await this.request("textDocument/prepareRename", {
      textDocument: { uri },
      position,
    });
    if (!result) return null;
    const asRange = result as LspRange & { range?: LspRange };
    return asRange.range ?? asRange;
  }

  async rename(uri: string, position: LspPosition, newName: string): Promise<WorkspaceEdit | null> {
    const result = await this.request("textDocument/rename", {
      textDocument: { uri },
      position,
      newName,
    });
    return (result as WorkspaceEdit) ?? null;
  }

  /** The edits that format the whole document, in document coordinates. */
  async formatting(
    uri: string,
    options: { tabSize: number; insertSpaces: boolean } = { tabSize: 4, insertSpaces: true },
  ): Promise<TextEdit[]> {
    const result = await this.request("textDocument/formatting", {
      textDocument: { uri },
      options,
    });
    return Array.isArray(result) ? (result as TextEdit[]) : [];
  }

  async typeDefinition(uri: string, position: LspPosition): Promise<LspLocation[]> {
    return normalizeLocations(
      await this.request("textDocument/typeDefinition", { textDocument: { uri }, position }),
    );
  }

  async implementation(uri: string, position: LspPosition): Promise<LspLocation[]> {
    return normalizeLocations(
      await this.request("textDocument/implementation", { textDocument: { uri }, position }),
    );
  }

  async declaration(uri: string, position: LspPosition): Promise<LspLocation[]> {
    return normalizeLocations(
      await this.request("textDocument/declaration", { textDocument: { uri }, position }),
    );
  }

  /**
   * The server's advertised capabilities, once the bridge has announced them.
   *
   * The browser never sends `initialize` — the bridge does it on our behalf —
   * so this is how the editor learns the semantic-token legend and which
   * features are worth offering.
   */
  onServerCapabilities(cb: (capabilities: ServerCapabilities) => void): () => void {
    if (this.capabilities) cb(this.capabilities);
    this.capabilityListeners.add(cb);
    return () => this.capabilityListeners.delete(cb);
  }

  get serverCapabilities(): ServerCapabilities | null {
    return this.capabilities;
  }

  /** Subscribe to server-pushed diagnostics. Returns an unsubscribe fn. */
  onDiagnostics(cb: (params: PublishDiagnosticsParams) => void): () => void {
    this.diagnosticsListeners.add(cb);
    return () => this.diagnosticsListeners.delete(cb);
  }

  dispose() {
    this.disposed = true;
    this.unsubscribe();
    for (const [, pending] of this.pending) {
      clearTimeout(pending.timer);
      pending.reject(new Error("LspClient disposed"));
    }
    this.pending.clear();
    this.documentSync.clear();
    this.diagnosticsListeners.clear();
    this.capabilityListeners.clear();
  }
}
</storycode:copy>

### Assemble `apps/web/src/lsp/client.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/client.ts"><storycode:paste select="#f-6bc953112fd0-LspLocation"/><storycode:paste select="#f-6bc953112fd0-LspDiagnostic"/><storycode:paste select="#f-6bc953112fd0-PublishDiagnosticsParams"/><storycode:paste select="#f-6bc953112fd0-HoverResult"/><storycode:paste select="#f-6bc953112fd0-CompletionItem"/><storycode:paste select="#f-6bc953112fd0-SignatureHelp"/><storycode:paste select="#f-6bc953112fd0-DocumentSymbol"/><storycode:paste select="#f-6bc953112fd0-InlayHint"/><storycode:paste select="#f-6bc953112fd0-FoldingRange"/><storycode:paste select="#f-6bc953112fd0-CodeAction"/><storycode:paste select="#f-6bc953112fd0-TextEdit"/><storycode:paste select="#f-6bc953112fd0-WorkspaceEdit"/><storycode:paste select="#f-6bc953112fd0-ServerCapabilities"/><storycode:paste select="#f-6bc953112fd0-Pending"/><storycode:paste select="#f-6bc953112fd0-LspRequestError"/><storycode:paste select="#f-6bc953112fd0-normalizeLocations"/><storycode:paste select="#f-6bc953112fd0-LspClient"/></storycode:file>

<a id="output-830f6d637cff"></a>
## `apps/web/src/lsp/cmLspFeatures.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A comparison keeps current code editable](../../editing/a-comparison-keeps-current-code-editable/contract.md), [Save Can Format First, With The File's Own Formatter](contract.md).

<a id="f-830f6d637cff-viewWith"></a>
### `viewWith`

<storycode:copy id="f-830f6d637cff-viewWith">
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { describe, expect, it } from "vitest";
import {
  diagnosticRanges,
  diagnosticsAt,
  lspDiagnosticField,
  offsetToPosition,
  positionToOffset,
  setLspDiagnostics,
} from "./cmLsp";
import {
  editsForUri,
  formatView,
  inlayHintField,
  lspFeatures,
  inlayText,
  semanticTokenField,
  setInlayHints,
  setSemanticTokens,
  signatureLabel,
  urisInEdit,
} from "./cmLspFeatures";

function viewWith(doc: string, extensions = [semanticTokenField, inlayHintField]) {
  return new EditorView({ state: EditorState.create({ doc, extensions }) });
}

</storycode:copy>
<a id="f-830f6d637cff-decorationCount"></a>
### `decorationCount`

<storycode:copy id="f-830f6d637cff-decorationCount">
function decorationCount(view: EditorView, field: typeof semanticTokenField) {
  let count = 0;
  view.state.field(field).between(0, view.state.doc.length, () => {
    count++;
  });
  return count;
}

</storycode:copy>
<a id="f-830f6d637cff-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-830f6d637cff-expression_statement">
describe("semantic token decorations", () => {
  it("paints the spans it is given", () => {
    const view = viewWith("def load(path):\n    return path\n");
    view.dispatch({
      effects: setSemanticTokens.of([
        { from: 0, to: 3, class: "cm-st-keyword" },
        { from: 4, to: 8, class: "cm-st-function" },
      ]),
    });
    expect(decorationCount(view, semanticTokenField)).toBe(2);
  });

  it("accepts spans in any order, because a server does not sort for us", () => {
    // RangeSetBuilder throws on descending ranges, so an unsorted reply used
    // to take the whole editor down rather than losing a colour.
    const view = viewWith("abcdefghij");
    expect(() =>
      view.dispatch({
        effects: setSemanticTokens.of([
          { from: 6, to: 8, class: "cm-st-variable" },
          { from: 0, to: 3, class: "cm-st-keyword" },
        ]),
      }),
    ).not.toThrow();
    expect(decorationCount(view, semanticTokenField)).toBe(2);
  });

  it("clamps a span that outlived the text it described", () => {
    // Tokens arrive after a debounce, so the document may have shrunk in the
    // meantime. Painting past the end would throw.
    const view = viewWith("short");
    view.dispatch({ effects: setSemanticTokens.of([{ from: 2, to: 900, class: "cm-st-type" }]) });
    expect(decorationCount(view, semanticTokenField)).toBe(1);
  });

  it("drops an empty span rather than painting a zero-width mark", () => {
    const view = viewWith("hello");
    view.dispatch({ effects: setSemanticTokens.of([{ from: 3, to: 3, class: "cm-st-type" }]) });
    expect(decorationCount(view, semanticTokenField)).toBe(0);
  });
});

</storycode:copy>
<a id="f-830f6d637cff-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-830f6d637cff-expression_statement-2">
describe("inlay hints", () => {
  it("places a widget at each hint position", () => {
    const view = viewWith("x = load()\n");
    view.dispatch({ effects: setInlayHints.of([{ at: 1, text: ": str" }]) });
    expect(decorationCount(view, inlayHintField)).toBe(1);
  });

  it("joins a label given as parts", () => {
    expect(inlayText({ position: { line: 0, character: 0 }, label: [{ value: ": " }, { value: "int" }] })).toBe(
      ": int",
    );
    expect(inlayText({ position: { line: 0, character: 0 }, label: "-> bool" })).toBe("-> bool");
  });
});

</storycode:copy>
<a id="f-830f6d637cff-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-830f6d637cff-expression_statement-3">
describe("diagnostics", () => {
  it("carries the server's message through to the decoration", () => {
    // An underline tells you WHERE the problem is and nothing about what it
    // is. The message rides on the decoration so the hover can read it back
    // without a second structure to keep in step.
    const view = viewWith("const x: number = 'oops';", [lspDiagnosticField]);
    view.dispatch({
      effects: setLspDiagnostics.of([
        { from: 18, to: 24, severity: 1, message: "Type 'string' is not assignable", source: "typescript" },
      ]),
    });
    const found = diagnosticsAt(view.state, 20);
    expect(found).toHaveLength(1);
    expect(found[0].message).toContain("not assignable");
    expect(found[0].source).toBe("typescript");
    expect(found[0].severity).toBe(1);
  });

  it("finds nothing where there is nothing wrong", () => {
    const view = viewWith("fine", [lspDiagnosticField]);
    view.dispatch({ effects: setLspDiagnostics.of([]) });
    expect(diagnosticsAt(view.state, 2)).toEqual([]);
  });

  it("keeps a zero-width diagnostic hoverable", () => {
    // An insertion-point diagnostic has start == end; left alone it would be
    // an underline with no width, which is a squiggle nobody can point at.
    const spans = diagnosticRanges(
      [{ range: r(0, 3, 0, 3), severity: 1, message: "expected `;`" }],
      (position) => position.character,
    );
    expect(spans[0].to).toBeGreaterThan(spans[0].from);
    expect(spans[0].message).toBe("expected `;`");
  });
});

</storycode:copy>
<a id="f-830f6d637cff-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-830f6d637cff-expression_statement-4">
describe("workspace edits", () => {
  const edit = {
    changes: { "hick:///a.md": [{ range: r(0, 0, 0, 4), newText: "next" }] },
    documentChanges: [
      {
        textDocument: { uri: "hick:///a.md", version: 1 },
        edits: [{ range: r(2, 0, 2, 4), newText: "next" }],
      },
      {
        textDocument: { uri: "hick:///b.md", version: 1 },
        edits: [{ range: r(0, 0, 0, 4), newText: "next" }],
      },
    ],
  };

  it("takes this document's edits from both shapes a server may use", () => {
    expect(editsForUri(edit, "hick:///a.md")).toHaveLength(2);
  });

  it("ignores edits addressed to another document", () => {
    // Applying another file's ranges to this buffer would corrupt it at
    // coordinates that happen to exist here.
    expect(editsForUri(edit, "hick:///a.md").every((e) => e.newText === "next")).toBe(true);
    expect(editsForUri(edit, "hick:///c.md")).toEqual([]);
  });

  it("accepts an encoded provider edit for the same spaced URI without applying a literal-percent filename", () => {
    const rename = { changes: { "hick:///Application%20Support/main.cpp": [{ range: r(0, 4, 0, 10), newText: "squared" }] } };
    expect(editsForUri(rename, "hick:///Application Support/main.cpp")).toHaveLength(1);
    expect(editsForUri(rename, "hick:///Application%2520Support/main.cpp")).toEqual([]);
    expect(editsForUri({ documentChanges: [{ textDocument: { uri: "hick:///Application%20Support/main.cpp", version: 1 }, edits: rename.changes["hick:///Application%20Support/main.cpp"] }] }, "hick:///Application Support/main.cpp")).toHaveLength(1);
  });

  it("names every document a rename touches, so the rest can be reported", () => {
    expect(urisInEdit(edit).sort()).toEqual(["hick:///a.md", "hick:///b.md"]);
  });

  it("treats a refused rename as no edits rather than an error", () => {
    expect(editsForUri(null, "hick:///a.md")).toEqual([]);
    expect(urisInEdit(null)).toEqual([]);
  });
});

</storycode:copy>
<a id="f-830f6d637cff-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-830f6d637cff-expression_statement-5">
describe("signature help", () => {
  it("shows the active signature, not the first", () => {
    expect(
      signatureLabel({
        signatures: [{ label: "(a: int)" }, { label: "(a: str, b: int)" }],
        activeSignature: 1,
      }),
    ).toBe("(a: str, b: int)");
  });

  it("is empty when a server answers with no signatures", () => {
    expect(signatureLabel({ signatures: [] })).toBe("");
  });
});

</storycode:copy>
<a id="f-830f6d637cff-r"></a>
### `r`

<storycode:copy id="f-830f6d637cff-r">
function r(sl: number, sc: number, el: number, ec: number) {
  return { start: { line: sl, character: sc }, end: { line: el, character: ec } };
}

</storycode:copy>
<a id="f-830f6d637cff-expression_statement-6"></a>
### `expression_statement`

Protects docs/guarantees/editor-intelligence/save-can-format-first.md

<storycode:copy id="f-830f6d637cff-expression_statement-6">
// Protects docs/guarantees/editor-intelligence/save-can-format-first.md
describe("formatting", () => {
  /** A client that answers formatting and nothing else, with the rest of the
   * bundle's requests quietly empty. */
  function clientWith(edits: { range: { start: { line: number; character: number }; end: { line: number; character: number } }; newText: string }[]) {
    return {
      serverCapabilities: null,
      formatting: async () => edits,
      semanticTokens: async () => null,
      inlayHints: async () => [],
      documentHighlight: async () => [],
      foldingRanges: async () => [],
      signatureHelp: async () => null,
    } as unknown as import("./client").LspClient;
  }

  function editorWith(doc: string, client: import("./client").LspClient) {
    return new EditorView({
      state: EditorState.create({
        doc,
        extensions: lspFeatures({
          client,
          uri: "hick:///a.rs",
          positionAt: (offset, view) => offsetToPosition(view.state.doc, offset),
          offsetAt: (position, view) => positionToOffset(view.state.doc, position),
        }),
      }),
    });
  }

  it("applies the server's edits back to front, as one user edit", async () => {
    // Two edits in server order. Applied forwards, the first would shift
    // the second onto the wrong bytes.
    const view = editorWith("a  b  c", clientWith([
      { range: { start: { line: 0, character: 1 }, end: { line: 0, character: 3 } }, newText: " " },
      { range: { start: { line: 0, character: 4 }, end: { line: 0, character: 6 } }, newText: " " },
    ]));
    expect(await formatView(view)).toBe(true);
    expect(view.state.doc.toString()).toBe("a b c");
  });

  it("says so, and changes nothing, when the server has no edits", async () => {
    const view = editorWith("fine", clientWith([]));
    expect(await formatView(view)).toBe(false);
    expect(view.state.doc.toString()).toBe("fine");
  });

  it("is not offered by an editor without a language client", async () => {
    const view = new EditorView({ state: EditorState.create({ doc: "x" }) });
    expect(await formatView(view)).toBe(false);
  });
});

</storycode:copy>
<a id="f-830f6d637cff-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-830f6d637cff-expression_statement-7">
// Guarantee: editing/a-comparison-keeps-current-code-editable.md.
describe("language edits in a source projection",()=>{
  const quietRequests = {
    serverCapabilities: null,
    semanticTokens: async () => null,
    inlayHints: async () => [],
    documentHighlight: async () => [],
    foldingRanges: async () => [],
    signatureHelp: async () => null,
  };
  it("refuses the entire formatting batch when an edit would cross reading fragments",async()=>{
    const client={...quietRequests,formatting:async()=>[
      {range:{start:{line:0,character:0},end:{line:0,character:1}},newText:"safe"},
      {range:{start:{line:0,character:0},end:{line:1,character:1}},newText:"would eat prose"},
    ]} as unknown as import("./client").LspClient;
    const view=new EditorView({state:EditorState.create({doc:"a\n# Explanation\nb",extensions:lspFeatures({
      client,uri:"hick:///a.py",positionAt:()=>({line:0,character:0}),offsetAt:()=>0,
      rangeAt:range=>range.end.line===0?{from:0,to:1}:null,
    })})});
    expect(await formatView(view)).toBe(false);
    expect(view.state.doc.toString()).toBe("a\n# Explanation\nb");
    view.destroy();
  });
  it("discards formatting answers that arrive after the current buffer changed",async()=>{
    let resolve!:(v:{range:{start:{line:number;character:number};end:{line:number;character:number}};newText:string}[])=>void;
    const client={...quietRequests,formatting:()=>new Promise(r=>{resolve=r;})} as unknown as import("./client").LspClient;
    const view=new EditorView({state:EditorState.create({doc:"old",extensions:lspFeatures({client,uri:"hick:///a.py",positionAt:()=>({line:0,character:0}),offsetAt:p=>p.character})})});
    const pending=formatView(view);
    view.dispatch({changes:{from:0,to:3,insert:"new"}});
    resolve([{range:{start:{line:0,character:0},end:{line:0,character:3}},newText:"stale"}]);
    expect(await pending).toBe(false); expect(view.state.doc.toString()).toBe("new"); view.destroy();
  });
});
</storycode:copy>

### Assemble `apps/web/src/lsp/cmLspFeatures.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/cmLspFeatures.test.ts"><storycode:paste select="#f-830f6d637cff-viewWith"/><storycode:paste select="#f-830f6d637cff-decorationCount"/><storycode:paste select="#f-830f6d637cff-expression_statement"/><storycode:paste select="#f-830f6d637cff-expression_statement-2"/><storycode:paste select="#f-830f6d637cff-expression_statement-3"/><storycode:paste select="#f-830f6d637cff-expression_statement-4"/><storycode:paste select="#f-830f6d637cff-expression_statement-5"/><storycode:paste select="#f-830f6d637cff-r"/><storycode:paste select="#f-830f6d637cff-expression_statement-6"/><storycode:paste select="#f-830f6d637cff-expression_statement-7"/></storycode:file>

<a id="output-a12307572b4c"></a>
## `apps/web/src/lsp/cmLspFeatures.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Every Shortcut Is A Setting](../every-shortcut-is-a-setting/contract.md), [Save Can Format First, With The File's Own Formatter](contract.md).

<a id="f-a12307572b4c-Span"></a>
### `Span`

<storycode:copy id="f-a12307572b4c-Span">
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

</storycode:copy>
<a id="f-a12307572b4c-setSemanticTokens"></a>
### `setSemanticTokens`

<storycode:copy id="f-a12307572b4c-setSemanticTokens">
// --- semantic tokens (colouring) -------------------------------------------

export const setSemanticTokens = StateEffect.define<Span[]>();

</storycode:copy>
<a id="f-a12307572b4c-semanticTokenField"></a>
### `semanticTokenField`

<storycode:copy id="f-a12307572b4c-semanticTokenField">
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

</storycode:copy>
<a id="f-a12307572b4c-setHighlights"></a>
### `setHighlights`

--- document highlight (the other uses of what the cursor is on) ----------

<storycode:copy id="f-a12307572b4c-setHighlights">
// --- document highlight (the other uses of what the cursor is on) ----------

export const setHighlights = StateEffect.define<Span[]>();

</storycode:copy>
<a id="f-a12307572b4c-highlightField"></a>
### `highlightField`

<storycode:copy id="f-a12307572b4c-highlightField">
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

</storycode:copy>
<a id="f-a12307572b4c-buildMarks"></a>
### `buildMarks`

Build mark decorations, sorted and clamped.
`RangeSetBuilder` requires ascending order and throws otherwise, and a
server's tokens can arrive after an edit that shortened the document — so
both the sort and the clamp are load-bearing, not defensive habit.

<storycode:copy id="f-a12307572b4c-buildMarks">
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

</storycode:copy>
<a id="f-a12307572b4c-InlayWidget"></a>
### `InlayWidget`

<storycode:copy id="f-a12307572b4c-InlayWidget">
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

</storycode:copy>
<a id="f-a12307572b4c-setInlayHints"></a>
### `setInlayHints`

<storycode:copy id="f-a12307572b4c-setInlayHints">
export const setInlayHints = StateEffect.define<{ at: number; text: string }[]>();

</storycode:copy>
<a id="f-a12307572b4c-inlayHintField"></a>
### `inlayHintField`

<storycode:copy id="f-a12307572b4c-inlayHintField">
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

</storycode:copy>
<a id="f-a12307572b4c-inlayText"></a>
### `inlayText`

An inlay hint's label, which is either a string or labelled parts.

<storycode:copy id="f-a12307572b4c-inlayText">
/** An inlay hint's label, which is either a string or labelled parts. */
export function inlayText(hint: InlayHint): string {
  if (typeof hint.label === "string") return hint.label;
  return hint.label.map((part) => part.value).join("");
}

</storycode:copy>
<a id="f-a12307572b4c-editsForUri"></a>
### `editsForUri`

--- turning a workspace edit into editor changes ---------------------------
The edits a workspace edit makes to ONE document.
A rename can touch several files; the notebook only holds this one, and an
edit to a document that is not open cannot be applied here. Returning just
this document's edits — and letting the caller report the rest — beats
applying half a rename silently.

<storycode:copy id="f-a12307572b4c-editsForUri">
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

</storycode:copy>
<a id="f-a12307572b4c-urisInEdit"></a>
### `urisInEdit`

Every URI a workspace edit touches, for telling the user what else moved.

<storycode:copy id="f-a12307572b4c-urisInEdit">
/** Every URI a workspace edit touches, for telling the user what else moved. */
export function urisInEdit(edit: WorkspaceEdit | null): string[] {
  if (!edit) return [];
  const uris = new Set<string>(Object.keys(edit.changes ?? {}));
  for (const change of edit.documentChanges ?? []) {
    if (change.textDocument?.uri) uris.add(change.textDocument.uri);
  }
  return [...uris];
}

</storycode:copy>
<a id="f-a12307572b4c-LspFeatureOptions"></a>
### `LspFeatureOptions`

<storycode:copy id="f-a12307572b4c-LspFeatureOptions">
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

</storycode:copy>
<a id="f-a12307572b4c-REFRESH_DELAY_MS"></a>
### `REFRESH_DELAY_MS`

<storycode:copy id="f-a12307572b4c-REFRESH_DELAY_MS">
const REFRESH_DELAY_MS = 250;

</storycode:copy>
<a id="f-a12307572b4c-lspFeatures"></a>
### `lspFeatures`

<storycode:copy id="f-a12307572b4c-lspFeatures">
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

</storycode:copy>
<a id="f-a12307572b4c-refreshPlugin"></a>
### `refreshPlugin`

Re-ask the server after the document settles.
Debounced because these are whole-document requests: asking on every
keystroke would queue work the next keystroke invalidates, and the tokens
that eventually arrived would be for text nobody is looking at any more.

<storycode:copy id="f-a12307572b4c-refreshPlugin">
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

</storycode:copy>
<a id="f-a12307572b4c-setSignatureHelp"></a>
### `setSignatureHelp`

--- signature help ---------------------------------------------------------
The signature of the call the cursor is inside, shown while typing
arguments.
Triggered by the characters that open and separate an argument list rather
than by every keystroke: those are the moments the answer changes, and
asking otherwise is a request per character for an unchanged reply.

<storycode:copy id="f-a12307572b4c-setSignatureHelp">
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
</storycode:copy>
<a id="f-a12307572b4c-signatureField"></a>
### `signatureField`

<storycode:copy id="f-a12307572b4c-signatureField">
export const signatureField = StateField.define<{ label: string; source: Text; at: number } | null>({
  create: () => null,
  update(value, transaction) {
    if (transaction.docChanged || transaction.selection) value = null;
    for (const effect of transaction.effects) if (effect.is(setSignatureHelp)) value = effect.value && effect.value.source === transaction.state.doc ? effect.value : null;
    return value;
  },
});

</storycode:copy>
<a id="f-a12307572b4c-signatureRenderer"></a>
### `signatureRenderer`

Preserve the existing bottom-pinned human signature surface. Its renderer
owns its node; model readings use the field, never inspect this DOM.

<storycode:copy id="f-a12307572b4c-signatureRenderer">
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


</storycode:copy>
<a id="f-a12307572b4c-signatureHelpTooltip"></a>
### `signatureHelpTooltip`

<storycode:copy id="f-a12307572b4c-signatureHelpTooltip">
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

</storycode:copy>
<a id="f-a12307572b4c-annotationReading"></a>
### `annotationReading`

<storycode:copy id="f-a12307572b4c-annotationReading">
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

</storycode:copy>
<a id="f-a12307572b4c-signatureLabel"></a>
### `signatureLabel`

The active signature's label, with the active parameter marked in text.

<storycode:copy id="f-a12307572b4c-signatureLabel">
/** The active signature's label, with the active parameter marked in text. */
export function signatureLabel(help: {
  signatures: { label: string; activeParameter?: number }[];
  activeSignature?: number;
}): string {
  const signature = help.signatures?.[help.activeSignature ?? 0];
  return signature?.label ?? "";
}

</storycode:copy>
<a id="f-a12307572b4c-foldFromServer"></a>
### `foldFromServer`

--- folding ----------------------------------------------------------------
Fold where the server says a construct is, not where the indentation
suggests it might be.
CodeMirror asks for one line at a time and wants an answer synchronously,
so the ranges are fetched in the background and answered from the last set
received. A fold that appears a moment after the document settles is
better than one computed from a grammar the document does not have.

<storycode:copy id="f-a12307572b4c-foldFromServer">
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

</storycode:copy>
<a id="f-a12307572b4c-FoldSpan"></a>
### `FoldSpan`

<storycode:copy id="f-a12307572b4c-FoldSpan">
interface FoldSpan {
  from: number;
  to: number;
}

</storycode:copy>
<a id="f-a12307572b4c-foldServiceFor"></a>
### `foldServiceFor`

CodeMirror's fold service, answered from whatever the server last sent.

<storycode:copy id="f-a12307572b4c-foldServiceFor">
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

</storycode:copy>
<a id="f-a12307572b4c-semanticLegend"></a>
### `semanticLegend`

The legend the bridge announced, or null if the server cannot colour.

<storycode:copy id="f-a12307572b4c-semanticLegend">
/** The legend the bridge announced, or null if the server cannot colour. */
export function semanticLegend(client: LspClient): SemanticLegend | null {
  const legend = client.serverCapabilities?.semanticTokensProvider?.legend;
  if (!legend?.tokenTypes) return null;
  return {
    tokenTypes: legend.tokenTypes,
    tokenModifiers: legend.tokenModifiers ?? [],
  };
}

</storycode:copy>
<a id="f-a12307572b4c-renameAt"></a>
### `renameAt`

<storycode:copy id="f-a12307572b4c-renameAt">
function renameAt(view: EditorView, opts: LspFeatureOptions): boolean {
  if (!opts.client || !opts.onRename || view.state.readOnly || !view.state.facet(EditorView.editable) || !opts.positionAt(view.state.selection.main.head, view)) return false;
  void renameSymbol(view, opts, opts.onRename);
  return true;
}

</storycode:copy>
<a id="f-a12307572b4c-renameSymbol"></a>
### `renameSymbol`

<storycode:copy id="f-a12307572b4c-renameSymbol">
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

</storycode:copy>
<a id="f-a12307572b4c-formatter"></a>
### `formatter`

--- formatting -------------------------------------------------------------
How to format the buffer this extension is in, for a caller outside the
editor — the workspace's Save, when format-on-save is on. A facet rather
than an exported function because the caller has the view and not the
options the view was built with.

<storycode:copy id="f-a12307572b4c-formatter">
// --- formatting -------------------------------------------------------------

/**
 * How to format the buffer this extension is in, for a caller outside the
 * editor — the workspace's Save, when format-on-save is on. A facet rather
 * than an exported function because the caller has the view and not the
 * options the view was built with.
 */
export const formatter = Facet.define<(view: EditorView) => Promise<boolean>>();

</storycode:copy>
<a id="f-a12307572b4c-formatView"></a>
### `formatView`

Format `view` through whatever formatter its extensions provide. Resolves
false when there is none, or the server had nothing to say.

<storycode:copy id="f-a12307572b4c-formatView">
/** Format `view` through whatever formatter its extensions provide. Resolves
 * false when there is none, or the server had nothing to say. */
export async function formatView(view: EditorView): Promise<boolean> {
  for (const format of view.state.facet(formatter)) if (await format(view)) return true;
  return false;
}

</storycode:copy>
<a id="f-a12307572b4c-formatDocument"></a>
### `formatDocument`

Ask the server for formatting and validate the entire original-coordinate
batch before one native editor transaction. Mark the real input event so
existing editor save/provenance policies process the result.

<storycode:copy id="f-a12307572b4c-formatDocument">
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

</storycode:copy>
<a id="f-a12307572b4c-applyLanguageBatch"></a>
### `applyLanguageBatch`

Validate all mapped edits before dispatching one source transaction.

<storycode:copy id="f-a12307572b4c-applyLanguageBatch">
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

</storycode:copy>
<a id="f-a12307572b4c-applyEdit"></a>
### `applyEdit`

<storycode:copy id="f-a12307572b4c-applyEdit">
function applyEdit(view: EditorView, edit: WorkspaceEdit | null, opts: LspFeatureOptions): boolean {
  const edits = editsForUri(edit, opts.uri);
  if (!edits.length) { opts.onMessage?.("The server made no changes."); return false; }
  const changed = applyLanguageBatch(view, edits, opts, "input.lsp");
  const elsewhere = urisInEdit(edit).filter(other => !sameDocumentUri(other, opts.uri));
  if (elsewhere.length) opts.onMessage?.(`Also changes ${elsewhere.length} other document${elsewhere.length === 1 ? "" : "s"}: ${elsewhere.join(", ")}`);
  return changed;
}
</storycode:copy>

### Assemble `apps/web/src/lsp/cmLspFeatures.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/cmLspFeatures.ts"><storycode:paste select="#f-a12307572b4c-Span"/><storycode:paste select="#f-a12307572b4c-setSemanticTokens"/><storycode:paste select="#f-a12307572b4c-semanticTokenField"/><storycode:paste select="#f-a12307572b4c-setHighlights"/><storycode:paste select="#f-a12307572b4c-highlightField"/><storycode:paste select="#f-a12307572b4c-buildMarks"/><storycode:paste select="#f-a12307572b4c-InlayWidget"/><storycode:paste select="#f-a12307572b4c-setInlayHints"/><storycode:paste select="#f-a12307572b4c-inlayHintField"/><storycode:paste select="#f-a12307572b4c-inlayText"/><storycode:paste select="#f-a12307572b4c-editsForUri"/><storycode:paste select="#f-a12307572b4c-urisInEdit"/><storycode:paste select="#f-a12307572b4c-LspFeatureOptions"/><storycode:paste select="#f-a12307572b4c-REFRESH_DELAY_MS"/><storycode:paste select="#f-a12307572b4c-lspFeatures"/><storycode:paste select="#f-a12307572b4c-refreshPlugin"/><storycode:paste select="#f-a12307572b4c-setSignatureHelp"/><storycode:paste select="#f-a12307572b4c-signatureField"/><storycode:paste select="#f-a12307572b4c-signatureRenderer"/><storycode:paste select="#f-a12307572b4c-signatureHelpTooltip"/><storycode:paste select="#f-a12307572b4c-annotationReading"/><storycode:paste select="#f-a12307572b4c-signatureLabel"/><storycode:paste select="#f-a12307572b4c-foldFromServer"/><storycode:paste select="#f-a12307572b4c-FoldSpan"/><storycode:paste select="#f-a12307572b4c-foldServiceFor"/><storycode:paste select="#f-a12307572b4c-semanticLegend"/><storycode:paste select="#f-a12307572b4c-renameAt"/><storycode:paste select="#f-a12307572b4c-renameSymbol"/><storycode:paste select="#f-a12307572b4c-formatter"/><storycode:paste select="#f-a12307572b4c-formatView"/><storycode:paste select="#f-a12307572b4c-formatDocument"/><storycode:paste select="#f-a12307572b4c-applyLanguageBatch"/><storycode:paste select="#f-a12307572b4c-applyEdit"/></storycode:file>

## Test the behavior: `a_plain_rust_file_formats_with_rustfmt_through_the_language_server`

This is the maintained test fragment from `crates/hickory-cli/tests/lsp_languages.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-7c4650d33fc8-a_plain_rust_file_formats_with_rustfmt_through_the_language_server"></a>
### `a_plain_rust_file_formats_with_rustfmt_through_the_language_server`

Protects docs/guarantees/editor-intelligence/save-can-format-first.md

<storycode:copy id="f-7c4650d33fc8-a_plain_rust_file_formats_with_rustfmt_through_the_language_server">
/// Protects docs/guarantees/editor-intelligence/save-can-format-first.md
#[test]
fn a_plain_rust_file_formats_with_rustfmt_through_the_language_server() {
    if cfg!(windows) {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    let file = dir.path().join("src/main.rs");
    let ugly = "fn main(){println!(\"x\");}\n";
    std::fs::write(&file, ugly).unwrap();
    if !server_available("rust", dir.path())
        || binary("rustfmt").is_none() && which("rustfmt").is_none()
    {
        skip(
            "rust",
            "rust-analyzer or rustfmt is not installed on this machine",
        );
        return;
    }
    let Some(mut client) = StdioClient::start(dir.path()) else {
        skip("rust", "hick-lsp has not been built into this target dir");
        return;
    };
    let root_uri = format!("file://{}", dir.path().display());
    let uri = format!("file://{}", file.display());
    let id = client.request(
        "initialize",
        json!({ "processId": std::process::id(), "rootUri": root_uri, "capabilities": {} }),
    );
    let capabilities = client
        .wait_for(id, Duration::from_secs(20))
        .expect("initialize");
    assert!(
        capabilities["capabilities"]["documentFormattingProvider"] != Value::Null,
        "formatting was not advertised: {capabilities}"
    );
    client.notify("initialized", json!({}));
    client.notify(
        "textDocument/didOpen",
        json!({"textDocument": {"uri": uri, "languageId": "rust", "version": 1, "text": ugly}}),
    );
    let edits = client.request_until(
        "textDocument/formatting",
        json!({"textDocument": {"uri": uri}, "options": {"tabSize": 4, "insertSpaces": true}}),
        Duration::from_secs(60),
    );
    // rustfmt answers with the smallest edits that get there, so the proof
    // is the text they produce rather than any one of them.
    let edits = edits.expect("formatting is answered");
    let formatted = apply_edits(ugly, &edits);
    assert_eq!(
        formatted, "fn main() {\n    println!(\"x\");\n}\n",
        "rustfmt's edits did not come back through the server: {edits}"
    );
    eprintln!("OK rust: a plain file formats with rustfmt");
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/lib/formatOnSave.ts](contract.md#output-c6800936a2be)
- [apps/web/src/lsp/client.ts](contract.md#output-6bc953112fd0)
- [apps/web/src/lsp/cmLspFeatures.ts](contract.md#output-a12307572b4c)
- [apps/web/src/views/SettingsView.tsx](../../../mechanisms/apps/web/src/views/SettingsView.tsx.md#output-38894484eb8f)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [crates/hick-lsp/src/backend.rs](../../../mechanisms/crates/hick-lsp/src/backend.rs.md#output-c936496a03c1)
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)
- [crates/hickory-cli/src/serve/lsp_bridge.rs](../../../mechanisms/crates/hickory-cli/src/serve/lsp_bridge.rs.md#output-a429fedbabb9)
- [crates/hickory-cli/src/serve/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/mod.rs.md#output-09ea558208cd)

### Tests and fixtures

- [apps/web/src/lsp/cmLspFeatures.test.ts](contract.md#output-830f6d637cff)
- [apps/web/src/views/SettingsView.test.tsx](../../../mechanisms/apps/web/src/views/SettingsView.tsx.md#output-5bfac230ae5d)
- [crates/hickory-cli/tests/lsp_languages.rs](../../../mechanisms/crates/hickory-cli/tests/lsp_languages.rs.md#output-7c4650d33fc8)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
