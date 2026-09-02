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

export interface LspDiagnostic {
  range: LspRange;
  severity?: number;
  source?: string;
  message: string;
}

export interface PublishDiagnosticsParams {
  uri: string;
  diagnostics: LspDiagnostic[];
}

export interface HoverResult {
  contents: unknown;
  range?: LspRange;
}

export interface CompletionItem {
  label: string;
  kind?: number;
  detail?: string;
  insertText?: string;
  [key: string]: unknown;
}

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

export interface InlayHint {
  position: LspPosition;
  label: string | { value: string }[];
  kind?: number;
  paddingLeft?: boolean;
  paddingRight?: boolean;
}

export interface FoldingRange {
  startLine: number;
  endLine: number;
  startCharacter?: number;
  endCharacter?: number;
  kind?: string;
}

export interface CodeAction {
  title: string;
  kind?: string;
  edit?: WorkspaceEdit;
  command?: { title: string; command: string; arguments?: unknown[] };
}

/** One replacement in one document. */
export interface TextEdit {
  range: LspRange;
  newText: string;
}

export interface WorkspaceEdit {
  changes?: Record<string, { range: LspRange; newText: string }[]>;
  documentChanges?: {
    textDocument?: { uri: string; version?: number | null };
    edits?: { range: LspRange; newText: string }[];
  }[];
}

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

type Pending = {
  resolve: (value: unknown) => void;
  reject: (err: Error) => void;
  timer: ReturnType<typeof setTimeout>;
};

export class LspRequestError extends Error {
  constructor(
    public readonly code: number,
    message: string,
  ) {
    super(message);
    this.name = "LspRequestError";
  }
}

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

export class LspClient {
  private nextId = 1;
  private pending = new Map<number, Pending>();
  private diagnosticsListeners = new Set<(params: PublishDiagnosticsParams) => void>();
  private capabilityListeners = new Set<(capabilities: ServerCapabilities) => void>();
  private capabilities: ServerCapabilities | null = null;
  private unsubscribe: () => void;
  private disposed = false;

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
    this.diagnosticsListeners.clear();
    this.capabilityListeners.clear();
  }
}
