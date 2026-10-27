import { test, expect } from "@playwright/test";

test("mobile local editing survives reload without a desktop", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/mobile.html");
  await expect(page.getByText(/Browser preview/)).toBeVisible();
  await page.getByRole("button", { name: "New document", exact: true }).click();
  await page.getByLabel("Document title").fill("Field notes");
  const editor = page.locator(".cm-content[contenteditable=true]");
  await editor.click(); await page.keyboard.insertText("Offline notes 🦀");
  await expect(page.getByRole("status")).toHaveText("Saved in this browser");
  await expect(page.getByRole("button", { name: "Run on desktop" })).toBeDisabled();
  await page.reload();
  await page.getByRole("button", { name: /Field notes/ }).click();
  await expect(editor).toContainText("Offline notes 🦀");
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.getByRole("button", { name: "Documents", exact: true }).click();
  await page.getByText("Connect a desktop", { exact: true }).click();
  await expect(page.getByRole("button", { name: "Pair", exact: true })).toBeDisabled();
});
