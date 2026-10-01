import { describe, expect, it, vi } from "vitest";
import { createBrowserTsChannel, locateBlock, toDocument, toVirtual, virtualFileOf } from "./browserTsServer";
import type { JsonRpcMessage } from "../../lsp/channel";

const DOCUMENT = [
  `<?xml version="1.0" encoding="UTF-8"?>`,
  `<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">`,
  `# Pricing`,
  ``,
  `<hick:file path="pricing.ts">`,
  `export function double(value: number): number {`,
  `  return value * 2;`,
  `}`,
  `</hick:file>`,
  `</hick:doc>`,
].join("\n");

/** 0-based document line of `export function double…`. */
const DEFINITION_LINE = 5;

describe("locating the block inside a document", () => {
  it("finds where the code starts, ends and what it is called", () => {
    const block = locateBlock(DOCUMENT);
    expect(block).toEqual({ firstLine: DEFINITION_LINE, lineCount: 3, path: "pricing.ts" });
  });

  it("gives the language service only the code, not the document", () => {
    // The compiler must never see the XML around the block: it would report a
    // syntax error on line 1 of a document that is perfectly valid.
    const block = locateBlock(DOCUMENT)!;
    const virtual = virtualFileOf(DOCUMENT, block);
    expect(virtual.startsWith("export function double")).toBe(true);
    expect(virtual).not.toContain("hick:doc");
  });

  it("reports nothing for a document with no code in it", () => {
    expect(locateBlock("# Just prose\n")).toBeNull();
  });
});

describe("coordinates", () => {
  const block = locateBlock(DOCUMENT)!;

  it("maps a document position into the block", () => {
    expect(toVirtual({ line: DEFINITION_LINE, character: 16 }, block)).toEqual({
      line: 0,
      character: 16,
    });
  });

  it("refuses a position in the prose above the block", () => {
    // Answering for prose would put a hover about TypeScript on a heading.
    expect(toVirtual({ line: 2, character: 0 }, block)).toBeNull();
  });

  it("refuses a position below the block", () => {
    expect(toVirtual({ line: 9, character: 0 }, block)).toBeNull();
  });

  it("round-trips back to the document line the reader is looking at", () => {
    const virtual = toVirtual({ line: DEFINITION_LINE + 1, character: 2 }, block)!;
    expect(toDocument(virtual, block)).toEqual({ line: DEFINITION_LINE + 1, character: 2 });
  });
});

