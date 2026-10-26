<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/e2e/spatialWorkspace.spec.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2b83f22a0d20"></a>
## `apps/web/e2e/spatialWorkspace.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Spatial Views Preserve The Editor](../../../../contract.md).

<a id="f-2b83f22a0d20-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-2b83f22a0d20-expression_statement">
import { expect, test, type Locator, type Page } from "@playwright/test";

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
// just test-spatial-ui supplies a native engine with note.md and notes/other.md.
test.use({ viewport: { width: 1200, height: 900 } });
</storycode:copy>
<a id="f-2b83f22a0d20-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-2b83f22a0d20-expression_statement-2">
test.beforeEach(async ({ page }) => {
  test.skip(!process.env.HICKORY_SPATIAL_URL, "Use just test-spatial-ui with a disposable workspace.");
  await page.goto(process.env.HICKORY_SPATIAL_URL!);
  await expect(page.getByRole("tablist", { name: "Center views" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Show left drawer" })).toBeVisible();
});
</storycode:copy>
<a id="f-2b83f22a0d20-center"></a>
### `center`

<storycode:copy id="f-2b83f22a0d20-center">
const center = (page: Page) => page.locator('[data-spatial-center=true]');
</storycode:copy>
<a id="f-2b83f22a0d20-tile"></a>
### `tile`

<storycode:copy id="f-2b83f22a0d20-tile">
const tile = (page: Page, path: string) => page.locator('[data-spatial-key="document:' + path + '"]');
</storycode:copy>
<a id="f-2b83f22a0d20-rect"></a>
### `rect`

<storycode:copy id="f-2b83f22a0d20-rect">
async function rect(element: Locator) {
  return element.evaluate(el => { const box = el.getBoundingClientRect(); return { x: box.x, y: box.y, width: box.width, height: box.height }; });
}
</storycode:copy>
<a id="f-2b83f22a0d20-open"></a>
### `open`

<storycode:copy id="f-2b83f22a0d20-open">
async function open(page: Page, path: string) {
  await page.getByRole("button", { name: "Show left drawer" }).click();
  await page.locator('[data-spatial-files=left] [data-tree-path="' + path + '"]').dblclick();
  await expect(tile(page, path)).toHaveAttribute("data-spatial-center", "true");
  await expect(page.getByRole("button", { name: "Show left drawer" })).toBeVisible();
  return tile(page, path);
}
</storycode:copy>
<a id="f-2b83f22a0d20-remember"></a>
### `remember`

<storycode:copy id="f-2b83f22a0d20-remember">
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
</storycode:copy>
<a id="f-2b83f22a0d20-preserved"></a>
### `preserved`

<storycode:copy id="f-2b83f22a0d20-preserved">
async function preserved(page: Page, selector: string) {
  return page.evaluate(async selector => {
    const module = "/node_modules/.vite/deps/@codemirror_view.js", { EditorView } = await import(/* @vite-ignore */ module);
    const witness = (window as unknown as { drawerWitness: { view: { state: { doc: { toString(): string }; selection: { toJSON(): unknown } } }; text: string; selection: unknown } }).drawerWitness;
    return EditorView.findFromDOM(document.querySelector(selector + " .cm-editor")) === witness.view && witness.view.state.doc.toString() === witness.text && JSON.stringify(witness.view.state.selection.toJSON()) === JSON.stringify(witness.selection);
  }, selector);
}

</storycode:copy>
<a id="f-2b83f22a0d20-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-2b83f22a0d20-expression_statement-3">
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

</storycode:copy>
<a id="f-2b83f22a0d20-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-2b83f22a0d20-expression_statement-4">
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

</storycode:copy>
<a id="f-2b83f22a0d20-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-2b83f22a0d20-expression_statement-5">
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

</storycode:copy>
<a id="f-2b83f22a0d20-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-2b83f22a0d20-expression_statement-6">
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

</storycode:copy>
<a id="f-2b83f22a0d20-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-2b83f22a0d20-expression_statement-7">
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

</storycode:copy>
<a id="f-2b83f22a0d20-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-2b83f22a0d20-expression_statement-8">
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

</storycode:copy>
<a id="f-2b83f22a0d20-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-2b83f22a0d20-expression_statement-9">
test("Files text rows own preview names and dragging the Files surface carries its pinned previews", async ({ page }) => {
  const errors: string[] = []; page.on("pageerror", error => errors.push(error.message));
  const note = await open(page, "note.md");
  await open(page, "notes/other.md");
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: "note.md", exact: true }).click();
  await remember(page, '[data-spatial-key="document:note.md"]');
  await center(page).getByRole("button", { name: "Pin view in drawer" }).click();
  const files = page.locator('[data-spatial-files=left]');
  const row = files.locator('[data-tree-path="note.md"]');
  await expect(row).toHaveAttribute("data-preview-pinned", "true");
  await expect(row).toHaveClass(/spatial-file-row--pinned/);
  await expect(page.locator('[data-spatial-strip="document:note.md"]')).toBeHidden();
  await expect.poll(async () => Math.abs((await rect(note)).y - (await rect(row)).y)).toBeLessThan(1);
  const textBefore = await files.locator(".cm-content").textContent();
  const toolbar = page.locator('[data-drawer-toolbar=left]'), before = await rect(toolbar), previewBefore = await rect(note);
  await page.mouse.move(before.x + 50, before.y + 18); await page.mouse.down();
  await page.mouse.move(before.x + 120, before.y + 63, { steps: 10 }); await page.mouse.up();
  await expect.poll(async () => (await rect(toolbar)).x).toBeCloseTo(before.x + 70, 0);
  await expect.poll(async () => (await rect(toolbar)).y).toBeCloseTo(before.y + 45, 0);
  await expect.poll(async () => (await rect(note)).x).toBeCloseTo(previewBefore.x + 70, 0);
  await expect.poll(async () => Math.abs((await rect(note)).y - (await rect(row)).y)).toBeLessThan(1);
  expect(await files.locator(".cm-content").textContent()).toBe(textBefore);
  const grip = files.locator('[data-spatial-drag="document:note.md"]');
  const start = await rect(grip), target = await rect(row);
  await page.mouse.move(start.x + start.width / 2, start.y + start.height / 2); await page.mouse.down();
  await page.mouse.move(start.x + start.width / 2, target.y + 45, { steps: 8 });
  await expect(note).toHaveClass(/spatial-view--tentative/);
  await page.mouse.up();
  await expect.poll(async () => (await rect(note)).y).toBeGreaterThan(target.y + 25);
  await expect.poll(async () => Math.abs((await rect(note)).y - (await rect(row)).y)).toBeLessThan(1);
  expect(await preserved(page, '[data-spatial-key="document:note.md"]')).toBe(true);
  expect(await files.locator(".cm-content").textContent()).toBe(textBefore);
  await row.click();
  const content = files.locator(".cm-content");
  await content.press("End"); await content.pressSequentially("-draft");
  await expect(row).toHaveText("note.md-draft");
  await expect(page.getByRole("toolbar", { name: "Unsaved Files changes" })).toBeVisible();
  await content.press("ControlOrMeta+z");
  await expect(page.getByRole("toolbar", { name: "Unsaved Files changes" })).toBeHidden();
  expect(await content.textContent()).toBe(textBefore);
  await page.screenshot({ path: "/tmp/hickory-files-preview-rows.png", fullPage: true });
  await page.reload();
  await expect.poll(async () => (await rect(toolbar)).x).toBeCloseTo(before.x + 70, 0);
  expect(errors).toEqual([]);
});

</storycode:copy>
<a id="f-2b83f22a0d20-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-2b83f22a0d20-expression_statement-10">
// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
test("scrolling an earlier Files preview around a pin never moves the pin", async ({ page }) => {
  const note = await open(page, "note.md");
  const other = await open(page, "notes/other.md");
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: "note.md", exact: true }).click();
  await center(page).getByRole("button", { name: "Pin view in drawer" }).click();
  const row = page.locator('[data-spatial-files=left] [data-tree-path="note.md"]');
  await expect.poll(async () => Math.abs((await rect(note)).y - (await rect(row)).y)).toBeLessThan(1);
  const pinned = await rect(note);
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: "Untitled", exact: true }).click();
  await page.evaluate(async () => {
    const module = "/node_modules/.vite/deps/@codemirror_view.js", { EditorView } = await import(/* @vite-ignore */ module);
    const view = EditorView.findFromDOM(document.querySelector('[data-spatial-center=true] .cm-editor'));
    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: Array.from({ length: 180 }, (_, n) => `Scroll passage ${n + 1}`).join("\n") } });
    document.querySelector(".spatial-shell")!.dispatchEvent(new CustomEvent("hickory:spatial-ribbon-open", { detail: { path: "notes/other.md", side: "left", central: false } }));
  });
  await expect(other).toBeVisible();
  const positions: number[] = [];
  for (const scroll of [0, 450, 80, 600, 0]) {
    await page.evaluate(async scroll => {
      const dom = document.querySelector<HTMLElement>('[data-spatial-center=true] .cm-scroller')!;
      dom.scrollTop = scroll;
      await new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
    }, scroll);
    await expect(other).not.toHaveClass(/spatial-view--crossing/);
    expect(await rect(note)).toEqual(pinned);
    const flowRow = page.locator('[data-spatial-files=left] [data-tree-path="notes/other.md"]');
    await expect.poll(async () => Math.abs((await rect(other)).y - (await rect(flowRow)).y)).toBeLessThan(1);
    await expect.poll(async () => Math.abs(pinned.y - (await rect(row)).y)).toBeLessThan(1);
    const flowing = await rect(other); positions.push(flowing.y);
    expect(flowing.y + flowing.height <= pinned.y || flowing.y >= pinned.y + pinned.height).toBe(true);
  }
  expect(Math.max(...positions) - Math.min(...positions)).toBeGreaterThan(100);
});

