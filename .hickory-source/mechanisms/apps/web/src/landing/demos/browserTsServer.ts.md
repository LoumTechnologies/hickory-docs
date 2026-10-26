<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/landing/demos/browserTsServer.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-338b6897f677"></a>
## `apps/web/src/landing/demos/browserTsServer.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-338b6897f677-BlockLocation"></a>
### `BlockLocation`

A real language server for the home page, running in the browser.
The claim the landing page makes about editing a `.md` document — that
the code inside it behaves like code, not like text in a box — is a claim
nobody believes from a screenshot. So the demo does it for real: this is
the actual TypeScript compiler's language service, answering hover,
completion, diagnostics and semantic colouring for the block in the
document above it.
## Why it is an LspChannel rather than a bespoke widget
Because then the demo drives the SAME client the desktop app drives.
`LspClient`, `lspSupport` and `lspFeatures` are unchanged and unaware; only
the thing on the far end of the channel differs — a WebSocket to `hick-lsp`
in the app, this module on the marketing site. A demo that reimplemented
the client would prove the demo works, which is not the interesting claim.
## What is genuinely the same, and what is not
The same: the weaving of a `hick:file` block into a virtual file, the
mapping of every position between document and virtual coordinates, the
JSON-RPC surface, and the answers, which come from `typescript` itself.
Not the same: in the app the servers are whichever ones YOUR machine has,
for whatever languages your document contains. Here it is TypeScript only,
because TypeScript is the one mainstream language whose whole compiler
happens to run in a browser. The page says so rather than implying a
browser could host rust-analyzer.
The compiler is ~2 MB gzipped, so it is imported dynamically: the page
loads without it and fetches it when a reader actually starts typing.

<storycode:copy id="f-338b6897f677-BlockLocation">
// A real language server for the home page, running in the browser.
//
// The claim the landing page makes about editing a `.md` document — that
// the code inside it behaves like code, not like text in a box — is a claim
// nobody believes from a screenshot. So the demo does it for real: this is
// the actual TypeScript compiler's language service, answering hover,
// completion, diagnostics and semantic colouring for the block in the
// document above it.
//
// ## Why it is an LspChannel rather than a bespoke widget
//
// Because then the demo drives the SAME client the desktop app drives.
// `LspClient`, `lspSupport` and `lspFeatures` are unchanged and unaware; only
// the thing on the far end of the channel differs — a WebSocket to `hick-lsp`
// in the app, this module on the marketing site. A demo that reimplemented
// the client would prove the demo works, which is not the interesting claim.
//
// ## What is genuinely the same, and what is not
//
// The same: the weaving of a `hick:file` block into a virtual file, the
// mapping of every position between document and virtual coordinates, the
// JSON-RPC surface, and the answers, which come from `typescript` itself.
//
// Not the same: in the app the servers are whichever ones YOUR machine has,
// for whatever languages your document contains. Here it is TypeScript only,
// because TypeScript is the one mainstream language whose whole compiler
// happens to run in a browser. The page says so rather than implying a
// browser could host rust-analyzer.
//
// The compiler is ~2 MB gzipped, so it is imported dynamically: the page
// loads without it and fetches it when a reader actually starts typing.

import type { JsonRpcMessage, LspChannel } from "../../lsp/channel";
import type { LspPosition } from "../../lsp/positions";

/** Where the block's code sits inside the document. */
export interface BlockLocation {
  /** 0-based document line of the block's first line of code. */
  firstLine: number;
  /** How many lines of code the block has. */
  lineCount: number;
  /** The virtual file's name; its extension is what picks the language. */
  path: string;
}

</storycode:copy>
<a id="f-338b6897f677-locateBlock"></a>
### `locateBlock`

Find the `hick:file` block in a document.
Deliberately a small parser rather than the real one: the demo document is
written right here and has exactly one block. Pulling the whole hick parser
into the marketing bundle to read a document we authored would be paying
for generality nobody uses.

<storycode:copy id="f-338b6897f677-locateBlock">
/**
 * Find the `hick:file` block in a document.
 *
 * Deliberately a small parser rather than the real one: the demo document is
 * written right here and has exactly one block. Pulling the whole hick parser
 * into the marketing bundle to read a document we authored would be paying
 * for generality nobody uses.
 */
