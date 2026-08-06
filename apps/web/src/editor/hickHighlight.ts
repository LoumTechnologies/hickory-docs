import { MatchDecorator, ViewPlugin, Decoration, EditorView } from "@codemirror/view";
import type { DecorationSet, ViewUpdate } from "@codemirror/view";

// Minimal .hick highlighting: only namespace-prefixed tags (`hick:`) are
// structured; everything else is raw text and stays plain. This mirrors the
// parser's no-escaping invariant — we highlight tags, nothing else.

const tagDeco = Decoration.mark({ class: "cm-hick-tag" });
const attrDeco = Decoration.mark({ class: "cm-hick-attr" });

const tagMatcher = new MatchDecorator({
  // Open, close, and self-closing hick: tags, including attributes.
  regexp: /<\/?hick:[\w-]+(?:\s+[^<>]*?)?\/?>/g,
  decorate: (add, from, to, match) => {
    add(from, to, tagDeco);
    // Highlight attribute names inside the tag a shade differently.
    const text = match[0];
    const attrRe = /([\w-]+)=("[^"]*"|'[^']*'|[^\s/>]+)/g;
    let m: RegExpExecArray | null;
    while ((m = attrRe.exec(text)) !== null) {
      add(from + m.index, from + m.index + m[1].length, attrDeco);
    }
  },
});

export const hickHighlight = [
  ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) {
        this.decorations = tagMatcher.createDeco(view);
      }
      update(update: ViewUpdate) {
        this.decorations = tagMatcher.updateDeco(update, this.decorations);
      }
    },
    { decorations: (v) => v.decorations },
  ),
  EditorView.theme({
    ".cm-hick-tag": { color: "var(--accent)", fontWeight: "500" },
    ".cm-hick-attr": { color: "var(--accent-2)" },
  }),
];
