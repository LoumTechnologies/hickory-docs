<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A fill handle extends formulas

For: Authors editing executable documents and the files those documents produce.

An editable rendered table shows one small plus handle at the bottom-right of
its selected cell or rectangular selection, including a selection made in
reverse. While editing a cell the handle is hidden. Dragging the handle up,
down, left or right previews the extended range; releasing fills that range
as one document edit and selects the result. Escape or losing window focus
cancels without changing data. A source or format edit during the drag invalidates
its frozen seed, so release cannot overwrite that newer edit.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-fill-handle-extends-formulas.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-2e2dc30f7f54"></a>
## `apps/web/e2e/tableFill.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A fill handle extends formulas](contract.md).

<a id="f-2e2dc30f7f54-for_in_statement"></a>
### `for_in_statement`

<storycode:copy id="f-2e2dc30f7f54-for_in_statement">
// Guarantee: docs/guarantees/authoring/a-fill-handle-extends-formulas.md
import { expect, test } from '@playwright/test';
for (const [row,column,formula] of [[0,2,'=B1*2'],[4,2,'=B5*2'],[2,0,'=#REF!*2'],[2,4,'=D3*2']] as const) {
  test(`drag the fill handle to ${row},${column}`,async({page})=>{
    test.skip(!process.env.HICKORY_E2E_URL,'Run against the full editor.');
    await page.goto('/');
    const pane=page.locator('.untitled-tab');
    await pane.locator('.cm-content').click();
    await page.keyboard.press('ControlOrMeta+a');
    await page.keyboard.insertText('<hick:table language="javascript" header="false" formats="C3:USD">\n1,2,3,4,5\n6,7,8,9,10\n11,12,=B3*2,14,15\n16,17,18,19,20\n21,22,23,24,25\n</hick:table>');
    await pane.getByRole('button',{name:/Table 1.*click to render/}).click();
    const cell=(r:number,c:number)=>pane.locator(`.table-panel__cell[data-row="${r}"][data-column="${c}"]`);
    await cell(2,2).click();
    const handle=pane.getByRole('button',{name:'Fill selected cells'});
    const start=(await handle.boundingBox())!;
    const end=(await cell(row,column).boundingBox())!;
    await page.mouse.move(start.x+start.width/2,start.y+start.height/2);
    await page.mouse.down();
    await page.mouse.move(end.x+end.width/2,end.y+end.height/2,{steps:8});
    await expect(pane.locator('.table-fill-preview').first()).toBeVisible();
    await page.mouse.up();
    await cell(row,column).click();
    await expect(pane.locator('.table-panel__formula')).toHaveValue(formula);
    await expect(pane.getByLabel('Cell number format')).toHaveValue('USD');
  });
}
</storycode:copy>
<a id="f-2e2dc30f7f54-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-2e2dc30f7f54-expression_statement">
test('fills a selected formula block beyond the bottom edge',async({page})=>{
  test.skip(!process.env.HICKORY_E2E_URL,'Run against the full editor.');
  await page.goto('/');
  const pane=page.locator('.untitled-tab');
  await pane.locator('.cm-content').click();
  await page.keyboard.press('ControlOrMeta+a');
  await page.keyboard.insertText('<hick:table language="python" header="false">\n1,=A1*2\n2,=A2*3\n3,\n4,\n</hick:table>');
  await pane.getByRole('button',{name:/Table 1.*click to render/}).click();
  const cell=(row:number)=>pane.locator(`.table-panel__cell[data-row="${row}"][data-column="1"]`);
  await cell(1).click();
  await cell(0).click({modifiers:['Shift']});
  const handle=pane.getByRole('button',{name:'Fill selected cells'});
  await expect(handle).toHaveCount(1);
  const start=(await handle.boundingBox())!;
  const last=(await cell(3).boundingBox())!;
  await page.mouse.move(start.x+start.width/2,start.y+start.height/2);
  await page.mouse.down();
  await page.mouse.move(last.x+last.width/2,last.y+last.height*2.5,{steps:8});
  await expect(pane.locator('.table-fill-range')).toContainText('B1:B6');
  await page.mouse.up();
  await cell(4).click();
  await expect(pane.locator('.table-panel__formula')).toHaveValue('=A5*2');
  await cell(5).click();
  await expect(pane.locator('.table-panel__formula')).toHaveValue('=A6*3');
});
</storycode:copy>

