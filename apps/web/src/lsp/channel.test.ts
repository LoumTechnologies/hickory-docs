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
      params: { textDocument: { uri: "hick:///guide.hick" }, position: { line: 3, character: 1 } },
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
