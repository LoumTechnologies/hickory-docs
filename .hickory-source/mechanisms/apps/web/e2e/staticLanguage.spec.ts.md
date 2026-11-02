<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Drive the static reader through captured language answers

A real browser must see compiled colours, a documentation popup and definition navigation while the document remains read-only. Serve a hash-bound fixture through the ordinary source-publication routes and reject any dependence on backend API traffic. This checks the production reader and shared CodeMirror handlers, not a separate demo renderer. It does not claim that a real LSP supplied this browser fixture; the provider boundary has its own focused witness.

At phone and landscape sizes, tap through the same symbol facts and confirm the panel fits the viewport, navigation remains reachable and the page has no horizontal overflow. Dragged and cancelled pointers must not inspect. Keyboard inspection and Escape dismissal remain available without a pointing device. These Chromium emulations do not establish physical iOS/Safari behavior.

<static:copy id="static-apps-web-e2e-staticLanguage-spec-ts-0">
import { test, expect, type Page } from "@playwright/test";
import { createHash } from "node:crypto";
import { gzipSync } from "node:zlib";
const hash = (text: string) => createHash("sha256").update(text).digest("hex");

async function publishedFixture(page: Page) {
  const source = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n# A published program\n<h:file path="a.ts">\nexport const count = 1;\ncount;\n</h:file>\n' + 'Readable prose must wrap inside the phone viewport without clipping. '.repeat(6) + '\nThis continuation belongs to the same paragraph. ' + 'More flowing prose. '.repeat(12) + '\n<h:file path="long.ts">\n// ' + 'long_identifier_'.repeat(40) + '\n</h:file>\n</h:doc>\n';
  const offset = source.lastIndexOf("count;"), declaration = source.indexOf("count =");
  const position = (at: number) => ({ line: source.slice(0, at).split("\n").length - 1, character: at - source.lastIndexOf("\n", at - 1) - 1 });
  const range = { start: position(declaration), end: position(declaration + 5) };
  const intelligence = { version: 1, source_hash: hash(source), state: "captured", spans: [], packed_spans: {
    classes: ["tok-keyword", "tok-variableName"], data: [source.indexOf("const"), 5, 0, offset, 5, 1],
  }, language: { capabilities: {}, queries: {}, diagnostics: [], shared: { count: { definition: { state: "captured", result: [{ uri: "hick:///book.md", range }] }, references: { state: "captured", result: [{ uri: "hick:///book.md", range }] } } }, occurrences: [
    { from: offset, to: offset + 5, position: position(offset), hover: { state: "captured", result: { contents: "```typescript\nconst count: 1\n```\nCaptured documentation" } }, symbolIds: ["count"] },
  ] } };
  const revision = "a".repeat(40), url = `documents/${"b".repeat(64)}.json.gz`;
  const document = { path: "book.md", title: "A program", sha256: hash(source), url, intelligence_sha256: hash(JSON.stringify(intelligence)) };
  const requests: string[] = [];
  page.on("request", request => { if (/^\/api(?:\/|$)|^\/ws(?:\/|$)/.test(new URL(request.url()).pathname)) requests.push(request.url()); });
  await page.route("**/source/index.json", route => route.fulfill({ json: { version: 1, revision, entry: "book.md", documents: [document] } }));
  await page.route(`**/source/${url}`, route => route.fulfill({ contentType: "application/gzip", body: gzipSync(JSON.stringify({ path: "book.md", revision, source, intelligence })) }));
  await page.goto("/source.html");
  return { range, requests };
}

test("static source replays colours, hover and definition without a backend", async ({ page }) => {
  const { range, requests } = await publishedFixture(page);
  const content = page.locator(".cm-content");
  await expect(content).toHaveAttribute("contenteditable", "false");
  await expect(content.locator(".tok-keyword").filter({ hasText: "const" }).first()).toBeVisible();
  const token = content.locator(".tok-variableName").filter({ hasText: "count" }).last();
  await token.hover();
  await expect(page.locator(".cm-lsp-hover")).toContainText("Captured documentation");
  await token.click();
  await page.keyboard.press("Alt+Enter");
  await expect(page.getByRole("complementary", { name: "Symbol inspector" })).toContainText("Captured documentation");
  await page.keyboard.press("Escape");
  await expect(page.getByRole("complementary", { name: "Symbol inspector" })).toHaveCount(0);
  await token.click({ modifiers: ["Control", "Shift"] });
  await expect(page.getByRole("complementary", { name: "Captured references" })).toContainText("book.md");
  await token.click({ modifiers: ["Control"] });
  await expect(page).toHaveURL(new RegExp(`#L${range.start.line + 1}:${range.start.character}$`));
  expect(requests).toEqual([]);
});