### Assemble `apps/web/e2e/tableFill.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/tableFill.spec.ts"><storycode:paste select="#f-2e2dc30f7f54-for_in_statement"/><storycode:paste select="#f-2e2dc30f7f54-expression_statement"/></storycode:file>

<a id="output-ca0207209fcf"></a>
## `apps/web/src/components/TableFill.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A fill handle extends formulas](contract.md).

<a id="f-ca0207209fcf-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ca0207209fcf-expression_statement">
// Guarantee: docs/guarantees/authoring/a-fill-handle-extends-formulas.md
import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { TablePanel } from './TablePanel';
afterEach(cleanup);
</storycode:copy>
<a id="f-ca0207209fcf-source"></a>
### `source`

<storycode:copy id="f-ca0207209fcf-source">
const source = '1,2,3,4,5\n6,7,8,9,10\n11,12,=B3*2,14,15\n16,17,18,19,20\n21,22,23,24,25\n';
</storycode:copy>
<a id="f-ca0207209fcf-cell"></a>
### `cell`

<storycode:copy id="f-ca0207209fcf-cell">
const cell = (row: number, column: number) => document.querySelector(`.table-panel__cell[data-row="${row}"][data-column="${column}"]`)!;
</storycode:copy>
<a id="f-ca0207209fcf-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ca0207209fcf-expression_statement-2">
it.each([[0,2,'=B1*2'],[4,2,'=B5*2'],[2,0,'=#REF!*2'],[2,4,'=D3*2']])('fills toward %s,%s once on release', (row,column,formula) => {
  const onChange = vi.fn();
  render(<TablePanel source={source} header={false} language="javascript" formats="C3:USD" onChange={onChange}/>);
  fireEvent.click(cell(2,2));
  fireEvent.mouseDown(screen.getByRole('button',{name:'Fill selected cells'}),{button:0});
  fireEvent.mouseEnter(cell(row,column));
  expect(document.querySelectorAll('.table-fill-preview').length).toBeGreaterThan(0);
  expect(onChange).not.toHaveBeenCalled();
  fireEvent.mouseUp(window);
  expect(onChange).toHaveBeenCalledTimes(1);
  expect(onChange.mock.calls[0][0].trim().split('\n')[row].split(',')[column]).toBe(formula);
  expect(onChange.mock.calls[0][1]).toContain(`${String.fromCharCode(65+Number(column))}${Number(row)+1}:USD`);
});
</storycode:copy>
<a id="f-ca0207209fcf-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-ca0207209fcf-expression_statement-3">
it('cancels with Escape and invalidates the seed after an external edit', () => {
  const onChange=vi.fn();
  const props={source,header:false,language:'python',onChange};
  const {rerender}=render(<TablePanel {...props}/>);
  fireEvent.click(cell(2,2));
  const begin=()=>{fireEvent.mouseDown(screen.getByRole('button',{name:'Fill selected cells'}),{button:0});fireEvent.mouseEnter(cell(4,2));};
  begin();fireEvent.keyDown(window,{key:'Escape'});fireEvent.mouseUp(window);
  expect(onChange).not.toHaveBeenCalled();
  begin();rerender(<TablePanel {...props} source={source.replace('25','99')}/>);fireEvent.mouseUp(window);
  expect(onChange).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-ca0207209fcf-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-ca0207209fcf-expression_statement-4">
it('puts one handle at the lower right of a reversed selection',()=>{
  render(<TablePanel source={source} header={false} onChange={vi.fn()}/>);
  fireEvent.mouseDown(cell(3,3),{button:0});fireEvent.mouseEnter(cell(1,1));fireEvent.mouseUp(window);
  const handle=screen.getByRole('button',{name:'Fill selected cells'});
  expect(handle.closest('td')?.dataset.fillRow).toBe('3');
  expect(handle.closest('td')?.dataset.fillColumn).toBe('3');
});
</storycode:copy>

### Assemble `apps/web/src/components/TableFill.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/TableFill.test.tsx"><storycode:paste select="#f-ca0207209fcf-expression_statement"/><storycode:paste select="#f-ca0207209fcf-source"/><storycode:paste select="#f-ca0207209fcf-cell"/><storycode:paste select="#f-ca0207209fcf-expression_statement-2"/><storycode:paste select="#f-ca0207209fcf-expression_statement-3"/><storycode:paste select="#f-ca0207209fcf-expression_statement-4"/></storycode:file>

<a id="output-70f98a0676ac"></a>
## `apps/web/src/components/useTableFill.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A fill handle extends formulas](contract.md).

<a id="f-70f98a0676ac-FillDrag"></a>
### `FillDrag`

<storycode:copy id="f-70f98a0676ac-FillDrag">
import { useEffect, useRef, useState, type RefObject } from "react";
import { type Csv } from "../lib/csv";
import { fillExtent, fillTable } from "../lib/tableFill";
import { boundsOf, containsCell, type Selection } from "../lib/tableSelection";

interface FillDrag {
  table: Csv; seed: Selection; target: Selection; revision: string; formulas: string | false;
}

</storycode:copy>
<a id="f-70f98a0676ac-useTableFill"></a>
### `useTableFill`

One drag owns a frozen seed and publishes exactly once, on mouseup.

<storycode:copy id="f-70f98a0676ac-useTableFill">
/** One drag owns a frozen seed and publishes exactly once, on mouseup. */
export function useTableFill({ table, selection, revision, language, gridRef, onFill }: {
  table: Csv; selection: Selection | null; revision: string; language?: string;
  gridRef: RefObject<HTMLTableElement | null>; onFill: (table: Csv, selection: Selection) => void;
}) {
  const drag = useRef<FillDrag | null>(null);
  const latest = useRef({ table, selection, revision, language, onFill });
  latest.current = { table, selection, revision, language, onFill };
  const [preview, setPreview] = useState<Selection | null>(null);
  const cancel = () => { drag.current = null; setPreview(null); };
  const over = (row: number, column: number) => {
    const current = drag.current;
    if (!current) return;
    const target = fillExtent(current.table, current.seed, { row, column });
    const a = boundsOf(target), b = boundsOf(current.target);
    if (a.top === b.top && a.bottom === b.bottom && a.left === b.left && a.right === b.right) return;
    current.target = target;
    setPreview(target);
  };
  const handlers = useRef({ over, cancel });
  handlers.current = { over, cancel };

  useEffect(() => {
    const move = (event: MouseEvent) => {
      if (!drag.current || !gridRef.current) return;
      event.preventDefault();
      const grid = gridRef.current;
      const hit = document.elementFromPoint?.(event.clientX, event.clientY)?.closest<HTMLElement>("td[data-fill-row]");
      if (hit && grid.contains(hit)) {
        handlers.current.over(Number(hit.dataset.fillRow), Number(hit.dataset.fillColumn));
        return;
      }
      // Outside the last cell, use its dimensions to preview new rows/columns.
      // At the top/left edge there is no row before 1 or column before A.
      const rowCells = Array.from(grid.querySelectorAll<HTMLElement>('td[data-fill-column="0"]'));
      const columnCells = Array.from(grid.querySelectorAll<HTMLElement>('td[data-fill-row="0"]'));
      const coordinate = (cells: HTMLElement[], point: number, axis: "x" | "y") => {
        if (!cells.length) return 0;
        for (let index = 0; index < cells.length; index++) {
          const rect = cells[index].getBoundingClientRect();
          if (point < (axis === "x" ? rect.right : rect.bottom)) return index;
        }
        const last = cells[cells.length - 1].getBoundingClientRect();
        const end = axis === "x" ? last.right : last.bottom;
        const size = axis === "x" ? last.width : last.height;
        return cells.length + Math.floor((point - end) / Math.max(1, size));
      };
      handlers.current.over(coordinate(rowCells, event.clientY, "y"), coordinate(columnCells, event.clientX, "x"));
      const viewport = grid.parentElement;
      if (viewport && typeof viewport.scrollBy === "function") {
        const box = viewport.getBoundingClientRect();
        viewport.scrollBy(event.clientX > box.right ? 24 : event.clientX < box.left ? -24 : 0,
          event.clientY > box.bottom ? 24 : event.clientY < box.top ? -24 : 0);
      }
    };
    const finish = () => {
      const current = drag.current;
      if (!current) return;
      handlers.current.cancel();
      // An external edit invalidates the frozen seed; never overwrite it.
      if (current.revision !== latest.current.revision) return;
      const a = boundsOf(current.seed), b = boundsOf(current.target);
      if (a.top === b.top && a.bottom === b.bottom && a.left === b.left && a.right === b.right) return;
      latest.current.onFill(fillTable(current.table, current.seed, current.target, current.formulas), current.target);
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape" && drag.current) {
        event.preventDefault();
        handlers.current.cancel();
      }
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", finish);
    window.addEventListener("keydown", escape);
    window.addEventListener("blur", handlers.current.cancel);
    const onBlur = handlers.current.cancel;
    return () => {
      drag.current = null;
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", finish);
      window.removeEventListener("keydown", escape);
      window.removeEventListener("blur", onBlur);
    };
  }, [gridRef]);

  const corner = selection ? boundsOf(selection) : null;
  return {
    at: (row: number, column: number) => corner?.bottom === row && corner.right === column,
    preview,
    includes: (row: number, column: number) => !!preview && containsCell(preview, row, column) &&
      !!selection && !containsCell(selection, row, column),
    over,
    start: (event: React.MouseEvent) => {
      if (event.button !== 0 || !selection) return;
      event.preventDefault(); event.stopPropagation();
      drag.current = { table, seed: selection, target: selection, revision, formulas: language || false };
      setPreview(selection);
    },
    // A ref rather than render state allows hover handlers to distinguish a drag
    // from ordinary selection without waiting for React to rerender.
    dragging: () => drag.current !== null,
  };
}
</storycode:copy>

### Assemble `apps/web/src/components/useTableFill.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/useTableFill.ts"><storycode:paste select="#f-70f98a0676ac-FillDrag"/><storycode:paste select="#f-70f98a0676ac-useTableFill"/></storycode:file>

<a id="output-3aa4c57ffe89"></a>
## `apps/web/src/editor/tableFill.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A fill handle extends formulas](contract.md).

<a id="f-3aa4c57ffe89-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-3aa4c57ffe89-expression_statement">
// Guarantee: docs/guarantees/authoring/a-fill-handle-extends-formulas.md
import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { EditorView } from '@codemirror/view';
import { undo } from '@codemirror/commands';
import { DocumentEditor } from './DocumentEditor';
import { LocalRealtime } from '../api/realtime';
afterEach(cleanup);
</storycode:copy>
<a id="f-3aa4c57ffe89-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-3aa4c57ffe89-expression_statement-2">
it('persists a formula fill and its format as one undoable document edit',async()=>{
  const source='Intro.\n\n<hick:table language="python" formats="B1:USD">\n2,=A1*2\n3,\n4,\n</hick:table>\n\nAfter.';
  const {container}=render(<DocumentEditor docId="fill-grid" initialSource={source} realtime={new LocalRealtime()} execBlocks={[]} runningCells={new Set()} onRunCell={()=>undefined} onChange={vi.fn()}/>);
  await waitFor(()=>expect(screen.getByTestId('table-panel')).toBeTruthy());
  const view=EditorView.findFromDOM(container.querySelector('.cm-editor')!)!;
  fireEvent.click(container.querySelector('.table-panel__cell[data-row="0"][data-column="1"]')!);
  fireEvent.mouseDown(screen.getByRole('button',{name:'Fill selected cells'}),{button:0});
  fireEvent.mouseEnter(container.querySelector('.table-panel__cell[data-row="2"][data-column="1"]')!);
  expect(view.state.doc.toString()).toBe(source);
  fireEvent.mouseUp(window);
  await waitFor(()=>expect(view.state.doc.toString()).toContain('4,=A3*2'));
  expect(view.state.doc.toString()).toContain('B3:USD');
  expect(view.state.doc.toString()).toContain('After.');
  undo(view);
  await waitFor(()=>expect(view.state.doc.toString()).toBe(source));
});
</storycode:copy>

### Assemble `apps/web/src/editor/tableFill.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/tableFill.test.tsx"><storycode:paste select="#f-3aa4c57ffe89-expression_statement"/><storycode:paste select="#f-3aa4c57ffe89-expression_statement-2"/></storycode:file>

<a id="output-086d60b2c1ac"></a>
## `apps/web/src/lib/formulaReferences.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A fill handle extends formulas](contract.md).

<a id="f-086d60b2c1ac-Cell"></a>
### `Cell`

The A1 references in a formula are part of the sheet, rather than part of
Python or JavaScript. Keeping this rewrite here means structural edits move
the data AND every formula's pointers to that data together.

<storycode:copy id="f-086d60b2c1ac-Cell">
// The A1 references in a formula are part of the sheet, rather than part of
// Python or JavaScript. Keeping this rewrite here means structural edits move
// the data AND every formula's pointers to that data together.

import { cellLabel, parseCellLabel } from "./cellRef";

type Cell = { row: number; column: number };

</storycode:copy>
<a id="f-086d60b2c1ac-rewriteFormulaReferences"></a>
### `rewriteFormulaReferences`

Rewrite every A1 reference (including both ends of an A1 range).

<storycode:copy id="f-086d60b2c1ac-rewriteFormulaReferences">
/** Rewrite every A1 reference (including both ends of an A1 range). */
export function rewriteFormulaReferences(formula: string, move: (cell: Cell) => Cell): string {
  if (!formula.trimStart().startsWith("=")) return formula;
  return formula.replace(
    /(^|[^A-Za-z0-9_])((?:\$?[A-Za-z]{1,2}\$?\d+)(?::\$?[A-Za-z]{1,2}\$?\d+)?)(?![A-Za-z0-9_])/g,
    (_whole, before: string, reference: string) => {
      const rewritten = reference.replace(/\$?[A-Za-z]{1,2}\$?\d+/g, (part: string) => {
        const columnAbsolute = part.startsWith("$");
        const rowAbsolute = /\$\d+$/.test(part);
        const cell = parseCellLabel(part.replace(/\$/g, "").toUpperCase());
        if (!cell) return part;
        const moved = move(cell);
        const next = cellLabel(moved.column, moved.row);
        const match = /^([A-Z]+)(\d+)$/.exec(next)!;
        return `${columnAbsolute ? "$" : ""}${match[1]}${rowAbsolute ? "$" : ""}${match[2]}`;
      });
      return before + rewritten;
    },
  );
}

</storycode:copy>
<a id="f-086d60b2c1ac-shiftFormulaReferences"></a>
### `shiftFormulaReferences`

Shift references that moved with an inserted row or column.

<storycode:copy id="f-086d60b2c1ac-shiftFormulaReferences">
/** Shift references that moved with an inserted row or column. */
export function shiftFormulaReferences(
  formula: string,
  axis: "row" | "column",
  at: number,
): string {
  return rewriteFormulaReferences(formula, (cell) =>
    axis === "row"
      ? { ...cell, row: cell.row >= at ? cell.row + 1 : cell.row }
      : { ...cell, column: cell.column >= at ? cell.column + 1 : cell.column },
  );
}

</storycode:copy>
<a id="f-086d60b2c1ac-moveFormulaReferences"></a>
### `moveFormulaReferences`

Follow a rectangular cut-and-paste to its destination.

<storycode:copy id="f-086d60b2c1ac-moveFormulaReferences">
/** Follow a rectangular cut-and-paste to its destination. */
export function moveFormulaReferences(
  formula: string,
  from: Cell,
  size: { rows: number; columns: number },
  to: Cell,
): string {
  return rewriteFormulaReferences(formula, (cell) => {
    const inside =
      cell.row >= from.row &&
      cell.row < from.row + size.rows &&
      cell.column >= from.column &&
      cell.column < from.column + size.columns;
    return inside
      ? { row: to.row + cell.row - from.row, column: to.column + cell.column - from.column }
      : cell;
  });
}

</storycode:copy>
<a id="f-086d60b2c1ac-copyFormulaReferences"></a>
### `copyFormulaReferences`

Copying changes relative references; insertion and cut/paste move even locks.

<storycode:copy id="f-086d60b2c1ac-copyFormulaReferences">
/** Copying changes relative references; insertion and cut/paste move even locks. */
export function copyFormulaReferences(formula: string, rows: number, columns: number, language = "javascript"): string {
  if (!formula.trimStart().startsWith("=")) return formula;
  // Skip string literals and comments. This is lexical, like reference discovery;
  // the expression still belongs to Python/JavaScript, not a spreadsheet parser.
  const strings = /"""[\s\S]*?"""|'''[\s\S]*?'''|"(?:\\[\s\S]|[^"\\])*"|'(?:\\[\s\S]|[^'\\])*'|`(?:\\[\s\S]|[^`\\])*`/.source;
  const references = /(?<![A-Za-z0-9_$])(\$?[A-Za-z]{1,2}\$?[1-9]\d*)(?![A-Za-z0-9_])/.source;
  const python = /^(python|python3|py)$/i.test(language.trim());
  const comments = python ? /#[^\n]*/.source : /\/\/[^\n]*|\/\*[\s\S]*?\*\//.source;
  const pattern = new RegExp(`${strings}|${comments}|${references}`, "g");
  return formula.replace(pattern,
    (whole: string, reference: string | undefined) => {
      if (!reference) return whole;
      const at = parseCellLabel(reference.replace(/\$/g, "").toUpperCase());
      if (!at) return whole;
      const columnAbsolute = reference.startsWith("$");
      const rowAbsolute = /\$\d+$/.test(reference);
      const row = at.row + (rowAbsolute ? 0 : rows);
      const column = at.column + (columnAbsolute ? 0 : columns);
      if (row < 0 || column < 0 || column >= 702) return "#REF!";
      const label = cellLabel(column, row).match(/^([A-Z]+)(\d+)$/)!;
      return `${columnAbsolute ? "$" : ""}${label[1]}${rowAbsolute ? "$" : ""}${label[2]}`;
    },
  );
}
</storycode:copy>

