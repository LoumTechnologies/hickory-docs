import { expect, test } from "@playwright/test";

// Contract: .hickory-source/guarantees/authoring/the-workspace-tree-edits-like-an-editor/contract.md
test("Files opens on click and keeps deletion behind confirmation", async ({ page }) => {
  test.skip(!process.env.HICKORY_E2E_URL, "Run against the URL printed by just dev.");
  const name = `zz-sidebar-${Date.now()}.txt`;
  try {
    expect((await page.request.post("/api/files/op", { data: { op: "create", path: name } })).ok()).toBeTruthy();
    await page.goto("/");
    await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "files" })));
    const row = page.getByRole("treeitem", { name, exact: true });
    await row.click();
    await expect(page.locator(`[data-spatial-key="file:${name}"]`)).toBeVisible();
    await row.focus();
    await row.press("Delete");
    await expect(page.getByRole("dialog")).toContainText(`Delete ${name}?`);
    expect(JSON.stringify(await (await page.request.get("/api/files")).json())).toContain(name);
    await page.getByRole("button", { name: "Cancel", exact: true }).click();
    await row.press("Delete");
    await page.getByRole("button", { name: "Delete", exact: true }).click();
    await expect(row).toHaveCount(0);
  } finally {
    await page.request.post("/api/files/op", { data: { op: "delete", path: name } });
  }
});

test("keeps neighboring Files gutter rows fixed while clicking a lower overflow action", async ({ page }) => {
  test.skip(!process.env.HICKORY_E2E_URL, "Run against the URL printed by just dev.");
  await page.route(url => url.pathname === "/api/files", async route => {
    const response = await route.fetch();
    const folder = await response.json();
    const tree = ["gutter-a", "gutter-b"].map(name => ({
      dir: true, name, path: name,
      children: [{ dir: false, name: "justfile", path: name + "/justfile" }],
    }));
    await route.fulfill({ response, json: { ...folder, folder_open: true, truncated: false, tree } });
  });
  await page.route(url => url.pathname === "/api/just-recipes", route => {
    const path = new URL(route.request().url()).searchParams.get("path");
    const names = path === "gutter-a/justfile" ? ["one", "two", "three", "four", "five"] : ["other"];
    return route.fulfill({ json: { recipes: names.map(name => ({ name, parameters: [] })) } });
  });
  await page.route(url => url.pathname === "/api/terminals", route =>
    route.request().method() === "POST" ? route.fulfill({ json: { id: "gutter-fixture" } }) : route.continue());
  await page.goto("/");
  await expect(page.locator(".untitled-tab .cm-content")).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "files" })));

  const tree = page.locator('[data-spatial-files="left"] .folder-tree');
  const more = tree.getByRole("button", { name: "Show 5 more gutter icons" });
  const nextIcon = tree.getByRole("button", { name: "Run just other in gutter-b" });
  const nextRow = tree.getByRole("treeitem", { name: "gutter-b" });
  await more.click({ trial: true });
  const dimensions = () => tree.evaluate(el => ({ width: el.clientWidth, scrollWidth: el.scrollWidth, scrollLeft: el.scrollLeft }));
  const before = await dimensions();
  expect(before.scrollWidth).toBe(before.width);
  expect(before.scrollLeft).toBe(0);
  const anchor = await nextIcon.boundingBox();
  const row = await nextRow.boundingBox();
  expect(anchor).not.toBeNull();
  expect(row).not.toBeNull();

  await more.click();
  await expect(tree.getByRole("dialog", { name: "Gutter icons" })).toBeVisible();
  expect(await dimensions()).toEqual(before);
  expect(await nextIcon.boundingBox()).toEqual(anchor);
  expect(await nextRow.boundingBox()).toEqual(row);
  await page.keyboard.press("Escape");
  await expect(more).toBeFocused();

  await more.click();
  const launched = page.waitForRequest(request => request.method() === "POST" && new URL(request.url()).pathname === "/api/terminals");
  await tree.getByRole("button", { name: "Run just five in gutter-a" }).click();
  expect((await launched).postDataJSON()).toEqual({
    title: "just five", cwd: "gutter-a", argv: ["just", "--justfile", "justfile", "--", "five"],
  });
});
