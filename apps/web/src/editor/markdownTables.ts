import { markdownLanguage } from "@codemirror/lang-markdown";
import { proseCodeFences, verbatimRanges, type HickBlock, type HickDocStructure } from "./hickDoc";

/** Presentation blocks only: Markdown tables never become language elements. */
export function markdownTableBlocks(structure: HickDocStructure, text: string): HickBlock[] {
  const excluded = [...verbatimRanges(structure.blocks),
    ...proseCodeFences(structure, text).map(f => [f.from, f.to]),
    ...structure.tags.map(t => [t.from, t.to])];
  // Keep offsets while removing non-prose from Markdown's view. In particular,
  // hick tags must not turn their prose children into an HTML block.
  const chars = text.split("");
  for (const [from, to] of excluded) {
    for (let i = from; i < to; i++) if (chars[i] !== "\n" && chars[i] !== "\r") chars[i] = " ";
  }
  const result: HickBlock[] = [];
  markdownLanguage.parser.parse(chars.join("")).iterate({
    enter(node) {
      if (node.name !== "Table" || node.node.parent?.name !== "Document") return;
      const { from, to } = node;
      const attrs = { format: "markdown" };
      result.push({ name: "markdown-table", from, to, contentFrom: from, contentTo: to, attrs,
        open: { name: "markdown-table", from, to: from, attrs, closing: false, selfClosing: false, attrNames: [] } });
      return false;
    },
  });
  return result;
}
