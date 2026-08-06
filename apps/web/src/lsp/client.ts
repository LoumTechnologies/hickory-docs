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
  }
}
