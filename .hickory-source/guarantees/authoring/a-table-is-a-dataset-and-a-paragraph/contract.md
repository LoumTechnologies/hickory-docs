<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Table Is A Dataset And A Paragraph

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:table>` in a document, when the document is woven, then the
reader sees a markdown table; and when the tag carries a `path`, then those
same rows are written there as CSV, byte for byte, for anything that is not
this app to read.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)
- [Requirements 3](requirements/03.md)
- [Requirements 4](requirements/04.md)
- [Requirements 5](requirements/05.md)
- [Requirements 6](requirements/06.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-table-is-a-dataset-and-a-paragraph.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />
- [Requirement detail](requirements/03.md)
<storycode:include file="requirements/03.md" />
- [Requirement detail](requirements/04.md)
<storycode:include file="requirements/04.md" />
- [Requirement detail](requirements/05.md)
<storycode:include file="requirements/05.md" />
- [Requirement detail](requirements/06.md)
<storycode:include file="requirements/06.md" />

## Implementation

<a id="output-f351518007ac"></a>
## `apps/web/src/components/ContextMenu.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Table Is A Dataset And A Paragraph](contract.md).

<a id="f-f351518007ac-ContextMenuItem"></a>
### `ContextMenuItem`

One right-click menu, for everything in the app that has one.
Extracted from the folder tree's menu (shell/TreeContextMenu.tsx), which is
now a caller rather than the only implementation. The behaviour is the part
worth having exactly once: it closes on Escape, on a click anywhere else,
and on a scroll or resize that would leave it floating over the wrong
thing; it takes focus on the first item so the Windows/Linux context-menu
key reaches the whole menu with no pointer involved; and it keeps itself
inside the window, because a menu opened near the bottom edge otherwise has
a last item nobody can click.
It never traps focus. A menu you cannot escape is worse than one you have
to aim at twice.

<storycode:copy id="f-f351518007ac-ContextMenuItem">
// One right-click menu, for everything in the app that has one.
//
// Extracted from the folder tree's menu (shell/TreeContextMenu.tsx), which is
// now a caller rather than the only implementation. The behaviour is the part
// worth having exactly once: it closes on Escape, on a click anywhere else,
// and on a scroll or resize that would leave it floating over the wrong
// thing; it takes focus on the first item so the Windows/Linux context-menu
// key reaches the whole menu with no pointer involved; and it keeps itself
// inside the window, because a menu opened near the bottom edge otherwise has
// a last item nobody can click.
//
// It never traps focus. A menu you cannot escape is worse than one you have
// to aim at twice.

import { useEffect, useId, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";

export interface ContextMenuItem {
  /** Stable across renders and platforms; what a test clicks by. */
  id: string;
  label: string;
  /** Offered but not available right now, with the tooltip saying why. An
   * item that vanishes teaches nobody where it went. */
  disabled?: boolean;
  /** Why it is disabled, or what it will do. */
  tip?: string;
  /** A separator line is drawn above this item. */
  group?: boolean;
  /** Removes something. Drawn so the hand slows down before it lands. */
  danger?: boolean;
}

</storycode:copy>
<a id="f-f351518007ac-ContextMenuProps"></a>
### `ContextMenuProps`

<storycode:copy id="f-f351518007ac-ContextMenuProps">
export interface ContextMenuProps {
  /** Viewport coordinates of the click that opened it. */
  x: number;
  y: number;
  items: readonly ContextMenuItem[];
  /** What is being acted on, as a heading. */
  subject: string;
  onPick: (id: string) => unknown | Promise<unknown>;
  onClose: () => unknown | Promise<unknown>;
  /** Extra class on the box, for a caller with its own look. */
  className?: string;
}

</storycode:copy>
<a id="f-f351518007ac-menuBox"></a>
### `menuBox`

Roughly how tall the menu will be, for keeping it on screen. Measured
against the CSS in styles.css (`.tree-menu__item`); a few pixels out only
moves the box a few pixels, which is why an estimate is enough.

<storycode:copy id="f-f351518007ac-menuBox">
/** Roughly how tall the menu will be, for keeping it on screen. Measured
 * against the CSS in styles.css (`.tree-menu__item`); a few pixels out only
 * moves the box a few pixels, which is why an estimate is enough. */
export function menuBox(
  items: readonly ContextMenuItem[],
  x: number,
  y: number,
  viewport: { width: number; height: number },
): { left: number; top: number; width: number } {
  const width = Math.min(240, Math.max(0, viewport.width - 8));
  const height = items.length * 28 + items.filter((i) => i.group).length * 9 + 36;
  return {
    left: Math.max(4, Math.min(x, viewport.width - width - 4)),
    top: Math.max(4, Math.min(y, viewport.height - height - 4)),
    width,
  };
}

</storycode:copy>
<a id="f-f351518007ac-ContextMenu"></a>
### `ContextMenu`

<storycode:copy id="f-f351518007ac-ContextMenu">
export function ContextMenu({
  x,
  y,
  items,
  subject,
  onPick,
  onClose,
  className,
}: ContextMenuProps) {
  const box = useRef<HTMLDivElement | null>(null);
  const parent = `context-menu:${useId()}`;
  const callbackRevision = useMemo(() => crypto.randomUUID(), [onPick, onClose]);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const lock = useRef(false), mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const revision = useUiRevision(() => [subject, items, callbackRevision, pending]);
  const pick = async (id: string): Promise<UiOutcome> => {
    const item = items.find(entry => entry.id === id);
    if (lock.current || !mounted.current || (!item || item.disabled || items.filter(entry => entry.id === id).length !== 1))
      return { status: "unavailable", reason: "Menu item is closed, busy, ambiguous or disabled" };
    lock.current = true; setPending(true); setError(null);
    try {
      await onPick(id);
      return { status: "completed", value: { callback_requested: true, item: id, scope: "Menu callback completed; inspect its owning operation for effects" } };
    } catch (cause) {
      if (mounted.current) setError(String(cause));
      return { status: "unavailable", reason: String(cause) };
    } finally { lock.current = false; if (mounted.current) setPending(false); }
  };
  useUiCommands([
    { key: `${parent}:close`, command: { parent, subject: `${parent}:visibility`, label: `Close menu for ${subject}`, parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true }), execute: async () => {
        await onClose();
        return { status: "completed", value: { dismissal_requested: true, operation_cancelled: false } };
      } } },
    ...items.map(item => ({ key: `${parent}:pick:${item.id}`, command: { parent, subject: parent, label: item.label, parameters: {},
      prerequisites: item.tip ? [item.tip] : [], read: () => ({ revision: revision(), enabled: !lock.current && !item.disabled }), execute: () => pick(item.id) } })),
  ]);
  useUiReading(`menu for ${subject}`, { subject, items, pending, error });

  useEffect(() => {
    const previous = document.activeElement;
    const menu = box.current;
    menu?.querySelector<HTMLButtonElement>("button:not([disabled])")?.focus();
    return () => {
      if ((document.activeElement === document.body || menu?.contains(document.activeElement)) &&
          previous instanceof HTMLElement && previous.isConnected) previous.focus();
    };
  }, []);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        onClose();
      }
    };
    const onPointer = (event: MouseEvent) => {
      if (!box.current?.contains(event.target as Node)) onClose();
    };
    const onScroll = (event: Event) => {
      if (!box.current?.contains(event.target as Node)) onClose();
    };
    window.addEventListener("keydown", onKey, true);
    window.addEventListener("mousedown", onPointer, true);
    window.addEventListener("contextmenu", onPointer, true);
    window.addEventListener("resize", onClose);
    window.addEventListener("scroll", onScroll, true);
    return () => {
      window.removeEventListener("keydown", onKey, true);
      window.removeEventListener("mousedown", onPointer, true);
      window.removeEventListener("contextmenu", onPointer, true);
      window.removeEventListener("resize", onClose);
      window.removeEventListener("scroll", onScroll, true);
    };
  }, [onClose]);

  const { left, top, width } = menuBox(items, x, y, {
    width: window.innerWidth,
    height: window.innerHeight,
  });

  const search = useRef({ text: "", time: 0 });
  return createPortal(
    <div
      ref={box}
      className={`tree-menu${className ? ` ${className}` : ""}`}
      onKeyDown={(event) => {
        const buttons = Array.from(box.current?.querySelectorAll<HTMLButtonElement>("button:not([disabled])") ?? []);
        if (event.key === "Tab") { onClose(); return; }
        if (event.altKey || event.ctrlKey || event.metaKey || !buttons.length) return;
        const current = buttons.indexOf(document.activeElement as HTMLButtonElement);
        let next: HTMLButtonElement | undefined;
        if (event.key === "ArrowDown") next = buttons[(current + 1) % buttons.length];
        else if (event.key === "ArrowUp") next = buttons[(current - 1 + buttons.length) % buttons.length];
        else if (event.key === "Home") next = buttons[0];
        else if (event.key === "End") next = buttons[buttons.length - 1];
        else if (event.key.length === 1 && event.key !== " ") {
          const now = Date.now();
          const letter = event.key.toLocaleLowerCase();
          search.current = { text: now - search.current.time > 700 ? letter : search.current.text + letter, time: now };
          const query = [...search.current.text].every(char => char === letter) ? letter : search.current.text;
          next = [...buttons.slice(current + 1), ...buttons.slice(0, current + 1)]
            .find(button => button.textContent?.trim().toLocaleLowerCase().startsWith(query));
        } else return;
        event.preventDefault();
        event.stopPropagation();
        next?.focus();
      }}
      role="menu"
      aria-label={`Actions for ${subject}`}
      style={{ left: `${left}px`, top: `${top}px`, width: `${width}px` }}
    >
      <p className="tree-menu__title" data-tip={subject}>
        {subject}
      </p>
      {error && <p role="alert">{error}</p>}
      {items.map((item) => (
        <button
          key={item.id}
          tabIndex={-1}
          onPointerMove={(event) => { if (!item.disabled && !pending) event.currentTarget.focus({ preventScroll: true }); }}
          type="button"
          role="menuitem"
          data-menu-item={item.id}
          disabled={item.disabled || pending}
          data-tip={item.tip}
          className={`tree-menu__item${item.group ? " tree-menu__item--group" : ""}${
            item.danger ? " tree-menu__item--danger" : ""
          }`}
          // The grid takes its selection on mousedown; a press on a menu item
          // must not move it out from under the action about to run.
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => void pick(item.id)}
        >
          {item.label}
        </button>
      ))}
    </div>,
    document.body,
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/ContextMenu.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ContextMenu.tsx"><storycode:paste select="#f-f351518007ac-ContextMenuItem"/><storycode:paste select="#f-f351518007ac-ContextMenuProps"/><storycode:paste select="#f-f351518007ac-menuBox"/><storycode:paste select="#f-f351518007ac-ContextMenu"/></storycode:file>

<a id="output-3c939bfc5a94"></a>
## `apps/web/src/components/FenceTable.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Table Is A Dataset And A Paragraph](contract.md).

<a id="f-3c939bfc5a94-FenceTable"></a>
### `FenceTable`

A ```csv fence, edited as a grid.
A fence is PROSE — a block of text in a document, no different from a
paragraph as far as the language is concerned. So this edits it in place
and changes nothing else: the fence stays a fence, the CSV stays the bytes
between the markers, and the diff is the cells that moved.
The other half is the promotion. A `<hick:table path="…">` is the same data
with a job: the CSV is written to a file, so a script or a query can read
it, and the document still shows a table. That is the step from "a table I
pasted into my notes" to "the dataset this document owns", and it is worth
being an explicit act rather than something that happens to a fence when
nobody was looking.

<storycode:copy id="f-3c939bfc5a94-FenceTable">
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

</storycode:copy>
<a id="f-3c939bfc5a94-tableElementFor"></a>
### `tableElementFor`

The `<hick:table>` a fence becomes.
The CSV is carried across untouched — that is the point of the conversion,
and a promotion that reformatted the data would be a promotion nobody could
check.

<storycode:copy id="f-3c939bfc5a94-tableElementFor">
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
</storycode:copy>

### Assemble `apps/web/src/components/FenceTable.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/FenceTable.tsx"><storycode:paste select="#f-3c939bfc5a94-FenceTable"/><storycode:paste select="#f-3c939bfc5a94-tableElementFor"/></storycode:file>

<a id="output-7ae5e1d2e203"></a>
## `apps/web/src/components/TableSizeDialog.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Table Is A Dataset And A Paragraph](contract.md).

<a id="f-7ae5e1d2e203-MAX_CELLS"></a>
### `MAX_CELLS`

