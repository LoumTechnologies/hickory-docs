<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/e2e/startup.spec.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4e63365d81de"></a>
## `apps/web/e2e/startup.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The App Starts As A Lightweight Editor](../../../../guarantees/authoring/the-app-starts-as-a-lightweight-editor/contract.md), [The Editor Does Not Wait For Workspace Preparation](../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md), [The Measure Says What It Measures](../../../../guarantees/authoring/the-measure-says-what-it-measures/contract.md).

<a id="f-4e63365d81de-introductionHeading"></a>
### `introductionHeading`

<storycode:copy id="f-4e63365d81de-introductionHeading">
import { expect, test } from "@playwright/test";
import { readFileSync } from "node:fs";

const introductionHeading = readFileSync(new URL("../src/content/startup.md", import.meta.url), "utf8").split("\n")[0];

</storycode:copy>
<a id="f-4e63365d81de-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-4e63365d81de-expression_statement">
// Guarantee: docs/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation.md
test("keeps the startup note editable while workspace requests await preparation", async ({ page }) => {
  test.skip(!process.env.HICKORY_E2E_URL, "Run against just dev.");
  test.setTimeout(45000);
  let ready = false;
  let dispatched = false;
  for (const path of ["drafts", "ui"]) {
    await page.route(`**/api/workspace/${path}`, (route) => route.fulfill({ json:
      route.request().method() === "PUT" ? { ok: true } : path === "ui" ? { state: null } : { drafts: [] },
    }));
  }
  await page.route("**/api/startup", (route) => route.fulfill({ json: { ready } }));
  await page.route("**/api/projects/local/docs", (route) => {
    if (!ready) return route.fulfill({ status: 503, headers: { "x-hickory-startup": "pending" }, json: { error: "Preparing generated files…" } });
    dispatched = true;
    return route.continue();
  });
  await page.goto("/");
  await expect(page.getByText(/You can keep writing your note/)).toBeVisible({ timeout: 15000 });
  const editor = page.locator(".untitled-tab .cm-content");
  await expect(editor).toBeVisible();
  await editor.click();
  await editor.press("ControlOrMeta+Home");
  await editor.pressSequentially("Writing during workspace preparation. ");
  await expect(editor).toContainText("Writing during workspace preparation.");
  expect(dispatched).toBe(false);
  ready = true;
  await expect(page.getByText(/You can keep writing your note/)).toHaveCount(0);
  await expect.poll(() => dispatched).toBe(true);
  await expect(editor).toContainText("Writing during workspace preparation.");
});

