<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/e2e/predictionPublication.spec.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Published executable results identify reviewed inputs](../../../../contract.md)

Captured SHA-256: `5f5a6013c8c91e0f20d9c5fb5397df3b58dde1042b6433547cd429d42ae341bf`.

<book:entry path="apps/web/e2e/predictionPublication.spec.ts">
import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import { resolve, sep } from "node:path";

// Guarantee: docs/guarantees/embedding/publication-is-bound-to-reviewed-inputs.md
// Actual recorded host-process/DAP exports; static hosting is a test fixture.
test("inspect the recorded authorization pair without a backend and refuse missing evidence", async ({ page }) => {
  test.skip(!process.env.HICKORY_PREDICTION_PUBLICATIONS, "Generate the explicitly selected authorization publications first");
  const root = resolve(process.env.HICKORY_PREDICTION_PUBLICATIONS!);
  const index = JSON.parse(readFileSync(resolve(root, "index.json"), "utf8"));
  const api: string[] = [];
  let removeEvidence = false;
  await page.route(url => url.pathname.startsWith("/api/"), route => { api.push(route.request().url()); return route.abort(); });
  await page.route("**/publications/review/**", async route => {
    const relative = new URL(route.request().url()).pathname.split("/publications/review/")[1];
    const path = resolve(root, relative);
    if (!path.startsWith(root + sep)) return route.abort();
    const value = JSON.parse(readFileSync(path, "utf8"));
    if (removeEvidence && value.prediction_history) value.prediction_history.history.ordering = value.prediction_history.history.ordering.filter((entry: { event: { kind: string } }) => entry.event.kind !== "finished");
    return route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(value) });
  });
  for (const name of ["vulnerable", "patched"]) {
    const document = index.documents.find((entry: { path: string }) => entry.path.endsWith(`${name}.md`));
    expect(document).toBeTruthy();
    await page.goto("/snapshot.html?index=publications/review/index.json");
    await page.getByRole("link", { name: `${document.path} · ${document.status}`, exact: true }).click();
    await expect(page.getByRole("heading", { name: `Verification: ${name === "vulnerable" ? "failed" : "passed"}`, exact: true })).toBeVisible();
    await expect(page.getByRole("heading", { name: /^execution_rejected · attempt/ })).toBeHidden();
    const requirement = page.locator("#requirement-0");
    await requirement.getByRole("link", { name: /^Recorded prediction attempt/ }).first().click();
    await expect(page.getByRole("heading", { name: /^execution_rejected · attempt/ })).toBeVisible();
    await expect(page.getByRole("heading", { name: /^verified_after_correction · attempt/ })).toBeVisible();
    await expect(page.getByText(/matches this captured document/)).toHaveCount(1);
    await expect(page.getByText(/different revision; source bytes not included/)).toHaveCount(1);
    await expect(page.getByText(/Current certification: unavailable/)).toHaveCount(2);
    await expect(page.getByText(/recorded capture; browsing does not execute a debugger/)).toHaveCount(2);
    await expect(page.getByRole("link", { name: /^Tested revision/ })).toHaveAttribute("href", `https://github.com/LoumTechnologies/hickory-docs/commit/${document.revision}`);
    const other = index.documents.find((entry: { source_hash: string; path: string }) => entry.path.includes("security-authorization/") && entry.source_hash !== document.source_hash);
    await page.getByRole("button", { name: `Inspect captured source revision ${other.source_hash}`, exact: true }).click();
    await expect(page.getByRole("heading", { name: `Verification: ${other.status}`, exact: true })).toBeVisible();
  }
  removeEvidence = true;
  await page.goto(`/snapshot.html?src=${encodeURIComponent(`publications/review/${index.documents.at(-1).snapshot}`)}`);
  await expect(page.getByRole("alert")).toContainText("Invalid selected prediction history");
  await expect(page.getByRole("heading", { name: "Verification: passed", exact: true })).toHaveCount(0);
  expect(api).toEqual([]);
});
</book:entry>

</book:doc>