### Assemble `apps/web/src/lib/formulaReferences.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/formulaReferences.ts"><storycode:paste select="#f-086d60b2c1ac-Cell"/><storycode:paste select="#f-086d60b2c1ac-rewriteFormulaReferences"/><storycode:paste select="#f-086d60b2c1ac-shiftFormulaReferences"/><storycode:paste select="#f-086d60b2c1ac-moveFormulaReferences"/><storycode:paste select="#f-086d60b2c1ac-copyFormulaReferences"/></storycode:file>

<a id="output-ee0541091f1a"></a>
## `apps/web/src/lib/tableFill.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A fill handle extends formulas](contract.md).

<a id="f-ee0541091f1a-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ee0541091f1a-expression_statement">
// Guarantee: docs/guarantees/authoring/a-fill-handle-extends-formulas.md
import { describe, expect, it } from "vitest";
import { copyFormulaReferences } from "./formulaReferences";
import { parseCsv } from "./csv";
import { fillSelection, fillTable } from "./tableFill";
import { boundsOf, cellSelection, type Selection } from "./tableSelection";

it("copies relative, absolute and mixed references and both range endpoints", () => {
  expect(copyFormulaReferences("=sum(A1:B3) + $A1 + B$2 + $C$4", 2, 3))
    .toBe("=sum(D3:E5) + $A3 + E$2 + $C$4");
  expect(copyFormulaReferences("=C4 + $C4 + C$4 + $C$4", -2, -1)).toBe("=B2 + $C2 + B$4 + $C$4");
});
</storycode:copy>
<a id="f-ee0541091f1a-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ee0541091f1a-expression_statement-2">
it("leaves language strings, comments and identifiers intact", () => {
  expect(copyFormulaReferences('=A1 + len("A1") + myA1 # B2', 1, 0, "python")).toBe('=A2 + len("A1") + myA1 # B2');
  expect(copyFormulaReferences("=A1 + 'B2'.length // C3", 0, 1)).toBe("=B1 + 'B2'.length // C3");
  expect(copyFormulaReferences("=A1 + `B2` /* C3 */", 1, 0)).toBe("=A2 + `B2` /* C3 */");
  expect(copyFormulaReferences("=len('''A1\nB2''') + C3", 1, 0)).toBe("=len('''A1\nB2''') + C4");
});
</storycode:copy>
<a id="f-ee0541091f1a-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-ee0541091f1a-expression_statement-3">
it("marks references moved before A1 invalid instead of wrapping them", () => {
  expect(copyFormulaReferences("=A1 + $A$1", -1, -1)).toBe("=#REF! + $A$1");
});

