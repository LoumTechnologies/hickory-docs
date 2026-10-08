// Guarantee: docs/guarantees/authoring/cell-formatting-keeps-numbers-numeric.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EditorView } from "@codemirror/view";
import { undo } from "@codemirror/commands";
import { DocumentEditor } from "./DocumentEditor";
import { LocalRealtime } from "../api/realtime";

afterEach(cleanup);

it.each([
  ['| Cost |\n| ---: |\n| 1234.567 |', '<!-- hickory-table-formats: A2:USD -->'],
  ['<hick:table path="cost.csv" language="python">\nCost\n1234.567\n</hick:table>', 'formats="A2:USD"'],
])("persists formatting in the live document, survives prose edits, and undoes it", async (table, metadata) => {
  const source = `Intro.\n\n${table}\n\nAfter.`;
  const { container, unmount } = render(<DocumentEditor docId="formatted-grid" initialSource={source}
    realtime={new LocalRealtime()} execBlocks={[]} runningCells={new Set()}
    onRunCell={() => undefined} onChange={vi.fn()} />);
  await waitFor(() => expect(screen.getByTestId("table-panel")).toBeTruthy());
  const view = EditorView.findFromDOM(container.querySelector('.cm-editor')!)!;
  view.dispatch({ changes: { from: 0, insert: '# Notes\n' } });
  fireEvent.click(screen.getByText('1234.567'));
  fireEvent.change(screen.getByLabelText('Cell number format'), { target: { value: 'USD' } });
  await waitFor(() => expect(view.state.doc.toString()).toContain(metadata));
  await waitFor(() => expect(screen.getByText('$1,234.57')).toBeTruthy());
  const saved = view.state.doc.toString();
  expect(saved).toContain('1234.567');
  expect(saved).toContain('After.');
  undo(view);
  await waitFor(() => expect(view.state.doc.toString()).toBe('# Notes\n' + source));
  unmount();
  render(<DocumentEditor docId="formatted-grid-reopened" initialSource={saved}
    realtime={new LocalRealtime()} execBlocks={[]} runningCells={new Set()}
    onRunCell={() => undefined} onChange={vi.fn()} />);
  await waitFor(() => expect(screen.getByText('$1,234.57')).toBeTruthy());
});