"3 × 2", typed rather than clicked.
The toolbar can add a row and remove a row, which is the right shape for
editing a table and the wrong shape for MAKING one: a person who wants a
12 × 5 table is not going to press "+ Row" eleven times, and if that is the
only way, they will type the CSV by hand instead — which is the outcome the
grid exists to make unnecessary.
So the size indicator, which already says what the size IS, is also where
it is set. That is the whole idea: the number that reports a fact is the
number you edit to change it.
Shrinking is a destructive act and says so before it happens. Not a
confirm-then-do dialog, which trains people to click through — the count of
what is about to go is simply on screen next to the button, while the
number is still being typed.

<storycode:copy id="f-7ae5e1d2e203-MAX_CELLS">
// "3 × 2", typed rather than clicked.
//
// The toolbar can add a row and remove a row, which is the right shape for
// editing a table and the wrong shape for MAKING one: a person who wants a
// 12 × 5 table is not going to press "+ Row" eleven times, and if that is the
// only way, they will type the CSV by hand instead — which is the outcome the
// grid exists to make unnecessary.
//
// So the size indicator, which already says what the size IS, is also where
// it is set. That is the whole idea: the number that reports a fact is the
// number you edit to change it.
//
// Shrinking is a destructive act and says so before it happens. Not a
// confirm-then-do dialog, which trains people to click through — the count of
// what is about to go is simply on screen next to the button, while the
// number is still being typed.

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiActor, UiCommand, UiOutcome } from "../lib/uiActions";
import { useId, useState } from "react";

/**
 * The most cells a table may be set to.
 *
 * The same 20,000 the formula route refuses past, for a related reason: a
 * grid that big is a dataset rather than a spreadsheet, and drawing it would
 * lock the window up for a table nobody can read anyway. A person who wants
 * one wants a script to write it.
 */
export const MAX_CELLS = 20_000;

</storycode:copy>
<a id="f-7ae5e1d2e203-TableSizeDialogProps"></a>
### `TableSizeDialogProps`

<storycode:copy id="f-7ae5e1d2e203-TableSizeDialogProps">
export interface TableSizeDialogProps {
  rows: number;
  columns: number;
  /** How many cells hold something now, per row and per column, so shrinking
   * can say what it is about to take rather than only how much. */
  filledRows: number;
  filledColumns: number;
  onApply: (rows: number, columns: number, actor?: UiActor) => void | boolean;
  sourceCurrent?: () => boolean;
  owner?: string;
  onCancel: () => void;
}

</storycode:copy>
<a id="f-7ae5e1d2e203-TableSizeDialog"></a>
### `TableSizeDialog`

<storycode:copy id="f-7ae5e1d2e203-TableSizeDialog">
export function TableSizeDialog({
  rows,
  columns,
  filledRows,
  filledColumns,
  onApply,
  sourceCurrent,
  owner,
  onCancel,
}: TableSizeDialogProps) {
  // Held as text, not as numbers: a number input that snaps an empty field
  // back to 1 cannot be cleared and retyped, which is the first thing anybody
  // does to it.
  const [wantRows, setWantRows] = useState(String(rows));
  const [wantColumns, setWantColumns] = useState(String(columns));

  const asked = { rows: Number(wantRows), columns: Number(wantColumns) };
  const whole = (value: number) => Number.isInteger(value) && value >= 1;
  const valid = whole(asked.rows) && whole(asked.columns);
  const tooBig = valid && asked.rows * asked.columns > MAX_CELLS;

  const losingRows = valid ? Math.max(0, filledRows - asked.rows) : 0;
  const losingColumns = valid ? Math.max(0, filledColumns - asked.columns) : 0;

  const parent = `table-size:${useId()}`;
  const [error, setError] = useState("");
  const current = () => sourceCurrent?.() ?? true;
  const revision = useUiRevision(() => [rows, columns, filledRows, filledColumns, wantRows, wantColumns, current(), error]);
  const submit = (actor: UiActor): UiOutcome => {
    if (!valid || tooBig || !current()) return { status: "unavailable", reason: "Choose valid dimensions for the current source (at most 20000 cells)" };
    const result = onApply(asked.rows, asked.columns, actor);
    return result === false ? { status: "unavailable", reason: "Owning table refused the resize; inspect current source" } : { status: "completed", value: { resize_handler_returned: true, source_write: result === true ? "dispatched" : "unconfirmed; inspect source", saved: "not established" } };
  };
  const commands: { key: string; command: UiCommand }[] = [
    { key: `${parent}:draft`, command: { label: "Set table size draft", parent, subject: parent, parameters: { rows: { type: "string", maxLength: 32 }, columns: { type: "string", maxLength: 32 } }, prerequisites: [], read: () => ({ revision: revision(), value: { owner: owner ?? null }, enabled: true }), execute: async args => { setWantRows(args.rows as string); setWantColumns(args.columns as string); return { status: "completed", value: { draft_changed: true, resized: false } }; } } },
    { key: `${parent}:apply`, command: { label: "Apply table size", parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), value: { owner: owner ?? null }, enabled: valid && !tooBig && current(), reason: !current() ? "Owning source changed/closed/read-only" : !valid || tooBig ? "Use positive whole dimensions and at most 20000 cells" : undefined }), execute: async (_args, actor) => submit(actor) } },
    { key: `${parent}:cancel`, command: { label: "Cancel table size dialog", parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), value: { owner: owner ?? null }, enabled: true }), execute: async () => { onCancel(); return { status: "completed", value: { close_requested: true } }; } } },
    { key: `${parent}:read`, command: { label: "Read table size state", parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), value: { owner: owner ?? null }, enabled: true }), execute: async () => ({ status: "completed", value: { owner: owner ?? null, rows, columns, draft_rows: wantRows, draft_columns: wantColumns, valid, too_big: tooBig, losing_rows: losingRows, losing_columns: losingColumns, write_scope_current: current(), error } }) } },
  ];
  const invoke = useUiCommands(commands);
  const click = (key: string) => { void invoke(`${parent}:${key}`).catch(reason => setError(String(reason))); };
  return (
    <div
      className="table-size"
      role="dialog"
      aria-label="Table size"
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          click("cancel");
        }
      }}
    >
      {error && <p role="alert">{error}</p>}
      <form
        className="table-size__form"
        onSubmit={(event) => {
          event.preventDefault();
          if (!valid || tooBig) return;
          click("apply");
        }}
      >
        <label className="table-size__field">
          Rows
          {/* eslint-disable-next-line jsx-a11y/no-autofocus */}
          <input
            autoFocus
            className="table-size__number"
            type="number"
            min={1}
            inputMode="numeric"
            value={wantRows}
            onChange={(event) => setWantRows(event.target.value)}
          />
        </label>
        <span className="table-size__by" aria-hidden="true">
          ×
        </span>
        <label className="table-size__field">
          Columns
          <input
            className="table-size__number"
            type="number"
            min={1}
            inputMode="numeric"
            value={wantColumns}
            onChange={(event) => setWantColumns(event.target.value)}
          />
        </label>
        <button type="submit" className="btn btn-small" disabled={!valid || tooBig || !current()}>
          Apply
        </button>
        <button type="button" className="btn btn-small" onClick={() => click("cancel")}>
          Cancel
        </button>
      </form>

      {/* One line, and only when it has something to say. A dialog that always
          carries a warning is a dialog nobody reads. */}
      {!valid ? (
        <p className="table-size__note muted" role="status">
          A table is at least one row by one column. Whole numbers only.
        </p>
      ) : tooBig ? (
        <p className="table-size__note table-size__note--bad" role="status">
          {asked.rows.toLocaleString()} × {asked.columns.toLocaleString()} is{" "}
          {(asked.rows * asked.columns).toLocaleString()} cells, past the{" "}
          {MAX_CELLS.toLocaleString()}-cell limit. A grid that size is a dataset rather than a
          spreadsheet — write it from an exec cell and point the table at the file.
        </p>
      ) : losingRows > 0 || losingColumns > 0 ? (
        <p className="table-size__note table-size__note--warn" role="status">
          Removes {describe(losingRows, "row")}
          {losingRows > 0 && losingColumns > 0 ? " and " : ""}
          {describe(losingColumns, "column")}, and what is in{" "}
          {losingRows + losingColumns === 1 ? "it" : "them"}.
        </p>
      ) : null}
    </div>
  );
}

</storycode:copy>
<a id="f-7ae5e1d2e203-describe"></a>
### `describe`

"1 row", "3 rows", or nothing at all — so the sentence above reads as a
sentence in every case rather than as "Removes 0 rows and 2 columns".

<storycode:copy id="f-7ae5e1d2e203-describe">
/** "1 row", "3 rows", or nothing at all — so the sentence above reads as a
 * sentence in every case rather than as "Removes 0 rows and 2 columns". */
function describe(count: number, noun: string): string {
  if (count === 0) return "";
  return `${count} ${noun}${count === 1 ? "" : "s"}`;
}
</storycode:copy>

### Assemble `apps/web/src/components/TableSizeDialog.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/TableSizeDialog.tsx"><storycode:paste select="#f-7ae5e1d2e203-MAX_CELLS"/><storycode:paste select="#f-7ae5e1d2e203-TableSizeDialogProps"/><storycode:paste select="#f-7ae5e1d2e203-TableSizeDialog"/><storycode:paste select="#f-7ae5e1d2e203-describe"/></storycode:file>

<a id="output-43450f4cc8c1"></a>
## `apps/web/src/lib/csv.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Table Is A Dataset And A Paragraph](contract.md).

<a id="f-43450f4cc8c1-rows"></a>
### `rows`

<storycode:copy id="f-43450f4cc8c1-rows">
import { describe, expect, it } from "vitest";
import {
  cellAt,
  clearCells,
  columnCount,
  filledSize,
  guessDelimiter,
  insertColumn,
  insertColumnWithFormulaReferences,
  insertRow,
  insertRowWithFormulaReferences,
  needsQuoting,
  parseCsv,
  pasteBlock,
  removeColumn,
  removeColumns,
  removeRow,
  removeRows,
  resizeTable,
  toMarkdown,
  withCell,
  writeCsv,
  writeField,
} from "./csv";

const rows = (text: string, delimiter?: string) => parseCsv(text, delimiter).rows;

</storycode:copy>
<a id="f-43450f4cc8c1-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-43450f4cc8c1-expression_statement">
describe("reading CSV", () => {
  it("reads plain rows and fields", () => {
    expect(rows("a,b\n1,2\n")).toEqual([
      ["a", "b"],
      ["1", "2"],
    ]);
  });

  it("reads a quoted field holding the delimiter", () => {
    expect(rows('name,note\nAda,"one, two"\n')).toEqual([
      ["name", "note"],
      ["Ada", "one, two"],
    ]);
  });

  it("reads a doubled quote as one quote", () => {
    expect(rows('a\n"she said ""hi"""\n')).toEqual([["a"], ['she said "hi"']]);
  });

  it("reads a newline inside a quoted field", () => {
    expect(rows('a,b\n"line one\nline two",x\n')).toEqual([
      ["a", "b"],
      ["line one\nline two", "x"],
    ]);
  });

  it("reads a bare quote in the middle of a field literally", () => {
    // What a spreadsheet export produces, and what RFC 4180 leaves to the
    // reader. Refusing it would be refusing a real file.
    expect(rows('a\n5" pipe\n')).toEqual([["a"], ['5" pipe']]);
  });

  it("keeps a ragged row ragged", () => {
    // Padding on read would write four fields back out and claim the file
    // said something it did not. The GRID pads for display.
    expect(rows("a,b,c\n1,2\n")).toEqual([
      ["a", "b", "c"],
      ["1", "2"],
    ]);
  });

  it("does not invent a row for the trailing newline", () => {
    expect(rows("a\nb\n")).toEqual([["a"], ["b"]]);
    expect(rows("a\nb")).toEqual([["a"], ["b"]]);
  });

  it("reads an empty text as an empty table", () => {
    expect(rows("")).toEqual([[""]]);
  });

  it("never throws on an unterminated quote", () => {
    // A table editor that refuses to open a file is useless exactly when it
    // is most needed.
    expect(() => parseCsv('a,"unterminated\n')).not.toThrow();
    expect(rows('a,"unterminated\n')).toEqual([["a", "unterminated\n"]]);
  });
});

</storycode:copy>
<a id="f-43450f4cc8c1-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-43450f4cc8c1-expression_statement-2">
describe("guessing the delimiter", () => {
  it("finds tabs, semicolons and pipes", () => {
    expect(guessDelimiter("a\tb\n1\t2\n")).toBe("\t");
    expect(guessDelimiter("a;b\n1;2\n")).toBe(";");
    expect(guessDelimiter("a|b\n1|2\n")).toBe("|");
  });

  it("prefers the one that is consistent per row", () => {
    // A comma inside prose is not a delimiter; a real one appears the same
    // number of times on every line.
    expect(guessDelimiter("a;b;c\none, two;three;four\nx;y;z\n")).toBe(";");
  });

  it("ignores delimiters inside quotes", () => {
    expect(guessDelimiter('a;b\n"one;two";x\n')).toBe(";");
  });

  it("falls back to a comma when there is nothing to go on", () => {
    expect(guessDelimiter("")).toBe(",");
    expect(guessDelimiter("just one column\n")).toBe(",");
  });
});

