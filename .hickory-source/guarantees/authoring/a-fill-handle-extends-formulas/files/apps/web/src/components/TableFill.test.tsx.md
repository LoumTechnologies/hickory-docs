<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/TableFill.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A fill handle extends formulas](../../../../../contract.md)

Captured SHA-256: `3f0b36340cb89ac0028b4ceae56f28591ccb7a4b2f02356f36f6da0a9b06cd78`.

<book:entry path="apps/web/src/components/TableFill.test.tsx">
// Guarantee: docs/guarantees/authoring/a-fill-handle-extends-formulas.md
import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { TablePanel } from './TablePanel';
afterEach(cleanup);
const source = '1,2,3,4,5\n6,7,8,9,10\n11,12,=B3*2,14,15\n16,17,18,19,20\n21,22,23,24,25\n';
const cell = (row: number, column: number) => document.querySelector(`.table-panel__cell[data-row="${row}"][data-column="${column}"]`)!;
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
it('puts one handle at the lower right of a reversed selection',()=>{
  render(<TablePanel source={source} header={false} onChange={vi.fn()}/>);
  fireEvent.mouseDown(cell(3,3),{button:0});fireEvent.mouseEnter(cell(1,1));fireEvent.mouseUp(window);
  const handle=screen.getByRole('button',{name:'Fill selected cells'});
  expect(handle.closest('td')?.dataset.fillRow).toBe('3');
  expect(handle.closest('td')?.dataset.fillColumn).toBe('3');
});
</book:entry>

</book:doc>
