// A ```csv fence, edited as a grid.
//
// A fence is PROSE — a block of text in a document, no different from a
// paragraph as far as the language is concerned. So this edits it in place
// and changes nothing else: the fence stays a fence, the CSV stays the bytes
// between the markers, and the diff is the cells that moved.
//
// The other half is the promotion. A `<hick:table path="…">` is the same data
// with a job: the CSV is written to a file, so a script or a query can read
// it, and the document still shows a table. That is the step from "a table I
// pasted into my notes" to "the dataset this document owns", and it is worth
// being an explicit act rather than something that happens to a fence when
// nobody was looking.

import { useState } from "react";

import { TablePanel } from "./TablePanel";

export function FenceTable({
  body,
  onChange,
  onPromote,
}: {
  /** The CSV between the fence markers. */
  body: string;
  /** Replace it. */
  onChange: (csv: string) => void;
  /** Turn the fence into a `<hick:table>`, writing its CSV to `path` when one
   * is given. */
  onPromote: (path: string) => void;
}) {
  const [path, setPath] = useState("");

  return (
    <div className="fence-table">
      <TablePanel source={body} onChange={onChange} />
      <div className="fence-table__promote">
        <p className="muted fence-table__why">
          Make it a dataset this document owns: the same rows, written to a
          file a script can read, still shown here as a table.
        </p>
        <div className="fence-table__row">
          <input
            className="fence-table__path mono"
            type="text"
            value={path}
            placeholder="data/sales.csv"
            aria-label="File to write the CSV to"
            onChange={(event) => setPath(event.target.value)}
          />
          <button
            type="button"
            className="btn btn-small btn-primary"
            onClick={() => onPromote(path.trim())}
            data-tip={
              path.trim()
                ? "Convert to a <hick:table>, writing the CSV to that file"
                : "Convert to a <hick:table> — leave the path empty for a table that writes no file"
            }
          >
            Make it a table
          </button>
        </div>
      </div>
    </div>
  );
}

/**
 * The `<hick:table>` a fence becomes.
 *
 * The CSV is carried across untouched — that is the point of the conversion,
 * and a promotion that reformatted the data would be a promotion nobody could
 * check.
 */
export function tableElementFor(body: string, path: string): string {
  const attrs = path ? ` path="${path.replace(/"/g, "&quot;")}"` : "";
  const csv = body.replace(/\n+$/, "");
  return `<hick:table${attrs}>\n${csv}\n</hick:table>`;
}