test("reader follows device appearance on load and when it changes", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  await page.addInitScript(() => localStorage.setItem("hickory.theme", "dark"));
  await publishedFixture(page);
  const content = page.locator(".cm-content");
  await expect(content).toBeVisible();
  const colours = () => page.evaluate(() => {
    const style = (selector: string) => getComputedStyle(document.querySelector(selector)!);
    return { theme: document.documentElement.dataset.theme, scheme: getComputedStyle(document.documentElement).colorScheme,
      page: style("body").backgroundColor, text: style(".hickory-embed").color,
      code: style(".cm-file-line").backgroundColor, keyword: style(".tok-keyword").color,
      inspector: style(".source-inspector").backgroundColor };
  });
  const token = content.locator(".tok-variableName").filter({ hasText: "count" }).last();
  await token.click(); await page.keyboard.press("Alt+Enter");
  const inspector = page.getByRole("complementary", { name: "Symbol inspector" });
  await expect(inspector).toContainText("Captured documentation");
  const light = await colours();
  expect(light).toMatchObject({ theme: "light", scheme: "light", page: "rgb(250, 249, 247)", text: "rgb(33, 32, 29)", code: "rgb(241, 239, 235)" });
  await page.emulateMedia({ colorScheme: "dark" });
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  const dark = await colours();
  expect(dark).toMatchObject({ scheme: "dark", page: "rgb(30, 34, 39)", text: "rgb(221, 227, 234)", code: "rgb(25, 29, 33)" });
  expect(dark.keyword).not.toBe(light.keyword);
  expect(dark.inspector).not.toBe(light.inspector);
  await expect(inspector).toContainText("Captured documentation");
  await expect(content).toHaveAttribute("contenteditable", "false");
  await page.emulateMedia({ colorScheme: "light" });
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  expect(await colours()).toEqual(light);
});

for (const viewport of [{ width: 390, height: 844 }, { width: 320, height: 568 }, { width: 844, height: 390 }]) {
  test.describe(`touch source ${viewport.width}x${viewport.height}`, () => {
    test.use({ viewport, hasTouch: true, isMobile: true });
    test("tap inspects, navigation stays reachable, drag and cancellation do not inspect", async ({ page }, testInfo) => {
      const { range, requests } = await publishedFixture(page);
      const content = page.locator(".cm-content");
      const token = content.locator(".tok-variableName").filter({ hasText: "count" }).last();
      await expect(token).toBeVisible();
      const prose = content.locator(".cm-line").filter({ hasText: "Readable prose must wrap" });
      await expect(prose).toContainText("This continuation belongs to the same paragraph.");
      expect((await prose.boundingBox())!.width).toBeLessThanOrEqual(viewport.width);
      expect((await prose.boundingBox())!.height).toBeGreaterThan(44);
      const longCode = content.locator(".cm-line").filter({ hasText: "long_identifier_" });
      // CodeMirror only mounts lines near the viewport; scroll to the long block first.
      await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
      await expect(longCode).toBeVisible();
      expect((await longCode.boundingBox())!.width).toBeLessThanOrEqual(viewport.width);
      expect((await longCode.boundingBox())!.height).toBeGreaterThan(44);
      await page.evaluate(() => window.scrollTo(0, 0));
      await expect(token).toBeVisible();
      if (viewport.width <= 700) {
        await expect(page.getByRole("navigation", { name: "Source files" })).toBeHidden();
        const files = page.getByRole("button", { name: "Files", exact: true });
        await files.tap();
        await expect(files).toHaveAttribute("aria-expanded", "true");
        await expect(page.getByRole("navigation", { name: "Source files" })).toBeVisible();
        await files.tap();
      }
      // Dragged/cancelled pointers belong to native scrolling/selection.
      await token.dispatchEvent("pointerdown", { pointerType: "touch", pointerId: 9, clientX: 50, clientY: 100 });
      await token.dispatchEvent("pointermove", { pointerType: "touch", pointerId: 9, clientX: 50, clientY: 140 });
      await token.dispatchEvent("pointerup", { pointerType: "touch", pointerId: 9, clientX: 50, clientY: 140 });
      await expect(page.getByRole("complementary", { name: "Symbol inspector" })).toHaveCount(0);
      await token.dispatchEvent("pointerdown", { pointerType: "touch", pointerId: 10 });
      await token.dispatchEvent("pointercancel", { pointerType: "touch", pointerId: 10 });
      await token.dispatchEvent("pointerup", { pointerType: "touch", pointerId: 10 });
      await expect(page.getByRole("complementary", { name: "Symbol inspector" })).toHaveCount(0);
      await token.tap();
      const inspector = page.getByRole("complementary", { name: "Symbol inspector" });
      await expect(inspector).toContainText("Captured documentation");
      await expect(inspector).toContainText("count");
      await expect(content).toHaveAttribute("contenteditable", "false");
      await inspector.getByText("References (1)", { exact: true }).tap();
      await expect(inspector.getByRole("link", { name: /book.md:/ })).toBeVisible();
      await page.screenshot({ path: testInfo.outputPath("mobile-inspector.png") });
      const bounds = await inspector.boundingBox();
      expect(bounds!.x).toBeGreaterThanOrEqual(0);
      expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(viewport.width + 1);
      expect(bounds!.y + bounds!.height).toBeLessThanOrEqual(viewport.height + 1);
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
      const close = page.getByRole("button", { name: "Close symbol inspector" });
      expect((await close.boundingBox())!.height).toBeGreaterThanOrEqual(44);
      await close.tap();
      await expect(inspector).toHaveCount(0);
      await token.tap();
      await inspector.getByText("Definitions (1)", { exact: true }).tap();
      await inspector.getByRole("link", { name: /book.md:/ }).tap();
      await expect(page).toHaveURL(new RegExp(`#L${range.start.line + 1}:${range.start.character}$`));
      await expect(inspector).toHaveCount(0);
      expect(requests).toEqual([]);
    });
  });
}
</static:copy>

<static:file path="apps/web/e2e/staticLanguage.spec.ts"><static:paste select="#static-apps-web-e2e-staticLanguage-spec-ts-0"/></static:file>
</static:doc>
