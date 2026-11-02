import { EditorState, StateEffect, StateField, type Extension, type Transaction } from "@codemirror/state";
import { ySyncFacet } from "y-codemirror.next";
import { parseStructure } from "./hickDoc";
import type { OutputFile } from "../api/types";
import { byteToChar } from "../lib/offsets";

type Range = { from: number; to: number };

function touches(tr: Transaction, ranges: Range[]): boolean {
  let blocked = false;
  tr.changes.iterChangedRanges((from, to) => {
    if (ranges.some(r => from === to ? r.from < from && from < r.to : from < r.to && to > r.from)) blocked = true;
  });
  return blocked;
}

export function immutableScaffolds(): Extension {
  return EditorState.transactionFilter.of(tr => {
    if (!tr.docChanged) return tr;
    // A Yjs receiving update has already changed the room. Let the view
    // reconcile; every local editor gesture is checked before it reaches Yjs.
    const room = tr.startState.facet(ySyncFacet);
    if (room && room.ytext.toString() === tr.newDoc.toString()) return tr;
    const ranges = parseStructure(tr.startState.doc.toString()).blocks
      .filter(b => b.name === "ingested" && !("key" in b.attrs))
      .map(b => ({ from: b.from, to: b.to }));
    // An edit outside the block must not hide it by changing the namespace
    // or enclosing it in a raw fence/entry, then make its bytes writable.
    const next = tr.newDoc.toString();
    const retained = parseStructure(next).blocks.filter(b => b.name === "ingested" && !("key" in b.attrs))
      .map(b => next.slice(b.from, b.to));
    for (const range of ranges) {
      const at = retained.indexOf(tr.startState.doc.sliceString(range.from, range.to));
      if (at < 0) return [];
      retained.splice(at, 1);
    }
    return tr;
  });
}
export const setScaffoldOutputRanges = StateEffect.define<Range[]>();
const scaffoldOutputRanges = StateField.define<Range[]>({
  create: () => [],
  update(ranges, tr) {
    ranges = ranges.map(r => ({ from: tr.changes.mapPos(r.from, 1), to: tr.changes.mapPos(r.to, -1) }));
    for (const effect of tr.effects) if (effect.is(setScaffoldOutputRanges)) ranges = effect.value;
    return ranges;
  },
});

export function capturedOutputRanges(file: OutputFile): Range[] {
  return file.provenance.filter(p => p.origin.kind === "ingested")
    .map(p => ({ from: byteToChar(file.content, p.start), to: byteToChar(file.content, p.end) }));
}

export function immutableScaffoldOutput(initial: Range[]): Extension {
  return [scaffoldOutputRanges.init(() => initial), EditorState.transactionFilter.of(tr =>
    touches(tr, tr.startState.field(scaffoldOutputRanges)) ? [] : tr)];
}
