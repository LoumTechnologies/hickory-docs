import { Engaged } from "../../components/Engaged";
import { TablePanel } from "../../components/TablePanel";
import { tableKey } from "../../lib/uiState";
import type { ElementView } from "../types";

/** `<hick:table>`: CSV in the document, drawn and edited as a grid. */
export const tableView: ElementView = {
  kind: "table",
  draws: (block) => block.name === "table",
  render(slot, cx) {
    // Named by the table's own `path` where it has one, so the size
    // survives prose being written above it — see `tableKey`.
    const key = tableKey(cx.path, slot.index, slot.table?.path);
    return (
      // Behind the engage gate: at rest the wheel scrolls the DOCUMENT
      // through the grid; clicking into the table is what buys its own
      // scrolling (styles.css hides the internal overflow until then).
      <Engaged className="rendered-table rendered-table--laned">
        <TablePanel
          laneRight
          layout={cx.tableLayouts?.[key]}
          onLayout={(size) => cx.onTableLayout?.(key, size)}
          source={slot.text}
          header={slot.table?.header ?? true}
          delimiter={slot.table?.delimiter}
          language={slot.table?.language}
          // An edit in the grid rewrites the CSV in the document, in place.
          // The document stays the source of truth — there is no second
          // copy of the table anywhere — which is what keeps the file a
          // file somebody reviews in a diff.
          onChange={(csv) => cx.replaceBlockContent(slot, csv)}
        />
      </Engaged>
    );
  },
};