</storycode:copy>
<a id="f-2b83f22a0d20-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-2b83f22a0d20-expression_statement-11">
// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
test("resizing Files puts the same preview beneath its filename or to its right", async ({ page }) => {
  await open(page, "note.md");
  const preview = await open(page, "notes/other.md");
  await remember(page, '[data-spatial-key="document:notes/other.md"]');
  await center(page).getByRole("button", { name: "Pin view in drawer" }).click();
  const files = page.locator('[data-spatial-files=left]');
  const row = files.locator('[data-tree-path="notes/other.md"]');
  const neighbor = files.locator('[data-tree-path="note.md"]');
  await expect(preview).toHaveAttribute("data-files-below", "false");
  const original = await rect(preview);
  const rule = page.locator('[data-spatial-column=left]');
  let boundary = await rect(rule);
  await page.mouse.move(boundary.x + boundary.width / 2, boundary.y + 100); await page.mouse.down();
  await page.mouse.move(boundary.x - 85, boundary.y + 100, { steps: 10 }); await page.mouse.up();
  await expect(preview).toHaveAttribute("data-files-below", "true");
  await expect(files).toHaveAttribute("data-files-below", "true");
  const narrow = await rect(preview), list = await rect(files), label = await rect(row), content = await rect(preview.locator(".spatial-view-body"));
  expect(narrow.x).toBeGreaterThanOrEqual(list.x);
  expect(narrow.x + narrow.width).toBeLessThanOrEqual(list.x + list.width);
  expect(content.y).toBeGreaterThan(label.y + label.height);
  expect(narrow.y).toBe(original.y);
  expect(narrow.height).toBe(original.height);
  expect((await rect(neighbor)).y).toBeGreaterThanOrEqual(narrow.y + narrow.height);
  expect(await preserved(page, '[data-spatial-key="document:notes/other.md"]')).toBe(true);
  await row.click();
  const text = files.locator(".cm-content");
  await text.press("End"); await text.pressSequentially("-responsive");
  await expect(row).toHaveText("  other.md-responsive");
  await text.press("ControlOrMeta+z");
  await expect(row).toHaveText("  other.md");
  await expect(preview.getByRole("button", { name: "Unpin view", exact: true })).toBeVisible();
  await page.screenshot({ path: "/tmp/hickory-files-responsive-below.png", fullPage: true });
  boundary = await rect(rule);
  await page.mouse.move(boundary.x + boundary.width / 2, boundary.y + 100); await page.mouse.down();
  await page.mouse.move(boundary.x + 90, boundary.y + 100, { steps: 10 }); await page.mouse.up();
  await expect(preview).toHaveAttribute("data-files-below", "false");
  const wide = await rect(preview), wideFiles = await rect(files);
  expect(wide.x).toBeGreaterThanOrEqual(wideFiles.x + wideFiles.width);
  expect(wide.y).toBe(original.y);
  expect(await preserved(page, '[data-spatial-key="document:notes/other.md"]')).toBe(true);
  await page.screenshot({ path: "/tmp/hickory-files-responsive-beside.png", fullPage: true });
});
</storycode:copy>

