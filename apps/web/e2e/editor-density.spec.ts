import { expect, test, type Locator, type Page } from "@playwright/test";

// Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
// Real native engine and CSS geometry; use just test-editor-density-ui.
test.use({ viewport: { width: 1200, height: 900 } });
const center = (page: Page) => page.locator('[data-spatial-center=true]');
const font = (element: Locator) => element.evaluate(el => parseFloat(getComputedStyle(el).fontSize));
const height = (element: Locator) => element.evaluate(el => el.getBoundingClientRect().height);
async function showFiles(page: Page) {
  const files = page.locator('[data-spatial-files=left]');
  const handle = page.getByRole('button', { name: 'Show left drawer' });
  if (await handle.isVisible()) await handle.click();
  await expect(files).toBeVisible();
}
async function setting(page: Page, name: string) {
  await page.goto('/#/settings');
  return page.getByRole("checkbox", { name, exact: true });
}
async function note(page: Page, name: string) {
  const source = '# Dense note\n\n<hick:file path="sample.py">\n' + Array.from({ length: 100 }, (_, i) => `value_${i} = ${i}`).join('\n') + '\n</hick:file>\n<hick:container name="shell" image="alpine:3.20" />\n<hick:exec container="shell">true</hick:exec>\n';
  const response = await page.request.post('/api/projects/local/docs', { data: { path: name, source } });
  expect(response.ok()).toBe(true);
  const doc = await response.json();
  await page.goto(`/#/docs/${doc.id}`);
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name, exact: true }).click();
  await expect(center(page).locator('.cm-content')).toContainText('value_0');
  return { id: doc.id, source, name };
}
test.beforeEach(async ({ page }) => {
  test.skip(!process.env.HICKORY_SPATIAL_URL, 'Use just test-editor-density-ui.');
  await page.goto(process.env.HICKORY_SPATIAL_URL!);
  await expect(page.getByRole('tablist', { name: 'Center views' })).toBeVisible();
});

test('header contains actions and view information; ruler and action rail remain available', async ({ page }) => {
  const doc = await note(page, 'density-header.md');
  const header = center(page).locator('.spatial-view-header');
  await expect(header.getByRole('button', { name: 'More actions for this document' })).toBeVisible();
  await expect(center(page).locator('.debug-block > .surface-info')).toHaveCount(0);
  await expect(center(page).locator('.debug-block > .doc-tab-toolbar')).toHaveCount(0);
  await expect(center(page).getByRole('slider')).toBeVisible();
  await expect(center(page).locator('.cm-right-rail')).toBeVisible();
  await expect(center(page).locator('.cm-card-rail')).toBeVisible();
  await header.getByRole('button', { name: /About Document/ }).click();
  await expect(header.locator('.surface-info p')).toContainText('Edits save automatically');
  await header.getByRole('button', { name: /About Document/ }).click();
  await header.getByRole('button', { name: 'More actions for this document' }).click();
  await expect(page.getByRole('menuitem', { name: 'Run', exact: true })).toBeVisible();
  await expect(page.getByRole('menuitem', { name: 'Test', exact: true })).toBeVisible();
  await page.keyboard.press('Escape');
  const api = await page.request.get(`/api/docs/${doc.id}`);
  expect((await api.json()).source).toBe(doc.source);
  await page.screenshot({ path: '/tmp/hickory-compact-editor.png' });
});

test('compact spacing shows more rows without shrinking text; Files touch targets are optional', async ({ page }) => {
  const doc = await note(page, 'density-spacing.md');
  const row = center(page).locator('.cm-line').filter({ hasText: /^value_0 =/ }).first();
  const compact = { height: await height(row), font: await font(row) };
  const topPadding = await center(page).locator('.cm-content').evaluate(el => parseFloat(getComputedStyle(el).paddingTop));
  expect(topPadding).toBeLessThanOrEqual(8);
  await page.goto('/#/settings');
  await page.getByRole('button', { name: 'Comfortable', exact: true }).click();
  await page.goto(`/#/docs/${doc.id}`);
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: doc.name, exact: true }).click();
  await expect(row).toBeVisible();
  expect(await height(row)).toBeGreaterThan(compact.height);
  expect(await font(row)).toBe(compact.font);
  await page.goto('/#/settings');
  await page.getByRole('button', { name: 'Compact', exact: true }).click();
  await page.goto(`/#/docs/${doc.id}`);
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: doc.name, exact: true }).click();
  await showFiles(page);
  const file = page.locator('[data-spatial-files=left] .filesystem-editor .cm-line').first();
  expect(await height(file)).toBeLessThan(28);
  await (await setting(page, 'Large Files touch targets')).check();
  await page.goto(`/#/docs/${doc.id}`);
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: doc.name, exact: true }).click();
  await showFiles(page);
  expect(await height(file)).toBeGreaterThanOrEqual(36);
});

