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

const emptyRectList = (): DOMRectList => {
  const list = [] as unknown as DOMRectList;
  Object.defineProperty(list, "item", { value: () => null });
  return list;
};

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

if (typeof HTMLCanvasElement !== "undefined") {
  const ctx = {
    fillStyle: "",
    fillRect() {},
    getImageData: () => ({ data: new Uint8ClampedArray([0, 0, 0, 255]) }),
  };
  HTMLCanvasElement.prototype.getContext = (() =>
    ctx) as unknown as HTMLCanvasElement["getContext"];
}

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
