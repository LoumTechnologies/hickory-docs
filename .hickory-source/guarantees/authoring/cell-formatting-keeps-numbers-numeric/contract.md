<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Cell formatting keeps numbers numeric

For: Authors editing executable documents and the files those documents produce.

Given a Markdown pipe table, a `<hick:table>`, or a CSV/TSV fence edited as a
Hickory grid, selecting cells offers General, Number, Percent, and money formats
for USD, EUR, GBP and JPY. Formatting changes the display of numeric cells and
computed formula results. Editing a cell and the formula bar show its underlying
value or expression. Formatting never rounds or replaces the stored number.
Text, blanks, unevaluated formulas and errors remain readable as themselves.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/cell-formatting-keeps-numbers-numeric.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-ccf2c8b200ba"></a>
## `apps/web/e2e/tableFormatting.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Cell formatting keeps numbers numeric](contract.md).

<a id="f-ccf2c8b200ba-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ccf2c8b200ba-expression_statement">
// Guarantee: docs/guarantees/authoring/cell-formatting-keeps-numbers-numeric.md
import { expect, test } from "@playwright/test";

test("money formatting and currency entry preserve precision in the full editor", async ({ page }) => {
  test.skip(!process.env.HICKORY_E2E_URL, "Run against the full editor (VITE_MOCK=1 is sufficient).");
  await page.goto('/');
  const pane = page.locator('.untitled-tab');
  const editor = pane.locator('.cm-content');
  await expect(editor).toBeVisible();
  await editor.click();
  await page.keyboard.press('ControlOrMeta+a');
  await page.keyboard.insertText('| Cost |\n| ---: |\n| 1234.567 |');
  await pane.getByRole('button', { name: /Table 1.*click to render/ }).click();
  await pane.getByText('1234.567', { exact: true }).click();
  await pane.getByLabel('Cell number format').selectOption('USD');
  await expect(pane.getByText('$1,234.57', { exact: true })).toBeVisible();
  await pane.getByText('$1,234.57', { exact: true }).dblclick();
  const input = pane.locator('.table-panel__input');
  await expect(input).toHaveValue('1234.567');
  await input.fill('$2,345.678');
  await input.press('Enter');
  await expect(pane.getByText('$2,345.68', { exact: true })).toBeVisible();
  await pane.getByText('$2,345.68', { exact: true }).click();
  await expect(pane.locator('.table-panel__formula')).toHaveValue('2345.678');
});

</storycode:copy>
<a id="f-ccf2c8b200ba-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ccf2c8b200ba-expression_statement-2">
test("CSV fence formatting survives promotion without changing the CSV value", async ({ page }) => {
  test.skip(!process.env.HICKORY_E2E_URL, "Run against the full editor.");
  await page.goto('/');
  const pane = page.locator('.untitled-tab');
  const editor = pane.locator('.cm-content');
  await editor.click();
  await page.keyboard.press('ControlOrMeta+a');
  await page.keyboard.insertText('```csv\nCost\n1234.50\n```');
  await pane.getByRole('button', { name: /Code fence \(csv\)/ }).click();
  const table = page.getByTestId('table-panel');
  await table.getByText('1234.50', { exact: true }).click();
  await table.getByLabel('Cell number format').selectOption('USD');
  await expect(table.getByText('$1,234.50', { exact: true })).toBeVisible();
  await expect(editor).toContainText('csv formats="A2:USD"');
  await page.getByRole('button', { name: 'Make it a table', exact: true }).click();
  await expect(editor).toContainText('<hick:table formats="A2:USD">');
  await expect(editor).toContainText('1234.50');
});
</storycode:copy>

### Assemble `apps/web/e2e/tableFormatting.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/tableFormatting.spec.ts"><storycode:paste select="#f-ccf2c8b200ba-expression_statement"/><storycode:paste select="#f-ccf2c8b200ba-expression_statement-2"/></storycode:file>

<a id="output-b5ed71b20ef0"></a>
## `apps/web/src/components/TableFormatting.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Cell formatting keeps numbers numeric](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/components/TableFormatting.test.tsx">
// Guarantee: docs/guarantees/authoring/cell-formatting-keeps-numbers-numeric.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { TablePanel } from "./TablePanel";
import { MarkdownTable } from "./MarkdownTable";
import { api } from "../api/client";

afterEach(() => { cleanup(); vi.restoreAllMocks(); });

