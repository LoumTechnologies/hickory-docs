// Blocks that show their RESULT instead of their source.
//
// A literate file is meant to be read. Opening one and finding
// `<hick:diagram renderer="mermaid">` above nine lines of graph syntax is
// reading the machinery instead of the document — so an exec cell and a
// diagram render by default, and their source is one click away.
//
// The mechanism is a fold, not an overlay: a block replacement decoration
// swaps the block's LINES for the rendered widget, exactly the way a
// collapsed region works. That distinction is what keeps the gutters honest.
// A block *widget* adds a screen row with no document line behind it, so the
// numbers step over a row that exists; a block *replacement* removes the rows
// it stands for, so the numbers step over lines that are genuinely not shown
// — which is what folding has always meant and what every editor's fold
// arrow already teaches. See
// docs/guarantees/authoring/the-gutters-never-skip-a-number.md.
//
// Which blocks are rendered is remembered as DOCUMENT POSITIONS, mapped
// through every change. Keying on "the third exec" would move the state to
// the wrong block the moment one is inserted above it; a mapped position
// follows the block it was taken from, and quietly disappears with it.

import { MapMode, StateEffect, StateField } from "@codemirror/state";
import type { EditorState, Extension, Range } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";

import { SlotRegistry, structureOf } from "./wysiwyg";
import { blocksNamed, codeRangesOf, execBlocksOf } from "./hickDoc";
import type { HickBlock, HickDocStructure } from "./hickDoc";

/** Render this block (identified by the offset its source starts at). */
export const renderBlock = StateEffect.define<number>();
/** Show this block's source instead. */
export const showBlockSource = StateEffect.define<number>();
/** Replace the whole set — how a freshly opened document is made readable. */
export const setRenderedBlocks = StateEffect.define<number[]>();

/**
 * The block starts currently rendered.
 *
 * Empty by default, and filled once when a document is opened (see
 * DocumentEditor). Deliberately NOT "everything is rendered unless excepted":
 * a block you are in the middle of typing must not turn into a picture under
 * the caret, and with this direction a new block simply is not in the set.
 */
export const renderedField = StateField.define<readonly number[]>({
  create: () => [],
  update(value, tr) {
    let next = value;
    if (tr.docChanged) {
      next = next
        .map((pos) => tr.changes.mapPos(pos, 1, MapMode.TrackDel))
        .filter((pos): pos is number => pos !== null && pos >= 0);
    }
    for (const effect of tr.effects) {
      if (effect.is(setRenderedBlocks)) next = [...new Set(effect.value)];
      else if (effect.is(renderBlock))
        next = next.includes(effect.value) ? next : [...next, effect.value];
      else if (effect.is(showBlockSource)) next = next.filter((pos) => pos !== effect.value);
    }
    return next;
  },
});

/** Whether the block starting at `at` is showing its result. */
export function isRendered(state: EditorState, at: number): boolean {
  return state.field(renderedField, false)?.includes(at) ?? false;
}

/** Every exec and diagram block of a document, in order — the blocks that
 * have something to render. */
export function renderableBlocks(structure: HickDocStructure): HickBlock[] {
  return [...execBlocksOf(structure), ...blocksNamed(structure, "diagram")].sort(
    (a, b) => a.from - b.from,
  );
}

// ---------------------------------------------------------------------------
// Slots: the bridge to React, the same one the environment chip uses.
// ---------------------------------------------------------------------------

export interface RenderedSlot {
  key: string;
  el: HTMLElement;
  index: number;
  kind: "exec" | "diagram";
  /** The block's start offset — what a toggle effect carries. */
  at: number;
  /** The block's whole source span, for matching the server's exec blocks. */
  span: [number, number];
  /** exec: the commands, with nested expect output removed. diagram: source. */
  text: string;
  /** diagram only. */
  renderer: string;
  asserts: string[];
}

export class RenderedRegistry extends SlotRegistry<RenderedSlot> {}

