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

import { readFormats, writeFormats } from "../lib/tableFormats";
import { useId, useMemo, useState } from "react";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiActor, UiOutcome } from "../lib/uiActions";
import { TablePanel } from "./TablePanel";

export function FenceTable({
  body,
  onChange,
  formats,
  onPromote,
  sourceCurrent,
}: {
  /** The CSV between the fence markers. */
  body: string;
  /** Replace it. */
  onChange: (csv: string, formats?: string, actor?: UiActor) => void;
  formats?: string;
  /** Turn the fence into a `<hick:table>`, writing its CSV to `path` when one
   * is given. */
  onPromote: (path: string) => boolean | void;
  sourceCurrent?: () => boolean;
}) {
  const [path, setPath] = useState("");
  const [error, setError] = useState<string | null>(null);
  const parent = `fence-table:${useId()}`;
  const callbackRevision = useMemo(() => crypto.randomUUID(), [onPromote, sourceCurrent]);
  const current = () => sourceCurrent?.() !== false;
  const revision = useUiRevision(() => [body, formats, path, callbackRevision, current()]);
  const changePath = (text: string) => { if (text.length <= 4096 && current()) setPath(text); };
  const promote = (): UiOutcome => {
    if (!current()) return { status: "unavailable", reason: "Fence source changed or became read-only; reopen its current controls" };
    try {
      const changed = onPromote(path.trim());
      if (changed === false) { const reason = "The owner refused changed source. Inspect and reopen the current fence."; setError(reason); return { status: "requires_input", request: `${parent}:source`, reason }; }
      return { status: "completed", value: { edit_requested: true, source_changed: changed === true, file_written: false, saved: false } };
    } catch (cause) { setError(String(cause)); return { status: "unavailable", reason: String(cause) }; }
  };
  useUiCommands([
    { key: `${parent}:path`, command: { parent, subject: parent, label: "Set fence dataset path", parameters: { path: { type: "string", maxLength: 4096 } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: current() }), execute: async args => { changePath(args.path as string); return { status: "completed", value: { draft_updated: true } }; } } },
    { key: `${parent}:promote`, command: { parent, subject: parent, label: "Promote fence to table element", parameters: {}, prerequisites: ["Changes document source; output writing requires the existing save/weave pipeline"],
      read: () => ({ revision: revision(), enabled: current() }), execute: async () => promote() } },
  ]);
  useUiReading("fence dataset promotion", { path, error, source_current: current(), scope: "Table data has its own grid reading. Promotion changes source; it does not itself save or write the output file." });

  return (
    <div className="fence-table">
      <TablePanel source={body} sourceCurrent={sourceCurrent} formats={formats} onChange={onChange} />
      <div className="fence-table__promote">
        {error && <p role="alert">{error}</p>}
        <p className="muted fence-table__why">
          Make it a dataset this document owns: the same rows, written to a
          file a script can read, still shown here as a table.
        </p>
        <div className="fence-table__row">
          <input
            className="fence-table__path mono"
            type="text"
            value={path}
            maxLength={4096}
            disabled={!current()}
            placeholder="data/sales.csv"
            aria-label="File to write the CSV to"
            onChange={(event) => changePath(event.target.value)}
          />
          <button
            type="button"
            className="btn btn-small btn-primary"
            disabled={!current()}
            onClick={() => promote()}
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
export function tableElementFor(body: string, path: string, formats = ""): string {
  let attrs = path ? ` path="${path.replace(/"/g, "&quot;")}"` : "";
  const normalized = writeFormats(readFormats(formats));
  if (normalized) attrs += ` formats="${normalized}"`;
  const csv = body.replace(/\n+$/, "");
  return `<hick:table${attrs}>\n${csv}\n</hick:table>`;
}
