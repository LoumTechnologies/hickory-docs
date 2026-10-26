<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/test-setup.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0e833cff7a55"></a>
## `apps/web/src/test-setup.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Editor Reads A Document With The Parser The Server Uses](../../../../guarantees/language/the-editor-reads-with-the-parser-the-server-uses/contract.md), [A Watched Cell Shows What The Terminal Showed](../../../../guarantees/terminal/a-watched-cell-shows-what-the-terminal-showed/contract.md).

<a id="f-0e833cff7a55-cwd"></a>
### `cwd`

The parser is WebAssembly, loaded from disk here because a test has a
filesystem and no fetch. Every test that parses a document needs it, and
loading it once here is what lets `parseHickDoc` stay synchronous.

<storycode:copy id="f-0e833cff7a55-cwd">
// The parser is WebAssembly, loaded from disk here because a test has a
// filesystem and no fetch. Every test that parses a document needs it, and
// loading it once here is what lets `parseHickDoc` stay synchronous.
import { readFileSync } from "node:fs";
import { loadHickLangSync } from "./editor/hickLang";

// Under the jsdom environment `import.meta.url` is the page's, not a file
// URL, so the path is taken from the working directory — Vitest's root,
// which is this package.
const cwd = (globalThis as { process?: { cwd(): string } }).process?.cwd() ?? ".";
</storycode:copy>
<a id="f-0e833cff7a55-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-0e833cff7a55-expression_statement">
loadHickLangSync(readFileSync(`${cwd}/src/editor/generated/hick-lang/hick_lang_bg.wasm`));

</storycode:copy>
<a id="f-0e833cff7a55-zeroRect"></a>
### `zeroRect`

jsdom gaps that CodeMirror measures through.
CodeMirror asks the DOM how big a character is by putting a Range around
text and calling `getClientRects()`. jsdom implements Range but not that
method, so the call throws — from inside a requestAnimationFrame, which
makes it an UNHANDLED error rather than a test failure: the suite goes
green and prints a stack, which is the worst of both.
The shim returns zeroes. That is honest — jsdom does no layout, so every
rect it could return would be zero anyway — and it is enough: the editor
tests here assert on structure (which rows exist, what is inside them),
never on measured geometry. Anything that genuinely depends on layout is
verified in a real browser instead.

<storycode:copy id="f-0e833cff7a55-zeroRect">
// jsdom gaps that CodeMirror measures through.
//
// CodeMirror asks the DOM how big a character is by putting a Range around
// text and calling `getClientRects()`. jsdom implements Range but not that
// method, so the call throws — from inside a requestAnimationFrame, which
// makes it an UNHANDLED error rather than a test failure: the suite goes
// green and prints a stack, which is the worst of both.
//
// The shim returns zeroes. That is honest — jsdom does no layout, so every
// rect it could return would be zero anyway — and it is enough: the editor
// tests here assert on structure (which rows exist, what is inside them),
// never on measured geometry. Anything that genuinely depends on layout is
// verified in a real browser instead.

const zeroRect = (): DOMRect => ({
  x: 0,
  y: 0,
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  width: 0,
  height: 0,
  toJSON: () => ({}),
});

</storycode:copy>
<a id="f-0e833cff7a55-emptyRectList"></a>
### `emptyRectList`

<storycode:copy id="f-0e833cff7a55-emptyRectList">
const emptyRectList = (): DOMRectList => {
  const list = [] as unknown as DOMRectList;
  Object.defineProperty(list, "item", { value: () => null });
  return list;
};

</storycode:copy>
<a id="f-0e833cff7a55-if_statement"></a>
### `if_statement`

Only fill what is missing: a future jsdom that implements these properly
must win over the shim.

<storycode:copy id="f-0e833cff7a55-if_statement">
// Only fill what is missing: a future jsdom that implements these properly
// must win over the shim.
if (typeof Range !== "undefined") {
  if (!Range.prototype.getClientRects) {
    Range.prototype.getClientRects = emptyRectList;
  }
  if (!Range.prototype.getBoundingClientRect) {
    Range.prototype.getBoundingClientRect = zeroRect;
  }
}

</storycode:copy>
<a id="f-0e833cff7a55-if_statement-2"></a>
### `if_statement`