test('palette takes no resting space and Escape preserves text, selection and focus', async ({ page }) => {
  await note(page, 'density-palette.md');
  const content = center(page).locator('.cm-content');
  await content.click(); await content.press('ControlOrMeta+Home');
  await content.press('ArrowRight'); await content.press('Shift+ArrowRight');
  const selected = await page.evaluate(() => window.getSelection()?.toString());
  const before = await center(page).boundingBox();
  await expect(page.getByRole('search')).toHaveCount(0);
  await content.press('ControlOrMeta+p');
  const query = page.getByRole('textbox', { name: 'Search open files and folder' });
  await expect(query).toBeFocused();
  await query.press('Escape');
  await expect(page.getByRole('search')).toHaveCount(0);
  await expect(content).toBeFocused();
  expect(await page.evaluate(() => window.getSelection()?.toString())).toBe(selected);
  expect(await center(page).boundingBox()).toEqual(before);
  await center(page).getByRole('button', { name: 'Open search' }).click();
  await expect(query).toBeFocused();
  await query.fill('>Settings');
  await page.getByRole('option', { name: /Settings/ }).click();
  await expect(page.getByRole('group', { name: 'Editor density' })).toBeVisible();
});

test('tab zoom scales document and code text but leaves the header fixed; right numbers remember their toggle', async ({ page }) => {
  const doc = await note(page, 'density-zoom.md');
  const content = center(page).locator('.cm-content');
  const initial = await font(content), headerHeight = await height(center(page).locator('.spatial-view-header'));
  await content.click(); await content.press('ControlOrMeta+Alt+-');
  await expect.poll(() => font(content)).toBeLessThan(initial);
  expect(await height(center(page).locator('.spatial-view-header'))).toBe(headerHeight);
  await content.press('ControlOrMeta+Alt+0');
  await expect.poll(() => font(content)).toBe(initial);
  await (await setting(page, 'Right-hand line numbers')).uncheck();
  await page.goto(`/#/docs/${doc.id}`);
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: doc.name, exact: true }).click();
  await expect(center(page).locator('.cm-right-rail')).toBeHidden();
  await expect(center(page).locator('.cm-card-rail')).toBeVisible();
  await expect(center(page).getByRole('slider')).toBeVisible();
  await page.reload();
  await expect(center(page).locator('.cm-right-rail')).toBeHidden();
  expect((await (await page.request.get(`/api/docs/${doc.id}`)).json()).source).toBe(doc.source);
  // The same tab zoom applies to an ordinary code file, not only documents.
  const created = await page.request.post('/api/files/op', { data: { op: 'create', path: 'zoom-code.py' } });
  expect(created.ok()).toBe(true);
  const saved = await page.request.put('/api/file', { data: { path: 'zoom-code.py', content: 'print(42)\n', force: true } });
  expect(saved.ok()).toBe(true);
  await page.reload();
  await showFiles(page);
  await page.locator('[data-spatial-files=left] [data-tree-path="zoom-code.py"]').dblclick();
  await page.getByRole('tablist', { name: 'Center views' }).getByRole('tab', { name: 'zoom-code.py', exact: true }).click();
  const code = center(page).locator('.cm-content');
  await expect(code).toContainText('print(42)');
  await center(page).getByLabel('More actions for this file').click();
  await expect(center(page).getByRole('button', { name: /^Debug / })).toBeVisible();
  await center(page).getByLabel('More actions for this file').click();
  const original = await font(code);
  await code.click(); await code.press('ControlOrMeta+Alt+-');
  await expect.poll(() => font(code)).toBeLessThan(original);
});

test('an ordinary note reserves no empty action column and adding a cell restores its actions', async ({ page }) => {
  const response = await page.request.post('/api/projects/local/docs', { data: { path: 'empty-rail.md', source: '# Ordinary note\n' } });
  expect(response.ok()).toBe(true);
  const doc = await response.json();
  await page.goto(`/#/docs/${doc.id}`);
  await page.getByRole('tablist', { name: 'Center views' }).getByRole('tab', { name: 'empty-rail.md', exact: true }).click();
  const rail = center(page).locator('.cm-card-rail');
  await expect(rail).toHaveAttribute('data-empty', 'true');
  expect((await rail.boundingBox())?.width).toBe(0);
  await expect(center(page).locator('.cm-right-rail')).toBeVisible();
  const content = center(page).locator('.cm-content');
  await content.click(); await content.press('ControlOrMeta+End');
  await content.pressSequentially('\n<hick:container name="shell" image="alpine:3.20" />\n<hick:exec container="shell">true</hick:exec>');
  await expect(rail).toHaveAttribute('data-empty', 'false');
  expect((await rail.boundingBox())!.width).toBeGreaterThan(0);
  await expect(rail.getByRole('button', { name: /click to run it/ })).toBeVisible();
});
