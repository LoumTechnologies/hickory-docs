<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/markdownTables.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Cell formatting keeps numbers numeric](../../../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md)
- [Markdown tables use the table editor](../../../../../../guarantees/authoring/markdown-tables-use-the-table-editor/contract.md)

Captured SHA-256: `036ca7897b6248b6e3c51e972862ae7e274adc88b6813ec925e1348ae00e4e9e`.

<book:entry path="apps/web/src/editor/markdownTables.ts">
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
      let { from } = node;
      const { to } = node;
      // Include only an immediately adjacent format comment in the table edit.
      const before = text.slice(0, from);
      const metadata = /(?:^|\n)(<!-- hickory-table-formats: [^\r\n]* -->\r?\n)$/.exec(before);
      if (metadata) from -= metadata[1].length;
      const attrs = { format: "markdown" };
      result.push({ name: "markdown-table", from, to, contentFrom: from, contentTo: to, attrs,
        open: { name: "markdown-table", from, to: from, attrs, closing: false, selfClosing: false, attrNames: [] } });
      return false;
    },
  });
  return result;
}
</book:entry>

</book:doc>
