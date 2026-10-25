<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/elements/table/view.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Cell formatting keeps numbers numeric](../../../../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md)

Captured SHA-256: `0e0c17bf0f75a379952f4f11e746f8c347815348141c9b8f3b1174f59c537bb7`.

<book:entry path="apps/web/src/elements/table/view.tsx">
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
</book:entry>

</book:doc>
