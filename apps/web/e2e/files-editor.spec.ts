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
    const toolbar = page.getByRole("toolbar", { name: "Unsaved Files changes" });
    await expect(toolbar).toBeVisible();
    await toolbar.getByRole("button", { name: "Dry Run" }).click();
    await expect(page.getByLabel("Files dry run")).toContainText(`Rename or move ${before} → ${after}`);
    const beforeApply = JSON.stringify(await (await page.request.get("/api/files")).json());
    expect(beforeApply).toContain(before);
    expect(beforeApply).not.toContain(after);
    await content.press("Control+s");
    await expect.poll(async () => JSON.stringify(await (await page.request.get("/api/files")).json())).toContain(after);
    await page.reload();
    await content.click();
    await content.press("Control+End");
    await expect(content.locator(`[data-tree-path="${after}"]`)).toBeVisible();

    const renamedLine = content.locator(`[data-tree-path="${after}"]`);
    await renamedLine.click();
    await content.press("End");
    await content.press("Enter");
    await content.pressSequentially(created);
    await page.getByRole("button", { name: "Dry Run" }).click();
    await expect(page.getByLabel("Files dry run")).toContainText(`Create file ${created}`);
    await page.getByRole("button", { name: "Apply" }).click();
    await expect.poll(async () => JSON.stringify(await (await page.request.get("/api/files")).json())).toContain(created);

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
    await page.getByRole("button", { name: "Dry Run" }).click();
    await expect(page.getByLabel("Files dry run")).toContainText(
      `Delete ${created} — confirmation required; there is no trash`,
    );
    await page.getByRole("button", { name: "Apply" }).click();
    await expect(page.getByText(`Delete ${created}? There is no trash.`)).toBeVisible();
    await page.getByRole("button", { name: "Delete" }).click();
    await expect.poll(async () => JSON.stringify(await (await page.request.get("/api/files")).json())).not.toContain(created);
  } finally {
    await page.request.post("/api/files/op", { data: { op: "delete", path: created } });
    await page.request.post("/api/files/op", { data: { op: "delete", path: after } });
    await page.request.post("/api/files/op", { data: { op: "delete", path: before } });
  }
});
