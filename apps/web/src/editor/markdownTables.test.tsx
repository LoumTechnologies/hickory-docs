// Guarantee: docs/guarantees/authoring/markdown-tables-use-the-table-editor.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EditorView } from "@codemirror/view";
import { DocumentEditor } from "./DocumentEditor";
import { LocalRealtime } from "../api/realtime";
import { showBlockSource } from "./rendered";

afterEach(cleanup);

it("opens a Markdown table as a grid, keeps edits after prose moves it, and reveals source", async () => {
  const onChange = vi.fn();
  const source = 'Intro.\n\n| Name | Count |\n| :--- | ---: |\n| North | 120 |\n\nAfter.';
  const { container } = render(<DocumentEditor docId="markdown-grid" initialSource={source}
    realtime={new LocalRealtime()} execBlocks={[]} runningCells={new Set()}
    onRunCell={() => undefined} onChange={onChange} />);
  await waitFor(() => expect(screen.getByTestId("table-panel")).toBeTruthy());
  const view = EditorView.findFromDOM(container.querySelector('.cm-editor')!)!;
  view.dispatch({ changes: { from: 0, insert: '# Notes\n' } });
  fireEvent.doubleClick(screen.getByText('120'));
  const input = screen.getByRole('textbox', { name: 'Cell contents' });
  fireEvent.change(input, { target: { value: '121' } });
  fireEvent.keyDown(input, { key: 'Enter' });
  await waitFor(() => expect(view.state.doc.toString()).toContain('| North | 121 |'));
  expect(view.state.doc.toString()).toBe('# Notes\n' + source.replace('120', '121'));
  expect(onChange).toHaveBeenLastCalledWith(view.state.doc.toString());
  expect(screen.getByTestId('table-panel')).toBeTruthy();
  // Editing the header replaces the table's first bytes too, and must keep
  // its rendered state and mounted grid rather than discard the fold.
  fireEvent.doubleClick(screen.getByText('Name'));
  const header = screen.getByRole('textbox', { name: 'Cell contents' });
  fireEvent.change(header, { target: { value: 'Region' } });
  fireEvent.keyDown(header, { key: 'Enter' });
  await waitFor(() => expect(view.state.doc.toString()).toContain('| Region | Count |'));
  expect(screen.getByTestId('table-panel')).toBeTruthy();
  const at = view.state.doc.toString().indexOf('| Region');
  view.dispatch({ effects: showBlockSource.of(at) });
  await waitFor(() => expect(screen.queryByTestId('table-panel')).toBeNull());
  expect(container.querySelector('.cm-content')?.textContent).toContain('| Region | Count |');
});