</storycode:copy>
<a id="f-43450f4cc8c1-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-43450f4cc8c1-expression_statement-3">
describe("writing CSV back", () => {
  it("quotes only what has to be quoted", () => {
    // Quoting everything is valid and unreadable, and it turns a one-cell
    // edit into a whole-file diff.
    expect(writeField("plain", ",")).toBe("plain");
    expect(writeField("one, two", ",")).toBe('"one, two"');
    expect(writeField('say "hi"', ",")).toBe('"say ""hi"""');
    expect(writeField("two\nlines", ",")).toBe('"two\nlines"');
    expect(writeField(" padded ", ",")).toBe('" padded "');
    expect(needsQuoting("plain", ",")).toBe(false);
  });

  it("round-trips a file byte for byte", () => {
    for (const text of [
      "a,b\n1,2\n",
      "a,b\n1,2",
      'name,note\nAda,"one, two"\n',
      "a\tb\n1\t2\n",
      'x\n"multi\nline"\n',
    ]) {
      expect(writeCsv(parseCsv(text))).toBe(text);
    }
  });

  it("keeps the line ending the file came with", () => {
    // A Windows CSV silently becoming a Unix one shows up as every line
    // changed.
    const text = "a,b\r\n1,2\r\n";
    expect(writeCsv(parseCsv(text))).toBe(text);
  });

  it("changes exactly one line when one cell is edited", () => {
    const before = "a,b\n1,2\n3,4\n";
    const after = writeCsv(withCell(parseCsv(before), 1, 1, "TWO"));
    expect(after).toBe("a,b\n1,TWO\n3,4\n");
  });
});

</storycode:copy>
<a id="f-43450f4cc8c1-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-43450f4cc8c1-expression_statement-4">
describe("editing the grid", () => {
  const table = parseCsv("a,b,c\n1,2,3\n");

  it("pads only as far as it must when a ragged row is edited", () => {
    const ragged = parseCsv("a,b,c\n1\n");
    expect(withCell(ragged, 1, 2, "z").rows[1]).toEqual(["1", "", "z"]);
  });

  it("inserts and removes rows", () => {
    expect(insertRow(table, 1).rows).toEqual([["a", "b", "c"], ["", "", ""], ["1", "2", "3"]]);
    expect(removeRow(table, 0).rows).toEqual([["1", "2", "3"]]);
  });

  it("inserts and removes columns in every row", () => {
    expect(insertColumn(table, 1).rows).toEqual([
      ["a", "", "b", "c"],
      ["1", "", "2", "3"],
    ]);
    expect(removeColumn(table, 1).rows).toEqual([
      ["a", "c"],
      ["1", "3"],
    ]);
  });

  it("moves formula references with inserted rows and columns", () => {
    expect(writeCsv(insertRowWithFormulaReferences(parseCsv("1,=A1*2\n"), 0))).toBe(
      ",\n1,=A2*2\n",
    );
    expect(writeCsv(insertColumnWithFormulaReferences(parseCsv("1,=A1*2\n"), 0))).toBe(
      ",1,=B1*2\n",
    );
  });

  it("survives removing the last row", () => {
    expect(removeRow(parseCsv("only\n"), 0).rows).toEqual([]);
  });

  it("counts columns from the widest row", () => {
    expect(columnCount(parseCsv("a,b,c\n1\n"))).toBe(3);
  });

  it("shows a short row's missing cells as empty", () => {
    const ragged = parseCsv("a,b,c\n1\n");
    expect(cellAt(ragged, 1, 0)).toBe("1");
    expect(cellAt(ragged, 1, 2)).toBe("");
    expect(cellAt(ragged, 9, 9)).toBe("");
  });
});

</storycode:copy>
<a id="f-43450f4cc8c1-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-43450f4cc8c1-expression_statement-5">
describe("the markdown a table weaves to", () => {
  it("writes a header row and a rule", () => {
    expect(toMarkdown(parseCsv("name,age\nAda,36\n"))).toBe(
      "| name | age |\n| --- | --- |\n| Ada | 36 |",
    );
  });

  it("escapes a pipe, which markdown has no quoting for", () => {
    // A cell containing `|` would silently become two cells.
    expect(toMarkdown(parseCsv('a\n"x|y"\n'))).toContain("x\\|y");
  });

  it("pads a ragged row out to the table's width", () => {
    expect(toMarkdown(parseCsv("a,b\n1\n"))).toBe("| a | b |\n| --- | --- |\n| 1 |  |");
  });

  it("gives a headerless table an empty header rather than inventing names", () => {
    const out = toMarkdown(parseCsv("1,2\n"), false);
    expect(out.split("\n")[0]).toBe("|  |  |");
    expect(out).toContain("| 1 | 2 |");
  });

  it("writes nothing for an empty table", () => {
    expect(toMarkdown({ rows: [] })).toBe("");
  });
});

</storycode:copy>
<a id="f-43450f4cc8c1-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-43450f4cc8c1-expression_statement-6">
describe("acting on more than one cell", () => {
  it("empties every cell it is given and leaves the rest alone", () => {
    const out = clearCells(parseCsv("a,b,c\n1,2,3\n"), [
      { row: 1, column: 0 },
      { row: 1, column: 2 },
    ]);
    expect(writeCsv(out)).toBe("a,b,c\n,2,\n");
  });

  it("will not pad a ragged row to clear a cell that was never in the file", () => {
    // The file keeps its ragged rows until somebody edits one.
    const out = clearCells(parseCsv("a,b,c\n1\n"), [{ row: 1, column: 2 }]);
    expect(writeCsv(out)).toBe("a,b,c\n1\n");
  });

  it("removes several rows without the earlier ones shifting the later ones", () => {
    expect(writeCsv(removeRows(parseCsv("1\n2\n3\n4\n"), [0, 2]))).toBe("2\n4\n");
  });

  it("removes several columns the same way", () => {
    expect(writeCsv(removeColumns(parseCsv("a,b,c,d\n1,2,3,4\n"), [1, 3]))).toBe("a,c\n1,3\n");
  });
});

</storycode:copy>
<a id="f-43450f4cc8c1-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-43450f4cc8c1-expression_statement-7">
describe("setting the size by hand", () => {
  it("grows with empty cells", () => {
    expect(writeCsv(resizeTable(parseCsv("a,b\n1,2\n"), 3, 3))).toBe("a,b,\n1,2,\n,,\n");
  });

  it("trims from the end, which is what a smaller number means", () => {
    expect(writeCsv(resizeTable(parseCsv("a,b,c\n1,2,3\n4,5,6\n"), 2, 2))).toBe("a,b\n1,2\n");
  });

  it("squares up a ragged row, because an explicit size is an instruction", () => {
    // The one place padding is right: everywhere else it would claim the file
    // said something it did not.
    expect(writeCsv(resizeTable(parseCsv("a,b,c\n1\n"), 2, 3))).toBe("a,b,c\n1,,\n");
  });

  it("keeps the line ending and the trailing newline the file came with", () => {
    expect(writeCsv(resizeTable(parseCsv("a,b\r\n1,2\r\n"), 1, 2))).toBe("a,b\r\n");
  });

  it("will not make a table with no rows or no columns", () => {
    expect(writeCsv(resizeTable(parseCsv("a,b\n1,2\n"), 0, 0))).toBe("a\n");
  });
});

</storycode:copy>
<a id="f-43450f4cc8c1-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-43450f4cc8c1-expression_statement-8">
describe("how much of a table holds something", () => {
  it("is smaller than the shape when the end is empty", () => {
    // What a shrink would really take, so "removes 4 rows" is said only when
    // there is something in them.
    expect(filledSize(parseCsv("a,b,\n1,2,\n,,\n"))).toEqual({ rows: 2, columns: 2 });
  });

  it("counts a cell of only spaces as empty, as the grid draws it", () => {
    expect(filledSize(parseCsv("a\n\" \"\n"))).toEqual({ rows: 1, columns: 1 });
  });

  it("is zero for a table with nothing in it", () => {
    expect(filledSize(parseCsv(",\n,\n"))).toEqual({ rows: 0, columns: 0 });
  });
});

