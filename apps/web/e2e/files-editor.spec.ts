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
