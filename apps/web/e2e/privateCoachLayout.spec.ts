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

// Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
// Exercises the actual native tour and shipped CSS, with synthetic inactive metadata.
test("native intro remains inside its anchored popup with clickable chrome", async ({ page }) => {
  const posts: string[] = [];
  const createdPath = `/synthetic-layout-workspace/${"long-folder-".repeat(30)}/practice`;
  await page.route("**/api/**", async route => {
    const path = new URL(route.request().url()).pathname;
    if (!path.startsWith("/api/")) { await route.fallback(); return; }
    if (route.request().method() === "POST") posts.push(path);
    await route.fulfill({ json: route.request().method() === "POST" && path === "/api/tour/practice" ? { path: createdPath, opened: false, open_error: "Synthetic layout response; no repository was created" } : { active: false } });
  });
  for (const width of [1100, 390]) {
    await page.setViewportSize({ width, height: 780 });
    await page.goto("/embed.html?native-coach");
    await page.getByRole("button", { name: "AI engineer tour", exact: true }).click();
    const popup = page.getByRole("region", { name: "AI engineer tour", exact: true });
    const chrome = popup.locator(":scope > header"), content = popup.getByRole("region", { name: "Agent-first practice" });
    const outer = (await popup.boundingBox())!, header = (await chrome.boundingBox())!, body = (await content.boundingBox())!;
    expect(body.x).toBeGreaterThanOrEqual(outer.x);
    expect(body.x + body.width).toBeLessThanOrEqual(outer.x + outer.width);
    expect(body.y).toBeGreaterThanOrEqual(header.y + header.height);
    expect(await popup.evaluate(element => element.scrollWidth <= element.clientWidth)).toBe(true);
    await popup.getByRole("button", { name: "Locate target", exact: true }).click();
    await expect(page.getByRole("button", { name: "Close AI engineer tour", exact: true })).toBeFocused();
    await expect(popup.getByText("The target is closed", { exact: false })).toHaveCount(0);
    const create = popup.getByRole("button", { name: "Create a private prediction practice", exact: true });
    await create.scrollIntoViewIfNeeded(); await expect(create).toBeVisible();
    await popup.getByRole("button", { name: "Create and open an isolated practice repository", exact: true }).click();
    await expect(popup.locator("code")).toHaveText(createdPath);
    expect(await popup.evaluate(element => element.scrollWidth <= element.clientWidth)).toBe(true);
    await page.screenshot({ path: test.info().outputPath(`native-intro-${width}.png`) });
    await popup.getByRole("button", { name: "Dismiss coach", exact: true }).click();
    await expect(popup).toHaveCount(0);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  }
  expect(posts).toEqual(["/api/tour/practice", "/api/tour/practice"]);
});
