import { expect, test, type Locator, type Page } from "@playwright/test";

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
// just test-spatial-ui supplies a native engine with note.md and notes/other.md.
test.use({ viewport: { width: 1200, height: 900 } });
test.beforeEach(async ({ page }) => {
  test.skip(!process.env.HICKORY_SPATIAL_URL, "Use just test-spatial-ui with a disposable workspace.");
  await page.goto(process.env.HICKORY_SPATIAL_URL!);
  await expect(page.getByRole("tablist", { name: "Center views" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Show left drawer" })).toBeVisible();
});
const center = (page: Page) => page.locator('[data-spatial-center=true]');
const tile = (page: Page, path: string) => page.locator('[data-spatial-key="document:' + path + '"]');
async function rect(element: Locator) {
  return element.evaluate(el => { const box = el.getBoundingClientRect(); return { x: box.x, y: box.y, width: box.width, height: box.height }; });
}
async function open(page: Page, path: string) {
  await page.getByRole("button", { name: "Show left drawer" }).click();
  await page.locator('[data-spatial-files=left] [data-tree-path="' + path + '"]').dblclick();
  await expect(tile(page, path)).toHaveAttribute("data-spatial-center", "true");
  await expect(page.getByRole("button", { name: "Show left drawer" })).toBeVisible();
  return tile(page, path);
}
async function remember(page: Page, selector: string) {
  await page.evaluate(async selector => {
    const module = "/node_modules/.vite/deps/@codemirror_view.js";
    const { EditorView } = await import(/* @vite-ignore */ module);
    const view = EditorView.findFromDOM(document.querySelector(selector + " .cm-editor"));
    const line = view.state.doc.line(Math.min(3, view.state.doc.lines));
    view.dispatch({ selection: { anchor: line.from, head: line.to }, scrollIntoView: true }); view.focus();
    (window as unknown as { drawerWitness: unknown }).drawerWitness = { view, text: view.state.doc.toString(), selection: view.state.selection.toJSON() };
  }, selector);
}
async function preserved(page: Page, selector: string) {
  return page.evaluate(async selector => {
    const module = "/node_modules/.vite/deps/@codemirror_view.js", { EditorView } = await import(/* @vite-ignore */ module);
    const witness = (window as unknown as { drawerWitness: { view: { state: { doc: { toString(): string }; selection: { toJSON(): unknown } } }; text: string; selection: unknown } }).drawerWitness;
    return EditorView.findFromDOM(document.querySelector(selector + " .cm-editor")) === witness.view && witness.view.state.doc.toString() === witness.text && JSON.stringify(witness.view.state.selection.toJSON()) === JSON.stringify(witness.selection);
  }, selector);
}

test("collapsed Files uses no space; overlay, pin, unpin and Escape preserve the native editor", async ({ page }) => {
  const shell = page.locator(".spatial-shell"), before = await rect(center(page));
  expect(before.width).toBe((await rect(shell)).width);
  await expect(page.locator('[data-spatial-files=left]')).toBeHidden();
  await expect(page.getByRole("button", { name: "Show right drawer" })).toHaveCount(0);
  await remember(page, "[data-spatial-center=true]");
  await page.screenshot({ path: "/tmp/hickory-drawers-collapsed.png", fullPage: true });
  await page.getByRole("button", { name: "Show left drawer" }).click();
  await expect(page.locator('[data-spatial-files=left]')).toBeVisible();
  expect(await rect(center(page))).toEqual(before);
  await page.screenshot({ path: "/tmp/hickory-drawers-overlay.png", fullPage: true });
  await page.getByRole("button", { name: "Pin left drawer" }).click();
  await expect.poll(async () => (await rect(center(page))).width).toBeLessThan(before.width);
  await page.getByRole("button", { name: "Unpin left drawer" }).click();
  await expect.poll(() => rect(center(page))).toEqual(before);
  await page.getByRole("button", { name: "Close left drawer" }).press("Escape");
  await expect(page.locator('[data-spatial-files=left]')).toBeHidden();
  expect(await preserved(page, "[data-spatial-center=true]")).toBe(true);
});

test("ordinary tabs stay out of context; a pinned view retains its native editor through drag, resize and Center", async ({ page }) => {
  const errors: string[] = []; page.on("pageerror", error => errors.push(error.message));
  const note = await open(page, "note.md");
  await open(page, "notes/other.md");
  await expect(note).toBeHidden();
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: "note.md", exact: true }).click();
  await remember(page, '[data-spatial-key="document:note.md"]');
  await center(page).getByRole("button", { name: "Pin view in drawer" }).click();
  await expect(note).toHaveAttribute("data-spatial-center", "false");
  await expect(note).toBeVisible();
  await expect.poll(async () => (await rect(note)).height).toBe(253);
  const initial = await rect(note);
  await page.mouse.move(initial.x + 20, initial.y + 18); await page.mouse.down(); await page.mouse.move(initial.x + 20, initial.y + 78, { steps: 8 });
  await expect(note).toHaveClass(/spatial-view--tentative/);
  const preview = await rect(note); await page.mouse.up();
  await expect.poll(() => rect(note)).toEqual(preview);
  for (const edge of ["top", "bottom"] as const) {
    const before = await rect(note), handle = page.locator('[data-spatial-resize="document:note.md"][data-resize-edge="' + edge + '"]');
    await expect(handle).toBeVisible();
    await handle.press(edge === "top" ? "ArrowUp" : "ArrowDown");
    await expect.poll(async () => (await rect(note)).height).toBe(before.height + 10);
  }
  await note.getByRole("button", { name: "Center", exact: true }).click();
  await expect(note).toHaveAttribute("data-spatial-center", "true");
  await note.getByRole("button", { name: "Restore previous center", exact: true }).click();
  await expect(note).toHaveAttribute("data-spatial-center", "false");
  expect(await preserved(page, '[data-spatial-key="document:note.md"]')).toBe(true);
  expect(errors).toEqual([]);
});

test("Files editing survives drawer dismissal and drawer pins survive reload", async ({ page }) => {
  await page.getByRole("button", { name: "Show left drawer" }).click();
  const files = page.locator('[data-spatial-files=left] .filesystem-editor .cm-content');
  await files.click(); await files.press("ControlOrMeta+End"); await files.press("End"); await files.pressSequentially("-tentative");
  await expect(page.getByRole("toolbar", { name: "Unsaved Files changes" })).toBeVisible();
  await page.getByRole("button", { name: "Close left drawer" }).click();
  await page.getByRole("button", { name: "Show left drawer" }).click();
  await expect(page.getByRole("toolbar", { name: "Unsaved Files changes" })).toBeVisible();
  await files.press("ControlOrMeta+z");
  await page.getByRole("button", { name: "Pin left drawer" }).click();
  await page.reload();
  await expect(page.getByRole("button", { name: "Unpin left drawer" })).toBeVisible();
  const root = await rect(page.locator(".spatial-shell"));
  expect((await rect(center(page))).width).toBeLessThan(root.width);
  await page.screenshot({ path: "/tmp/hickory-drawers-pinned.png", fullPage: true });
});

test("overlays leave center tabs usable and reclaim their space at narrow widths", async ({ page }) => {
  await page.setViewportSize({ width: 500, height: 800 });
  const shell = page.locator(".spatial-shell");
  await expect.poll(async () => (await rect(center(page))).width).toBe((await rect(shell)).width);
  await page.getByRole("button", { name: "Show left drawer" }).click();
  await expect(page.getByRole("tablist", { name: "Center views" })).toBeVisible();
  const tabs = await rect(page.getByRole("tablist", { name: "Center views" })), drawer = await rect(page.locator('[data-drawer-toolbar=left]'));
  expect(drawer.y).toBeGreaterThanOrEqual(tabs.y + tabs.height);
  await page.getByRole("button", { name: "Close left drawer" }).click();
  await expect.poll(async () => (await rect(center(page))).width).toBe((await rect(shell)).width);
  expect(await page.evaluate(() => document.documentElement.scrollHeight > window.innerHeight + 1)).toBe(false);
});

test("tab X closes an inactive tab and dragging between drawers and center preserves the editor", async ({ page }) => {
  const note = await open(page, "note.md");
  await open(page, "notes/other.md");
  const tabs = page.getByRole("tablist", { name: "Center views" });
  await tabs.getByRole("button", { name: "Close note.md", exact: true }).click();
  await expect(note).toHaveCount(0);
  await open(page, "note.md");
  await remember(page, '[data-spatial-key="document:note.md"]');
  await center(page).getByRole("button", { name: "Pin view in drawer" }).click();
  await expect(note).toBeVisible();
  const before = await rect(note), destination = await rect(center(page));
  await page.mouse.move(before.x + 30, before.y + 18); await page.mouse.down();
  await page.mouse.move(destination.x + destination.width / 2, destination.y + destination.height / 2, { steps: 10 });
  await expect(page.locator('[data-spatial-drop-preview]')).toBeVisible();
  await page.mouse.up();
  await expect(note).toHaveAttribute("data-spatial-center", "true");
  expect(await preserved(page, '[data-spatial-key="document:note.md"]')).toBe(true);
});

test("dragging a tab to a center edge creates a tile and closing reclaims space", async ({ page }) => {
  const note = await open(page, "note.md");
  const other = await open(page, "notes/other.md");
  await remember(page, '[data-spatial-key="document:note.md"]');
  const tabs = page.getByRole("tablist", { name: "Center views" });
  const label = tabs.getByRole("tab", { name: "note.md", exact: true });
  const from = await rect(label), target = await rect(other);
  await page.mouse.move(from.x + from.width / 2, from.y + from.height / 2); await page.mouse.down();
  await page.mouse.move(target.x + target.width - 12, target.y + target.height / 2, { steps: 10 });
  await expect(page.locator('[data-spatial-drop-preview]')).toHaveText("Split right");
  await page.mouse.up();
  await expect(note).toHaveAttribute("data-spatial-tiled", "true");
  await expect.poll(async () => (await rect(note)).x).toBeGreaterThan((await rect(other)).x);
  expect(await preserved(page, '[data-spatial-key="document:note.md"]')).toBe(true);
  const noteBox = await rect(note), otherBox = await rect(other);
  expect(noteBox.width + otherBox.width + 6).toBeCloseTo(target.width, 0);
  await page.screenshot({ path: "/tmp/hickory-center-tiles.png", fullPage: true });
  await note.getByRole("button", { name: "Pin view", exact: true }).click();
  await expect(note).toHaveAttribute("data-spatial-tiled", "false");
  await expect(note).toBeVisible();
  expect(await preserved(page, '[data-spatial-key="document:note.md"]')).toBe(true);
  await tabs.getByRole("button", { name: "Close note.md", exact: true }).click();
  await page.getByRole("button", { name: "Unpin left drawer" }).click();
  await page.getByRole("button", { name: "Close left drawer" }).click();
  await expect.poll(async () => (await rect(other)).width).toBeCloseTo(target.width, 0);
});