export function locateBlock(document: string): BlockLocation | null {
  const lines = document.split("\n");
  const open = lines.findIndex((line) => line.includes("<hick:file"));
  if (open === -1) return null;
  const close = lines.findIndex((line, index) => index > open && line.includes("</hick:file>"));
  if (close === -1) return null;
  const path = /path="([^"]+)"/.exec(lines[open])?.[1] ?? "demo.ts";
  return { firstLine: open + 1, lineCount: close - open - 1, path };
}

</storycode:copy>
<a id="f-338b6897f677-virtualFileOf"></a>
### `virtualFileOf`

The block's code, as the virtual file the language service sees.

<storycode:copy id="f-338b6897f677-virtualFileOf">
/** The block's code, as the virtual file the language service sees. */
export function virtualFileOf(document: string, block: BlockLocation): string {
  return document
    .split("\n")
    .slice(block.firstLine, block.firstLine + block.lineCount)
    .join("\n");
}

</storycode:copy>
<a id="f-338b6897f677-toVirtual"></a>
### `toVirtual`

Document position → virtual file position, or null if outside the block.

<storycode:copy id="f-338b6897f677-toVirtual">
/** Document position → virtual file position, or null if outside the block. */
export function toVirtual(position: LspPosition, block: BlockLocation): LspPosition | null {
  const line = position.line - block.firstLine;
  if (line < 0 || line >= block.lineCount) return null;
  return { line, character: position.character };
}

</storycode:copy>
<a id="f-338b6897f677-toDocument"></a>
### `toDocument`

Virtual file position → document position.

<storycode:copy id="f-338b6897f677-toDocument">
/** Virtual file position → document position. */
export function toDocument(position: LspPosition, block: BlockLocation): LspPosition {
  return { line: position.line + block.firstLine, character: position.character };
}

</storycode:copy>
<a id="f-338b6897f677-DEMO_TOKEN_TYPES"></a>
### `DEMO_TOKEN_TYPES`

The token types this demo reports, in the order the client is told.
A short legend rather than the full LSP list: these are the kinds
TypeScript's classifier actually distinguishes, and a legend containing
entries nothing ever emits would be a promise the demo does not keep.

<storycode:copy id="f-338b6897f677-DEMO_TOKEN_TYPES">
/**
 * The token types this demo reports, in the order the client is told.
 *
 * A short legend rather than the full LSP list: these are the kinds
 * TypeScript's classifier actually distinguishes, and a legend containing
 * entries nothing ever emits would be a promise the demo does not keep.
 */
export const DEMO_TOKEN_TYPES = [
  "keyword",
  "string",
  "number",
  "comment",
  "type",
  "class",
  "interface",
  "enum",
  "typeParameter",
  "function",
  "method",
  "property",
  "variable",
  "parameter",
] as const;

</storycode:copy>
<a id="f-338b6897f677-DEMO_TOKEN_MODIFIERS"></a>
### `DEMO_TOKEN_MODIFIERS`

<storycode:copy id="f-338b6897f677-DEMO_TOKEN_MODIFIERS">
export const DEMO_TOKEN_MODIFIERS = ["declaration", "readonly", "static"] as const;

</storycode:copy>
<a id="f-338b6897f677-Ts"></a>
### `Ts`

<storycode:copy id="f-338b6897f677-Ts">
type Ts = typeof import("typescript");

</storycode:copy>
<a id="f-338b6897f677-createBrowserTsChannel"></a>
### `createBrowserTsChannel`

Start the in-browser server over a document, and return the channel the
`LspClient` sits on.
`onReady` fires once the compiler has loaded, so the page can say it is
fetching a few megabytes rather than looking broken for a moment.

<storycode:copy id="f-338b6897f677-createBrowserTsChannel">
/**
 * Start the in-browser server over a document, and return the channel the
 * `LspClient` sits on.
 *
 * `onReady` fires once the compiler has loaded, so the page can say it is
 * fetching a few megabytes rather than looking broken for a moment.
 */