</storycode:copy>
<a id="f-ee0541091f1a-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-ee0541091f1a-expression_statement-4">
describe("four directions", () => {
  const seed = cellSelection(2, 2);
  const table = parseCsv("0,0,0,0,0\n0,0,0,0,0\n0,0,=B3 * 2,0,0\n0,0,0,0,0\n0,0,0,0,0");
  it.each([
    [{ row: 4, column: 2 }, 4, 2, "=B5 * 2"],
    [{ row: 0, column: 2 }, 0, 2, "=B1 * 2"],
    [{ row: 2, column: 4 }, 2, 4, "=D3 * 2"],
    [{ row: 2, column: 1 }, 2, 1, "=A3 * 2"],
  ] as const)("fills toward %j", (at, row, column, expected) => {
    const filled = fillTable(table, seed, fillSelection(seed, at, 5, 5), true);
    expect(filled.rows[row][column]).toBe(expected);
    expect(filled.rows[2][2]).toBe("=B3 * 2");
    expect(table.rows[row][column]).toBe("0");
  });
});
</storycode:copy>
<a id="f-ee0541091f1a-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-ee0541091f1a-expression_statement-5">
it("tiles a selected block upward and downward from the frozen seed, carrying formats", () => {
  const table = { ...parseCsv("0\n0\n=A3 * 2\n=A4 * 3\n0\n0"), formats: { A3: "USD", A4: "percent", A1: "number" } as const };
  const seed: Selection = { anchor: { row: 3, column: 0 }, focus: { row: 2, column: 0 }, kind: "cells" };
  const up = fillTable(table, seed, fillSelection(seed, { row: 0, column: 0 }, 6, 1), true);
  expect(up.rows.slice(0, 4)).toEqual([["=A1 * 2"], ["=A2 * 3"], ["=A3 * 2"], ["=A4 * 3"]]);
  expect(up.formats?.A1).toBe("USD");
  expect(up.formats?.A2).toBe("percent");
  const down = fillTable(table, seed, fillSelection(seed, { row: 5, column: 0 }, 6, 1), true);
  expect(down.rows.slice(4)).toEqual([["=A5 * 2"], ["=A6 * 3"]]);
});
</storycode:copy>
<a id="f-ee0541091f1a-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-ee0541091f1a-expression_statement-6">
it("repeats literals and treats equals as text when no language is declared", () => {
  const table = parseCsv("=A1,hello\n0,0");
  const seed: Selection = { anchor: { row: 0, column: 0 }, focus: { row: 0, column: 1 }, kind: "cells" };
  expect(fillTable(table, seed, fillSelection(seed, { row: 1, column: 1 }, 2, 2), false).rows[1]).toEqual(["=A1", "hello"]);
});
</storycode:copy>
<a id="f-ee0541091f1a-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-ee0541091f1a-expression_statement-7">
it("grows within the grid budget and selects only one axis on a diagonal", () => {
  const seed = cellSelection(0, 0);
  expect(boundsOf(fillSelection(seed, { row: 100_000, column: 0 }, 1, 2))).toEqual({ top: 0, bottom: 9999, left: 0, right: 0 });
  expect(boundsOf(fillSelection(seed, { row: 0, column: 100_000 }, 2, 1))).toEqual({ top: 0, bottom: 0, left: 0, right: 9999 });
  const target = fillSelection(seed, { row: 3, column: 2 }, 1, 1);
  expect(boundsOf(target)).toEqual({ top: 0, bottom: 3, left: 0, right: 0 });
  expect(fillTable(parseCsv("=B1"), seed, target, true).rows).toEqual([["=B1"], ["=B2"], ["=B3"], ["=B4"]]);
});

