<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/e2e/spatialWalkthrough.spec.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-02482d7f5018"></a>
## `apps/web/e2e/spatialWalkthrough.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Spatial Views Preserve The Editor](../../../../contract.md).

<a id="f-02482d7f5018-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-02482d7f5018-expression_statement">
import { expect, test, type Locator, type Page } from "@playwright/test";

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
// Runs against a disposable copy supplied by just test-spatial-walkthrough.
test.use({ viewport: { width: 1200, height: 900 } });
</storycode:copy>
<a id="f-02482d7f5018-pane"></a>
### `pane`

<storycode:copy id="f-02482d7f5018-pane">
const pane = (page: Page, path: string) => page.locator(`.spatial-view[data-shell-target="${path}"]`);
</storycode:copy>
<a id="f-02482d7f5018-box"></a>
### `box`

<storycode:copy id="f-02482d7f5018-box">
const box = (element: Locator) => element.evaluate(el => { const r = el.getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: r.height }; });
</storycode:copy>
<a id="f-02482d7f5018-open"></a>
### `open`

<storycode:copy id="f-02482d7f5018-open">
async function open(page: Page, path: string) {
  const show = page.getByRole("button", { name: "Show left drawer" });
  await expect(page.locator(".spatial-shell")).toBeVisible();
  if (!(await page.locator("[data-spatial-files=left]").isVisible())) await show.click();
  await page.locator(`[data-spatial-files=left] [data-tree-path="${path}"]`).dblclick();
  await expect(pane(page, path)).toHaveAttribute("data-spatial-center", "true");
}

</storycode:copy>
<a id="f-02482d7f5018-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-02482d7f5018-expression_statement-2">
test("walkthrough receipt bypass moves its filename and window together while the pin stays fixed", async ({ page }) => {
  test.setTimeout(45000);
  test.skip(!process.env.HICKORY_SPATIAL_PROJECT, "Use just test-spatial-walkthrough with an existing project.");
  const errors: string[] = []; page.on("pageerror", error => errors.push(error.message));
  await page.goto(process.env.HICKORY_SPATIAL_URL!);
  await open(page, "library.py"); await open(page, "app.py"); await open(page, "project.md");
  const source = pane(page, "project.md"), pinned = pane(page, "library.py"), flowing = pane(page, "app.py");
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: "library.py", exact: true }).click();
  await page.locator('[data-spatial-center=true]').getByRole("button", { name: "Pin view in drawer" }).click();
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: "project.md", exact: true }).click();
  await expect(source).toHaveAttribute("data-spatial-center", "true");
  const fixedRow = page.locator('[data-spatial-files=left] [data-tree-path="library.py"]');
  await expect.poll(async () => Math.abs((await box(pinned)).y - (await box(fixedRow)).y)).toBeLessThan(1);
  const fixed = await box(pinned);
  await page.evaluate(async () => {
    const module = "/node_modules/.vite/deps/@codemirror_view.js", { EditorView } = await import(/* @vite-ignore */ module);
    const view = EditorView.findFromDOM(document.querySelector('[data-spatial-center=true] .cm-editor'));
    view.dispatch({ changes: { from: 0, insert: Array.from({ length: 80 }, (_, i) => `Walkthrough receipt scroll ${i + 1}\n`).join("") + "\n" } });
    document.querySelector(".spatial-shell")!.dispatchEvent(new CustomEvent("hickory:spatial-ribbon-open", { detail: { path: "app.py", side: "left", central: false } }));
  });
  await page.evaluate(() => {
    const samples: { delta: number; y: number }[] = [];
    (window as unknown as { bypassSamples: typeof samples }).bypassSamples = samples;
    document.querySelector(".spatial-shell")!.addEventListener("hickory-spatial-frame", () => {
      const flow = document.querySelector<HTMLElement>('.spatial-view[data-shell-target="app.py"]')!;
      const row = document.querySelector<HTMLElement>('[data-spatial-files=left] [data-tree-path="app.py"]');
      if (row && flow.classList.contains("spatial-view--crossing")) samples.push({ delta: Math.abs(row.getBoundingClientRect().top - flow.getBoundingClientRect().top), y: flow.getBoundingClientRect().top });
    });
  });
  const positions: number[] = [];
  let sawBypass = false;
  for (const scroll of [0, 1600, 0, 1600, 0]) {
    await source.locator(".cm-scroller").first().evaluate(async (dom, scroll) => {
      dom.scrollTop = scroll;
      await new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
    }, scroll);
    sawBypass ||= await flowing.evaluate(el => el.classList.contains("spatial-view--crossing"));
    await expect(flowing).not.toHaveClass(/spatial-view--crossing/);
    await expect(flowing).toBeVisible();
    const row = page.locator('[data-spatial-files=left] [data-tree-path="app.py"]');
    await expect.poll(async () => Math.abs((await box(row)).y - (await box(flowing)).y)).toBeLessThan(1);
    expect(await box(pinned)).toEqual(fixed);
    expect(Math.abs((await box(fixedRow)).y - fixed.y)).toBeLessThan(1);
    const current = await box(flowing); positions.push(current.y);
    expect(current.y + current.h <= fixed.y || current.y >= fixed.y + fixed.h).toBe(true);
  }
  expect(Math.max(...positions) - Math.min(...positions)).toBeGreaterThan(100);
  expect(sawBypass).toBe(true);
  const samples = await page.evaluate(() => (window as unknown as { bypassSamples: { delta: number; y: number }[] }).bypassSamples);
  expect(samples.length).toBeGreaterThan(3);
  expect(Math.max(...samples.map(sample => sample.delta))).toBeLessThan(2);
  expect(Math.max(...samples.map(sample => sample.y)) - Math.min(...samples.map(sample => sample.y))).toBeGreaterThan(100);
  const movingRow = page.locator('[data-spatial-files=left] [data-tree-path="app.py"]');
  await movingRow.click();
  const filesText = page.locator('[data-spatial-files=left] .filesystem-editor .cm-content');
  await filesText.press("End"); await filesText.pressSequentially("-edit");
  await expect(movingRow).toHaveText("app.py-edit");
  await filesText.press("ControlOrMeta+z");
  await expect(movingRow).toHaveText("app.py");
  await page.screenshot({ path: "/tmp/hickory-walkthrough-bypass.png", fullPage: true });
  expect(errors).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/e2e/spatialWalkthrough.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/spatialWalkthrough.spec.ts"><storycode:paste select="#f-02482d7f5018-expression_statement"/><storycode:paste select="#f-02482d7f5018-pane"/><storycode:paste select="#f-02482d7f5018-box"/><storycode:paste select="#f-02482d7f5018-open"/><storycode:paste select="#f-02482d7f5018-expression_statement-2"/></storycode:file>

</storycode:doc>
