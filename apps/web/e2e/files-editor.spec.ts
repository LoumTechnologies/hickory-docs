import { expect, test } from "@playwright/test";

// Guarantee: docs/guarantees/authoring/the-workspace-tree-edits-like-an-editor.md
test("the live Files pane navigates and renames like an editor", async ({ page }) => {
  test.setTimeout(30_000);
  test.skip(!process.env.HICKORY_E2E_URL, "Run against the URL printed by just dev.");
  const before = `zz-editor-tree-${Date.now()}.txt`;
  const after = before.replace("editor-tree", "renamed-tree");
  const created = before.replace("editor-tree", "created-tree");
  try {
    await page.request.post("/api/files/op", { data: { op: "create", path: before } });
    await page.goto("/");
    const content = page.locator(".filesystem-editor .cm-content");
    await content.click();
    await content.press("Control+End");
    const line = content.locator(".cm-line", { hasText: before });
    await line.click();
    await expect(content).toBeFocused();
    await content.press("Home");
    await content.press("Shift+End");
    await content.pressSequentially(after);
    await content.press("Control+s");
    await expect(content.locator(".cm-line", { hasText: after })).toBeVisible();
    await expect(page.getByText("Applied 1 filesystem edit.")).toBeVisible();

    const renamedLine = content.locator(".cm-line", { hasText: after });
    await renamedLine.click();
    await content.press("End");
    await content.press("Enter");
    await content.pressSequentially(created);
    await content.press("Control+s");
    await expect(content.locator(`[data-tree-path="${created}"]`)).toBeVisible();

    // Saving regenerates this lens from the filesystem. Reload once so the
    // deletion gesture starts against that settled server projection rather
    // than the editor instance that is being replaced.
    await page.reload();
    const refreshedContent = page.locator(".filesystem-editor .cm-content");
    await refreshedContent.click();
    await refreshedContent.press("Control+End");
    const createdLine = refreshedContent.locator(`[data-tree-path="${created}"]`);
    await expect(createdLine).toBeVisible();
    await createdLine.click();
    await refreshedContent.press("Home");
    await refreshedContent.press("Shift+End");
    await refreshedContent.press("Backspace");
    await refreshedContent.press("Backspace");
    await refreshedContent.press("Control+s");
    await expect(page.getByText(`Delete ${created}? There is no trash.`)).toBeVisible();
    await page.getByRole("button", { name: "Delete" }).click();
    await expect(page.getByText(`Deleted ${created}.`)).toBeVisible();
  } finally {
    await page.request.post("/api/files/op", { data: { op: "delete", path: created } });
    await page.request.post("/api/files/op", { data: { op: "delete", path: after } });
    await page.request.post("/api/files/op", { data: { op: "delete", path: before } });
  }
});