</storycode:copy>
<a id="f-ee0541091f1a-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-ee0541091f1a-expression_statement-8">
it('keeps Python floor division while moving its operand references', () => {
  expect(copyFormulaReferences('=A1 // B1 # C1', 1, 0, 'python')).toBe('=A2 // B2 # C1');
});
</storycode:copy>

### Assemble `apps/web/src/lib/tableFill.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tableFill.test.ts"><storycode:paste select="#f-ee0541091f1a-expression_statement"/><storycode:paste select="#f-ee0541091f1a-expression_statement-2"/><storycode:paste select="#f-ee0541091f1a-expression_statement-3"/><storycode:paste select="#f-ee0541091f1a-expression_statement-4"/><storycode:paste select="#f-ee0541091f1a-expression_statement-5"/><storycode:paste select="#f-ee0541091f1a-expression_statement-6"/><storycode:paste select="#f-ee0541091f1a-expression_statement-7"/><storycode:paste select="#f-ee0541091f1a-expression_statement-8"/></storycode:file>

<a id="output-b8bb84031a3f"></a>
## `apps/web/src/lib/tableFill.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A fill handle extends formulas](contract.md).

<a id="f-b8bb84031a3f-fillSelection"></a>
### `fillSelection`

<storycode:copy id="f-b8bb84031a3f-fillSelection">
import { cellLabel } from "./cellRef";
import { columnCount, type Csv } from "./csv";
import { copyFormulaReferences } from "./formulaReferences";
import { boundsOf, type Cell, type Selection } from "./tableSelection";

