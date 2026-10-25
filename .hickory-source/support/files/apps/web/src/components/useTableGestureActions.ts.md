<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/useTableGestureActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `8e26ca16ea557c2f03fc507d598946d21e7b60c1c68975ff874298ddc8962ab6`.

<book:entry path="apps/web/src/components/useTableGestureActions.ts">
import { useUiCommands } from "../lib/uiActionBridge";
import type { UiActor, UiCommand, UiOutcome } from "../lib/uiActions";
export type TableClipboard = Pick<DataTransfer, "getData" | "setData">;
/** Native events and typed payloads use the same parser/pointing/layout handlers. */
export function useTableGestureActions(options: {
  parent: string; revision: () => string; current: () => boolean; editable: boolean; selected: boolean; editing: boolean; pointing: boolean;
  rows: number; columns: number; input: () => HTMLInputElement | null; viewport: () => HTMLElement | null;
  begin: (row: number, column: number, from: "grid" | "bar") => void; change: (value: string) => void;
  point: (row: number, column: number, endRow: number, endColumn: number) => boolean;
  copy: (data: TableClipboard) => boolean; paste: (data: TableClipboard, actor: UiActor) => boolean;
  resize: (height: number) => void;
}) {
  const { parent } = options;
  const commands: { key: string; command: UiCommand }[] = [];
  const integer = { type: "number" as const, integer: true, minimum: 0, maximum: 1000000 };
  const snapshot = () => { const input = options.input(), viewport = options.viewport(); return JSON.stringify([options.revision(), input?.selectionStart, input?.selectionEnd, viewport?.scrollTop, viewport?.scrollLeft, viewport?.clientHeight, viewport?.clientWidth, options.current()]); };
  const add = (id: string, label: string, parameters: UiCommand["parameters"], enabled: () => boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:gestures:${id}`, command: {
    parent, subject: parent, label, parameters, prerequisites: ["Current table source, selection, formula draft and mounted viewport; clipboard permissions remain browser-owned"],
    read: () => ({ revision: snapshot(), enabled: options.current() && enabled(), reason: "Select a current cell or formula/viewport; finish or cancel conflicting edits" }), execute,
  } });
  add("begin", "Begin table cell edit", { row: integer, column: integer, input: { type: "string", maxLength: 8, choices: ["grid", "bar"] } }, () => options.editable && !options.editing, async args => {
    if ((args.row as number) >= options.rows || (args.column as number) >= options.columns) return { status: "unavailable", reason: "Cell is outside the current table" };
    options.begin(args.row as number, args.column as number, args.input as "grid" | "bar"); return { status: "completed", value: { editing: true, source_written: false } };
  });
  add("draft", "Change table cell edit draft", { value: { type: "string", maxLength: 1048576 } }, () => options.editable && options.editing, async args => { options.change(String(args.value)); return { status: "completed", value: { draft_changed: true, committed: false } }; });
  add("caret", "Select table formula draft range", { anchor: integer, head: integer }, () => options.editable && options.editing && !!options.input(), async args => {
    const input = options.input(), anchor = args.anchor as number, head = args.head as number;
    if (!input || anchor > input.value.length || head > input.value.length) return { status: "unavailable", reason: "Range is outside the current table draft" };
    input.focus(); input.setSelectionRange(Math.min(anchor, head), Math.max(anchor, head), anchor <= head ? "forward" : "backward");
    return { status: "completed", value: { anchor, head, source_written: false } };
  });
  add("point", "Point table formula to cell range", { row: integer, column: integer, end_row: integer, end_column: integer }, () => options.editable && options.pointing, async args => {
    const row = args.row as number, column = args.column as number, endRow = args.end_row as number, endColumn = args.end_column as number;
    if ([row, endRow].some(value => value >= options.rows) || [column, endColumn].some(value => value >= options.columns)) return { status: "unavailable", reason: "Formula reference is outside the current table" };
    if (!options.point(row, column, endRow, endColumn)) return { status: "unavailable", reason: "The current formula caret cannot accept that reference" };
    return { status: "completed", value: { draft_changed: true, committed: false } };
  });
  add("height", "Resize table viewport height", { pixels: { type: "number", minimum: 64, maximum: 10000 } }, () => true, async args => { options.resize(args.pixels as number); return { status: "completed", value: { layout_change_requested: true, source_written: false } }; });
  add("viewport", "Read table viewport", {}, () => !!options.viewport(), async () => {
    const view = options.viewport(); if (!view) return { status: "unavailable", reason: "The table viewport was closed" };
    return { status: "completed", value: { top: view.scrollTop, left: view.scrollLeft, height: view.clientHeight, width: view.clientWidth, total_height: view.scrollHeight, total_width: view.scrollWidth, scope: "Geometry only; cell contents require explicit region reading" } };
  });
  add("scroll", "Scroll table viewport", { top: integer, left: integer }, () => !!options.viewport(), async args => {
    const view = options.viewport(); if (!view) return { status: "unavailable", reason: "The table viewport was closed" };
    view.scrollTop = args.top as number; view.scrollLeft = args.left as number;
    return { status: "completed", value: { top: view.scrollTop, left: view.scrollLeft, source_written: false } };
  });
  const clipboard = () => { const data = new Map<string, string>(); return { getData: (kind: string) => data.get(kind) ?? "", setData: (kind: string, value: string) => { data.set(kind, value); } }; };
  add("paste", "Paste table clipboard contents", { text: { type: "string", maxLength: 1048576 }, html: { type: "string", maxLength: 1048576 } }, () => options.editable && options.selected && !options.editing, async (args, actor) => {
    const data = clipboard(); data.setData("text/plain", String(args.text)); data.setData("text/html", String(args.html));
    return options.paste(data, actor) ? { status: "completed", value: { source_write_requested: true, saved: false, verification: "not requested" } } : { status: "unavailable", reason: "The current table refused this clipboard payload" };
  });
  add("copy", "Copy selected table cells", {}, () => options.selected && !options.editing, async (): Promise<UiOutcome> => {
    const data = clipboard(); if (!options.copy(data)) return { status: "unavailable", reason: "No current table selection to copy" };
    const captured = snapshot();
    try {
      if (!navigator.clipboard?.write || typeof ClipboardItem === "undefined") throw Error("Rich clipboard writing is unavailable");
      await navigator.clipboard.write([new ClipboardItem({ "text/plain": new Blob([data.getData("text/plain")], { type: "text/plain" }), "text/html": new Blob([data.getData("text/html")], { type: "text/html" }) })]);
      return { status: "completed", value: { copied: true, current_at_completion: snapshot() === captured, source_written: false } };
    } catch { return { status: "requires_input", request: `${parent}:copy-table-cells`, reason: "Use the native Copy gesture for this selection; the browser requires clipboard permission/activation. No table source was changed." }; }
  });
  add("cut", "Cut selected table cells", {}, () => options.editable && options.selected && !options.editing, async () => ({ status: "requires_input", request: `${parent}:cut-table-cells`, reason: "Use native Cut for the selected cells so the browser supplies its clipboard event and private move marker before source deletion. No cut or clipboard write was performed by this request." }));
  useUiCommands(commands);
}
</book:entry>

</book:doc>