export function createBrowserTsChannel(options: {
  document: string;
  onReady?: () => void;
  onError?: (message: string) => void;
}): LspChannel & { update(document: string): void; dispose(): void } {
  const listeners = new Set<(message: JsonRpcMessage) => void>();
  const queue: JsonRpcMessage[] = [];
  let ts: Ts | null = null;
  let service: import("typescript").LanguageService | null = null;
  let document = options.document;
  let block = locateBlock(document);
  let version = 0;
  let disposed = false;

  const emit = (message: JsonRpcMessage) => {
    for (const listener of listeners) listener(message);
  };

  const contents = () => (block ? virtualFileOf(document, block) : "");

  // The compiler arrives asynchronously; anything asked before it lands is
  // queued and answered once, rather than dropped. An editor that silently
  // lost the first hover would look broken exactly when a reader is paying
  // the most attention.
  // The standard library comes with it, and lazily for the same reason:
  // without it the compiler has no `Array`, so `lines.reduce(...)` in the
  // demo document is an ERROR — a first impression of three fabricated type
  // errors on correct code would teach the reader the product is wrong about
  // their code. Imported HERE rather than at module scope so its 216 KB
  // travels in the lazy chunk instead of the page's first load.
  void Promise.all([import("typescript"), import("typescript/lib/lib.es5.d.ts?raw")])
    .then(([module, lib]) => {
      if (disposed) return;
      const libEs5 = lib.default;
      ts = module.default ?? (module as unknown as Ts);
      const fileName = block?.path ?? "demo.ts";
      const LIB = "lib.d.ts";
      const host: import("typescript").LanguageServiceHost = {
        getScriptFileNames: () => [fileName],
        getScriptVersion: (name) => (name === LIB ? "1" : String(version)),
        getScriptSnapshot: (name) => {
          if (name === LIB) return ts!.ScriptSnapshot.fromString(libEs5);
          return name === fileName ? ts!.ScriptSnapshot.fromString(contents()) : undefined;
        },
        getCurrentDirectory: () => "/",
        getCompilationSettings: () => ({
          target: ts!.ScriptTarget.ES2020,
          strict: true,
          // No module resolution: there is no filesystem here, and a demo
          // that reported "cannot find module" for every import would be
          // teaching the reader something false about the product.
          noResolve: true,
        }),
        getDefaultLibFileName: () => LIB,
        fileExists: (name) => name === fileName || name === LIB,
        readFile: (name) => {
          if (name === LIB) return libEs5;
          return name === fileName ? contents() : undefined;
        },
      };
      service = ts.createLanguageService(host, ts.createDocumentRegistry());
      options.onReady?.();
      for (const message of queue.splice(0)) handle(message);
    })
    .catch((error: unknown) => {
      options.onError?.(
        `The TypeScript compiler could not be loaded, so this demo is showing plain text. ${String(error)}`,
      );
    });

  function offsetOf(position: LspPosition): number | null {
    if (!block) return null;
    const virtual = toVirtual(position, block);
    if (!virtual) return null;
    const lines = contents().split("\n");
    let offset = 0;
    for (let i = 0; i < virtual.line; i++) offset += lines[i].length + 1;
    return offset + virtual.character;
  }

  function positionOf(offset: number): LspPosition {
    const text = contents().slice(0, offset);
    const lines = text.split("\n");
    return { line: lines.length - 1, character: lines[lines.length - 1].length };
  }

  function documentRange(start: number, length: number) {
    if (!block) return null;
    return {
      start: toDocument(positionOf(start), block),
      end: toDocument(positionOf(start + length), block),
    };
  }

  function reply(id: number | string, result: unknown) {
    emit({ jsonrpc: "2.0", id, result });
  }

  function publishDiagnostics(uri: string) {
    if (!ts || !service || !block) return;
    const fileName = block.path;
    const raw = [
      ...service.getSyntacticDiagnostics(fileName),
      ...service.getSemanticDiagnostics(fileName),
    ];
    const diagnostics = raw
      .map((diagnostic) => {
        const range = documentRange(diagnostic.start ?? 0, diagnostic.length ?? 1);
        if (!range) return null;
        return {
          range,
          severity: diagnostic.category === ts!.DiagnosticCategory.Error ? 1 : 2,
          source: "typescript",
          message: ts!.flattenDiagnosticMessageText(diagnostic.messageText, " "),
        };
      })
      .filter(Boolean);
    emit({
      jsonrpc: "2.0",
      method: "textDocument/publishDiagnostics",
      params: { uri, diagnostics },
    });
  }

  function handle(message: JsonRpcMessage) {
    if (!("method" in message)) return;
    const { method } = message;
    const params = (message.params ?? {}) as Record<string, never> & {
      textDocument?: { uri: string; text?: string };
      position?: LspPosition;
      contentChanges?: { text: string }[];
    };
    const id = "id" in message ? message.id : null;
    const uri = params.textDocument?.uri ?? "hick:///demo.md";

    if (!service) {
      // Requests are queued; a notification that changes the text is applied
      // immediately so nothing is lost while the compiler loads.
      if (method === "textDocument/didChange" && params.contentChanges?.[0]) {
        document = params.contentChanges[0].text;
        block = locateBlock(document);
        version++;
      }
      queue.push(message);
      return;
    }

    switch (method) {
      case "initialize":
        if (id !== null) {
          reply(id, {
            capabilities: {
              hoverProvider: true,
              completionProvider: { triggerCharacters: ["."] },
              definitionProvider: true,
              semanticTokensProvider: {
                legend: {
                  tokenTypes: [...DEMO_TOKEN_TYPES],
                  tokenModifiers: [...DEMO_TOKEN_MODIFIERS],
                },
                full: true,
              },
            },
          });
        }
        // The same announcement the desktop bridge makes, for the same
        // reason: the client cannot decode a token type without the legend.
        emit({
          jsonrpc: "2.0",
          method: "hick/serverCapabilities",
          params: {
            capabilities: {
              semanticTokensProvider: {
                legend: {
                  tokenTypes: [...DEMO_TOKEN_TYPES],
                  tokenModifiers: [...DEMO_TOKEN_MODIFIERS],
                },
              },
            },
          },
        });
        return;

      case "textDocument/didOpen":
        publishDiagnostics(uri);
        return;

      case "textDocument/didChange": {
        const next = params.contentChanges?.[0]?.text;
        if (typeof next === "string") {
          document = next;
          block = locateBlock(document);
          version++;
        }
        publishDiagnostics(uri);
        return;
      }

      case "textDocument/hover": {
        if (id === null) return;
        const offset = params.position ? offsetOf(params.position) : null;
        if (offset === null || !block) return reply(id, null);
        const info = service.getQuickInfoAtPosition(block.path, offset);
        if (!info) return reply(id, null);
        const text = ts!.displayPartsToString(info.displayParts);
        const documentation = ts!.displayPartsToString(info.documentation);
        return reply(id, {
          contents: {
            kind: "plaintext",
            value: documentation ? `${text}\n\n${documentation}` : text,
          },
          range: documentRange(info.textSpan.start, info.textSpan.length),
        });
      }

      case "textDocument/completion": {
        if (id === null) return;
        const offset = params.position ? offsetOf(params.position) : null;
        if (offset === null || !block) return reply(id, { isIncomplete: false, items: [] });
        const completions = service.getCompletionsAtPosition(block.path, offset, {});
        return reply(id, {
          isIncomplete: false,
          items: (completions?.entries ?? []).slice(0, 50).map((entry) => ({
            label: entry.name,
            detail: entry.kind,
          })),
        });
      }

      case "textDocument/definition": {
        if (id === null) return;
        const offset = params.position ? offsetOf(params.position) : null;
        if (offset === null || !block) return reply(id, null);
        const definitions = service.getDefinitionAtPosition(block.path, offset) ?? [];
        return reply(
          id,
          definitions
            .map((definition) => {
              const range = documentRange(definition.textSpan.start, definition.textSpan.length);
              return range ? { uri, range } : null;
            })
            .filter(Boolean),
        );
      }

      case "textDocument/semanticTokens/full": {
        if (id === null) return;
        if (!block) return reply(id, { data: [] });
        return reply(id, { data: semanticTokens(ts!, service, block) });
      }

      default:
        // Everything else is answered with "nothing", exactly as a child
        // server that lacks the capability would be. The editor degrades to
        // what this server can do rather than erroring.
        if (id !== null) reply(id, null);
    }
  }

  return {
    send(message) {
      handle(message);
    },
    onMessage(cb) {
      listeners.add(cb);
      return () => listeners.delete(cb);
    },
    handleFrame() {
      // There is no socket here: nothing ever arrives as a raw frame.
      return false;
    },
    update(next: string) {
      document = next;
      block = locateBlock(next);
      version++;
    },
    dispose() {
      disposed = true;
      listeners.clear();
      service?.dispose();
      service = null;
    },
  };
}