/** Extend along one axis. Diagonal drags use the larger distance outside the seed. */
export function fillSelection(seed: Selection, at: Cell, height: number, width: number, limit = 20_000): Selection {
  const bounds = boundsOf(seed);
  const row = Math.max(0, Math.min(at.row, Math.max(height, Math.floor(limit / width)) - 1));
  const column = Math.max(0, Math.min(at.column, Math.max(width, Math.floor(limit / height)) - 1));
  const vertical = row < bounds.top ? row - bounds.top : Math.max(0, row - bounds.bottom);
  const horizontal = column < bounds.left ? column - bounds.left : Math.max(0, column - bounds.right);
  return {
    kind: "cells",
    anchor: { row: Math.abs(vertical) >= Math.abs(horizontal) ? Math.min(bounds.top, row) : bounds.top,
      column: Math.abs(horizontal) > Math.abs(vertical) ? Math.min(bounds.left, column) : bounds.left },
    focus: { row: Math.abs(vertical) >= Math.abs(horizontal) ? Math.max(bounds.bottom, row) : bounds.bottom,
      column: Math.abs(horizontal) > Math.abs(vertical) ? Math.max(bounds.right, column) : bounds.right },
  };
}

</storycode:copy>
<a id="f-b8bb84031a3f-fillTable"></a>
### `fillTable`