/** Drive the channel the way `LspClient` does, and collect what comes back. */
async function open(document = DOCUMENT) {
  const received: JsonRpcMessage[] = [];
  const ready = vi.fn();
  const channel = createBrowserTsChannel({ document, onReady: ready });
  channel.onMessage((message) => received.push(message));
  // The compiler is imported dynamically; give it a moment to land.
  for (let i = 0; i < 100 && ready.mock.calls.length === 0; i++) {
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  return { channel, received };
}

function replyTo(received: JsonRpcMessage[], id: number) {
  return received.find(
    (message) => "id" in message && message.id === id && "result" in message,
  ) as { result: unknown } | undefined;
}

describe("the compiler, answering through the channel", () => {
  it("hovers a function with its real signature", async () => {
    const { channel, received } = await open();
    channel.send({
      jsonrpc: "2.0",
      id: 1,
      method: "textDocument/hover",
      params: {
        textDocument: { uri: "hick:///doc.md" },
        position: { line: DEFINITION_LINE, character: 18 },
      },
    });
    const result = replyTo(received, 1)?.result as { contents?: { value?: string } };
    expect(result?.contents?.value).toContain("double");
    expect(result?.contents?.value).toContain("number");
    channel.dispose();
  }, 20_000);

  it("reports a real type error, at the document's line", async () => {
    // The demo's whole claim: break the code and the compiler notices. A
    // string where a number belongs is the smallest honest break.
    const broken = DOCUMENT.replace("return value * 2;", 'return value * "two";');
    const { channel, received } = await open(broken);
    channel.send({
      jsonrpc: "2.0",
      method: "textDocument/didOpen",
      params: { textDocument: { uri: "hick:///doc.md" } },
    });
    const published = received.find(
      (message) => "method" in message && message.method === "textDocument/publishDiagnostics",
    ) as { params: { diagnostics: { message: string; range: { start: { line: number } } }[] } };
    expect(published.params.diagnostics.length).toBeGreaterThan(0);
    // In DOCUMENT coordinates: the line the reader is looking at, not line 1
    // of a virtual file they cannot see.
    expect(published.params.diagnostics[0].range.start.line).toBe(DEFINITION_LINE + 1);
    channel.dispose();
  }, 20_000);

  it("says nothing is wrong with code that is fine", async () => {
    const { channel, received } = await open();
    channel.send({
      jsonrpc: "2.0",
      method: "textDocument/didOpen",
      params: { textDocument: { uri: "hick:///doc.md" } },
    });
    const published = received.find(
      (message) => "method" in message && message.method === "textDocument/publishDiagnostics",
    ) as { params: { diagnostics: unknown[] } };
    // Without the standard library the compiler has no `Array` and invents
    // errors in correct code — the worst possible first impression, and the
    // reason the lib is shipped with the demo.
    expect(published.params.diagnostics).toEqual([]);
    channel.dispose();
  }, 20_000);

  it("colours the code, in document coordinates", async () => {
    const { channel, received } = await open();
    channel.send({
      jsonrpc: "2.0",
      id: 2,
      method: "textDocument/semanticTokens/full",
      params: { textDocument: { uri: "hick:///doc.md" } },
    });
    const data = (replyTo(received, 2)?.result as { data: number[] }).data;
    expect(data.length).toBeGreaterThan(0);
    expect(data.length % 5).toBe(0);
    // Every delta is non-negative, or the protocol cannot express it.
    for (let i = 0; i < data.length; i += 5) expect(data[i]).toBeGreaterThanOrEqual(0);
    channel.dispose();
  }, 20_000);

  it("colours the code, not the document around it", async () => {
    // The bug this pins: the tokens came back in VIRTUAL-file coordinates
    // while the client — which does no mapping, because hick-lsp maps
    // server-side — read them as document lines. Every token landed about
    // five lines high, so the demo coloured `version=` in the XML header as
    // an interface and a line of prose as a property. It looked like syntax
    // highlighting right up until you read which words it had picked.
    const { channel, received } = await open();
    channel.send({
      jsonrpc: "2.0",
      id: 9,
      method: "textDocument/semanticTokens/full",
      params: { textDocument: { uri: "hick:///doc.md" } },
    });
    const data = (replyTo(received, 9)?.result as { data: number[] }).data;
    // The first token's line delta is absolute, and every token must fall
    // inside the block — never in the prose or the XML above it.
    expect(data[0]).toBeGreaterThanOrEqual(DEFINITION_LINE);
    let line = 0;
    for (let i = 0; i < data.length; i += 5) {
      line += data[i];
      expect(line).toBeGreaterThanOrEqual(DEFINITION_LINE);
      expect(line).toBeLessThan(DEFINITION_LINE + 3);
    }
    channel.dispose();
  }, 20_000);

  it("announces its legend, because token types are integers", async () => {
    const { channel, received } = await open();
    channel.send({ jsonrpc: "2.0", id: 3, method: "initialize", params: {} });
    const announcement = received.find(
      (message) => "method" in message && message.method === "hick/serverCapabilities",
    ) as { params: { capabilities: { semanticTokensProvider: { legend: { tokenTypes: string[] } } } } };
    expect(announcement.params.capabilities.semanticTokensProvider.legend.tokenTypes).toContain(
      "function",
    );
    channel.dispose();
  }, 20_000);

  it("answers nothing, rather than erroring, for what it cannot do", async () => {
    // The degradation rule: an editor asking for a feature this server lacks
    // must get a reply, so the request does not hang forever.
    const { channel, received } = await open();
    channel.send({
      jsonrpc: "2.0",
      id: 4,
      method: "textDocument/codeLens",
      params: { textDocument: { uri: "hick:///doc.md" } },
    });
    expect(replyTo(received, 4)).toBeDefined();
    expect(replyTo(received, 4)?.result).toBeNull();
    channel.dispose();
  }, 20_000);
});