</storycode:copy>
<a id="f-338b6897f677-semanticTokens"></a>
### `semanticTokens`

TypeScript's classifications, encoded the way the protocol wants them.
In DOCUMENT coordinates, because that is what the client is handed
everywhere else: `hick-lsp` maps positions server-side, so the editor
bindings do no mapping of their own. Returning virtual-file lines here
puts every token about five lines too high — the demo coloured `version=`
in the XML header as an interface and a line of prose as a property, which
looked like syntax highlighting right up until you read which words it had
picked.

<storycode:copy id="f-338b6897f677-semanticTokens">
/**
 * TypeScript's classifications, encoded the way the protocol wants them.
 *
 * In DOCUMENT coordinates, because that is what the client is handed
 * everywhere else: `hick-lsp` maps positions server-side, so the editor
 * bindings do no mapping of their own. Returning virtual-file lines here
 * puts every token about five lines too high — the demo coloured `version=`
 * in the XML header as an interface and a line of prose as a property, which
 * looked like syntax highlighting right up until you read which words it had
 * picked.
 */
function semanticTokens(
  ts: Ts,
  service: import("typescript").LanguageService,
  block: BlockLocation,
): number[] {
  const classified = service.getEncodedSemanticClassifications(
    block.path,
    { start: 0, length: Number.MAX_SAFE_INTEGER },
    ts.SemanticClassificationFormat.TwentyTwenty,
  );
  const text = service.getProgram()?.getSourceFile(block.path)?.getFullText() ?? "";
  const lineStarts: number[] = [0];
  for (let i = 0; i < text.length; i++) if (text[i] === "\n") lineStarts.push(i + 1);
  const positionAt = (offset: number) => {
    let line = 0;
    while (line + 1 < lineStarts.length && lineStarts[line + 1] <= offset) line++;
    return { line, character: offset - lineStarts[line] };
  };

  const out: number[] = [];
  let lastLine = 0;
  let lastStart = 0;
  // TypeScript emits [start, length, classification] triples, where the
  // classification is `(kind + 1) << 8 | modifiers`. Reading it as a whole
  // number and looking that up in a table only ever matches the tokens that
  // happen to carry NO modifiers — which is why a first attempt at this
  // coloured two identifiers in the whole file and looked like the compiler
  // was barely running.
  for (let i = 0; i + 2 < classified.spans.length; i += 3) {
    const [start, length, classification] = classified.spans.slice(i, i + 3);
    const kind = (classification >> 8) - 1;
    const modifiers = classification & 0xff;
    const type = TS_TOKEN_TYPES[kind];
    if (type === undefined) continue;
    const index = DEMO_TOKEN_TYPES.indexOf(type);
    if (index === -1) continue;
    const { line: virtualLine, character } = positionAt(start);
    const { line } = toDocument({ line: virtualLine, character }, block);
    const deltaLine = line - lastLine;
    const deltaStart = deltaLine === 0 ? character - lastStart : character;
    out.push(deltaLine, deltaStart, length, index, encodeModifiers(modifiers));
    lastLine = line;
    lastStart = character;
  }
  return out;
}

