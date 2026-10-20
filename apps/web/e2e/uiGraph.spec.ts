import { test, expect } from "@playwright/test";

// Guarantee: docs/guarantees/authoring/a-drawn-diagram-is-document-text.md
// Actual React Flow and native gestures; no paid provider, package or deployment witness.
test("native graph drag and label typing share the document with agent movement", async ({ page }) => {
  const unexpected: string[] = [];
  await page.route(url => url.pathname.startsWith("/api/"), route => { unexpected.push(route.request().url()); return route.abort(); });
  await page.goto("/embed.html?graph");
  const node = page.getByTestId("graph-node-api");
  await expect(node).toBeVisible();
  await node.dblclick();
  const label = node.getByRole("textbox");
  await expect(label).toBeVisible();
  await label.fill("Native API"); await label.press("Enter");
  await expect(node).toHaveText("Native API");
  const source = page.getByLabel("Graph document source");
  await expect(source).toContainText('"label": "Native API"');
  const before = await node.boundingBox();
  if (!before) throw Error("Node has no live canvas bounds");
  await page.mouse.move(before.x + before.width / 2, before.y + before.height / 2);
  await page.mouse.down(); await page.mouse.move(before.x + before.width / 2 + 70, before.y + before.height / 2 + 35, { steps: 8 }); await page.mouse.up();
  await expect.poll(async () => JSON.parse(await source.innerText()).layout.api.x).not.toBe(0);
  const dragged = await node.boundingBox();
  await page.getByRole("button", { name: "Agent move API" }).click();
  await expect(page.getByLabel("Graph action receipt")).toContainText('"commit_callback_requested":true');
  await expect.poll(async () => JSON.parse(await source.innerText()).layout.api).toMatchObject({ x: 300, y: 96 });
  await expect(node).toHaveText("Native API");
  const moved = await node.boundingBox();
  expect(moved?.x).not.toBe(dragged?.x);
  expect(unexpected).toEqual([]);
});
