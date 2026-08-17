// A fading highlight on text that changed because of an edit somewhere else.
//
// Both directions of the two-way edit flow end with text appearing in a
// buffer the user was not typing in: an output edit resolves into the
// document, and a document edit re-weaves into every generated pane. The
// update itself is silent — the flash is the sign that says WHAT moved, and
// then gets out of the way (~1.2s, CSS-animated fade, self-removing).
//
// Mechanics: `flashSpans` installs the field on first use (the same
// appendConfig trick rangeHighlight.ts uses for the document editor), adds a
// batch of mark decorations, and schedules that batch's removal after the
// fade completes. Ranges map through document changes in between, so a flash
// stays on its text while the user keeps typing; a batch is dropped whole
// when its timer fires, never someone else's still-fading batch.

import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";

import { computeEdits } from "../lib/diff";

export interface FlashRange {
  from: number;
  to: number;
}

/** How long a flash lives. The CSS animation is 1.2s; the decorations come
 * off a beat later so the fade is never cut short. */
export const FLASH_MS = 1400;

let nextBatch = 1;

/** Add a batch of flash marks. Positions map through concurrent changes. */
export const addFlash = StateEffect.define<{ batch: number; ranges: FlashRange[] }>({
  map: (value, mapping) => ({
    batch: value.batch,
    ranges: value.ranges.map((r) => ({
      from: mapping.mapPos(r.from, 1),
      to: mapping.mapPos(r.to, -1),
    })),
  }),
});

/** Remove one batch's marks (its fade has finished). */
export const expireFlash = StateEffect.define<number>();

export const changeFlashField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const e of tr.effects) {
      if (e.is(addFlash)) {
        const max = tr.state.doc.length;
        const marks = [];
        for (const r of e.value.ranges) {
          const from = Math.max(0, Math.min(r.from, max));
          const to = Math.max(0, Math.min(r.to, max));
          if (to > from) {
            marks.push(
              Decoration.mark({
                class: "cm-change-flash",
                batch: e.value.batch,
              }).range(from, to),
            );
          }
        }
        if (marks.length > 0) deco = deco.update({ add: marks, sort: true });
      } else if (e.is(expireFlash)) {
        const batch = e.value;
        deco = deco.update({ filter: (_f, _t, value) => value.spec.batch !== batch });
      }
    }
    return deco;
  },
  provide: (f) => EditorView.decorations.from(f),
});

/**
 * Flash `ranges` in `view`: install the field if this editor has never
 * flashed, add the marks, and remove them once the fade is over.
 *
 * Empty and collapsed ranges are dropped; a call with nothing visible
 * dispatches nothing.
 */
export function flashSpans(view: EditorView, ranges: FlashRange[]): void {
  const visible = ranges.filter((r) => r.to > r.from);
  if (visible.length === 0) return;
  const batch = nextBatch++;
  const effects: StateEffect<unknown>[] = [addFlash.of({ batch, ranges: visible })];
  if (!view.state.field(changeFlashField, false)) {
    effects.unshift(StateEffect.appendConfig.of(changeFlashField));
  }
  view.dispatch({ effects });
  setTimeout(() => {
    // The view may be gone by now (tab closed mid-fade); a dispatch into a
    // destroyed view throws, and a flash is never worth an error.
    try {
      view.dispatch({ effects: expireFlash.of(batch) });
    } catch {
      /* view destroyed */
    }
  }, FLASH_MS);
}

/**
 * Bring `view`'s buffer to `content` with minimal edits, flashing what
 * arrived. Returns whether anything changed.
 *
 * This is the receiving end of a change made SOMEWHERE ELSE (the up-loop
 * re-wove this pane's file). The own-edit guard is the diff itself: content
 * identical to the buffer — the round-trip of what the user just typed here —
 * produces no edits, so it neither disturbs the cursor nor flashes.
 *
 * The dispatch carries no user event, so an `onLocalEdit`-style listener
 * keyed on input/delete/undo will not echo it back to the server.
 */
export function syncAndFlash(view: EditorView, content: string): boolean {
  const edits = computeEdits(view.state.doc.toString(), content);
  if (edits.length === 0) return false;
  view.dispatch({
    changes: edits.map((e) => ({ from: e.start, to: e.end, insert: e.text })),
  });
  // Where each edit's new text sits after ALL edits applied: earlier edits
  // shift later ones by their length delta (edits are ascending, disjoint).
  const ranges: FlashRange[] = [];
  let delta = 0;
  for (const e of edits) {
    const from = e.start + delta;
    ranges.push({ from, to: from + e.text.length });
    delta += e.text.length - (e.end - e.start);
  }
  flashSpans(view, ranges);
  return true;
}
