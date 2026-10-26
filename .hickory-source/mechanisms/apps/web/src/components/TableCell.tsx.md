<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/TableCell.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-59dbc5d33133"></a>
## `apps/web/src/components/TableCell.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-59dbc5d33133-TableCellProps"></a>
### `TableCellProps`

One editable or selected cell in TablePanel's grid.

<storycode:copy id="f-59dbc5d33133-TableCellProps">
// One editable or selected cell in TablePanel's grid.

import type React from "react";

export interface TableCellProps {
  row: number;
  column: number;
  value: string;
  shown: string;
  problem?: string;
  selected: boolean;
  within: boolean;
  stepping: boolean;
  read: boolean;
  pointedAt: boolean;
  editing: boolean;
  draft: string;
  editable: boolean;
  inputRef: React.MutableRefObject<HTMLInputElement | null>;
  selectedRef: React.MutableRefObject<HTMLElement | null>;
  onDown: (event: React.MouseEvent) => void;
  onEnter: () => void;
  onSelect: (event: React.MouseEvent) => void;
  onEdit: () => void;
  onDraft: (value: string) => void;
  onPoint: (event: React.KeyboardEvent<HTMLInputElement>) => boolean;
  onKeys: (event: React.KeyboardEvent) => void;
  onMove: (dRow: number, dColumn: number) => void;
  onDone: () => void;
  onCancel: () => void;
}

</storycode:copy>
<a id="f-59dbc5d33133-TableCell"></a>
### `TableCell`

<storycode:copy id="f-59dbc5d33133-TableCell">
export function TableCell({
  row,
  column,
  value,
  shown,
  problem,
  selected,
  within,
  stepping,
  read,
  pointedAt,
  editing,
  draft,
  editable,
  inputRef,
  selectedRef,
  onDown,
  onEnter,
  onSelect,
  onEdit,
  onDraft,
  onPoint,
  onKeys,
  onMove,
  onDone,
  onCancel,
}: TableCellProps) {
  if (!editing)
    return (
      <span
        ref={(node) => {
          if (selected) selectedRef.current = node;
        }}
        data-row={row}
        data-column={column}
        className={
          "table-panel__cell" +
          (problem ? " table-panel__cell--bad" : "") +
          (value !== shown ? " table-panel__cell--computed" : "") +
          (selected ? " table-panel__cell--selected" : "") +
          (within ? " table-panel__cell--within" : "") +
          (stepping ? " table-panel__cell--stepping" : "") +
          (read ? " table-panel__cell--read" : "") +
          (pointedAt ? " table-panel__cell--pointed" : "")
        }
        role={editable ? "gridcell" : undefined}
        tabIndex={editable ? 0 : undefined}
        data-tip={problem ?? (value !== shown ? value : undefined)}
        onMouseDown={
          editable
            ? (event) => {
                if (event.button === 2) return;
                event.preventDefault();
                onDown(event);
              }
            : undefined
        }
        onMouseEnter={editable ? onEnter : undefined}
        onClick={editable ? onSelect : undefined}
        onDoubleClick={editable ? onEdit : undefined}
        onKeyDown={editable ? onKeys : undefined}
      >
        {shown === "" ? " " : shown}
      </span>
    );
  return (
    <input
      ref={inputRef}
      className="table-panel__input"
      value={draft}
      onChange={(event) => onDraft(event.target.value)}
      onBlur={onDone}
      onKeyDown={(event) => {
        if (event.key.startsWith("Arrow") && onPoint(event)) {
          event.preventDefault();
          return;
        }
        if (event.key === "Enter") {
          event.preventDefault();
          onMove(event.shiftKey ? -1 : 1, 0);
        } else if (event.key === "Tab") {
          event.preventDefault();
          onMove(0, event.shiftKey ? -1 : 1);
        } else if (event.key === "Escape") {
          event.preventDefault();
          onCancel();
        } else if (event.key === "ArrowUp" || event.key === "ArrowDown") {
          event.preventDefault();
          onMove(event.key === "ArrowDown" ? 1 : -1, 0);
        } else if (
          event.key === "ArrowLeft" &&
          (event.currentTarget.selectionStart ?? 0) === 0 &&
          (event.currentTarget.selectionEnd ?? 0) === 0
        ) {
          event.preventDefault();
          onMove(0, -1);
        } else if (
          event.key === "ArrowRight" &&
          (event.currentTarget.selectionStart ?? draft.length) ===
            draft.length &&
          (event.currentTarget.selectionEnd ?? draft.length) === draft.length
        ) {
          event.preventDefault();
          onMove(0, 1);
        }
      }}
    />
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/TableCell.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/TableCell.tsx"><storycode:paste select="#f-59dbc5d33133-TableCellProps"/><storycode:paste select="#f-59dbc5d33133-TableCell"/></storycode:file>

</storycode:doc>
