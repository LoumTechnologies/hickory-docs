<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Read source-wrapped prose as flowing paragraphs

The editable document already uses `paragraphUnwrapChanges` to find Markdown prose soft breaks, including lists, quotes and setext headings, while excluding code payloads, tags, explicit hard breaks and other byte-sensitive structures. The static reader must make the same reading decision without applying those edits: hashes, language facts and anchors refer to the original source. Turn the planned joins into replacement decorations instead. Supply them directly from a state field because replacing a newline changes CodeMirror's height map; a viewport plugin cannot safely do that. Each visible space conceals only the selected newline and continuation indentation. The document and all offsets stay unchanged.

<static:copy id="static-apps-web-src-editor-paragraphReading-ts-0">
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
</static:copy>

<static:file path="apps/web/src/editor/paragraphReading.ts"><static:paste select="#static-apps-web-src-editor-paragraphReading-ts-0"/></static:file>
</static:doc>