</storycode:copy>
<a id="f-338b6897f677-encodeModifiers"></a>
### `encodeModifiers`

TypeScript's modifier bits, re-indexed onto this demo's legend.
The two sets are not the same list — TypeScript has `local` and
`defaultLibrary`, the demo's legend does not — so a bit is carried over
only when both sides have a name for it.

<storycode:copy id="f-338b6897f677-encodeModifiers">
/**
 * TypeScript's modifier bits, re-indexed onto this demo's legend.
 *
 * The two sets are not the same list — TypeScript has `local` and
 * `defaultLibrary`, the demo's legend does not — so a bit is carried over
 * only when both sides have a name for it.
 */
function encodeModifiers(bits: number): number {
  let out = 0;
  for (const [tsBit, name] of TS_MODIFIER_BITS) {
    if (bits & (1 << tsBit)) {
      const index = DEMO_TOKEN_MODIFIERS.indexOf(name);
      if (index !== -1) out |= 1 << index;
    }
  }
  return out;
}

</storycode:copy>
<a id="f-338b6897f677-TS_TOKEN_TYPES"></a>
### `TS_TOKEN_TYPES`

`TokenType`, in the compiler's own order (class = 0), mapped onto the
legend names this demo advertises. `namespace`, `enumMember` and `member`
have no colour here and are simply absent: a token whose type the legend
does not carry is skipped rather than coloured as something else.

<storycode:copy id="f-338b6897f677-TS_TOKEN_TYPES">
/**
 * `TokenType`, in the compiler's own order (class = 0), mapped onto the
 * legend names this demo advertises. `namespace`, `enumMember` and `member`
 * have no colour here and are simply absent: a token whose type the legend
 * does not carry is skipped rather than coloured as something else.
 */