Repeat the frozen seed, adjusting each formula from its own source position.

<storycode:copy id="f-b8bb84031a3f-fillTable">
/** Repeat the frozen seed, adjusting each formula from its own source position. */
export function fillTable(table: Csv, seed: Selection, target: Selection, formulas: boolean | string): Csv {
  const from = boundsOf(seed);
  const to = boundsOf(target);
  const rows = table.rows.map(row => [...row]);
  const formats = { ...table.formats };
  const modulo = (n: number, divisor: number) => ((n % divisor) + divisor) % divisor;
  for (let row = to.top; row <= to.bottom; row++) {
    while (rows.length <= row) rows.push([]);
    for (let column = to.left; column <= to.right; column++) {
      if (row >= from.top && row <= from.bottom && column >= from.left && column <= from.right) continue;
      const sourceRow = from.top + modulo(row - from.top, from.bottom - from.top + 1);
      const sourceColumn = from.left + modulo(column - from.left, from.right - from.left + 1);
      const value = table.rows[sourceRow]?.[sourceColumn] ?? "";
      while (rows[row].length <= column) rows[row].push("");
      rows[row][column] = formulas ? copyFormulaReferences(value, row - sourceRow, column - sourceColumn, typeof formulas === "string" ? formulas : undefined) : value;
      const label = cellLabel(column, row);
      const format = table.formats?.[cellLabel(sourceColumn, sourceRow)];
      if (format) formats[label] = format;
      else delete formats[label];
    }
  }
  return { ...table, rows, ...(table.formats ? { formats } : {}) };
}

