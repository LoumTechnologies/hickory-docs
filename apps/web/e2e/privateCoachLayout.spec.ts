import { expect, test } from "@playwright/test";
// Guarantee: verification/predictions-precede-managed-observation.md.
// Real browser layout with synthetic private metadata, no engine/provider witness.
test("private coach form fits native and narrow viewports without covering its target", async ({ page }) => {
  const state = { document: "layout", source: "practice.md", source_hash: "a".repeat(64), current: true,
    question: { requirement_reference: "One total budget" }, target: {}, environment_scope: "Synthetic layout fixture",
    actual_values: "withheld", application_integration: "incomplete",
    retained_plan: { initial: { requirement_reference: "Initial stop" }, next: { requirement_reference: "Future stop" } },
    retained_debugger: { phase: "not_started", actual_values: "withheld", last: null } };
  const posts: string[] = [];
  await page.route("**/api/**", async route => {
    const path = new URL(route.request().url()).pathname;
    if (!path.startsWith("/api/")) { await route.fallback(); return; }
    if (route.request().method() === "POST") posts.push(path);
    const json = path === "/api/prediction/state" ? state : path === "/api/prediction/tool" ? { ok: true, source: "# Captured layout fixture\n" }
      : path === "/api/prediction/agent" ? { ready: true, backend: "python", edits: { mode: "review", changes: [] }, run: null }
      : path === "/api/prediction/evidence" ? { evidence: null } : {};
    await route.fulfill({ json });
  });
  for (const width of [825, 390]) {
    await page.setViewportSize({ width, height: 585 });
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Predict, investigate and review" })).toBeVisible();
    const request = page.getByRole("textbox", { name: "Investigation request" });
    const provider = page.getByRole("textbox", { name: "Provider", exact: true });
    const requestBox = (await request.boundingBox())!, providerBox = (await provider.boundingBox())!;
    expect(requestBox.width).toBeGreaterThan(width - 60);
    expect(providerBox.y).toBeGreaterThan(requestBox.y + requestBox.height);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.getByRole("button", { name: "Locate target", exact: true }).click();
    const delegate = page.getByRole("button", { name: "Delegate investigation", exact: true });
    await expect(delegate).toBeFocused();
    const target = (await delegate.boundingBox())!, popup = (await page.getByRole("region", { name: "Delegate prediction and investigation" }).boundingBox())!;
    expect(target.x + target.width <= popup.x || popup.x + popup.width <= target.x || target.y + target.height <= popup.y || popup.y + popup.height <= target.y).toBe(true);
    await page.screenshot({ path: test.info().outputPath(`private-coach-${width}.png`) });
  }
  expect(posts.every(path => path === "/api/prediction/tool" || path.startsWith("/api/v1/ui/windows"))).toBe(true);
});