</storycode:copy>
<a id="f-43450f4cc8c1-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-43450f4cc8c1-expression_statement-9">
describe("writing a block of cells in", () => {
  it("lands it with its corner where it was asked for", () => {
    const out = pasteBlock(parseCsv("a,b,c\n1,2,3\n4,5,6\n"), 1, 1, [
      ["x", "y"],
      ["z", "w"],
    ]);
    expect(writeCsv(out)).toBe("a,b,c\n1,x,y\n4,z,w\n");
  });

  it("grows the table rather than dropping what does not fit", () => {
    // A paste is an instruction: four rows pasted into the last row means the
    // table now has three more.
    const out = pasteBlock(parseCsv("a\n1\n"), 1, 0, [["x"], ["y"], ["z"]]);
    expect(writeCsv(out)).toBe("a\n x\ny\nz\n".replace(" ", ""));
  });

  it("grows sideways the same way", () => {
    expect(writeCsv(pasteBlock(parseCsv("a\n1\n"), 0, 1, [["b", "c"]]))).toBe("a,b,c\n1\n");
  });

  it("leaves a ragged row it did not touch exactly as ragged", () => {
    expect(writeCsv(pasteBlock(parseCsv("a,b,c\n1\n2,3,4\n"), 2, 0, [["x"]]))).toBe(
      "a,b,c\n1\nx,3,4\n",
    );
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/csv.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/csv.test.ts"><storycode:paste select="#f-43450f4cc8c1-rows"/><storycode:paste select="#f-43450f4cc8c1-expression_statement"/><storycode:paste select="#f-43450f4cc8c1-expression_statement-2"/><storycode:paste select="#f-43450f4cc8c1-expression_statement-3"/><storycode:paste select="#f-43450f4cc8c1-expression_statement-4"/><storycode:paste select="#f-43450f4cc8c1-expression_statement-5"/><storycode:paste select="#f-43450f4cc8c1-expression_statement-6"/><storycode:paste select="#f-43450f4cc8c1-expression_statement-7"/><storycode:paste select="#f-43450f4cc8c1-expression_statement-8"/><storycode:paste select="#f-43450f4cc8c1-expression_statement-9"/></storycode:file>

<a id="output-4460b44bfc5a"></a>
## `apps/web/src/lib/csv.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Table Is A Dataset And A Paragraph](contract.md), [Cell formatting keeps numbers numeric](../cell-formatting-keeps-numbers-numeric/contract.md).

<a id="f-4460b44bfc5a-Csv"></a>
### `Csv`

CSV, parsed and written back byte-for-byte where nothing changed.
A table editor's whole job is to be a nicer way to edit a file that is
still a file. That puts an unusual demand on the parser: it is not enough
to read a table correctly, it has to WRITE one that a person would have
typed, because the result lands in a document somebody else reviews in a
diff. A parser that round-trips `a,b` as `"a","b"` turns a one-cell edit
into a whole-file rewrite, and the diff stops being reviewable.
So the rules here are RFC 4180's, plus one of our own:
- A field is quoted only when it HAS to be — when it holds the delimiter,
a quote, a newline, or leading/trailing space. Quoting everything is
valid and unreadable.
- The line ending the file already used is the line ending it keeps. A
Windows CSV edited here does not silently become a Unix one, which would
show up as every line changed.
- A trailing newline is preserved if it was there and not invented if it
was not.
Ragged rows are kept ragged on the way in — a row with three fields where
the header has four is a real thing that happens, and padding it on read
would write four fields back out and claim the file said something it did
not. The GRID pads for display; the file keeps what it had until a cell in
that row is actually edited.

<storycode:copy id="f-4460b44bfc5a-Csv">
// CSV, parsed and written back byte-for-byte where nothing changed.
//
// A table editor's whole job is to be a nicer way to edit a file that is
// still a file. That puts an unusual demand on the parser: it is not enough
// to read a table correctly, it has to WRITE one that a person would have
// typed, because the result lands in a document somebody else reviews in a
// diff. A parser that round-trips `a,b` as `"a","b"` turns a one-cell edit
// into a whole-file rewrite, and the diff stops being reviewable.
//
// So the rules here are RFC 4180's, plus one of our own:
//
//  - A field is quoted only when it HAS to be — when it holds the delimiter,
//    a quote, a newline, or leading/trailing space. Quoting everything is
//    valid and unreadable.
//  - The line ending the file already used is the line ending it keeps. A
//    Windows CSV edited here does not silently become a Unix one, which would
//    show up as every line changed.
//  - A trailing newline is preserved if it was there and not invented if it
//    was not.
//
// Ragged rows are kept ragged on the way in — a row with three fields where
// the header has four is a real thing that happens, and padding it on read
// would write four fields back out and claim the file said something it did
// not. The GRID pads for display; the file keeps what it had until a cell in
// that row is actually edited.

import { moveFormats, type CellFormats } from "./tableFormats";
import { shiftFormulaReferences } from "./formulaReferences";

/** A parsed table: rows of fields, exactly as many as each row had. */
export interface Csv {
  rows: string[][];
  formats?: CellFormats;
  /** The delimiter that was found (or the one that was asked for). */
  delimiter: string;
  /** "\n" or "\r\n" — whichever the text used. */
  newline: string;
  /** Whether the text ended with a line break. */
  trailingNewline: boolean;
}

</storycode:copy>
<a id="f-4460b44bfc5a-CANDIDATES"></a>
### `CANDIDATES`

Delimiters worth guessing between. Tab and semicolon are what a European
spreadsheet and a TSV export produce, and both are common enough that
guessing wrong once is worse than looking.

<storycode:copy id="f-4460b44bfc5a-CANDIDATES">
/** Delimiters worth guessing between. Tab and semicolon are what a European
 * spreadsheet and a TSV export produce, and both are common enough that
 * guessing wrong once is worse than looking. */
const CANDIDATES = [",", "\t", ";", "|"] as const;

</storycode:copy>
<a id="f-4460b44bfc5a-guessDelimiter"></a>
### `guessDelimiter`

Which delimiter a text uses.
Counted OUTSIDE quotes, on the first few lines, and the winner is the one
that appears the same number of times on every line — a real delimiter is
consistent per row, and a comma inside prose is not. Ties go to the comma,
because it is the one the format is named after.

<storycode:copy id="f-4460b44bfc5a-guessDelimiter">
/**
 * Which delimiter a text uses.
 *
 * Counted OUTSIDE quotes, on the first few lines, and the winner is the one
 * that appears the same number of times on every line — a real delimiter is
 * consistent per row, and a comma inside prose is not. Ties go to the comma,
 * because it is the one the format is named after.
 */
export function guessDelimiter(text: string): string {
  const sample = text.split(/\r?\n/).filter((l) => l.trim() !== "").slice(0, 10);
  if (sample.length === 0) return ",";
  let best = ",";
  let bestScore = -1;
  for (const candidate of CANDIDATES) {
    const counts = sample.map((line) => countOutsideQuotes(line, candidate));
    if (counts[0] === 0) continue;
    const consistent = counts.every((n) => n === counts[0]);
    // Consistency first, then how many fields it makes: `a,b;c,d` is two
    // semicolon-fields and three comma-fields, and the commas are the table.
    const score = (consistent ? 1000 : 0) + counts[0];
    if (score > bestScore) {
      bestScore = score;
      best = candidate;
    }
  }
  return bestScore < 0 ? "," : best;
}

</storycode:copy>
<a id="f-4460b44bfc5a-countOutsideQuotes"></a>
### `countOutsideQuotes`

<storycode:copy id="f-4460b44bfc5a-countOutsideQuotes">
function countOutsideQuotes(line: string, delimiter: string): number {
  let count = 0;
  let quoted = false;
  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (ch === '"') {
      // A doubled quote inside a quoted field is an escaped quote, not a
      // close followed by an open.
      if (quoted && line[i + 1] === '"') i++;
      else quoted = !quoted;
    } else if (!quoted && ch === delimiter) {
      count++;
    }
  }
  return count;
}

</storycode:copy>
<a id="f-4460b44bfc5a-parseCsv"></a>
### `parseCsv`

Parse `text` as CSV. Never throws: malformed input is read as best it can
be, because a table editor that refuses to open a file is useless exactly
when it is most needed.

<storycode:copy id="f-4460b44bfc5a-parseCsv">
/** Parse `text` as CSV. Never throws: malformed input is read as best it can
 * be, because a table editor that refuses to open a file is useless exactly
 * when it is most needed. */
export function parseCsv(text: string, delimiter?: string): Csv {
  const sep = delimiter ?? guessDelimiter(text);
  const newline = /\r\n/.test(text) ? "\r\n" : "\n";
  const trailingNewline = /\r?\n$/.test(text);
  const rows: string[][] = [];
  let row: string[] = [];
  let field = "";
  let quoted = false;
  let i = 0;

  const endField = () => {
    row.push(field);
    field = "";
  };
  const endRow = () => {
    endField();
    rows.push(row);
    row = [];
  };

  while (i < text.length) {
    const ch = text[i];
    if (quoted) {
      if (ch === '"') {
        if (text[i + 1] === '"') {
          field += '"';
          i += 2;
          continue;
        }
        quoted = false;
        i++;
        continue;
      }
      field += ch;
      i++;
      continue;
    }
    if (ch === '"' && field === "") {
      // Only at the START of a field: a quote in the middle of `ab"cd` is a
      // literal quote, which is what a spreadsheet export produces.
      quoted = true;
      i++;
      continue;
    }
    if (ch === sep) {
      endField();
      i++;
      continue;
    }
    if (ch === "\r" && text[i + 1] === "\n") {
      endRow();
      i += 2;
      continue;
    }
    if (ch === "\n" || ch === "\r") {
      endRow();
      i++;
      continue;
    }
    field += ch;
    i++;
  }
  // An unterminated quoted field still ends the row it was in, rather than
  // losing it.
  if (field !== "" || row.length > 0 || !trailingNewline) endRow();

  // A trailing newline produces one empty trailing row; it is the line ending
  // of the last row, not a row of its own.
  if (trailingNewline && rows.length > 0) {
    const last = rows[rows.length - 1];
    if (last.length === 1 && last[0] === "") rows.pop();
  }
  return { rows, delimiter: sep, newline, trailingNewline };
}

</storycode:copy>
<a id="f-4460b44bfc5a-needsQuoting"></a>
### `needsQuoting`

Whether a field has to be quoted to survive a round trip.

<storycode:copy id="f-4460b44bfc5a-needsQuoting">
/** Whether a field has to be quoted to survive a round trip. */
export function needsQuoting(field: string, delimiter: string): boolean {
  return (
    field.includes(delimiter) ||
    field.includes('"') ||
    field.includes("\n") ||
    field.includes("\r") ||
    field !== field.trim()
  );
}

</storycode:copy>
<a id="f-4460b44bfc5a-writeField"></a>
### `writeField`

One field, written the way a person would have typed it.

<storycode:copy id="f-4460b44bfc5a-writeField">
/** One field, written the way a person would have typed it. */
export function writeField(field: string, delimiter: string): string {
  if (!needsQuoting(field, delimiter)) return field;
  return `"${field.replace(/"/g, '""')}"`;
}

</storycode:copy>
<a id="f-4460b44bfc5a-writeCsv"></a>
### `writeCsv`

Write a table back out.
Quotes only what has to be quoted, and keeps the line ending and the
trailing newline the file came with. The point is a reviewable diff: a
one-cell edit should be a one-line change.

<storycode:copy id="f-4460b44bfc5a-writeCsv">
/**
 * Write a table back out.
 *
 * Quotes only what has to be quoted, and keeps the line ending and the
 * trailing newline the file came with. The point is a reviewable diff: a
 * one-cell edit should be a one-line change.
 */
export function writeCsv(table: Csv): string {
  const body = table.rows
    .map((row) => row.map((field) => writeField(field, table.delimiter)).join(table.delimiter))
    .join(table.newline);
  return table.trailingNewline && body !== "" ? body + table.newline : body;
}

</storycode:copy>
<a id="f-4460b44bfc5a-columnCount"></a>
### `columnCount`

The widest row's length — how many columns the grid draws.

<storycode:copy id="f-4460b44bfc5a-columnCount">
/** The widest row's length — how many columns the grid draws. */
export function columnCount(table: Pick<Csv, "rows">): number {
  return table.rows.reduce((n, row) => Math.max(n, row.length), 0);
}

</storycode:copy>
<a id="f-4460b44bfc5a-cellAt"></a>
### `cellAt`

The value of one cell, padding for DISPLAY only.
A row shorter than the header is a real thing that happens; the grid shows
it as empty cells, and the file keeps its ragged row until somebody edits
one of them.

<storycode:copy id="f-4460b44bfc5a-cellAt">
/**
 * The value of one cell, padding for DISPLAY only.
 *
 * A row shorter than the header is a real thing that happens; the grid shows
 * it as empty cells, and the file keeps its ragged row until somebody edits
 * one of them.
 */
export function cellAt(table: Pick<Csv, "rows">, row: number, column: number): string {
  return table.rows[row]?.[column] ?? "";
}

</storycode:copy>
<a id="f-4460b44bfc5a-withCell"></a>
### `withCell`

The table with one cell replaced, padding that row only as far as it must.

<storycode:copy id="f-4460b44bfc5a-withCell">
/** The table with one cell replaced, padding that row only as far as it must. */
export function withCell(table: Csv, row: number, column: number, value: string): Csv {
  const rows = table.rows.map((r) => [...r]);
  while (rows.length <= row) rows.push([]);
  const target = rows[row];
  while (target.length <= column) target.push("");
  target[column] = value;
  return { ...table, rows };
}

</storycode:copy>
<a id="f-4460b44bfc5a-clearCells"></a>
### `clearCells`

The table with a set of cells emptied.
A cell PAST the end of its own row is skipped rather than padded: the file
keeps its ragged rows until somebody edits one, and clearing a cell that
was never there would write fields the file did not have.

<storycode:copy id="f-4460b44bfc5a-clearCells">
/**
 * The table with a set of cells emptied.
 *
 * A cell PAST the end of its own row is skipped rather than padded: the file
 * keeps its ragged rows until somebody edits one, and clearing a cell that
 * was never there would write fields the file did not have.
 */
export function clearCells(table: Csv, cells: { row: number; column: number }[]): Csv {
  const rows = table.rows.map((r) => [...r]);
  for (const { row, column } of cells) {
    const target = rows[row];
    if (target && column < target.length) target[column] = "";
  }
  return { ...table, rows };
}

</storycode:copy>
<a id="f-4460b44bfc5a-removeRows"></a>
### `removeRows`

Remove several rows at once — a selection of them, from the toolbar.
Removed from the bottom up, so earlier removals do not shift the indices
of the ones still to come.

<storycode:copy id="f-4460b44bfc5a-removeRows">
/** Remove several rows at once — a selection of them, from the toolbar.
 * Removed from the bottom up, so earlier removals do not shift the indices
 * of the ones still to come. */
export function removeRows(table: Csv, rows: number[]): Csv {
  return [...rows].sort((a, b) => b - a).reduce(removeRow, table);
}

</storycode:copy>
<a id="f-4460b44bfc5a-removeColumns"></a>
### `removeColumns`

Remove several columns at once, bottom-up for the same reason.

<storycode:copy id="f-4460b44bfc5a-removeColumns">
/** Remove several columns at once, bottom-up for the same reason. */
export function removeColumns(table: Csv, columns: number[]): Csv {
  return [...columns].sort((a, b) => b - a).reduce(removeColumn, table);
}

</storycode:copy>
<a id="f-4460b44bfc5a-pasteBlock"></a>
### `pasteBlock`

A block of cells written into the table with its top-left corner at
(`row`, `column`), growing the table as far as it has to.
A paste is an instruction, so it grows: pasting four rows into the last row
of a table means the table now has three more. It does not, however, tidy
anything it did not touch — a ragged row two rows above stays ragged.

<storycode:copy id="f-4460b44bfc5a-pasteBlock">
/**
 * A block of cells written into the table with its top-left corner at
 * (`row`, `column`), growing the table as far as it has to.
 *
 * A paste is an instruction, so it grows: pasting four rows into the last row
 * of a table means the table now has three more. It does not, however, tidy
 * anything it did not touch — a ragged row two rows above stays ragged.
 */
export function pasteBlock(
  table: Csv,
  row: number,
  column: number,
  block: string[][],
): Csv {
  const rows = table.rows.map((r) => [...r]);
  block.forEach((fields, down) => {
    const at = row + down;
    while (rows.length <= at) rows.push([]);
    const target = rows[at];
    fields.forEach((field, across) => {
      const into = column + across;
      while (target.length <= into) target.push("");
      target[into] = field;
    });
  });
  return { ...table, rows };
}

</storycode:copy>
<a id="f-4460b44bfc5a-filledSize"></a>
### `filledSize`

How much of the table actually holds something.
Not the same as its shape: a table can be twelve rows tall with the last
four empty. What this answers is what a shrink would really take, so
"removes 4 rows" can be said only when there is something in them.

<storycode:copy id="f-4460b44bfc5a-filledSize">
/**
 * How much of the table actually holds something.
 *
 * Not the same as its shape: a table can be twelve rows tall with the last
 * four empty. What this answers is what a shrink would really take, so
 * "removes 4 rows" can be said only when there is something in them.
 */
export function filledSize(table: Pick<Csv, "rows">): { rows: number; columns: number } {
  let rows = 0;
  let columns = 0;
  table.rows.forEach((row, index) => {
    row.forEach((field, column) => {
      if (field.trim() === "") return;
      rows = Math.max(rows, index + 1);
      columns = Math.max(columns, column + 1);
    });
  });
  return { rows, columns };
}

</storycode:copy>
<a id="f-4460b44bfc5a-resizeTable"></a>
### `resizeTable`

The table at exactly `rows` × `columns`.
Grows with empty cells and trims from the end — the end, because a table is
read from the top left, and taking rows off the bottom is what "make it
smaller" means to the person who typed a smaller number.
This is the one place a ragged row is squared up, and deliberately: padding
for DISPLAY would claim the file said something it did not, but an explicit
"make this table four columns wide" is an instruction rather than a guess.

<storycode:copy id="f-4460b44bfc5a-resizeTable">
/**
 * The table at exactly `rows` × `columns`.
 *
 * Grows with empty cells and trims from the end — the end, because a table is
 * read from the top left, and taking rows off the bottom is what "make it
 * smaller" means to the person who typed a smaller number.
 *
 * This is the one place a ragged row is squared up, and deliberately: padding
 * for DISPLAY would claim the file said something it did not, but an explicit
 * "make this table four columns wide" is an instruction rather than a guess.
 */
export function resizeTable(table: Csv, rows: number, columns: number): Csv {
  const wanted = Math.max(1, Math.floor(rows));
  const wide = Math.max(1, Math.floor(columns));
  const next: string[][] = [];
  for (let row = 0; row < wanted; row++) {
    const existing = table.rows[row] ?? [];
    const fields = existing.slice(0, wide);
    while (fields.length < wide) fields.push("");
    next.push(fields);
  }
  return { ...table, rows: next, formats: moveFormats(table.formats, (row, column) => row < wanted && column < wide ? { row, column } : null) };
}

</storycode:copy>
<a id="f-4460b44bfc5a-insertRow"></a>
### `insertRow`

Insert an empty row at `at` (0-based; `rows.length` appends).

<storycode:copy id="f-4460b44bfc5a-insertRow">
/** Insert an empty row at `at` (0-based; `rows.length` appends). */
export function insertRow(table: Csv, at: number): Csv {
  const width = columnCount(table);
  const rows = table.rows.map((r) => [...r]);
  const index = Math.max(0, Math.min(at, rows.length));
  rows.splice(index, 0, new Array(width).fill(""));
  return { ...table, rows, formats: moveFormats(table.formats, (row, column) => ({ row: row >= index ? row + 1 : row, column })) };
}

</storycode:copy>
<a id="f-4460b44bfc5a-insertRowWithFormulaReferences"></a>
### `insertRowWithFormulaReferences`

Insert a row and keep formulas aimed at the cells that moved.

<storycode:copy id="f-4460b44bfc5a-insertRowWithFormulaReferences">
/** Insert a row and keep formulas aimed at the cells that moved. */
export function insertRowWithFormulaReferences(table: Csv, at: number): Csv {
  const index = Math.max(0, Math.min(at, table.rows.length));
  const inserted = insertRow(table, index);
  return rewriteFormulas(inserted, (formula) => shiftFormulaReferences(formula, "row", index));
}

</storycode:copy>
<a id="f-4460b44bfc5a-removeRow"></a>
### `removeRow`

Remove a row. Removing the last one leaves an empty table, not a broken
one.

<storycode:copy id="f-4460b44bfc5a-removeRow">
/** Remove a row. Removing the last one leaves an empty table, not a broken
 * one. */
export function removeRow(table: Csv, at: number): Csv {
  const rows = table.rows.map((r) => [...r]);
  if (at >= 0 && at < rows.length) rows.splice(at, 1);
  return { ...table, rows, formats: moveFormats(table.formats, (row, column) => row === at ? null : ({ row: row > at ? row - 1 : row, column })) };
}

</storycode:copy>
<a id="f-4460b44bfc5a-insertColumn"></a>
### `insertColumn`

<storycode:copy id="f-4460b44bfc5a-insertColumn">
/** Insert an empty column at `at` in every row. */
export function insertColumn(table: Csv, at: number): Csv {
  const width = columnCount(table);
  const index = Math.max(0, Math.min(at, width));
  const rows = table.rows.map((row) => {
    const next = [...row];
    // Pad only up to the insertion point: a ragged row stays as ragged as it
    // was on the right-hand side.
    while (next.length < index) next.push("");
    next.splice(index, 0, "");
    return next;
  });
  return { ...table, rows, formats: moveFormats(table.formats, (row, column) => ({ row, column: column >= index ? column + 1 : column })) };
}

</storycode:copy>
<a id="f-4460b44bfc5a-insertColumnWithFormulaReferences"></a>
### `insertColumnWithFormulaReferences`

Insert a column and keep formulas aimed at the cells that moved.

<storycode:copy id="f-4460b44bfc5a-insertColumnWithFormulaReferences">
/** Insert a column and keep formulas aimed at the cells that moved. */
export function insertColumnWithFormulaReferences(table: Csv, at: number): Csv {
  const index = Math.max(0, Math.min(at, columnCount(table)));
  const inserted = insertColumn(table, index);
  return rewriteFormulas(inserted, (formula) => shiftFormulaReferences(formula, "column", index));
}

</storycode:copy>
<a id="f-4460b44bfc5a-rewriteFormulas"></a>
### `rewriteFormulas`

<storycode:copy id="f-4460b44bfc5a-rewriteFormulas">
function rewriteFormulas(table: Csv, rewrite: (formula: string) => string): Csv {
  return { ...table, rows: table.rows.map((row) => row.map(rewrite)) };
}

</storycode:copy>
<a id="f-4460b44bfc5a-removeColumn"></a>
### `removeColumn`

<storycode:copy id="f-4460b44bfc5a-removeColumn">
/** Remove a column from every row. */
export function removeColumn(table: Csv, at: number): Csv {
  const rows = table.rows.map((row) => {
    const next = [...row];
    if (at >= 0 && at < next.length) next.splice(at, 1);
    return next;
  });
  return { ...table, rows, formats: moveFormats(table.formats, (row, column) => column === at ? null : ({ row, column: column > at ? column - 1 : column })) };
}

</storycode:copy>
<a id="f-4460b44bfc5a-toMarkdown"></a>
### `toMarkdown`

A markdown table, for the weave.
The JS twin of what `hick weave` writes, used by the browser demo and by
anything that needs the rendered form without a round trip. Pipes and
backslashes are escaped because a markdown table has no quoting of its own
— a cell containing `|` would silently become two cells.

<storycode:copy id="f-4460b44bfc5a-toMarkdown">
/**
 * A markdown table, for the weave.
 *
 * The JS twin of what `hick weave` writes, used by the browser demo and by
 * anything that needs the rendered form without a round trip. Pipes and
 * backslashes are escaped because a markdown table has no quoting of its own
 * — a cell containing `|` would silently become two cells.
 */
export function toMarkdown(table: Pick<Csv, "rows">, header = true): string {
  const width = columnCount(table);
  if (width === 0 || table.rows.length === 0) return "";
  const cell = (value: string) => value.replace(/\\/g, "\\\\").replace(/\|/g, "\\|");
  const line = (row: string[]) =>
    `| ${Array.from({ length: width }, (_, i) => cell(row[i] ?? "")).join(" | ")} |`;
  const out: string[] = [];
  const [first, ...rest] = table.rows;
  if (header) {
    out.push(line(first));
    out.push(`|${" --- |".repeat(width)}`);
    for (const row of rest) out.push(line(row));
  } else {
    // No header row means an empty one: markdown has no table without a
    // header, and inventing column names would be inventing content.
    out.push(`|${"  |".repeat(width)}`);
    out.push(`|${" --- |".repeat(width)}`);
    for (const row of table.rows) out.push(line(row));
  }
  return out.join("\n");
}

</storycode:copy>
<a id="f-4460b44bfc5a-isTabularFence"></a>
### `isTabularFence`

Whether a fence's info string names a table.
`csv` and `tsv` only. Not `data`, not `txt`: a fence gets a GRID instead of
the "make it a cell" converter on the strength of this, and guessing wrong
takes away the converter from something that wanted it.

<storycode:copy id="f-4460b44bfc5a-isTabularFence">
/**
 * Whether a fence's info string names a table.
 *
 * `csv` and `tsv` only. Not `data`, not `txt`: a fence gets a GRID instead of
 * the "make it a cell" converter on the strength of this, and guessing wrong
 * takes away the converter from something that wanted it.
 */
export function isTabularFence(info: string): boolean {
  const language = info.trim().split(/\s+/)[0]?.toLowerCase() ?? "";
  return language === "csv" || language === "tsv";
}
</storycode:copy>

### Assemble `apps/web/src/lib/csv.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/csv.ts"><storycode:paste select="#f-4460b44bfc5a-Csv"/><storycode:paste select="#f-4460b44bfc5a-CANDIDATES"/><storycode:paste select="#f-4460b44bfc5a-guessDelimiter"/><storycode:paste select="#f-4460b44bfc5a-countOutsideQuotes"/><storycode:paste select="#f-4460b44bfc5a-parseCsv"/><storycode:paste select="#f-4460b44bfc5a-needsQuoting"/><storycode:paste select="#f-4460b44bfc5a-writeField"/><storycode:paste select="#f-4460b44bfc5a-writeCsv"/><storycode:paste select="#f-4460b44bfc5a-columnCount"/><storycode:paste select="#f-4460b44bfc5a-cellAt"/><storycode:paste select="#f-4460b44bfc5a-withCell"/><storycode:paste select="#f-4460b44bfc5a-clearCells"/><storycode:paste select="#f-4460b44bfc5a-removeRows"/><storycode:paste select="#f-4460b44bfc5a-removeColumns"/><storycode:paste select="#f-4460b44bfc5a-pasteBlock"/><storycode:paste select="#f-4460b44bfc5a-filledSize"/><storycode:paste select="#f-4460b44bfc5a-resizeTable"/><storycode:paste select="#f-4460b44bfc5a-insertRow"/><storycode:paste select="#f-4460b44bfc5a-insertRowWithFormulaReferences"/><storycode:paste select="#f-4460b44bfc5a-removeRow"/><storycode:paste select="#f-4460b44bfc5a-insertColumn"/><storycode:paste select="#f-4460b44bfc5a-insertColumnWithFormulaReferences"/><storycode:paste select="#f-4460b44bfc5a-rewriteFormulas"/><storycode:paste select="#f-4460b44bfc5a-removeColumn"/><storycode:paste select="#f-4460b44bfc5a-toMarkdown"/><storycode:paste select="#f-4460b44bfc5a-isTabularFence"/></storycode:file>

<a id="output-fc3e01f3b793"></a>
## `apps/web/src/lib/tableClipboard.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Table Is A Dataset And A Paragraph](contract.md).

<a id="f-fc3e01f3b793-Block"></a>
### `Block`

The grid's half of the clipboard.
A table people can select a rectangle in is a table people will try to copy
out of and paste into, and the other end of that is almost never this app:
it is Excel, or Google Sheets, or a `<table>` on a web page, or a block of
CSV in a chat message. So the formats here are the ones those programs
already speak, and the rule is to WRITE what everything reads and READ
whatever arrives.
Writing is two flavours of the same rectangle, because a clipboard carries
several and the receiving program picks:
- `text/plain` as TAB-separated lines. This is what a spreadsheet writes
and what one expects to be given; comma-separated text pasted into
Excel lands in a single column, which is the single most annoying thing
a clipboard can do.
- `text/html` as a real `<table>`, which is what Excel, Word and Sheets
read when they want structure rather than text.
Reading prefers the HTML when it holds a table, because that is the flavour
that survived a cell containing a newline; otherwise the plain text is
parsed as delimited data, with the delimiter guessed exactly as an opened
file's is (lib/csv.ts).

<storycode:copy id="f-fc3e01f3b793-Block">
// The grid's half of the clipboard.
//
// A table people can select a rectangle in is a table people will try to copy
// out of and paste into, and the other end of that is almost never this app:
// it is Excel, or Google Sheets, or a `<table>` on a web page, or a block of
// CSV in a chat message. So the formats here are the ones those programs
// already speak, and the rule is to WRITE what everything reads and READ
// whatever arrives.
//
// Writing is two flavours of the same rectangle, because a clipboard carries
// several and the receiving program picks:
//
//   - `text/plain` as TAB-separated lines. This is what a spreadsheet writes
//     and what one expects to be given; comma-separated text pasted into
//     Excel lands in a single column, which is the single most annoying thing
//     a clipboard can do.
//   - `text/html` as a real `<table>`, which is what Excel, Word and Sheets
//     read when they want structure rather than text.
//
// Reading prefers the HTML when it holds a table, because that is the flavour
// that survived a cell containing a newline; otherwise the plain text is
// parsed as delimited data, with the delimiter guessed exactly as an opened
// file's is (lib/csv.ts).

import { currencyInput } from "./tableFormats";
import { parseCsv } from "./csv";

/** A rectangle of cells, row-major. */
export type Block = string[][];

</storycode:copy>
<a id="f-fc3e01f3b793-toTabSeparated"></a>
### `toTabSeparated`

The rectangle as `text/plain`.
A field is quoted only when it has to be — when it holds a tab, a newline,
or a quote — which is the rule Excel writes by, and the rule that keeps an
ordinary table pasteable into anything at all.

<storycode:copy id="f-fc3e01f3b793-toTabSeparated">
/**
 * The rectangle as `text/plain`.
 *
 * A field is quoted only when it has to be — when it holds a tab, a newline,
 * or a quote — which is the rule Excel writes by, and the rule that keeps an
 * ordinary table pasteable into anything at all.
 */
export function toTabSeparated(block: Block): string {
  return block
    .map((row) =>
      row
        .map((field) =>
          /[\t\r\n"]/.test(field) ? `"${field.replace(/"/g, '""')}"` : field,
        )
        .join("\t"),
    )
    // CRLF, which is what every spreadsheet puts between rows and what some of
    // them still need to see.
    .join("\r\n");
}

</storycode:copy>
<a id="f-fc3e01f3b793-toHtmlTable"></a>
### `toHtmlTable`

The rectangle as `text/html`: a real table, so a program that wants
structure gets structure.

<storycode:copy id="f-fc3e01f3b793-toHtmlTable">
/** The rectangle as `text/html`: a real table, so a program that wants
 * structure gets structure. */
export function toHtmlTable(block: Block): string {
  const escape = (text: string) =>
    text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      // A newline inside a cell is the reason the HTML flavour exists; keeping
      // it as a `<br>` is what makes it survive the trip.
      .replace(/\n/g, "<br>");
  const rows = block
    .map((row) => `<tr>${row.map((field) => `<td>${escape(field)}</td>`).join("")}</tr>`)
    .join("");
  return `<table>${rows}</table>`;
}

</storycode:copy>
<a id="f-fc3e01f3b793-parseHtmlTable"></a>
### `parseHtmlTable`

The first table in a fragment of HTML, or null if there is not one.
`colspan` pads with empty cells so the columns after it still line up.
`rowspan` is NOT reconstructed — the cells below a spanned one shift left,
exactly as they do when the same table is pasted into a spreadsheet as
values. Saying so is better than a half-implementation that is right for
one merge and wrong for two.

<storycode:copy id="f-fc3e01f3b793-parseHtmlTable">
/**
 * The first table in a fragment of HTML, or null if there is not one.
 *
 * `colspan` pads with empty cells so the columns after it still line up.
 * `rowspan` is NOT reconstructed — the cells below a spanned one shift left,
 * exactly as they do when the same table is pasted into a spreadsheet as
 * values. Saying so is better than a half-implementation that is right for
 * one merge and wrong for two.
 */
export function parseHtmlTable(html: string): Block | null {
  if (!html.trim()) return null;
  let document: Document;
  try {
    document = new DOMParser().parseFromString(html, "text/html");
  } catch {
    return null;
  }
  const table = document.querySelector("table");
  if (!table) return null;
  const block: Block = [];
  for (const row of Array.from(table.querySelectorAll("tr"))) {
    const cells: string[] = [];
    for (const cell of Array.from(row.querySelectorAll("th, td"))) {
      // `<br>` is a line break in a cell, not a space between two words.
      for (const br of Array.from(cell.querySelectorAll("br"))) {
        br.replaceWith("\n");
      }
      cells.push((cell.textContent ?? "").replace(/ /g, " ").trim());
      const span = Number((cell as HTMLTableCellElement).getAttribute("colspan") ?? "1");
      for (let extra = 1; extra < span; extra++) cells.push("");
    }
    if (cells.length > 0) block.push(cells);
  }
  return block.length > 0 ? block : null;
}

</storycode:copy>
<a id="f-fc3e01f3b793-parseClipboardTable"></a>
### `parseClipboardTable`

Whatever the clipboard is holding, as a rectangle — or null if it holds
nothing this grid can use.
A single cell of plain text is a perfectly good 1 × 1 rectangle: pasting a
word into a cell is the commonest paste there is, and it must not need a
table on the clipboard to work.

<storycode:copy id="f-fc3e01f3b793-parseClipboardTable">
/**
 * Whatever the clipboard is holding, as a rectangle — or null if it holds
 * nothing this grid can use.
 *
 * A single cell of plain text is a perfectly good 1 × 1 rectangle: pasting a
 * word into a cell is the commonest paste there is, and it must not need a
 * table on the clipboard to work.
 */
export function parseClipboardTable(html: string | null, text: string | null): Block | null {
  const fromHtml = html ? parseHtmlTable(html) : null;
  if (fromHtml) return fromHtml;
  if (!text) return null;
  // No trailing-newline row: a spreadsheet ends its last line, and reading
  // that as an extra empty row would wipe the cells under a paste.
  const trimmed = text.replace(/\r?\n$/, "");
  if (trimmed === "") return null;
  // Thousands separators in a monetary cell are not column delimiters.
  const lines = trimmed.split(/\r?\n/);
  if (lines.every(line => currencyInput(line))) return lines.map(line => [line]);
  const tabs = parseCsv(trimmed, "\t");
  const parsed = tabs.rows.some(row => row.length > 1) ? tabs : parseCsv(trimmed);
  return parsed.rows.length > 0 ? parsed.rows : null;
}
</storycode:copy>

### Assemble `apps/web/src/lib/tableClipboard.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tableClipboard.ts"><storycode:paste select="#f-fc3e01f3b793-Block"/><storycode:paste select="#f-fc3e01f3b793-toTabSeparated"/><storycode:paste select="#f-fc3e01f3b793-toHtmlTable"/><storycode:paste select="#f-fc3e01f3b793-parseHtmlTable"/><storycode:paste select="#f-fc3e01f3b793-parseClipboardTable"/></storycode:file>

<a id="output-e37176c0f701"></a>
## `apps/web/src/lib/tableMenu.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Table Is A Dataset And A Paragraph](contract.md).

<a id="f-e37176c0f701-at"></a>
### `at`

<storycode:copy id="f-e37176c0f701-at">
import { describe, expect, it } from "vitest";
import { columnsPhrase, rowsPhrase, tableMenuItems } from "./tableMenu";

const at = (rows: number[], columns: number[], height = 5, width = 4) => ({
  rows,
  columns,
  height,
  width,
});

</storycode:copy>
<a id="f-e37176c0f701-byId"></a>
### `byId`

<storycode:copy id="f-e37176c0f701-byId">
const byId = (items: ReturnType<typeof tableMenuItems>, id: string) =>
  items.find((item) => item.id === id)!;

</storycode:copy>
<a id="f-e37176c0f701-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e37176c0f701-expression_statement">
describe("what a right-click offers", () => {
  it("names the rows and columns the click actually landed on", () => {
    const items = tableMenuItems(at([2], [1]));
    expect(byId(items, "delete-rows").label).toBe("Delete row");
    expect(byId(items, "delete-rows").tip).toBe("Remove row 3");
    expect(byId(items, "delete-columns").tip).toBe("Remove column B");
  });

  it("counts a sweep, in the numbering a person reads off the grid", () => {
    const items = tableMenuItems(at([1, 2, 3], [0, 1]));
    expect(byId(items, "delete-rows").label).toBe("Delete 3 rows");
    expect(byId(items, "delete-rows").tip).toBe("Remove rows 2–4");
    expect(byId(items, "delete-columns").label).toBe("Delete 2 columns");
    expect(byId(items, "delete-columns").tip).toBe("Remove columns A–B");
  });

  it("inserts as many as are selected, which is what a spreadsheet does", () => {
    const items = tableMenuItems(at([1, 2], [0]));
    expect(byId(items, "insert-rows-above").label).toBe("Insert 2 rows above");
    expect(byId(items, "insert-columns-left").label).toBe("Insert a column left");
  });

  it("offers and disables the delete that would empty the table", () => {
    const items = tableMenuItems(at([0, 1, 2], [0, 1], 3, 2));
    expect(byId(items, "delete-rows").disabled).toBe(true);
    expect(byId(items, "delete-rows").tip).toContain("every row");
    expect(byId(items, "delete-columns").disabled).toBe(true);
    expect(byId(items, "delete-columns").tip).toContain("every column");
  });

  it("marks the destructive half so the hand slows down", () => {
    const items = tableMenuItems(at([0], [0]));
    expect(items.filter((i) => i.danger).map((i) => i.id)).toEqual([
      "delete-rows",
      "delete-columns",
    ]);
  });
});

</storycode:copy>
<a id="f-e37176c0f701-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-e37176c0f701-expression_statement-2">
describe("phrases", () => {
  it("reads the way a person would say it", () => {
    expect(rowsPhrase([0])).toBe("row 1");
    expect(rowsPhrase([0, 1, 2])).toBe("rows 1–3");
    expect(rowsPhrase([])).toBe("no rows");
    expect(columnsPhrase([25, 26])).toBe("columns Z–AA");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/tableMenu.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tableMenu.test.ts"><storycode:paste select="#f-e37176c0f701-at"/><storycode:paste select="#f-e37176c0f701-byId"/><storycode:paste select="#f-e37176c0f701-expression_statement"/><storycode:paste select="#f-e37176c0f701-expression_statement-2"/></storycode:file>

<a id="output-fb752e4d823e"></a>
## `apps/web/src/lib/tableMenu.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Table Is A Dataset And A Paragraph](contract.md).

<a id="f-fb752e4d823e-TableMenuAction"></a>
### `TableMenuAction`

What a right-click on a table cell offers, and what each item does.
The toolbar under the grid can add a row at the bottom and remove the
selected ones. That is the right set of buttons and the wrong set of
gestures: a row belongs in the MIDDLE of a table at least as often as at
the end, and the hand that wants to remove row 12 is already on row 12. So
the same operations are also where a spreadsheet has always put them —
under the pointer, on the thing being acted on.
Pure: the items are computed from the selection and the table's size, and
the caller applies the edit. That is what lets the labels be tested (they
name the actual rows, which is the part that goes wrong) without a grid.

<storycode:copy id="f-fb752e4d823e-TableMenuAction">
// What a right-click on a table cell offers, and what each item does.
//
// The toolbar under the grid can add a row at the bottom and remove the
// selected ones. That is the right set of buttons and the wrong set of
// gestures: a row belongs in the MIDDLE of a table at least as often as at
// the end, and the hand that wants to remove row 12 is already on row 12. So
// the same operations are also where a spreadsheet has always put them —
// under the pointer, on the thing being acted on.
//
// Pure: the items are computed from the selection and the table's size, and
// the caller applies the edit. That is what lets the labels be tested (they
// name the actual rows, which is the part that goes wrong) without a grid.

import { columnLabel } from "./cellRef";
import type { ContextMenuItem } from "../components/ContextMenu";

/** Every action the menu can ask for. */
export type TableMenuAction =
  | "insert-rows-above"
  | "insert-rows-below"
  | "insert-columns-left"
  | "insert-columns-right"
  | "delete-rows"
  | "delete-columns";

</storycode:copy>
<a id="f-fb752e4d823e-TableMenuContext"></a>
### `TableMenuContext`

What the menu needs to know about where the click landed.

<storycode:copy id="f-fb752e4d823e-TableMenuContext">
/** What the menu needs to know about where the click landed. */
export interface TableMenuContext {
  /** Rows the selection covers, ascending. Empty means nothing is selected,
   * which the caller avoids by selecting the clicked cell first. */
  rows: readonly number[];
  /** Columns the selection covers, ascending. */
  columns: readonly number[];
  /** The table's size, which is what makes "delete every row" refusable. */
  height: number;
  width: number;
}

</storycode:copy>
<a id="f-fb752e4d823e-rowsPhrase"></a>
### `rowsPhrase`

"row 3" / "rows 3–5", in the numbering a person reads off the grid.

<storycode:copy id="f-fb752e4d823e-rowsPhrase">
/** "row 3" / "rows 3–5", in the numbering a person reads off the grid. */
export function rowsPhrase(rows: readonly number[]): string {
  if (rows.length === 0) return "no rows";
  if (rows.length === 1) return `row ${rows[0] + 1}`;
  return `rows ${rows[0] + 1}–${rows[rows.length - 1] + 1}`;
}

</storycode:copy>
<a id="f-fb752e4d823e-columnsPhrase"></a>
### `columnsPhrase`

<storycode:copy id="f-fb752e4d823e-columnsPhrase">
/** "column B" / "columns B–D". */
export function columnsPhrase(columns: readonly number[]): string {
  if (columns.length === 0) return "no columns";
  if (columns.length === 1) return `column ${columnLabel(columns[0])}`;
  return `columns ${columnLabel(columns[0])}–${columnLabel(columns[columns.length - 1])}`;
}

</storycode:copy>
<a id="f-fb752e4d823e-count"></a>
### `count`

"a row" / "3 rows" — the count an insert is about to add.

<storycode:copy id="f-fb752e4d823e-count">
/** "a row" / "3 rows" — the count an insert is about to add. */
function count(n: number, singular: string): string {
  return n === 1 ? `a ${singular}` : `${n} ${singular}s`;
}

</storycode:copy>
<a id="f-fb752e4d823e-tableMenuItems"></a>
### `tableMenuItems`

The menu for this selection.
Removing every row (or every column) is offered and DISABLED rather than
hidden, with the tooltip saying why: an item that disappears when you
select the whole table teaches nobody that the whole table was the problem.

<storycode:copy id="f-fb752e4d823e-tableMenuItems">
/**
 * The menu for this selection.
 *
 * Removing every row (or every column) is offered and DISABLED rather than
 * hidden, with the tooltip saying why: an item that disappears when you
 * select the whole table teaches nobody that the whole table was the problem.
 */
export function tableMenuItems(at: TableMenuContext): ContextMenuItem[] {
  const rows = at.rows.length;
  const columns = at.columns.length;
  const allRows = rows >= at.height;
  const allColumns = columns >= at.width;
  return [
    {
      id: "insert-rows-above",
      label: `Insert ${count(rows, "row")} above`,
      tip: `Add ${count(rows, "row")} above ${rowsPhrase(at.rows)}`,
    },
    {
      id: "insert-rows-below",
      label: `Insert ${count(rows, "row")} below`,
      tip: `Add ${count(rows, "row")} below ${rowsPhrase(at.rows)}`,
    },
    {
      id: "insert-columns-left",
      label: `Insert ${count(columns, "column")} left`,
      group: true,
      tip: `Add ${count(columns, "column")} to the left of ${columnsPhrase(at.columns)}`,
    },
    {
      id: "insert-columns-right",
      label: `Insert ${count(columns, "column")} right`,
      tip: `Add ${count(columns, "column")} to the right of ${columnsPhrase(at.columns)}`,
    },
    {
      id: "delete-rows",
      label: rows > 1 ? `Delete ${rows} rows` : "Delete row",
      group: true,
      danger: true,
      disabled: rows === 0 || allRows,
      tip: allRows
        ? "That is every row — a table with none left is not a table"
        : `Remove ${rowsPhrase(at.rows)}`,
    },
    {
      id: "delete-columns",
      label: columns > 1 ? `Delete ${columns} columns` : "Delete column",
      danger: true,
      disabled: columns === 0 || allColumns,
      tip: allColumns
        ? "That is every column — a table with none left is not a table"
        : `Remove ${columnsPhrase(at.columns)}`,
    },
  ];
}
</storycode:copy>

### Assemble `apps/web/src/lib/tableMenu.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tableMenu.ts"><storycode:paste select="#f-fb752e4d823e-TableMenuAction"/><storycode:paste select="#f-fb752e4d823e-TableMenuContext"/><storycode:paste select="#f-fb752e4d823e-rowsPhrase"/><storycode:paste select="#f-fb752e4d823e-columnsPhrase"/><storycode:paste select="#f-fb752e4d823e-count"/><storycode:paste select="#f-fb752e4d823e-tableMenuItems"/></storycode:file>

<a id="output-16ff13760366"></a>
## `apps/web/src/lib/tableSelection.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Table Is A Dataset And A Paragraph](contract.md).

<a id="f-16ff13760366-expression_statement"></a>
### `expression_statement`

Protects docs/guarantees/authoring/a-table-is-a-dataset-and-a-paragraph.md
— "selection is a rectangle, not a cursor".

<storycode:copy id="f-16ff13760366-expression_statement">
// Protects docs/guarantees/authoring/a-table-is-a-dataset-and-a-paragraph.md
// — "selection is a rectangle, not a cursor".

import { describe, expect, it } from "vitest";

import {
  allSelection,
  boundsOf,
  cellSelection,
  cellsIn,
  columnSelection,
  columnsIn,
  containsCell,
  extendTo,
  isSingleCell,
  rowSelection,
  rowsIn,
  selectionLabel,
  selectionSize,
} from "./tableSelection";

describe("a selection is an anchor and a reach, not two sorted corners", () => {
  it("keeps the anchor still when the reach goes backwards", () => {
    // The anchor is the ACTIVE cell — what the formula bar edits and what a
    // keystroke replaces. Normalising early would throw it away.
    const dragged = extendTo(cellSelection(3, 3), 1, 1, 8, 8);
    expect(dragged.anchor).toEqual({ row: 3, column: 3 });
    expect(boundsOf(dragged)).toEqual({ top: 1, left: 1, bottom: 3, right: 3 });
  });

  it("shrinks rather than flips when a drag comes back over its start", () => {
    const out = extendTo(cellSelection(0, 0), 4, 4, 8, 8);
    const back = extendTo(out, 1, 1, 8, 8);
    expect(selectionSize(back)).toEqual({ rows: 2, columns: 2 });
  });

  it("will not reach past the grid", () => {
    const out = extendTo(cellSelection(0, 0), 99, 99, 3, 2);
    expect(boundsOf(out)).toEqual({ top: 0, left: 0, bottom: 2, right: 1 });
  });
});

</storycode:copy>
<a id="f-16ff13760366-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-16ff13760366-expression_statement-2">
describe("whole rows and whole columns", () => {
  it("selects a column down its whole height", () => {
    const out = columnSelection(1, 4);
    expect(out.kind).toBe("columns");
    expect(boundsOf(out)).toEqual({ top: 0, left: 1, bottom: 3, right: 1 });
  });

  it("selects a row across its whole width", () => {
    const out = rowSelection(2, 3);
    expect(out.kind).toBe("rows");
    expect(boundsOf(out)).toEqual({ top: 2, left: 0, bottom: 2, right: 2 });
  });

  it("drags across the letters and never selects half a column", () => {
    // Dragging B → D is three whole columns. A partial one would be a
    // rectangle nobody asked for.
    const out = extendTo(columnSelection(1, 4), 2, 3, 4, 5);
    expect(boundsOf(out)).toEqual({ top: 0, left: 1, bottom: 3, right: 3 });
    expect(columnsIn(out)).toEqual([1, 2, 3]);
  });

  it("drags down the numbers and never selects half a row", () => {
    const out = extendTo(rowSelection(0, 3), 2, 1, 4, 3);
    expect(boundsOf(out)).toEqual({ top: 0, left: 0, bottom: 2, right: 2 });
    expect(rowsIn(out)).toEqual([0, 1, 2]);
  });

  it("selects everything from the corner", () => {
    expect(boundsOf(allSelection(3, 2))).toEqual({ top: 0, left: 0, bottom: 2, right: 1 });
  });
});

</storycode:copy>
<a id="f-16ff13760366-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-16ff13760366-expression_statement-3">
describe("what is in it", () => {
  it("knows a single cell from a range", () => {
    expect(isSingleCell(cellSelection(1, 1))).toBe(true);
    expect(isSingleCell(extendTo(cellSelection(1, 1), 1, 2, 4, 4))).toBe(false);
  });

  it("contains every cell in the rectangle and nothing outside it", () => {
    const out = extendTo(cellSelection(1, 1), 2, 2, 5, 5);
    expect(containsCell(out, 2, 1)).toBe(true);
    expect(containsCell(out, 0, 1)).toBe(false);
    expect(containsCell(out, 2, 3)).toBe(false);
  });

  it("lists its cells in reading order", () => {
    expect(cellsIn(extendTo(cellSelection(0, 0), 1, 1, 4, 4))).toEqual([
      { row: 0, column: 0 },
      { row: 0, column: 1 },
      { row: 1, column: 0 },
      { row: 1, column: 1 },
    ]);
  });
});

</storycode:copy>
<a id="f-16ff13760366-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-16ff13760366-expression_statement-4">
describe("what the name box says", () => {
  it("names one cell in A1 notation", () => {
    expect(selectionLabel(cellSelection(2, 1))).toBe("B3");
  });

  it("names a range by its corners", () => {
    expect(selectionLabel(extendTo(cellSelection(0, 0), 2, 1, 4, 4))).toBe("A1:B3");
  });

  it("names a whole column without claiming how tall it is", () => {
    // `B1:B7` would stop being true the moment somebody adds a row.
    expect(selectionLabel(columnSelection(1, 7))).toBe("B:B");
    expect(selectionLabel(extendTo(columnSelection(1, 7), 0, 3, 7, 5))).toBe("B:D");
  });

  it("names whole rows by their numbers", () => {
    expect(selectionLabel(extendTo(rowSelection(0, 3), 2, 0, 4, 3))).toBe("1:3");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/tableSelection.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tableSelection.test.ts"><storycode:paste select="#f-16ff13760366-expression_statement"/><storycode:paste select="#f-16ff13760366-expression_statement-2"/><storycode:paste select="#f-16ff13760366-expression_statement-3"/><storycode:paste select="#f-16ff13760366-expression_statement-4"/></storycode:file>

<a id="output-9d925d85cf99"></a>
## `apps/web/src/lib/tableSelection.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Table Is A Dataset And A Paragraph](contract.md).

<a id="f-9d925d85cf99-Cell"></a>
### `Cell`

What is selected in a table, as a range rather than a cursor.
A spreadsheet's selection is a rectangle with a corner you started from,
and that corner matters: it is the cell the formula bar edits, the cell a
keystroke replaces, and the cell an extend keeps still. So this is an
ANCHOR and a FOCUS, not a pair of sorted corners — normalising early would
throw away the only part of a drag that says which direction it went.
The `kind` is here for the same reason. A column selection and a
cell selection that happens to cover a whole column look identical as
rectangles and are not the same thing: one says "column B", and asking to
remove a row from it should be refused rather than quietly emptying the
table. Keeping what the person actually clicked is cheaper than inferring
it back from the shape.
Everything here is pure and takes the grid's measurements as arguments —
the panel owns the state, this owns the arithmetic.

<storycode:copy id="f-9d925d85cf99-Cell">
// What is selected in a table, as a range rather than a cursor.
//
// A spreadsheet's selection is a rectangle with a corner you started from,
// and that corner matters: it is the cell the formula bar edits, the cell a
// keystroke replaces, and the cell an extend keeps still. So this is an
// ANCHOR and a FOCUS, not a pair of sorted corners — normalising early would
// throw away the only part of a drag that says which direction it went.
//
// The `kind` is here for the same reason. A column selection and a
// cell selection that happens to cover a whole column look identical as
// rectangles and are not the same thing: one says "column B", and asking to
// remove a row from it should be refused rather than quietly emptying the
// table. Keeping what the person actually clicked is cheaper than inferring
// it back from the shape.
//
// Everything here is pure and takes the grid's measurements as arguments —
// the panel owns the state, this owns the arithmetic.

import { cellLabel, columnLabel } from "./cellRef";

export interface Cell {
  row: number;
  column: number;
}

</storycode:copy>
<a id="f-9d925d85cf99-SelectionKind"></a>
### `SelectionKind`

What was clicked to make this selection: cells, whole rows, or whole
columns. Not a rendering detail — see the note above.

<storycode:copy id="f-9d925d85cf99-SelectionKind">
/** What was clicked to make this selection: cells, whole rows, or whole
 * columns. Not a rendering detail — see the note above. */
export type SelectionKind = "cells" | "rows" | "columns";

</storycode:copy>
<a id="f-9d925d85cf99-Selection"></a>
### `Selection`

<storycode:copy id="f-9d925d85cf99-Selection">
export interface Selection {
  /** Where the selection started. The ACTIVE cell: what the formula bar
   * edits and what a keystroke replaces. */
  anchor: Cell;
  /** Where it currently reaches. Equal to the anchor for a single cell. */
  focus: Cell;
  kind: SelectionKind;
}

</storycode:copy>
<a id="f-9d925d85cf99-Bounds"></a>
### `Bounds`

A normalised rectangle, inclusive on every side.

<storycode:copy id="f-9d925d85cf99-Bounds">
/** A normalised rectangle, inclusive on every side. */
export interface Bounds {
  top: number;
  left: number;
  bottom: number;
  right: number;
}

</storycode:copy>
<a id="f-9d925d85cf99-clamp"></a>
### `clamp`

<storycode:copy id="f-9d925d85cf99-clamp">
const clamp = (value: number, limit: number) => Math.max(0, Math.min(value, limit - 1));

</storycode:copy>
<a id="f-9d925d85cf99-cellSelection"></a>
### `cellSelection`

<storycode:copy id="f-9d925d85cf99-cellSelection">
/** One cell, selected on its own. */
export function cellSelection(row: number, column: number): Selection {
  const at = { row, column };
  return { anchor: at, focus: at, kind: "cells" };
}

</storycode:copy>
<a id="f-9d925d85cf99-columnSelection"></a>
### `columnSelection`

<storycode:copy id="f-9d925d85cf99-columnSelection">
/** A whole column, from its letter. */
export function columnSelection(column: number, height: number): Selection {
  return {
    anchor: { row: 0, column },
    focus: { row: Math.max(0, height - 1), column },
    kind: "columns",
  };
}

</storycode:copy>
<a id="f-9d925d85cf99-rowSelection"></a>
### `rowSelection`

<storycode:copy id="f-9d925d85cf99-rowSelection">
/** A whole row, from its number. */
export function rowSelection(row: number, width: number): Selection {
  return {
    anchor: { row, column: 0 },
    focus: { row, column: Math.max(0, width - 1) },
    kind: "rows",
  };
}

</storycode:copy>
<a id="f-9d925d85cf99-allSelection"></a>
### `allSelection`

<storycode:copy id="f-9d925d85cf99-allSelection">
/** Everything — the corner box, and Ctrl+A. */
export function allSelection(height: number, width: number): Selection {
  return {
    anchor: { row: 0, column: 0 },
    focus: { row: Math.max(0, height - 1), column: Math.max(0, width - 1) },
    kind: "cells",
  };
}

</storycode:copy>
<a id="f-9d925d85cf99-extendTo"></a>
### `extendTo`

The same selection reaching to another cell — a drag, or a shift-click.
The anchor never moves, which is what makes dragging back over the start
shrink the range instead of dropping it. A row or column selection extends
along its own axis and stays full along the other: dragging across the
letters B, C, D selects three whole columns, never a partial one.

<storycode:copy id="f-9d925d85cf99-extendTo">
/**
 * The same selection reaching to another cell — a drag, or a shift-click.
 *
 * The anchor never moves, which is what makes dragging back over the start
 * shrink the range instead of dropping it. A row or column selection extends
 * along its own axis and stays full along the other: dragging across the
 * letters B, C, D selects three whole columns, never a partial one.
 */
export function extendTo(
  selection: Selection,
  row: number,
  column: number,
  height: number,
  width: number,
): Selection {
  const at = { row: clamp(row, height), column: clamp(column, width) };
  if (selection.kind === "columns") {
    return {
      ...selection,
      anchor: { row: 0, column: selection.anchor.column },
      focus: { row: Math.max(0, height - 1), column: at.column },
    };
  }
  if (selection.kind === "rows") {
    return {
      ...selection,
      anchor: { row: selection.anchor.row, column: 0 },
      focus: { row: at.row, column: Math.max(0, width - 1) },
    };
  }
  return { ...selection, focus: at };
}

</storycode:copy>
<a id="f-9d925d85cf99-boundsOf"></a>
### `boundsOf`

The rectangle, with its corners in reading order.

<storycode:copy id="f-9d925d85cf99-boundsOf">
/** The rectangle, with its corners in reading order. */
export function boundsOf(selection: Selection): Bounds {
  return {
    top: Math.min(selection.anchor.row, selection.focus.row),
    bottom: Math.max(selection.anchor.row, selection.focus.row),
    left: Math.min(selection.anchor.column, selection.focus.column),
    right: Math.max(selection.anchor.column, selection.focus.column),
  };
}

</storycode:copy>
<a id="f-9d925d85cf99-containsCell"></a>
### `containsCell`

<storycode:copy id="f-9d925d85cf99-containsCell">
export function containsCell(selection: Selection, row: number, column: number): boolean {
  const at = boundsOf(selection);
  return row >= at.top && row <= at.bottom && column >= at.left && column <= at.right;
}

</storycode:copy>
<a id="f-9d925d85cf99-isSingleCell"></a>
### `isSingleCell`

Whether this is one cell — the state a spreadsheet spends most of its time
in, and the one where a range highlight would be noise.

<storycode:copy id="f-9d925d85cf99-isSingleCell">
/** Whether this is one cell — the state a spreadsheet spends most of its time
 * in, and the one where a range highlight would be noise. */
export function isSingleCell(selection: Selection): boolean {
  return (
    selection.anchor.row === selection.focus.row &&
    selection.anchor.column === selection.focus.column
  );
}

</storycode:copy>
<a id="f-9d925d85cf99-rowsIn"></a>
### `rowsIn`

<storycode:copy id="f-9d925d85cf99-rowsIn">
export function rowsIn(selection: Selection): number[] {
  const at = boundsOf(selection);
  return Array.from({ length: at.bottom - at.top + 1 }, (_, i) => at.top + i);
}

</storycode:copy>
<a id="f-9d925d85cf99-columnsIn"></a>
### `columnsIn`

<storycode:copy id="f-9d925d85cf99-columnsIn">
export function columnsIn(selection: Selection): number[] {
  const at = boundsOf(selection);
  return Array.from({ length: at.right - at.left + 1 }, (_, i) => at.left + i);
}

</storycode:copy>
<a id="f-9d925d85cf99-cellsIn"></a>
### `cellsIn`

Every cell in the rectangle, in reading order.

<storycode:copy id="f-9d925d85cf99-cellsIn">
/** Every cell in the rectangle, in reading order. */
export function cellsIn(selection: Selection): Cell[] {
  const cells: Cell[] = [];
  for (const row of rowsIn(selection)) {
    for (const column of columnsIn(selection)) cells.push({ row, column });
  }
  return cells;
}

</storycode:copy>
<a id="f-9d925d85cf99-selectionSize"></a>
### `selectionSize`

How big it is, for the "3R × 2C" a spreadsheet shows while you drag.

<storycode:copy id="f-9d925d85cf99-selectionSize">
/** How big it is, for the "3R × 2C" a spreadsheet shows while you drag. */
export function selectionSize(selection: Selection): { rows: number; columns: number } {
  const at = boundsOf(selection);
  return { rows: at.bottom - at.top + 1, columns: at.right - at.left + 1 };
}

</storycode:copy>
<a id="f-9d925d85cf99-selectionLabel"></a>
### `selectionLabel`

What the name box says.
A1 range notation, because it is what a spreadsheet's name box says and
what the person will type back. A whole column is `B:B` rather than
`B1:B7` — the bounded spelling would claim the selection is seven rows
tall, which stops being true the moment a row is added.

<storycode:copy id="f-9d925d85cf99-selectionLabel">
/**
 * What the name box says.
 *
 * A1 range notation, because it is what a spreadsheet's name box says and
 * what the person will type back. A whole column is `B:B` rather than
 * `B1:B7` — the bounded spelling would claim the selection is seven rows
 * tall, which stops being true the moment a row is added.
 */
export function selectionLabel(selection: Selection): string {
  const at = boundsOf(selection);
  if (isSingleCell(selection)) return cellLabel(at.left, at.top);
  if (selection.kind === "columns") {
    return `${columnLabel(at.left)}:${columnLabel(at.right)}`;
  }
  if (selection.kind === "rows") return `${at.top + 1}:${at.bottom + 1}`;
  return `${cellLabel(at.left, at.top)}:${cellLabel(at.right, at.bottom)}`;
}
</storycode:copy>

### Assemble `apps/web/src/lib/tableSelection.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tableSelection.ts"><storycode:paste select="#f-9d925d85cf99-Cell"/><storycode:paste select="#f-9d925d85cf99-SelectionKind"/><storycode:paste select="#f-9d925d85cf99-Selection"/><storycode:paste select="#f-9d925d85cf99-Bounds"/><storycode:paste select="#f-9d925d85cf99-clamp"/><storycode:paste select="#f-9d925d85cf99-cellSelection"/><storycode:paste select="#f-9d925d85cf99-columnSelection"/><storycode:paste select="#f-9d925d85cf99-rowSelection"/><storycode:paste select="#f-9d925d85cf99-allSelection"/><storycode:paste select="#f-9d925d85cf99-extendTo"/><storycode:paste select="#f-9d925d85cf99-boundsOf"/><storycode:paste select="#f-9d925d85cf99-containsCell"/><storycode:paste select="#f-9d925d85cf99-isSingleCell"/><storycode:paste select="#f-9d925d85cf99-rowsIn"/><storycode:paste select="#f-9d925d85cf99-columnsIn"/><storycode:paste select="#f-9d925d85cf99-cellsIn"/><storycode:paste select="#f-9d925d85cf99-selectionSize"/><storycode:paste select="#f-9d925d85cf99-selectionLabel"/></storycode:file>

## Test the behavior: `hick`

This is the maintained test fragment from `crates/hickory-cli/tests/tables.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e0de523c23a6-hick"></a>
### `hick`

Tables: a dataset that is also prose.
Protects docs/guarantees/authoring/a-table-is-a-dataset-and-a-paragraph.md
The two halves are the whole point. The CONTENT is CSV, so a script, a
spreadsheet, and a query can all read it; the WEAVE is a markdown table,
so a reader opening the document gets the data rather than the delimiters.

<storycode:copy id="f-e0de523c23a6-hick">
//! Tables: a dataset that is also prose.
//!
//! Protects docs/guarantees/authoring/a-table-is-a-dataset-and-a-paragraph.md
//!
//! The two halves are the whole point. The CONTENT is CSV, so a script, a
//! spreadsheet, and a query can all read it; the WEAVE is a markdown table,
//! so a reader opening the document gets the data rather than the delimiters.

use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/ContextMenu.tsx](contract.md#output-f351518007ac)
- [apps/web/src/components/FenceTable.tsx](contract.md#output-3c939bfc5a94)
- [apps/web/src/components/TablePanel.tsx](../../../mechanisms/apps/web/src/components/TablePanel.tsx.md#output-38d5395d7736)
- [apps/web/src/components/TableSizeDialog.tsx](contract.md#output-7ae5e1d2e203)
- [apps/web/src/editor/CardRail.tsx](../../../mechanisms/apps/web/src/editor/CardRail.tsx.md#output-f6019c4774f8)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/cards.ts](../../../mechanisms/apps/web/src/editor/cards.ts.md#output-84385185ee44)
- [apps/web/src/editor/rendered.ts](../../../mechanisms/apps/web/src/editor/rendered.ts.md#output-4f5444d21411)
- [apps/web/src/lib/csv.ts](contract.md#output-4460b44bfc5a)
- [apps/web/src/lib/insertCatalog.ts](../../../mechanisms/apps/web/src/lib/insertCatalog.ts.md#output-8620a55486dc)
- [apps/web/src/lib/railActions.ts](../../../mechanisms/apps/web/src/lib/railActions.ts.md#output-499155452cf6)
- [apps/web/src/lib/tableClipboard.ts](contract.md#output-fc3e01f3b793)
- [apps/web/src/lib/tableMenu.ts](contract.md#output-fb752e4d823e)
- [apps/web/src/lib/tableSelection.ts](contract.md#output-9d925d85cf99)
- [apps/web/src/lib/uiState.ts](../../../mechanisms/apps/web/src/lib/uiState.ts.md#output-7183338fb20a)
- [apps/web/src/shell/TreeContextMenu.tsx](../a-tree-row-opens-in-the-platform/contract.md#output-779a633f282e)
- [apps/web/src/styles.css](../../../mechanisms/apps/web/src/styles.css.md#output-8695905e2358)
- [crates/hick-literate/src/csv_table.rs](../../../mechanisms/crates/hick-literate/src/csv_table.rs.md#output-99033ffa7355)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hick-literate/src/render.rs](../../../mechanisms/crates/hick-literate/src/render.rs.md#output-4a7f39f68eb8)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)

### Tests and fixtures

- [apps/web/src/components/FenceTable.test.tsx](../../../mechanisms/apps/web/src/components/FenceTable.test.tsx.md#output-effdc999f8dc)
- [apps/web/src/components/TablePanel.test.tsx](../../../mechanisms/apps/web/src/components/TablePanel.test.tsx.md#output-694220ace684)
- [apps/web/src/editor/rendered.test.ts](../../../mechanisms/apps/web/src/editor/rendered.ts.md#output-1e1e27143ad4)
- [apps/web/src/lib/csv.test.ts](contract.md#output-43450f4cc8c1)
- [apps/web/src/lib/tableClipboard.test.ts](../../../mechanisms/apps/web/src/lib/tableClipboard.test.ts.md#output-65703cf530ea)
- [apps/web/src/lib/tableMenu.test.ts](contract.md#output-e37176c0f701)
- [apps/web/src/lib/tableSelection.test.ts](contract.md#output-16ff13760366)
- [apps/web/src/lib/uiState.test.ts](../../../mechanisms/apps/web/src/lib/uiState.ts.md#output-43efcbce9427)
- [apps/web/src/styles.test.ts](../../../mechanisms/apps/web/src/styles.test.ts.md#output-9d71c5372ba0)
- [crates/hickory-cli/tests/tables.rs](../../../mechanisms/crates/hickory-cli/tests/tables.rs.md#output-e0de523c23a6)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
