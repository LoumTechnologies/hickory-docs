# Formula-range drag: handoff for a browser-level fix

**Audience:** the next engineer or coding agent debugging a real browser drag
inside the rendered table editor.

## The unresolved failure

An author reports that they still cannot drag across cells to insert an A1
range while editing a table formula. This report remains true after the
changes in the accompanying commit.

Do not close this by pointing to the unit test. The unit test passes, but it
dispatches synthetic events directly at the React component and does not prove
that Chrome sends the same sequence while the pointer is held down over the
rendered table.

The expected interaction is:

1. Open a formula cell in a `<hick:table language="python">`.
2. Type `=sum(` so the caret is after the opening parenthesis.
3. Press in cell `A2`, hold the mouse button, and drag to `A4`.
4. The formula editor should contain `=sum(A2:A4` before the mouse button is
   released. Releasing must keep the formula editor open.

This must also work from the formula bar, not only from the grid input.

## What was attempted

`apps/web/src/components/TablePanel.tsx` now has:

- `rangeAnchor` and `rangeDragged` refs set when a formula-pointing drag
  begins;
- `pointRangeTo`, which writes `A2:A4` through the existing `pointAt` helper;
- `pointRangeOver`, invoked from both the inner cell's `mouseenter` and its
  containing `<td>`'s `mousemove`.

The latest theory was that the 5px column/row resize handles sit above the
inner `.table-panel__cell` span, so an inner-span `mouseenter` can be skipped
during a real drag. Moving tracking to the `<td>` did not solve the author's
actual interaction.

`TablePanel.test.tsx` covers this with `mouseDown` on `A2`, `mouseMove` on
`A4`, and `click` on `A4`. That is not evidence that an operating browser
delivers the same events.

## Reproduce before changing code

Run `just dev`, open a document containing this table, and use an actual
mouse drag:

```hick
<hick:table language="python">
value,total
1,=sum(
2,
3,
</hick:table>
```

Use browser devtools or a temporary event log to record, in order:

- `mousedown` on the source cell;
- `mousemove`, `mouseover`, `mouseenter`, and pointer-event equivalents on
  the target cell and its `<td>`;
- `mouseup` and `click`;
- the focused element and the formula input's selection range after each.

Also reproduce with a drag that crosses a row or column resize edge and with
the formula bar as the focused editor. The `Engaged` wrapper and rendered
document widget may change the event path compared with the isolated component
test.

## Constraints that must remain true

- A normal drag, outside formula-point mode, still selects table cells.
- A click in formula-point mode still inserts one A1 reference.
- The active selection remains the formula cell; the range being pointed at
  must not silently select or edit its endpoint.
- A drag must not blur or commit the formula input before its range is written.
- A table with no `language` treats `=` as ordinary text and must not enter
  formula-point mode.

## Related completed work in this commit

The formula engine now understands ranges once text such as `A2:A4` reaches
it: the host expands the rectangle, binds it as a list, and rewrites the
backend expression to a legal temporary identifier. Python and the
JavaScript-backed TypeScript expression path both have backend tests for
`sum(A1:A3)`. This is not the reported failure; the unresolved part is getting
the range text into the editor through a real drag.

Formula references also follow inserted rows, inserted columns, and this
editor's private cut-and-paste move. Those changes should be retained while
the drag interaction is replaced or repaired.

