import { ChangeSet, StateField, type Extension, type Range } from "@codemirror/state";
import { Decoration, EditorView, WidgetType, type DecorationSet } from "@codemirror/view";
import { paragraphUnwrapChanges } from "./unwrapParagraphs";

class ParagraphSpace extends WidgetType {
  eq() { return true; }
  toDOM() {
    const space = document.createElement("span");
    space.className = "cm-paragraph-space";
    space.textContent = " ";
    return space;
  }
}

function paragraphJoins(source: string): DecorationSet {
  const joins: Range<Decoration>[] = [];
  const plan = ChangeSet.of(paragraphUnwrapChanges(source), source.length);
  plan.iterChanges((from, to) => {
    joins.push(Decoration.replace({ widget: new ParagraphSpace() }).range(from, to));
  });
  return Decoration.set(joins, true);
}

/** A reading projection, never a source rewrite or a persisted preference. */
export function paragraphReading(): Extension {
  return StateField.define<DecorationSet>({
    create: state => paragraphJoins(state.doc.toString()),
    update: (joins, transaction) => transaction.docChanged ? paragraphJoins(transaction.newDoc.toString()) : joins,
    provide: field => EditorView.decorations.from(field),
  });
}
