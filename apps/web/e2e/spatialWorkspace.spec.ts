import { expect, test, type Locator, type Page } from "@playwright/test";

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
// Run against a scratch engine with note.md and notes/other.md, as described in the guarantee.
test.use({ viewport: { width: 1800, height: 1100 } });
test.beforeEach(async ({ page }) => {
  test.skip(!process.env.HICKORY_SPATIAL_URL, "Use just test-spatial-browser with a scratch workspace URL.");
  await page.goto(process.env.HICKORY_SPATIAL_URL!);
  await expect(page.locator("[data-spatial-files]")).toHaveCount(2);
});
async function open(page: Page, path: string, side: "left" | "right") {
  await page.locator(`[data-spatial-files=${side}] [data-tree-path="${path}"]`).dblclick();
  const tile = page.locator(`[data-spatial-key="document:${path}"]`);
  await expect(tile).toBeVisible(); return tile;
}
async function rect(tile: Locator) { return tile.evaluate(el => { const r = el.getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height }; }); }
async function allRects(page: Page) { return page.locator("[data-spatial-key]").evaluateAll(elements => elements.filter(el => !el.hasAttribute("hidden")).map(el => ({ key: el.getAttribute("data-spatial-key"), rect: el.getBoundingClientRect().toJSON() }))); }
async function rememberEditor(page: Page, selector: string) {
  await page.evaluate(async selector => {
    // Vite exports the same real CodeMirror class used by the running application.
    const module = "/node_modules/.vite/deps/@codemirror_view.js";
    const { EditorView } = await import(/* @vite-ignore */ module);
    const view = EditorView.findFromDOM(document.querySelector(`${selector} .cm-editor`));
    const line = view.state.doc.line(Math.min(3, view.state.doc.lines));
    view.dispatch({ selection: { anchor: line.from + Math.min(2, line.length), head: line.to }, scrollIntoView: true });
    view.focus();
    (window as unknown as { spatialWitness: unknown }).spatialWitness = { view, text: view.state.doc.toString(), selection: view.state.selection.toJSON() };
  }, selector);
}
async function unchanged(page: Page, selector: string) {
  return page.evaluate(async selector => {
    const module = "/node_modules/.vite/deps/@codemirror_view.js", { EditorView } = await import(/* @vite-ignore */ module);
    const witness = (window as unknown as { spatialWitness: { view: { state: { doc: { toString(): string }; selection: { toJSON(): unknown } } }; text: string; selection: unknown } }).spatialWitness;
    const same = EditorView.findFromDOM(document.querySelector(`${selector} .cm-editor`)) === witness.view && witness.text === witness.view.state.doc.toString() && JSON.stringify(witness.selection) === JSON.stringify(witness.view.state.selection.toJSON());
    if (!same) console.debug("Spatial editor witness", JSON.stringify({ sameView: EditorView.findFromDOM(document.querySelector(`${selector} .cm-editor`)) === witness.view, sameText: witness.text === witness.view.state.doc.toString(), selection: witness.selection, current: witness.view.state.selection.toJSON() }));
    return same;
  }, selector);
}

test("pin/unpin freezes every rectangle and keeps the native header and Files buffer stable", async ({ page }) => {
  const tile = await open(page, "note.md", "left");
  await tile.getByRole("tab").click();
  await expect(tile).toHaveAttribute("data-expanded", "true");
  await expect.poll(async () => (await rect(tile)).height).toBe(253);
  await rememberEditor(page, '[data-spatial-key="document:note.md"]');
  const before = await allRects(page), treeBefore = await page.locator("[data-spatial-files=left] [data-tree-path=note\\.md]").boundingBox();
  await tile.getByRole("button", { name: "Pin view", exact: true }).click();
  await expect(tile).toHaveClass(/spatial-view--pinned/);
  expect(await allRects(page)).toEqual(before);
  const strip = page.locator('[data-spatial-strip="document:note.md"]');
  expect((await rect(strip)).y).toBe((await rect(tile)).y);
  await tile.getByRole("button", { name: "Unpin view", exact: true }).click();
  await expect(tile).not.toHaveClass(/spatial-view--pinned/);
  expect(await allRects(page)).toEqual(before);
  expect(await page.locator('[data-spatial-files=left] [data-tree-path="note.md"]').boundingBox()).toEqual(treeBefore);
  expect(await unchanged(page, '[data-spatial-key="document:note.md"]')).toBe(true);
});

test("drag previews commit exactly, both resize edges work, and Center/Restore preserve the editor", async ({ page }) => {
  page.on("console", message => { if (message.text().startsWith("Spatial editor witness")) console.log(message.text()); });
  const errors: string[] = []; page.on("pageerror", error => errors.push(error.message));
  const tile = await open(page, "note.md", "left");
  await tile.getByRole("tab").click();
  await expect.poll(async () => (await rect(tile)).height).toBe(253);
  await rememberEditor(page, '[data-spatial-key="document:note.md"]');
  await tile.getByRole("button", { name: "Pin view", exact: true }).click();
  const box = await rect(tile);
  await page.mouse.move(box.x + 30, box.y + 18); await page.mouse.down(); await page.mouse.move(box.x + 30, box.y + 130, { steps: 10 });
  await expect(tile).toHaveClass(/spatial-view--tentative/);
  const preview = await rect(tile); await page.mouse.up();
  await expect(tile).not.toHaveClass(/spatial-view--tentative/);
  expect(await rect(tile)).toEqual(preview);
  for (const edge of ["top", "bottom"] as const) {
    const beforeResize = await rect(tile);
    const handle = page.locator(`[data-spatial-resize="document:note.md"][data-resize-edge=${edge}]`), h = (await handle.boundingBox())!;
    await page.mouse.move(h.x + h.width / 2, h.y + 4); await page.mouse.down(); await page.mouse.move(h.x + h.width / 2, h.y + (edge === "top" ? -36 : 44), { steps: 6 }); await page.mouse.up();
    await expect.poll(async () => (await rect(tile)).height).toBe(beforeResize.height + 40);
  }
  expect((await rect(tile)).height).toBeGreaterThan(preview.height);
  const pin = await rect(tile);
  await tile.getByRole("button", { name: "Center", exact: true }).click();
  await expect(tile).toHaveAttribute("data-spatial-center", "true");
  await expect(tile.locator(".cm-gutters")).toBeVisible(); await expect(tile.locator(".cm-right-rail")).toBeVisible();
  await tile.getByRole("button", { name: "Restore previous center", exact: true }).click();
  await expect(tile).toHaveAttribute("data-spatial-center", "false");
  await expect.poll(() => rect(tile)).toEqual(pin);
  await expect.poll(() => unchanged(page, '[data-spatial-key="document:note.md"]')).toBe(true);
  expect(errors).toEqual([]);
});