const TS_TOKEN_TYPES: ((typeof DEMO_TOKEN_TYPES)[number] | undefined)[] = [
  "class",
  "enum",
  "interface",
  undefined, // namespace
  "typeParameter",
  "type",
  "parameter",
  "variable",
  undefined, // enumMember
  "property",
  "function",
  "method", // `member`
];

</storycode:copy>
<a id="f-338b6897f677-TS_MODIFIER_BITS"></a>
### `TS_MODIFIER_BITS`

`TokenModifier`, in the compiler's order, paired with our legend names.

<storycode:copy id="f-338b6897f677-TS_MODIFIER_BITS">
/** `TokenModifier`, in the compiler's order, paired with our legend names. */
const TS_MODIFIER_BITS: [number, (typeof DEMO_TOKEN_MODIFIERS)[number]][] = [
  [0, "declaration"],
  [1, "static"],
  [3, "readonly"],
];
</storycode:copy>

### Assemble `apps/web/src/landing/demos/browserTsServer.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/landing/demos/browserTsServer.ts"><storycode:paste select="#f-338b6897f677-BlockLocation"/><storycode:paste select="#f-338b6897f677-locateBlock"/><storycode:paste select="#f-338b6897f677-virtualFileOf"/><storycode:paste select="#f-338b6897f677-toVirtual"/><storycode:paste select="#f-338b6897f677-toDocument"/><storycode:paste select="#f-338b6897f677-DEMO_TOKEN_TYPES"/><storycode:paste select="#f-338b6897f677-DEMO_TOKEN_MODIFIERS"/><storycode:paste select="#f-338b6897f677-Ts"/><storycode:paste select="#f-338b6897f677-createBrowserTsChannel"/><storycode:paste select="#f-338b6897f677-semanticTokens"/><storycode:paste select="#f-338b6897f677-encodeModifiers"/><storycode:paste select="#f-338b6897f677-TS_TOKEN_TYPES"/><storycode:paste select="#f-338b6897f677-TS_MODIFIER_BITS"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-5b2005bf0cdb"></a>
## `apps/web/src/landing/demos/browserTsServer.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-5b2005bf0cdb-DOCUMENT"></a>
### `DOCUMENT`

<storycode:copy id="f-5b2005bf0cdb-DOCUMENT">
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

</storycode:copy>
<a id="f-5b2005bf0cdb-DEFINITION_LINE"></a>
### `DEFINITION_LINE`

0-based document line of `export function double…`.

<storycode:copy id="f-5b2005bf0cdb-DEFINITION_LINE">
/** 0-based document line of `export function double…`. */
const DEFINITION_LINE = 5;

</storycode:copy>
<a id="f-5b2005bf0cdb-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-5b2005bf0cdb-expression_statement">
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

</storycode:copy>
<a id="f-5b2005bf0cdb-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-5b2005bf0cdb-expression_statement-2">
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

</storycode:copy>
<a id="f-5b2005bf0cdb-open"></a>
### `open`

Drive the channel the way `LspClient` does, and collect what comes back.

<storycode:copy id="f-5b2005bf0cdb-open">
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

</storycode:copy>
<a id="f-5b2005bf0cdb-replyTo"></a>
### `replyTo`

<storycode:copy id="f-5b2005bf0cdb-replyTo">
function replyTo(received: JsonRpcMessage[], id: number) {
  return received.find(
    (message) => "id" in message && message.id === id && "result" in message,
  ) as { result: unknown } | undefined;
}

</storycode:copy>
<a id="f-5b2005bf0cdb-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-5b2005bf0cdb-expression_statement-3">
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
</storycode:copy>

### Assemble `apps/web/src/landing/demos/browserTsServer.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/landing/demos/browserTsServer.test.ts"><storycode:paste select="#f-5b2005bf0cdb-DOCUMENT"/><storycode:paste select="#f-5b2005bf0cdb-DEFINITION_LINE"/><storycode:paste select="#f-5b2005bf0cdb-expression_statement"/><storycode:paste select="#f-5b2005bf0cdb-expression_statement-2"/><storycode:paste select="#f-5b2005bf0cdb-open"/><storycode:paste select="#f-5b2005bf0cdb-replyTo"/><storycode:paste select="#f-5b2005bf0cdb-expression_statement-3"/></storycode:file>


</storycode:doc>
