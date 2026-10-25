<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/e2e/tableFormatting.spec.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Cell formatting keeps numbers numeric](../../../../contract.md)

Captured SHA-256: `981d19a42545af9bef6dc5555c0cebde4f9260febe6001e66133d22e7a5fe254`.

<book:entry path="apps/web/e2e/tableFormatting.spec.ts">
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
</book:entry>

</book:doc>
