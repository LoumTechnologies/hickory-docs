<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/markdownTables.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6f87bd812952"></a>
## `apps/web/src/editor/markdownTables.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Cell formatting keeps numbers numeric](../../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md), [Markdown tables use the table editor](../../../../../guarantees/authoring/markdown-tables-use-the-table-editor/contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/editor/markdownTables.ts">
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
</storycode:entry>

</storycode:doc>
