import { expect, test } from "@playwright/test";

// Guarantee: docs/guarantees/authoring/the-app-starts-as-a-lightweight-editor.md
test("startup is an editable unsaved introduction that expands on demand", async ({ page }) => {
  test.skip(!process.env.HICKORY_E2E_URL, "Run against just dev.");
  page.on("dialog", (dialog) => void dialog.accept());
  const before = await (await page.request.get("/api/files")).json();
  let created = false;
  page.on("request", (request) => {
    if (request.method() === "POST" && /\/docs$/.test(request.url())) created = true;
  });
  await page.goto("/");
  const editor = page.locator(".untitled-tab .cm-content");
  await expect(editor).toContainText("Start with a note. Grow into an IDE.");
  await expect(page.getByRole("tab", { name: /Untitled/ })).toContainText("*");
  await expect(page.getByRole("tab")).toHaveCount(1);
  await expect(editor).toBeFocused();
  await editor.press("ControlOrMeta+Home");
  await editor.pressSequentially("My startup note");
  await editor.press("Enter");
  await expect(editor).toContainText("My startup note");
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "files" })));
  await expect(page.locator(".filesystem-editor")).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  await expect(page.getByRole("tab", { name: /Agent/ })).toBeVisible();
  await expect(editor).toContainText("My startup note");
  expect(created).toBe(false);
  expect(await (await page.request.get("/api/files")).json()).toEqual(before);
  // Allow the expanded layout to be stored: startup must override it.
  await page.waitForTimeout(900);
  await page.reload();
  await expect(editor).toContainText("Start with a note. Grow into an IDE.");
  await expect(page.getByRole("tab")).toHaveCount(1);
  await expect(page.locator(".filesystem-editor")).toHaveCount(0);
  await page.getByRole("button", { name: "Close Untitled", exact: true }).click();
  await page.getByRole("button", { name: /Discard|Close and retain/ }).click();
  await expect(editor).toHaveCount(0);
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "new" })));
  await expect(editor).toHaveText("");
  await expect(editor).toBeFocused();
});
