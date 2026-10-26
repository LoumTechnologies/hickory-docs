<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lsp/channel.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9cdcca2616ec"></a>
## `apps/web/src/lsp/channel.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-9cdcca2616ec-CHANNEL_LSP"></a>
### `CHANNEL_LSP`

Channel 0x02 framing for the LSP bridge (api.md v0.3): each WebSocket
binary frame is a 1-byte channel prefix + exactly one JSON-RPC 2.0 message
in UTF-8 (no Content-Length headers).
This module deliberately does NOT import the realtime socket: it talks to a
narrow `LspWire` interface. The integration step wires it up by handing the
realtime layer's raw-frame send/receive hooks to `createLspChannel`.

<storycode:copy id="f-9cdcca2616ec-CHANNEL_LSP">
// Channel 0x02 framing for the LSP bridge (api.md v0.3): each WebSocket
// binary frame is a 1-byte channel prefix + exactly one JSON-RPC 2.0 message
// in UTF-8 (no Content-Length headers).
//
// This module deliberately does NOT import the realtime socket: it talks to a
// narrow `LspWire` interface. The integration step wires it up by handing the
// realtime layer's raw-frame send/receive hooks to `createLspChannel`.

export const CHANNEL_LSP = 0x02;

</storycode:copy>
<a id="f-9cdcca2616ec-JsonRpcRequest"></a>
### `JsonRpcRequest`

JSON-RPC 2.0 message shapes carried on the channel.

<storycode:copy id="f-9cdcca2616ec-JsonRpcRequest">
/** JSON-RPC 2.0 message shapes carried on the channel. */
export interface JsonRpcRequest {
  jsonrpc: "2.0";
  id: number | string;
  method: string;
  params?: unknown;
}
</storycode:copy>
<a id="f-9cdcca2616ec-JsonRpcNotification"></a>
### `JsonRpcNotification`

<storycode:copy id="f-9cdcca2616ec-JsonRpcNotification">
export interface JsonRpcNotification {
  jsonrpc: "2.0";
  method: string;
  params?: unknown;
}
</storycode:copy>
<a id="f-9cdcca2616ec-JsonRpcResponse"></a>
### `JsonRpcResponse`

<storycode:copy id="f-9cdcca2616ec-JsonRpcResponse">
export interface JsonRpcResponse {
  jsonrpc: "2.0";
  id: number | string | null;
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
}
</storycode:copy>
<a id="f-9cdcca2616ec-JsonRpcMessage"></a>
### `JsonRpcMessage`

<storycode:copy id="f-9cdcca2616ec-JsonRpcMessage">
export type JsonRpcMessage = JsonRpcRequest | JsonRpcNotification | JsonRpcResponse;

</storycode:copy>
<a id="f-9cdcca2616ec-LspWire"></a>
### `LspWire`

The only thing the channel needs from the socket: raw binary frame out.

<storycode:copy id="f-9cdcca2616ec-LspWire">
/** The only thing the channel needs from the socket: raw binary frame out. */
export interface LspWire {
  send(frame: Uint8Array): void;
}

</storycode:copy>
<a id="f-9cdcca2616ec-encodeLspFrame"></a>
### `encodeLspFrame`

Encode one JSON-RPC message as a 0x02-prefixed frame.

<storycode:copy id="f-9cdcca2616ec-encodeLspFrame">
/** Encode one JSON-RPC message as a 0x02-prefixed frame. */
export function encodeLspFrame(message: JsonRpcMessage): Uint8Array {
  const json = new TextEncoder().encode(JSON.stringify(message));
  const frame = new Uint8Array(json.length + 1);
  frame[0] = CHANNEL_LSP;
  frame.set(json, 1);
  return frame;
}

</storycode:copy>
<a id="f-9cdcca2616ec-decodeLspFrame"></a>
### `decodeLspFrame`

Decode a 0x02 frame back into its JSON-RPC message. Returns `null` for
frames on other channels or unparseable payloads.

<storycode:copy id="f-9cdcca2616ec-decodeLspFrame">
/**
 * Decode a 0x02 frame back into its JSON-RPC message. Returns `null` for
 * frames on other channels or unparseable payloads.
 */
export function decodeLspFrame(frame: Uint8Array): JsonRpcMessage | null {
  if (frame.length < 2 || frame[0] !== CHANNEL_LSP) return null;
  try {
    return JSON.parse(new TextDecoder().decode(frame.subarray(1))) as JsonRpcMessage;
  } catch {
    return null;
  }
}

</storycode:copy>
<a id="f-9cdcca2616ec-LspChannel"></a>
### `LspChannel`

Bidirectional message pipe the `LspClient` sits on.

<storycode:copy id="f-9cdcca2616ec-LspChannel">
/** Bidirectional message pipe the `LspClient` sits on. */
export interface LspChannel {
  send(message: JsonRpcMessage): void;
  /** Subscribe to inbound messages. Returns an unsubscribe fn. */
  onMessage(cb: (message: JsonRpcMessage) => void): () => void;
  /**
   * Feed a raw socket frame in. Returns true when the frame was a 0x02 frame
   * and has been consumed (so the caller can skip its other channel handlers).
   */
  handleFrame(frame: Uint8Array): boolean;
}

</storycode:copy>
<a id="f-9cdcca2616ec-createLspChannel"></a>
### `createLspChannel`

Create the channel over a wire. The integration step calls
`channel.handleFrame(bytes)` from the socket's onmessage for every binary
frame; non-LSP frames are left untouched (returns false).