### Assemble `apps/web/e2e/spatialWorkspace.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/spatialWorkspace.spec.ts"><storycode:paste select="#f-2b83f22a0d20-expression_statement"/><storycode:paste select="#f-2b83f22a0d20-expression_statement-2"/><storycode:paste select="#f-2b83f22a0d20-center"/><storycode:paste select="#f-2b83f22a0d20-tile"/><storycode:paste select="#f-2b83f22a0d20-rect"/><storycode:paste select="#f-2b83f22a0d20-open"/><storycode:paste select="#f-2b83f22a0d20-remember"/><storycode:paste select="#f-2b83f22a0d20-preserved"/><storycode:paste select="#f-2b83f22a0d20-expression_statement-3"/><storycode:paste select="#f-2b83f22a0d20-expression_statement-4"/><storycode:paste select="#f-2b83f22a0d20-expression_statement-5"/><storycode:paste select="#f-2b83f22a0d20-expression_statement-6"/><storycode:paste select="#f-2b83f22a0d20-expression_statement-7"/><storycode:paste select="#f-2b83f22a0d20-expression_statement-8"/><storycode:paste select="#f-2b83f22a0d20-expression_statement-9"/><storycode:paste select="#f-2b83f22a0d20-expression_statement-10"/><storycode:paste select="#f-2b83f22a0d20-expression_statement-11"/></storycode:file>

</storycode:doc>