it.each(["python", "javascript"])("pastes money as numeric input to %s formulas and formats their result", async language => {
  const evaluate = vi.spyOn(api, "evaluateFormulas").mockResolvedValue({ values: { B3: "2469" }, errors: {} });
  const save = vi.fn();
  function Grid() {
    const [source, setSource] = useState("name,amount\na,0\ntotal,=B2 * 2\n");
    const [formats, setFormats] = useState("B3:USD");
    return <TablePanel source={source} formats={formats} language={language}
      onChange={(next, fmt = "") => { save(next, fmt); setSource(next); setFormats(fmt); }} />;
  }
  render(<Grid />);
  fireEvent.click(screen.getByText("0"));
  fireEvent.paste(screen.getByTestId("table-panel"), { clipboardData: { getData: (type: string) => type === "text/plain" ? "$1,234.50" : "" } });
  expect(save).toHaveBeenLastCalledWith("name,amount\na,1234.50\ntotal,=B2 * 2\n", "B2:USD B3:USD");
  expect(screen.getByText("$1,234.50")).toBeTruthy();
  await waitFor(() => expect(evaluate).toHaveBeenCalledWith(language, [["name", "amount"], ["a", "1234.50"], ["total", "=B2 * 2"]]));
  await waitFor(() => expect(screen.getByText("$2,469.00")).toBeTruthy());
  fireEvent.doubleClick(screen.getByText("$1,234.50"));
  expect((screen.getByRole("textbox", { name: "Cell contents" }) as HTMLInputElement).value).toBe("1234.50");
});

it("saves Markdown formats adjacent to numeric cells and reloads them", () => {
  const save = vi.fn();
  const { rerender } = render(<MarkdownTable source={"| Cost |\n| ---: |\n| 1234.567 |"} onChange={save} />);
  fireEvent.click(screen.getByText("1234.567"));
  fireEvent.change(screen.getByLabelText("Cell number format"), { target: { value: "USD" } });
  const source = '<!-- hickory-table-formats: A2:USD -->\n| Cost |\n| ---: |\n| 1234.567 |';
  expect(save).toHaveBeenLastCalledWith(source);
  rerender(<MarkdownTable source={source} onChange={save} />);
  expect(screen.getByText("$1,234.57")).toBeTruthy();
  fireEvent.change(screen.getByLabelText("Cell number format"), { target: { value: "general" } });
  expect(save.mock.lastCall?.[0]).toContain("A2:general");
  expect(save.mock.lastCall?.[0]).toContain("1234.567");
});
</storycode:entry>

<a id="output-d48a9b43e133"></a>
## `apps/web/src/editor/tableEdit.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Cell formatting keeps numbers numeric](contract.md).

<a id="f-d48a9b43e133-replaceTable"></a>
### `replaceTable`

<storycode:copy id="f-d48a9b43e133-replaceTable">
import type { RenderedSlot } from "./rendered";
import type { SlotContext } from "../elements/types";
import { structureOf } from "./wysiwyg";

