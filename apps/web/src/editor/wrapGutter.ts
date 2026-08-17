// Wrap marks for the LEFT line-number gutter.
//
// CodeMirror's lineNumbers() draws one cell spanning a soft-wrapped line,
// number at the top — every continuation row is blank, and a brace or band
// spanning the tall line reads as unexplained height. This puts a muted
// wrap mark on each continuation row, over the gutter, so the rows read as
// "still that line".
//
// Deliberately NOT a replacement gutter. A custom number gutter would have
// to re-implement everything lineNumbers() already carries here — the
// lineNumberMarkers hover tint (editor/lineHighlight.ts), width sizing from
// the last line number, widget-row parity with the breakpoint gutter — and a
// GutterMarker builds its DOM during the update phase, BEFORE layout, when
// the number of visual rows is not yet knowable (the height map reports a
// wrapped line as one block, and height ÷ defaultLineHeight miscounts every
// styled line — measured live: a wrapped heading is two 35.5px rows against
// a 25.1px default). A layer's `markers` callback runs in the measure/read
// phase, post-layout, where editor/wrapRows.ts can read the true rows — so
// lineNumbers() stays exactly as it is and this layer paints only what it
// adds. Pointer events pass through (breakpoint clicks, gutter clicks are
// untouched), and the layer is display-only by construction.

import { BlockType, layer } from "@codemirror/view";
import type { EditorView, LayerMarker } from "@codemirror/view";

import { rowBoxes, textExtent } from "../lib/rightRail";
import { measureRowTops } from "./wrapRows";

/** The one glyph both gutters use for a continuation row. */
export const WRAP_MARK = "⤷";

/** One wrap mark, in the layer's scroll-space coordinates. */
class WrapMarker implements LayerMarker {
  constructor(
    private readonly left: number,
    private readonly top: number,
    private readonly width: number,
    private readonly height: number,
  ) {}

  eq(other: WrapMarker): boolean {
    return (
      Math.abs(other.left - this.left) < 0.5 &&
      Math.abs(other.top - this.top) < 0.5 &&
      Math.abs(other.width - this.width) < 0.5 &&
      Math.abs(other.height - this.height) < 0.5
    );
  }

  draw(): HTMLElement {
    const el = document.createElement("div");
    el.className = "cm-wrap-marker";
    el.textContent = WRAP_MARK;
    el.style.left = `${this.left}px`;
    el.style.top = `${this.top}px`;
    el.style.width = `${this.width}px`;
    el.style.height = `${this.height}px`;
    el.style.lineHeight = `${this.height}px`;
    return el;
  }
}

/** The wrap marks for the current viewport, measured post-layout. */
function wrapMarkers(view: EditorView): readonly LayerMarker[] {
  const gutter = view.dom.querySelector(".cm-gutter.cm-lineNumbers");
  if (!gutter) return [];
  // The layer's coordinate space is the scroller's scrolled canvas — the
  // same base CodeMirror's own layers use (client coords minus the scroll
  // container's origin, scroll offsets added back).
  const scRect = view.scrollDOM.getBoundingClientRect();
  const baseLeft = scRect.left - view.scrollDOM.scrollLeft;
  const baseTop = scRect.top - view.scrollDOM.scrollTop;
  const gRect = gutter.getBoundingClientRect();
  const out: LayerMarker[] = [];
  for (const block of view.viewportLineBlocks) {
    // Widget rows (file-block headers, cell panels) are not wrapping: the
    // mark belongs only inside the line's TEXT extent, exactly as the rail
    // and the number itself do.
    const children = Array.isArray(block.type)
      ? block.type.map((c) => ({
          text: c.type === BlockType.Text,
          top: c.top,
          height: c.height,
        }))
      : null;
    const extent = textExtent(block, children);
    // Too short for two rows: skip the DOM read (the common case).
    if (extent.height < view.defaultLineHeight * 1.5) continue;
    const tops = measureRowTops(view, block.from);
    if (!tops || tops.length < 2) continue;
    const boxes = rowBoxes(extent, tops.map((top) => top - view.documentTop));
    for (let i = 1; i < boxes.length; i++) {
      out.push(
        new WrapMarker(
          gRect.left - baseLeft,
          boxes[i].top + view.documentTop - baseTop,
          gRect.width,
          boxes[i].height,
        ),
      );
    }
  }
  return out;
}

/**
 * The extension: add alongside lineNumbers(). Display-only; changes no
 * line's height and intercepts no event.
 */
export function wrapGutterMarkers() {
  return layer({
    above: true,
    class: "cm-wrap-gutter-layer",
    update: (u) => u.docChanged || u.viewportChanged || u.geometryChanged,
    markers: wrapMarkers,
    // Above the gutters (z-index 200), which paint after layers: the marks
    // live ON the gutter, and a themed gutter background must not bury them.
    mount: (dom) => {
      dom.style.zIndex = "201";
    },
  });
}
