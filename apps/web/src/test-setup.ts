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