</storycode:copy>
<a id="f-4e63365d81de-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-4e63365d81de-expression_statement-2">
// Guarantee: docs/guarantees/authoring/the-app-starts-as-a-lightweight-editor.md
test("startup is an editable unsaved introduction that expands on demand", async ({ page }) => {
  test.skip(!process.env.HICKORY_E2E_URL, "Run against just dev.");
  test.setTimeout(45000);
  // Exercise a fresh folder arrangement even when the live QA workspace has
  // state from a previous run; later reads verify real persisted restoration.
  let fresh = true;
  const previous = await (await page.request.get("/api/workspace/drafts")).json();
  const previousDrafts = new Set((previous.drafts ?? []).map((draft: { path: string }) => draft.path));
  for (const path of ["drafts", "ui"]) {
    await page.route(`**/api/workspace/${path}`, async (route) => {
      if (route.request().method() !== "GET") return route.continue();
      if (fresh) return route.fulfill({ json: path === "ui" ? { state: null } : { drafts: [] } });
      if (path === "ui") return route.continue();
      const response = await route.fetch();
      const state = await response.json();
      return route.fulfill({ response, json: { drafts: state.drafts.filter((draft: { path: string }) => !previousDrafts.has(draft.path)) } });
    });
  }
  page.on("dialog", (dialog) => void dialog.accept());
  const before = await (await page.request.get("/api/files")).json();
  let created = false;
  page.on("request", (request) => {
    if (request.method() === "POST" && /\/docs$/.test(request.url())) created = true;
  });
  await page.goto("/");
  const editor = page.locator(".untitled-tab .cm-content");
  await expect(editor).toContainText(introductionHeading);
  await expect(page.getByRole("tab", { includeHidden: true })).toHaveCount(before.folder_open === false ? 1 : 3);
  await expect(editor).toBeFocused();
  // Guarantee: docs/guarantees/authoring/the-measure-says-what-it-measures.md
  const marker = page.locator(".untitled-tab").getByRole("slider", { name: "Where prose wraps" });
  const initialColumn = Number(await marker.getAttribute("aria-valuenow"));
  const box = await marker.boundingBox();
  expect(box).not.toBeNull();
  await page.mouse.move(box!.x + box!.width / 2, box!.y + box!.height / 2);
  await page.mouse.down();
  const direction = initialColumn <= 40 ? 1 : -1;
  await page.mouse.move(box!.x + direction * 100, box!.y + box!.height / 2, { steps: 5 });
  const draggedColumn = Number(await marker.getAttribute("aria-valuenow"));
  expect(draggedColumn).not.toBe(initialColumn);
  await page.mouse.up();
  await expect(marker).toHaveAttribute("aria-valuenow", String(draggedColumn));
  await expect.poll(async () => {
    const ui = await (await page.request.get("/api/workspace/ui")).json();
    return ui.state?.wrap?.untitled;
  }).toBe(draggedColumn);
  await editor.click();
  await editor.press("ControlOrMeta+Home");
  await editor.pressSequentially("My startup note");
  await editor.press("Enter");
  await expect(editor).toContainText("My startup note");
  await expect(page.getByRole("tab", { name: /Untitled/ })).toContainText("*");
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "files" })));
  await expect(page.locator("[data-spatial-files=left] .filesystem-editor")).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  await expect(page.getByRole("tab", { name: /Agent/ })).toBeVisible();
  await expect(editor).toContainText("My startup note");
  expect(created).toBe(false);
  expect(await (await page.request.get("/api/files")).json()).toEqual(before);
  // Allow the folder's expanded layout and unnamed draft to be stored.
  const expandedTabs = await page.getByRole("tab", { includeHidden: true }).count();
  await expect.poll(async () => {
    const [ui, recovery] = await Promise.all([
      page.request.get("/api/workspace/ui").then((response) => response.json()),
      page.request.get("/api/workspace/drafts").then((response) => response.json()),
    ]);
    return JSON.stringify(ui.state?.layout).includes('"kind":"chat"') && recovery.drafts.some(
      (draft: { path: string; contents: string }) => !previousDrafts.has(draft.path) && draft.contents.includes("My startup note"),
    );
  }).toBe(true);
  fresh = false;
  await page.reload();
  // Choose the restored note before inspecting its ruler; Agent has an
  // independent editor and ruler, and may occupy the center after restoration.
  const restoredNote = page.getByRole("tab", { name: /Untitled/ });
  // The recovered dirty marker distinguishes the restored buffer from the
  // temporary startup introduction that appears while hydration is pending.
  await expect(restoredNote).toContainText("*");
  await restoredNote.click({ timeout: 15000 });
  await expect(restoredNote).toHaveAttribute("aria-selected", "true");
  await expect(marker).toHaveAttribute("aria-valuenow", String(draggedColumn));
  await expect(editor).toContainText("My startup note");
  await expect(page.getByRole("tab", { includeHidden: true })).toHaveCount(expandedTabs);
  await expect(page.locator(".filesystem-editor")).toHaveCount(before.folder_open === false ? 0 : 2);
  await page.getByRole("button", { name: "Close Untitled", exact: true }).click();
  await page.getByRole("button", { name: /Discard|Close and retain/ }).click();
  await expect(editor).toHaveCount(0);
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "new" })));
  await expect(editor.locator(".cm-placeholder")).toBeVisible();
  await expect(editor).not.toContainText(introductionHeading);
  await expect(editor).toBeFocused();
});
</storycode:copy>

### Assemble `apps/web/e2e/startup.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/startup.spec.ts"><storycode:paste select="#f-4e63365d81de-introductionHeading"/><storycode:paste select="#f-4e63365d81de-expression_statement"/><storycode:paste select="#f-4e63365d81de-expression_statement-2"/></storycode:file>

</storycode:doc>
