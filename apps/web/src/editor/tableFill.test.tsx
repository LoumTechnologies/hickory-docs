// Guarantee: docs/guarantees/authoring/a-fill-handle-extends-formulas.md
import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { EditorView } from '@codemirror/view';
import { undo } from '@codemirror/commands';
import { DocumentEditor } from './DocumentEditor';
import { LocalRealtime } from '../api/realtime';
afterEach(cleanup);
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