/** Commit data and presentation together, so undo cannot separate them. */
export function replaceTable(slot: RenderedSlot, cx: SlotContext, csv: string, formats: string, agent = false): void {
  const view = cx.view;
  if (!view) return;
  const block = structureOf(view.state).blocks.find(b => b.from === slot.at && b.name === "table");
  if (!block) return;
  const opening = view.state.doc.sliceString(block.from, block.contentFrom);
  const stripped = opening.replace(/\s+formats=(?:"[^"]*"|'[^']*')/, "");
  const end = stripped.lastIndexOf(">");
  const next = formats ? `${stripped.slice(0, end)} formats="${formats}"${stripped.slice(end)}` : stripped;
  const prefix = view.state.doc.sliceString(block.contentFrom, block.contentTo).match(/^\r?\n/)?.[0] ?? "";
  const body = prefix + (csv.endsWith("\n") ? csv : `${csv}\n`);
  view.dispatch({ changes: [
    { from: block.from, to: block.contentFrom, insert: next },
    { from: block.contentFrom, to: block.contentTo, insert: body },
  ], userEvent: agent ? "input.uiAgentTable" : "input.table" });
}
</storycode:copy>

### Assemble `apps/web/src/editor/tableEdit.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/tableEdit.ts"><storycode:paste select="#f-d48a9b43e133-replaceTable"/></storycode:file>

<a id="output-3f26edc92ce9"></a>
## `apps/web/src/editor/tableFormatting.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Cell formatting keeps numbers numeric](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/editor/tableFormatting.test.tsx">
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
</storycode:entry>

<a id="output-ae3b10a489c4"></a>
## `apps/web/src/lib/tableFormats.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Cell formatting keeps numbers numeric](contract.md).

<a id="f-ae3b10a489c4-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ae3b10a489c4-expression_statement">
// Guarantee: docs/guarantees/authoring/cell-formatting-keeps-numbers-numeric.md
import { describe, expect, it } from "vitest";
import { currencyInput, displayNumber, readFormats, writeFormats } from "./tableFormats";
import { insertColumn, insertRow, parseCsv, removeColumns, removeRows, resizeTable } from "./csv";

describe("currency input", () => {
  it.each([
    ["$1,234.50", "1234.50", "USD"], ["-$12.00", "-12.00", "USD"],
    ["($1,234.50)", "-1234.50", "USD"], ["€25.99", "25.99", "EUR"],
    ["25.99 €", "25.99", "EUR"], ["£-2.50", "-2.50", "GBP"], ["¥300", "300", "JPY"],
  ])("reads %s without storing its symbol", (raw, value, format) => {
    expect(currencyInput(raw)).toEqual({ value, format });
  });
  it.each(["$12,34", "€12,50", "$1,23,456", "cost $5", "=A1 + 2", "", "$-+2", "(-$2)"])("leaves ambiguous or nonnumeric %s alone", raw => {
    expect(currencyInput(raw)).toBeNull();
  });
});

</storycode:copy>
<a id="f-ae3b10a489c4-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ae3b10a489c4-expression_statement-2">
it("formats numbers without rounding the stored value or inventing a numeric value for text", () => {
  expect(displayNumber("1234.567", "USD")).toBe("$1,234.57");
  expect(displayNumber("0.125", "percent")).toBe("12.5%");
  expect(displayNumber("1234.5", "number")).toBe("1,234.50");
  expect(displayNumber("", "USD")).toBe("");
  expect(displayNumber("=A1", "USD")).toBe("=A1");
  expect(displayNumber("label", "USD")).toBe("label");
});

</storycode:copy>
<a id="f-ae3b10a489c4-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-ae3b10a489c4-expression_statement-3">
it("round-trips per-cell formats, including explicit General, through structural edits", () => {
  const table = { ...parseCsv("name,amount\na,1234.50\nb,5\n"), formats: readFormats("B2:USD B3:general") };
  expect(writeFormats(insertColumn(insertRow(table, 1), 0).formats)).toBe("C3:USD C4:general");
  expect(writeFormats(removeRows(table, [1]).formats)).toBe("B2:general");
  expect(writeFormats(removeColumns(table, [1]).formats)).toBe("");
  expect(writeFormats(resizeTable(table, 2, 2).formats)).toBe("B2:USD");
  expect(readFormats("B2:USD garbage Z0:USD C4:unknown")).toEqual({ B2: "USD" });
});
</storycode:copy>

### Assemble `apps/web/src/lib/tableFormats.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tableFormats.test.ts"><storycode:paste select="#f-ae3b10a489c4-expression_statement"/><storycode:paste select="#f-ae3b10a489c4-expression_statement-2"/><storycode:paste select="#f-ae3b10a489c4-expression_statement-3"/></storycode:file>

<a id="output-87d8a5e26ee3"></a>
## `apps/web/src/lib/tableFormats.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Cell formatting keeps numbers numeric](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/lib/tableFormats.ts">
import { cellLabel, parseCellLabel } from "./cellRef";

export const NUMBER_FORMATS = {
  general: "General", number: "Number (1,234.50)", percent: "Percent",
  USD: "Money ($)", EUR: "Money (€)", GBP: "Money (£)", JPY: "Money (¥)",
} as const;
export type NumberFormat = keyof typeof NUMBER_FORMATS;
export type CellFormats = Record<string, NumberFormat>;
const CURRENCY_SYMBOLS = { "$": "USD", "€": "EUR", "£": "GBP", "¥": "JPY" } as const;

/** Compact, diffable metadata: B2:USD B3:percent. No quotes or XML escaping. */
export function readFormats(text = ""): CellFormats {
  const formats: CellFormats = {};
  for (const entry of text.split(/\s+/)) {
    const [label, format] = entry.split(":");
    if (parseCellLabel(label ?? "") && Object.hasOwn(NUMBER_FORMATS, format)) {
      formats[label] = format as NumberFormat;
    }
  }
  return formats;
}
export function writeFormats(formats: CellFormats = {}): string {
  return Object.entries(formats).sort(([a], [b]) => a.localeCompare(b))
    .map(([label, format]) => `${label}:${format}`).join(" ");
}

/** Explicit symbols only. Ambiguous decimal-comma currency remains text. */
export function currencyInput(raw: string): { value: string; format: NumberFormat } | null {
  let text = raw.trim();
  const accounting = text.startsWith("(") && text.endsWith(")");
  if (accounting) text = text.slice(1, -1).trim();
  const number = "(\\d+(?:\\.\\d+)?|\\d{1,3}(?:,\\d{3})+(?:\\.\\d+)?)";
  const prefix = new RegExp(`^([+-]?)\\s*([$€£¥])\\s*([+-]?)\\s*${number}$`).exec(text);
  const suffix = new RegExp(`^([+-]?)\\s*${number}\\s*([$€£¥])$`).exec(text);
  const match = prefix ?? (suffix ? [suffix[0], suffix[1], suffix[3], "", suffix[2]] : null);
  if (!match) return null;
  const symbol = match[2];
  const digits = match[4];
  if (!digits || (match[1] && match[3]) || (accounting && (match[1] || match[3]))) return null;
  const value = `${accounting ? "-" : match[1] || match[3]}${digits.replace(/,/g, "")}`;
  if (!Number.isFinite(Number(value))) return null;
  return { value, format: CURRENCY_SYMBOLS[symbol as keyof typeof CURRENCY_SYMBOLS] };
}

export function displayNumber(raw: string, format?: NumberFormat): string {
  if (!format || format === "general" || !/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?$/i.test(raw.trim())) return raw;
  const value = Number(raw);
  if (!Number.isFinite(value)) return raw;
  const options: Intl.NumberFormatOptions = format === "number"
    ? { minimumFractionDigits: 2, maximumFractionDigits: 2 }
    : format === "percent" ? { style: "percent", maximumFractionDigits: 2 }
    : { style: "currency", currency: format };
  return new Intl.NumberFormat("en-US", options).format(value);
}

/** Formats follow cells through structural edits; removed cells lose theirs. */
export function moveFormats(formats: CellFormats | undefined, move: (row: number, column: number) => { row: number; column: number } | null): CellFormats {
  const next: CellFormats = {};
  for (const [label, format] of Object.entries(formats ?? {})) {
    const at = parseCellLabel(label);
    const target = at && move(at.row, at.column);
    if (target) next[cellLabel(target.column, target.row)] = format;
  }
  return next;
}

export function splitFormatComment(source: string): { body: string; formats: string } {
  const match = /^(?:<!-- hickory-table-formats: ([^\r\n]*?) -->\r?\n)/.exec(source);
  return match ? { body: source.slice(match[0].length), formats: match[1] } : { body: source, formats: "" };
}
export function withFormatComment(body: string, formats: string): string {
  return formats ? `<!-- hickory-table-formats: ${formats} -->${body.includes("\r\n") ? "\r\n" : "\n"}${body}` : body;
}
</storycode:entry>

## Test the behavior: `monetary_numbers_are_numeric_in_python_and_javascript`

This is the maintained test fragment from `crates/hick-formula/tests/backends.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-6d63eb1fec85-monetary_numbers_are_numeric_in_python_and_javascript"></a>
### `monetary_numbers_are_numeric_in_python_and_javascript`

<storycode:copy id="f-6d63eb1fec85-monetary_numbers_are_numeric_in_python_and_javascript">
// Guarantee: docs/guarantees/authoring/cell-formatting-keeps-numbers-numeric.md
#[tokio::test(flavor = "multi_thread")]
async fn monetary_numbers_are_numeric_in_python_and_javascript() {
    // The editor saves $1,234.50 as 1234.50 with a separate B2:USD format.
    let entries = [("B2", "1234.50"), ("B3", "=B2 * 2"), ("B4", "=B3 + 0.25")];
    for language in ["python", "javascript"] {
        let interpreters = if language == "python" {
            &["python3", "python"][..]
        } else {
            &["node"][..]
        };
        if !have(interpreters) {
            eprintln!("skipped: currency verification needs {language}");
            continue;
        }
        let dir = tempfile::tempdir().unwrap();
        let computed = evaluate_sheet(dir.path(), language, &sheet(&entries))
            .await
            .unwrap();
        assert_eq!(at(&computed, "B3").as_deref(), Some("2469"), "{language}");
        assert_eq!(
            at(&computed, "B4").as_deref(),
            Some("2469.25"),
            "{language}"
        );
        assert!(computed.errors.is_empty());
    }
}
</storycode:copy>

## Test the behavior: `number_formats_stay_in_the_document_and_out_of_the_csv_dataset`

This is the maintained test fragment from `crates/hickory-cli/tests/tables.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e0de523c23a6-number_formats_stay_in_the_document_and_out_of_the_csv_dataset"></a>
### `number_formats_stay_in_the_document_and_out_of_the_csv_dataset`

<storycode:copy id="f-e0de523c23a6-number_formats_stay_in_the_document_and_out_of_the_csv_dataset">
// Guarantee: docs/guarantees/authoring/cell-formatting-keeps-numbers-numeric.md
#[test]
fn number_formats_stay_in_the_document_and_out_of_the_csv_dataset() {
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("cost.md");
    let source = "<hick:table path=\"cost.csv\" formats=\"B2:USD\">\nitem,amount\nnorth,1234.50\n</hick:table>\n";
    std::fs::write(&doc, source).unwrap();
    // Exercise this build directly; a running desktop engine owns a different
    // executable and must not handle a test's temporary output directory.
    let out = hick()
        .env("HICKORY_ENGINE_WORKER", "1")
        .arg("weave")
        .arg(&doc)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(std::fs::read_to_string(&doc).unwrap(), source);
    let csv = std::fs::read_to_string(dir.path().join("cost.csv")).unwrap();
    assert_eq!(csv, "item,amount\nnorth,1234.50\n");
}
</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/MarkdownTable.tsx](../../../mechanisms/apps/web/src/components/MarkdownTable.tsx.md#output-04adf2e0e8b7)
- [apps/web/src/components/TableFormatControl.tsx](../../../mechanisms/apps/web/src/components/TableFormatControl.tsx.md#output-2faa595b6a67)
- [apps/web/src/components/TablePanel.tsx](../../../mechanisms/apps/web/src/components/TablePanel.tsx.md#output-38d5395d7736)
- [apps/web/src/editor/markdownTables.ts](../../../mechanisms/apps/web/src/editor/markdownTables.ts.md#output-6f87bd812952)
- [apps/web/src/editor/rendered.ts](../../../mechanisms/apps/web/src/editor/rendered.ts.md#output-4f5444d21411)
- [apps/web/src/editor/tableEdit.ts](contract.md#output-d48a9b43e133)
- [apps/web/src/elements/table/view.tsx](../../../mechanisms/apps/web/src/elements/table/view.tsx.md#output-2971b5282a4d)
- [apps/web/src/lib/csv.ts](../a-table-is-a-dataset-and-a-paragraph/contract.md#output-4460b44bfc5a)
- [apps/web/src/lib/tableFormats.ts](contract.md#output-87d8a5e26ee3)
- [crates/hick-blocks/src/lib.rs](../../../mechanisms/crates/hick-blocks/src/lib.rs.md#output-5260198b0a7b)
- [crates/hick-literate/src/csv_table.rs](../../../mechanisms/crates/hick-literate/src/csv_table.rs.md#output-99033ffa7355)

### Tests and fixtures

- [apps/web/e2e/tableFormatting.spec.ts](contract.md#output-ccf2c8b200ba)
- [apps/web/src/components/FenceTable.test.tsx](../../../mechanisms/apps/web/src/components/FenceTable.test.tsx.md#output-effdc999f8dc)
- [apps/web/src/components/TableFormatting.test.tsx](contract.md#output-b5ed71b20ef0)
- [apps/web/src/components/TablePanel.test.tsx](../../../mechanisms/apps/web/src/components/TablePanel.test.tsx.md#output-694220ace684)
- [apps/web/src/editor/tableFormatting.test.tsx](contract.md#output-3f26edc92ce9)
- [apps/web/src/lib/tableClipboard.test.ts](../../../mechanisms/apps/web/src/lib/tableClipboard.test.ts.md#output-65703cf530ea)
- [apps/web/src/lib/tableFormats.test.ts](contract.md#output-ae3b10a489c4)
- [crates/hick-formula/tests/backends.rs](../../../mechanisms/crates/hick-formula/tests/backends.rs.md#output-6d63eb1fec85)
- [crates/hickory-cli/tests/tables.rs](../../../mechanisms/crates/hickory-cli/tests/tables.rs.md#output-e0de523c23a6)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