<storycode:copy id="f-9cdcca2616ec-createLspChannel">
/**
 * Create the channel over a wire. The integration step calls
 * `channel.handleFrame(bytes)` from the socket's onmessage for every binary
 * frame; non-LSP frames are left untouched (returns false).
 */
export function createLspChannel(wire: LspWire): LspChannel {
  const listeners = new Set<(message: JsonRpcMessage) => void>();
  return {
    send(message) {
      wire.send(encodeLspFrame(message));
    },
    onMessage(cb) {
      listeners.add(cb);
      return () => listeners.delete(cb);
    },
    handleFrame(frame) {
      if (frame.length === 0 || frame[0] !== CHANNEL_LSP) return false;
      const message = decodeLspFrame(frame);
      if (message !== null) for (const cb of listeners) cb(message);
      return true;
    },
  };
}
</storycode:copy>

### Assemble `apps/web/src/lsp/channel.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/channel.ts"><storycode:paste select="#f-9cdcca2616ec-CHANNEL_LSP"/><storycode:paste select="#f-9cdcca2616ec-JsonRpcRequest"/><storycode:paste select="#f-9cdcca2616ec-JsonRpcNotification"/><storycode:paste select="#f-9cdcca2616ec-JsonRpcResponse"/><storycode:paste select="#f-9cdcca2616ec-JsonRpcMessage"/><storycode:paste select="#f-9cdcca2616ec-LspWire"/><storycode:paste select="#f-9cdcca2616ec-encodeLspFrame"/><storycode:paste select="#f-9cdcca2616ec-decodeLspFrame"/><storycode:paste select="#f-9cdcca2616ec-LspChannel"/><storycode:paste select="#f-9cdcca2616ec-createLspChannel"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-df96879f2cc3"></a>
## `apps/web/src/lsp/channel.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-df96879f2cc3-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-df96879f2cc3-expression_statement">
import { describe, expect, it } from "vitest";
import {
  CHANNEL_LSP,
  createLspChannel,
  decodeLspFrame,
  encodeLspFrame,
  type JsonRpcMessage,
} from "./channel";

describe("0x02 framing", () => {
  it("round-trips a JSON-RPC message", () => {
    const msg: JsonRpcMessage = {
      jsonrpc: "2.0",
      id: 7,
      method: "textDocument/hover",
      params: { textDocument: { uri: "hick:///guide.md" }, position: { line: 3, character: 1 } },
    };
    const frame = encodeLspFrame(msg);
    expect(frame[0]).toBe(CHANNEL_LSP);
    expect(decodeLspFrame(frame)).toEqual(msg);
  });

  it("handles multi-byte UTF-8 payloads", () => {
    const msg: JsonRpcMessage = { jsonrpc: "2.0", method: "note", params: { text: "héllo — 𝄞€" } };
    expect(decodeLspFrame(encodeLspFrame(msg))).toEqual(msg);
  });

  it("rejects frames on other channels", () => {
    const yjs = new Uint8Array([0x00, 1, 2, 3]);
    const run = new Uint8Array([0x01, 123]);
    expect(decodeLspFrame(yjs)).toBeNull();
    expect(decodeLspFrame(run)).toBeNull();
    expect(decodeLspFrame(new Uint8Array([]))).toBeNull();
  });

  it("returns null for unparseable payloads", () => {
    const bad = new Uint8Array([CHANNEL_LSP, 0x7b, 0x7b]); // "{{"
    expect(decodeLspFrame(bad)).toBeNull();
  });
});

</storycode:copy>
<a id="f-df96879f2cc3-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-df96879f2cc3-expression_statement-2">
describe("createLspChannel", () => {
  it("sends framed messages over the wire", () => {
    const sent: Uint8Array[] = [];
    const channel = createLspChannel({ send: (f) => sent.push(f) });
    channel.send({ jsonrpc: "2.0", id: 1, method: "shutdown" });
    expect(sent).toHaveLength(1);
    expect(sent[0][0]).toBe(CHANNEL_LSP);
    expect(decodeLspFrame(sent[0])).toEqual({ jsonrpc: "2.0", id: 1, method: "shutdown" });
  });

  it("dispatches inbound 0x02 frames and ignores other channels", () => {
    const channel = createLspChannel({ send: () => {} });
    const received: JsonRpcMessage[] = [];
    const off = channel.onMessage((m) => received.push(m));

    const lspMsg: JsonRpcMessage = { jsonrpc: "2.0", id: 2, result: null };
    expect(channel.handleFrame(encodeLspFrame(lspMsg))).toBe(true);
    expect(channel.handleFrame(new Uint8Array([0x00, 9, 9]))).toBe(false);
    expect(channel.handleFrame(new Uint8Array([0x01, 9]))).toBe(false);
    expect(received).toEqual([lspMsg]);

    off();
    channel.handleFrame(encodeLspFrame(lspMsg));
    expect(received).toHaveLength(1);
  });

  it("consumes malformed 0x02 frames without dispatching", () => {
    const channel = createLspChannel({ send: () => {} });
    const received: JsonRpcMessage[] = [];
    channel.onMessage((m) => received.push(m));
    expect(channel.handleFrame(new Uint8Array([CHANNEL_LSP, 0x21]))).toBe(true);
    expect(received).toHaveLength(0);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lsp/channel.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/channel.test.ts"><storycode:paste select="#f-df96879f2cc3-expression_statement"/><storycode:paste select="#f-df96879f2cc3-expression_statement-2"/></storycode:file>


</storycode:doc>