class RenderedWidget extends WidgetType {
  constructor(
    private registry: RenderedRegistry,
    private slot: Omit<RenderedSlot, "el">,
  ) {
    super();
  }

  eq(other: RenderedWidget) {
    const a = this.slot;
    const b = other.slot;
    // Rebuild only when the rendered CONTENT would differ. Editing prose
    // elsewhere moves the span, which must not restart a diagram engine.
    return (
      a.key === b.key &&
      a.kind === b.kind &&
      a.text === b.text &&
      a.renderer === b.renderer &&
      a.asserts.join(" ") === b.asserts.join(" ")
    );
  }

  toDOM() {
    const el = document.createElement("div");
    el.className = `cm-rendered cm-rendered-${this.slot.kind}`;
    this.registry.register({ ...this.slot, el });
    return el;
  }

  destroy(el: HTMLElement) {
    this.registry.unregister(this.slot.key, el);
  }

  get estimatedHeight() {
    return this.slot.kind === "diagram" ? 180 : 90;
  }

  ignoreEvent() {
    // Interactive React UI: the editor must not read a click in here as a
    // click into text that is not being shown.
    return true;
  }
}

/** The command text of an exec cell: its content minus any nested expect. */
function commandOf(state: EditorState, structure: HickDocStructure, block: HickBlock): string {
  const ranges = codeRangesOf(structure, block);
  const text = ranges.length
    ? ranges.map(([from, to]) => state.doc.sliceString(from, to)).join("")
    : state.doc.sliceString(block.contentFrom, block.contentTo);
  return text.trim();
}

function buildRendered(state: EditorState, registry: RenderedRegistry): DecorationSet {
  const rendered = state.field(renderedField, false);
  if (!rendered || rendered.length === 0) return Decoration.none;
  const structure = structureOf(state);
  const doc = state.doc;
  const ranges: Range<Decoration>[] = [];
  const counts = { exec: 0, diagram: 0 };

  for (const block of renderableBlocks(structure)) {
    const kind = block.name === "diagram" ? "diagram" : "exec";
    const index = counts[kind]++;
    if (!rendered.includes(block.from)) continue;
    // A block replacement must cover whole lines, or CodeMirror cannot take
    // the rows out of the height map.
    const first = doc.lineAt(Math.min(block.from, doc.length));
    const last = doc.lineAt(Math.min(Math.max(block.to - 1, block.from), doc.length));
    if (last.to <= first.from) continue;
    ranges.push(
      Decoration.replace({
        widget: new RenderedWidget(registry, {
          key: `${kind}-${index}`,
          index,
          kind,
          at: block.from,
          span: [block.from, block.to],
          text:
            kind === "diagram"
              ? doc.sliceString(block.contentFrom, block.contentTo)
              : commandOf(state, structure, block),
          renderer: block.attrs.renderer ?? "mermaid",
          asserts: (block.attrs.asserts ?? "")
            .split(/\s+/)
            .filter(Boolean)
            .map((selector) => selector.replace(/^#/, "")),
        }),
        block: true,
      }).range(first.from, last.to),
    );
  }
  ranges.sort((a, b) => a.from - b.from);
  return Decoration.set(ranges, true);
}

/**
 * The rendered-block extension.
 *
 * A StateField rather than a ViewPlugin, and not negotiable: these
 * decorations change vertical layout, and CodeMirror only recomputes its
 * height map in the same update for field-provided decorations.
 */
export function renderedBlocks(registry: RenderedRegistry): Extension {
  return [
    renderedField,
    StateField.define<DecorationSet>({
      create: (state) => buildRendered(state, registry),
      update(value, tr) {
        const toggled = tr.effects.some(
          (e) => e.is(renderBlock) || e.is(showBlockSource) || e.is(setRenderedBlocks),
        );
        return tr.docChanged || toggled ? buildRendered(tr.state, registry) : value;
      },
      provide: (f) => EditorView.decorations.from(f),
    }),
  ];
}