test("documents can be the center; file buffers remain editable and the workspace has no outer scrollbar", async ({ page }) => {
  const first = await open(page, "note.md", "left"), second = await open(page, "notes/other.md", "right");
  await first.getByRole("button", { name: "Center", exact: true }).click();
  await expect(first).toHaveAttribute("data-spatial-center", "true");
  await expect(second).toBeVisible();
  const source = page.locator('[data-spatial-files=right] .filesystem-editor .cm-content');
  await source.click(); await source.press("ControlOrMeta+End"); await source.press("End"); await source.pressSequentially("-tentative");
  await expect(page.locator('[data-spatial-files=right]').getByRole("toolbar", { name: "Unsaved Files changes" })).toBeVisible();
  await source.press("ControlOrMeta+z");
  await expect(page.locator('[data-spatial-files=right]').getByRole("toolbar", { name: "Unsaved Files changes" })).toHaveCount(0);
  const overflow = await page.evaluate(() => ({ body: document.documentElement.scrollHeight > window.innerHeight + 1, host: document.querySelector(".spatial-shell")!.scrollHeight > document.querySelector(".spatial-shell")!.clientHeight + 1 }));
  expect(overflow).toEqual({ body: false, host: false });
  const boxes = await allRects(page);
  for (let i = 0; i < boxes.length; i++) for (let j = i + 1; j < boxes.length; j++) { const a = boxes[i].rect, b = boxes[j].rect; expect(a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top).toBe(false); }
});

test("scrolling completes an edge crossing without overlapping a pin, including at narrow widths", async ({ page }) => {
  const first = await open(page, "note.md", "left"), pinned = await open(page, "notes/other.md", "right");
  await first.getByRole("button", { name: "Center", exact: true }).click();
  await pinned.getByRole("button", { name: "Pin view", exact: true }).click();
  await page.evaluate(() => {
    const witness = { overlaps: [] as string[], frames: 0 };
    (window as unknown as { spatialMotion: unknown }).spatialMotion = witness;
    const watch = () => {
      const root = document.querySelector(".spatial-shell")!.getBoundingClientRect();
      const boxes = [...document.querySelectorAll<HTMLElement>("[data-spatial-key]")].filter(el => !el.hidden && getComputedStyle(el).visibility !== "hidden").map(el => ({ id: el.dataset.spatialKey!, r: el.getBoundingClientRect() })).filter(({ r }) => r.bottom > root.top && r.top < root.bottom);
      for (let i = 0; i < boxes.length; i++) for (let j = i + 1; j < boxes.length; j++) { const a = boxes[i].r, b = boxes[j].r; if (a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top) witness.overlaps.push(`${boxes[i].id} / ${boxes[j].id}`); }
      if (++witness.frames < 90) requestAnimationFrame(watch);
    }; requestAnimationFrame(watch);
  });
  const source = first.locator(".cm-scroller");
  await source.evaluate(el => { el.scrollTop = 1000; });
  await expect.poll(() => page.evaluate(() => (window as unknown as { spatialMotion: { frames: number } }).spatialMotion.frames)).toBe(90);
  expect(await page.evaluate(() => (window as unknown as { spatialMotion: { overlaps: string[] } }).spatialMotion.overlaps)).toEqual([]);
  expect(await page.locator(".spatial-view--crossing").count()).toBe(0);
  await page.setViewportSize({ width: 390, height: 844 });
  await expect.poll(async () => { const boxes = await allRects(page); return boxes.some(({ rect: a }, i) => boxes.some(({ rect: b }, j) => i !== j && a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top)); }).toBe(false);
});

test("a native folder fold hides its unpinned descendant and becomes its brace endpoint", async ({ page }) => {
  const tile = await open(page, "notes/other.md", "left");
  const rail = page.locator("[data-spatial-files=left]"), folder = rail.locator('[data-tree-path="notes"]'), row = (await folder.boundingBox())!, gutter = (await rail.locator(".cm-foldGutter").boundingBox())!;
  await page.mouse.click(gutter.x + gutter.width / 2, row.y + 18);
  await expect(tile).toBeHidden();
  const endpoint = await page.evaluate(async () => {
    const path = "/src/shell/Ribbons.tsx", { findTerminal } = await import(/* @vite-ignore */ path);
    const terminal = findTerminal(document.body, "document", "notes/other.md");
    return terminal && { ends: terminal.ends, top: terminal.rect.top };
  });
  expect(endpoint?.ends).toBe("tree"); expect(endpoint?.top).toBe((await folder.boundingBox())!.y);
});
