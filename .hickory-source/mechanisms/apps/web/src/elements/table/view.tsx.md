<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/elements/table/view.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2971b5282a4d"></a>
## `apps/web/src/elements/table/view.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Cell formatting keeps numbers numeric](../../../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md).

<a id="f-2971b5282a4d-tableView"></a>
### `tableView`

<storycode:copy id="f-2971b5282a4d-tableView">
import { markdownTableBlocks } from "../../editor/markdownTables";
import { structureOf } from "../../editor/wysiwyg";
import { replaceTable } from "../../editor/tableEdit";
import { MarkdownTable } from "../../components/MarkdownTable";
import { Engaged } from "../../components/Engaged";
import { TablePanel } from "../../components/TablePanel";
import { tableKey } from "../../lib/uiState";
import type { ElementView } from "../types";

/** `<hick:table>`: CSV in the document, drawn and edited as a grid. */
export const tableView: ElementView = {
  kind: "table",
  draws: (block) => block.name === "table" || block.name === "markdown-table",
  render(slot, cx) {
    // Named by the table's own `path` where it has one, so the size
    // survives prose being written above it — see `tableKey`.
    const document = cx.view?.state.doc;
    const structure = cx.view ? structureOf(cx.view.state) : null;
    const block = structure && document ? structure.blocks.find(b => b.from === slot.at) ?? markdownTableBlocks(structure, document.toString()).find(b => b.from === slot.at) : null;
    const anchored = !!block && cx.view?.state.doc.sliceString(block.contentFrom, block.contentTo) === slot.text;
    const sourceCurrent = () => anchored && !!cx.view?.dom.isConnected && !cx.view.state.readOnly && cx.view.state.doc === document;
    const Panel = slot.table?.format === "markdown" ? MarkdownTable : TablePanel;
    const key = tableKey(cx.path, slot.index, slot.table?.path);
    return (
      // Behind the engage gate: at rest the wheel scrolls the DOCUMENT
      // through the grid; clicking into the table is what buys its own
      // scrolling (styles.css hides the internal overflow until then).
      <Engaged className="rendered-table rendered-table--laned">
        <Panel
          sourceCurrent={sourceCurrent}
          sourceIdentity={`${cx.path ?? "unnamed document"}#table:${slot.index}@${slot.at}`}
          laneRight
          layout={cx.tableLayouts?.[key]}
          onLayout={(size) => cx.onTableLayout?.(key, size)}
          source={slot.table?.format === "markdown" ? slot.text : slot.text.replace(/^\r?\n/, "")}
          header={slot.table?.header ?? true}
          delimiter={slot.table?.delimiter}
          language={slot.table?.language}
          formats={slot.table?.formats ?? ""}
          // An edit in the grid rewrites the CSV in the document, in place.
          // The document stays the source of truth — there is no second
          // copy of the table anywhere — which is what keeps the file a
          // file somebody reviews in a diff.
          onChange={cx.view && !cx.view.state.readOnly ? (csv, formats = "", actor) => slot.table?.format === "markdown"
            ? cx.replaceBlockContent(slot, csv, actor?.kind === "agent" ? "input.uiAgentTable" : "input.table") : replaceTable(slot, cx, csv, formats, actor?.kind === "agent") : undefined}
        />
      </Engaged>
    );
  },
};
</storycode:copy>

### Assemble `apps/web/src/elements/table/view.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/table/view.tsx"><storycode:paste select="#f-2971b5282a4d-tableView"/></storycode:file>

</storycode:doc>
