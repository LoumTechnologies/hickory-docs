<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/FenceTable.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-effdc999f8dc"></a>
## `apps/web/src/components/FenceTable.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Table Is A Dataset And A Paragraph](../../../../../guarantees/authoring/a-table-is-a-dataset-and-a-paragraph/contract.md), [Cell formatting keeps numbers numeric](../../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md).

<a id="f-effdc999f8dc-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-effdc999f8dc-expression_statement">
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";

import { FenceTable, tableElementFor } from "./FenceTable";

afterEach(() => {
  cleanup();
});

</storycode:copy>
<a id="f-effdc999f8dc-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-effdc999f8dc-expression_statement-2">
describe("the element a fence becomes", () => {
  it("carries the CSV across untouched", () => {
    // A promotion that reformatted the data would be a promotion nobody
    // could check.
    expect(tableElementFor("a,b\n1,2\n", "data/x.csv")).toBe(
      '<hick:table path="data/x.csv">\na,b\n1,2\n</hick:table>',
    );
  });

  it("writes no path attribute when no file was asked for", () => {
    expect(tableElementFor("a\n", "")).toBe("<hick:table>\na\n</hick:table>");
  });

  it("escapes a quote in the path rather than breaking the tag", () => {
    expect(tableElementFor("a\n", 'we"ird.csv')).toContain('path="we&quot;ird.csv"');
  });
});

</storycode:copy>
<a id="f-effdc999f8dc-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-effdc999f8dc-expression_statement-3">
describe("editing a fence as a grid", () => {
  it("writes the cells back as CSV", () => {
    const onChange = vi.fn();
    render(<FenceTable body="a,b\n1,2\n" onChange={onChange} onPromote={() => {}} />);
    expect(screen.getByTestId("table-panel")).toBeTruthy();
  });

  it("promotes with the path that was typed", () => {
    const onPromote = vi.fn();
    render(
      <FenceTable body={"a,b\n1,2\n"} onChange={() => {}} onPromote={onPromote} />,
    );
    fireEvent.change(screen.getByLabelText(/file to write the csv to/i), {
      target: { value: " data/sales.csv " },
    });
    fireEvent.click(screen.getByRole("button", { name: /make it a table/i }));
    expect(onPromote).toHaveBeenCalledWith("data/sales.csv");
  });

  it("promotes with no path at all, which is a table that writes nothing", () => {
    const onPromote = vi.fn();
    render(<FenceTable body={"a\n"} onChange={() => {}} onPromote={onPromote} />);
    fireEvent.click(screen.getByRole("button", { name: /make it a table/i }));
    expect(onPromote).toHaveBeenCalledWith("");
  });
});

</storycode:copy>
<a id="f-effdc999f8dc-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-effdc999f8dc-expression_statement-4">
// Guarantee: docs/guarantees/authoring/cell-formatting-keeps-numbers-numeric.md
it("promotes fence formats to an attribute, leaving its dataset numeric", () => {
  expect(tableElementFor("cost\n1234.50\n", "cost.csv", "A2:USD")).toBe(
    '<hick:table path="cost.csv" formats="A2:USD">\ncost\n1234.50\n</hick:table>',
  );
});
</storycode:copy>

### Assemble `apps/web/src/components/FenceTable.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/FenceTable.test.tsx"><storycode:paste select="#f-effdc999f8dc-expression_statement"/><storycode:paste select="#f-effdc999f8dc-expression_statement-2"/><storycode:paste select="#f-effdc999f8dc-expression_statement-3"/><storycode:paste select="#f-effdc999f8dc-expression_statement-4"/></storycode:file>

</storycode:doc>
