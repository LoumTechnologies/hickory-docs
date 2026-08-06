// Channel 0x02 framing for the LSP bridge (api.md v0.3): each WebSocket
// binary frame is a 1-byte channel prefix + exactly one JSON-RPC 2.0 message
// in UTF-8 (no Content-Length headers).
//
// This module deliberately does NOT import the realtime socket: it talks to a
// narrow `LspWire` interface. The integration step wires it up by handing the
// realtime layer's raw-frame send/receive hooks to `createLspChannel`.

export const CHANNEL_LSP = 0x02;

/** JSON-RPC 2.0 message shapes carried on the channel. */
export interface JsonRpcRequest {
  jsonrpc: "2.0";
  id: number | string;
  method: string;
  params?: unknown;
}
export interface JsonRpcNotification {
  jsonrpc: "2.0";
  method: string;
  params?: unknown;
}
export interface JsonRpcResponse {
  jsonrpc: "2.0";
  id: number | string | null;
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
}
export type JsonRpcMessage = JsonRpcRequest | JsonRpcNotification | JsonRpcResponse;

/** The only thing the channel needs from the socket: raw binary frame out. */
export interface LspWire {
  send(frame: Uint8Array): void;
}

/** Encode one JSON-RPC message as a 0x02-prefixed frame. */
export function encodeLspFrame(message: JsonRpcMessage): Uint8Array {
  const json = new TextEncoder().encode(JSON.stringify(message));
  const frame = new Uint8Array(json.length + 1);
  frame[0] = CHANNEL_LSP;
  frame.set(json, 1);
  return frame;
}

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
