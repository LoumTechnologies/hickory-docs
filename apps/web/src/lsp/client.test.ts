import { describe, expect, it } from "vitest";
import { createLspChannel, encodeLspFrame, type JsonRpcMessage } from "./channel";
import { LspClient, normalizeLocations } from "./client";

/** A channel over a loopback wire plus a hook to inject server messages. */
function loopback() {
  const outbound: JsonRpcMessage[] = [];
  const channel = createLspChannel({
    send: (frame) => {
      outbound.push(JSON.parse(new TextDecoder().decode(frame.subarray(1))));
    },
  });
  const inject = (msg: JsonRpcMessage) => channel.handleFrame(encodeLspFrame(msg));
  return { channel, outbound, inject };
}

describe("LspClient", () => {
  it("matches responses to requests by id", async () => {
    const { channel, outbound, inject } = loopback();
    const client = new LspClient(channel);

    const hover = client.hover("hick:///a.md", { line: 1, character: 2 });
    const refs = client.references("hick:///a.md", { line: 1, character: 2 });

    expect(outbound).toHaveLength(2);
    const [hoverReq, refsReq] = outbound as Array<{ id: number; method: string; params: unknown }>;
    expect(hoverReq.method).toBe("textDocument/hover");
    expect(refsReq.method).toBe("textDocument/references");
    expect(refsReq.params).toMatchObject({ context: { includeDeclaration: true } });

    // Answer out of order.
    inject({
      jsonrpc: "2.0",
      id: refsReq.id,
      result: [
        {
          uri: "hick:///a.md",
          range: { start: { line: 0, character: 0 }, end: { line: 0, character: 3 } },
        },
      ],
    });
    inject({ jsonrpc: "2.0", id: hoverReq.id, result: null });

    expect(await hover).toBeNull();
    expect(await refs).toEqual([
      {
        uri: "hick:///a.md",
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 3 } },
      },
    ]);
    client.dispose();
  });

  it("didOpen/didChange are notifications with full text", () => {
    const { channel, outbound } = loopback();
    const client = new LspClient(channel);
    client.didOpen("hick:///a.md", "hello");
    client.didChange("hick:///a.md", "hello world", 2);
    expect(outbound[0]).toEqual({
      jsonrpc: "2.0",
      method: "textDocument/didOpen",
      params: { textDocument: { uri: "hick:///a.md", languageId: "hick", version: 1, text: "hello" } },
    });
    expect(outbound[1]).toEqual({
      jsonrpc: "2.0",
      method: "textDocument/didChange",
      params: { textDocument: { uri: "hick:///a.md", version: 2 }, contentChanges: [{ text: "hello world" }] },
    });
    client.dispose();
  });

  it("delivers diagnostics to subscribers until unsubscribed", () => {
    const { channel, inject } = loopback();
    const client = new LspClient(channel);
    const seen: string[] = [];
    const off = client.onDiagnostics((p) => seen.push(p.uri));

    inject({
      jsonrpc: "2.0",
      method: "textDocument/publishDiagnostics",
      params: { uri: "hick:///a.md", diagnostics: [] },
    });
    off();
    inject({
      jsonrpc: "2.0",
      method: "textDocument/publishDiagnostics",
      params: { uri: "hick:///b.md", diagnostics: [] },
    });
    expect(seen).toEqual(["hick:///a.md"]);
    client.dispose();
  });

  it("rejects on JSON-RPC error responses", async () => {
    const { channel, outbound, inject } = loopback();
    const client = new LspClient(channel);
    const req = client.request("textDocument/definition", {});
    const id = (outbound[0] as { id: number }).id;
    inject({ jsonrpc: "2.0", id, error: { code: -32601, message: "nope" } });
    await expect(req).rejects.toThrow("nope");
    client.dispose();
  });

  it("normalizes scalar locations and location links", () => {
    const range = { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } };
    expect(normalizeLocations(null)).toEqual([]);
    expect(normalizeLocations({ uri: "hick:///a", range })).toEqual([{ uri: "hick:///a", range }]);
    expect(
      normalizeLocations([{ targetUri: "hick-output:///main.rs", targetRange: range }]),
    ).toEqual([{ uri: "hick-output:///main.rs", range }]);
  });
});