jsdom gaps that xterm.js measures through.
The watching terminal (`terminal/WatchingTerminal.tsx`) mounts a real
emulator, which is the point: the assertion that ANSI was *interpreted*
rather than shown as garbage is only worth anything against the real
parser. Two browser APIs xterm reaches for on `open()` do not exist here.
`matchMedia` is how it watches for a device-pixel-ratio change. The stub
reports "no match" and never fires — honest, because jsdom's DPR never
moves — and carries the deprecated `addListener`/`removeListener` pair as
well as the modern one, because xterm 5 still calls the old names.
`getContext` is used by xterm's colour code to parse a CSS colour by
painting it and reading the pixel back. The stub cannot do that, so any
colour it resolves is wrong — which does not matter for what is asserted:
the 16 ANSI colours are carried as palette indices on the cell
(`getFgColorMode()`), never through this path. Without the stub jsdom
prints a "not implemented" stack to stderr beside a green suite.

<storycode:copy id="f-0e833cff7a55-if_statement-2">
// jsdom gaps that xterm.js measures through.
//
// The watching terminal (`terminal/WatchingTerminal.tsx`) mounts a real
// emulator, which is the point: the assertion that ANSI was *interpreted*
// rather than shown as garbage is only worth anything against the real
// parser. Two browser APIs xterm reaches for on `open()` do not exist here.
//
// `matchMedia` is how it watches for a device-pixel-ratio change. The stub
// reports "no match" and never fires — honest, because jsdom's DPR never
// moves — and carries the deprecated `addListener`/`removeListener` pair as
// well as the modern one, because xterm 5 still calls the old names.
//
// `getContext` is used by xterm's colour code to parse a CSS colour by
// painting it and reading the pixel back. The stub cannot do that, so any
// colour it resolves is wrong — which does not matter for what is asserted:
// the 16 ANSI colours are carried as palette indices on the cell
// (`getFgColorMode()`), never through this path. Without the stub jsdom
// prints a "not implemented" stack to stderr beside a green suite.
if (typeof window !== "undefined" && !window.matchMedia) {
  const stub = {
    matches: false,
    media: "",
    onchange: null,
    addListener() {},
    removeListener() {},
    addEventListener() {},
    removeEventListener() {},
    dispatchEvent: () => false,
  };
  window.matchMedia = (() => stub) as unknown as typeof window.matchMedia;
}

</storycode:copy>
<a id="f-0e833cff7a55-if_statement-3"></a>
### `if_statement`

<storycode:copy id="f-0e833cff7a55-if_statement-3">
if (typeof HTMLCanvasElement !== "undefined") {
  const ctx = {
    fillStyle: "",
    fillRect() {},
    getImageData: () => ({ data: new Uint8ClampedArray([0, 0, 0, 255]) }),
  };
  HTMLCanvasElement.prototype.getContext = (() =>
    ctx) as unknown as HTMLCanvasElement["getContext"];
}

</storycode:copy>
<a id="f-0e833cff7a55-if_statement-4"></a>
### `if_statement`

`ResizeObserver` is how a pane learns it changed width. jsdom does no
layout, so a real one would never have anything to report; the stub
therefore never fires, which is the correct behaviour rather than a
stand-in for it. Components that need a size are expected to have a
fallback for the first paint, and that fallback is what tests measure.

<storycode:copy id="f-0e833cff7a55-if_statement-4">
// `ResizeObserver` is how a pane learns it changed width. jsdom does no
// layout, so a real one would never have anything to report; the stub
// therefore never fires, which is the correct behaviour rather than a
// stand-in for it. Components that need a size are expected to have a
// fallback for the first paint, and that fallback is what tests measure.
if (typeof globalThis.ResizeObserver === "undefined") {
  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  } as unknown as typeof ResizeObserver;
}
</storycode:copy>

### Assemble `apps/web/src/test-setup.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/test-setup.ts"><storycode:paste select="#f-0e833cff7a55-cwd"/><storycode:paste select="#f-0e833cff7a55-expression_statement"/><storycode:paste select="#f-0e833cff7a55-zeroRect"/><storycode:paste select="#f-0e833cff7a55-emptyRectList"/><storycode:paste select="#f-0e833cff7a55-if_statement"/><storycode:paste select="#f-0e833cff7a55-if_statement-2"/><storycode:paste select="#f-0e833cff7a55-if_statement-3"/><storycode:paste select="#f-0e833cff7a55-if_statement-4"/></storycode:file>

</storycode:doc>