</storycode:copy>
<a id="f-b8bb84031a3f-fillExtent"></a>
### `fillExtent`

<storycode:copy id="f-b8bb84031a3f-fillExtent">
export function fillExtent(table: Csv, seed: Selection, at: Cell): Selection {
  return fillSelection(seed, at, Math.max(1, table.rows.length), Math.max(1, columnCount(table)));
}
</storycode:copy>

### Assemble `apps/web/src/lib/tableFill.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tableFill.ts"><storycode:paste select="#f-b8bb84031a3f-fillSelection"/><storycode:paste select="#f-b8bb84031a3f-fillTable"/><storycode:paste select="#f-b8bb84031a3f-fillExtent"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/TableFormatControl.tsx](../../../mechanisms/apps/web/src/components/TableFormatControl.tsx.md#output-2faa595b6a67)
- [apps/web/src/components/TablePanel.tsx](../../../mechanisms/apps/web/src/components/TablePanel.tsx.md#output-38d5395d7736)
- [apps/web/src/components/useTableFill.ts](contract.md#output-70f98a0676ac)
- [apps/web/src/lib/formulaReferences.ts](contract.md#output-086d60b2c1ac)
- [apps/web/src/lib/tableFill.ts](contract.md#output-b8bb84031a3f)

### Tests and fixtures

- [apps/web/e2e/tableFill.spec.ts](contract.md#output-2e2dc30f7f54)
- [apps/web/src/components/TableFill.test.tsx](contract.md#output-ca0207209fcf)
- [apps/web/src/editor/tableFill.test.tsx](contract.md#output-3aa4c57ffe89)
- [apps/web/src/lib/tableFill.test.ts](contract.md#output-ee0541091f1a)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
