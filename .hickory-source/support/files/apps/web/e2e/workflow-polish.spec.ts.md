<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/e2e/workflow-polish.spec.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The Agent pane is a live document](../../../../../guarantees/agent/the-agent-pane-is-a-live-document/contract.md)
- [Settings preserves the open workspace](../../../../../guarantees/authoring/settings-preserves-the-open-workspace/contract.md)
- [Spatial Views Preserve The Editor](../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md)

Captured SHA-256: `e9cf35eac0c45556cf640fb6fdd8b81330e92f0f7e33e9761724eb049fbc770e`.

<book:entry path="apps/web/e2e/workflow-polish.spec.ts">
import { expect, test } from "@playwright/test";

// Real native engine: just test-workflow-ui. No provider credentials or paid turns.
test.use({ viewport: { width: 1200, height: 900 } });
test.beforeEach(async ({ page }) => {
  test.skip(!process.env.HICKORY_SPATIAL_URL, "Use just test-workflow-ui.");
  await page.goto(process.env.HICKORY_SPATIAL_URL!);
  await expect(page.getByRole("tablist", { name: "Center views" })).toBeVisible();
});

test("Run and Test are discoverable by name and act on the focused document", async ({ page }) => {
  await page.getByRole("button", { name: "Show left drawer" }).click();
  await page.locator('[data-spatial-files=left] [data-tree-path="note.md"]').dblclick();
  const note = page.locator('[data-spatial-key="document:note.md"]');
  await expect(note).toHaveAttribute("data-spatial-center", "true");
  await page.keyboard.press("ControlOrMeta+Shift+P");
  const input = page.getByRole("textbox", { name: "Commands" });
  await input.fill(">Test document");
  await expect(page.getByRole("option", { name: /Test document.*note.md/ })).toBeVisible();
  await input.press("Enter");
  await expect(note).toContainText("Test passed");
  await page.keyboard.press("ControlOrMeta+Shift+P");
  await input.fill(">Run document");
  await expect(page.getByRole("option", { name: /Run document.*note.md/ })).toBeVisible();
  await input.press("Enter");
  await expect(note).toContainText("Re-woven");
});

test("opening Agent starts typing in the draft, and Write a response returns to it", async ({ page }) => {
  await page.keyboard.press("ControlOrMeta+Shift+P");
  const input = page.getByRole("textbox", { name: "Commands" });
  await input.fill(">Agent");
  await expect(page.getByRole("option", { name: /Agent — show/ })).toBeVisible();
  await input.press("Enter");
  const response = page.getByRole("textbox", { name: "Agent conversation and response" });
  await expect(response).toBeFocused();
  await page.keyboard.type("Explain the generated files");
  await expect(response).toContainText("Explain the generated files");
  await response.press("ControlOrMeta+Home");
  await page.getByRole("button", { name: "Write a response" }).click();
  await expect(response).toBeFocused();
  await page.keyboard.type(" and their lineage.");
  await expect(response).toContainText("Explain the generated files and their lineage.");
  await expect(response).toContainText("<hick:session");
  await page.screenshot({ path: "/tmp/hickory-agent-response-polish.png" });
});

test("Settings preserves the unnamed editor, draft, selection and open tabs", async ({ page }) => {
  const editor = page.locator('[data-spatial-center=true] .cm-content');
  await editor.click();
  await editor.press("ControlOrMeta+End");
  await editor.pressSequentially("A draft that must survive settings");
  await page.evaluate(async () => {
    const { EditorView } = await import("/node_modules/.vite/deps/@codemirror_view.js");
    const view = EditorView.findFromDOM(document.querySelector('[data-spatial-center=true] .cm-editor'));
    view.dispatch({ selection: { anchor: view.state.doc.length - 8, head: view.state.doc.length } });
    (window as any).settingsWitness = { view, text: view.state.doc.toString(), selection: view.state.selection.toJSON(), tabs: document.querySelectorAll('.spatial-center-tabs [role=tab]').length };
  });
  await page.keyboard.press("ControlOrMeta+Shift+P");
  const input = page.getByRole("textbox", { name: "Commands" });
  await input.fill(">Settings");
  await expect(page.getByRole("option", { name: /^Settings/ })).toBeVisible();
  await input.press("Enter");
  await expect(page.getByRole("heading", { name: "Settings", exact: true })).toBeVisible();
  await page.keyboard.press("ControlOrMeta+w");
  await expect(page.getByRole("heading", { name: "Settings", exact: true })).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(editor).toBeVisible();
  await expect(page).not.toHaveTitle("Settings — Hickory Docs");
  expect(await page.evaluate(async () => {
    const { EditorView } = await import("/node_modules/.vite/deps/@codemirror_view.js");
    const witness = (window as any).settingsWitness;
    const view = EditorView.findFromDOM(document.querySelector('[data-spatial-center=true] .cm-editor'));
    return view === witness.view && view.state.doc.toString() === witness.text
      && JSON.stringify(view.state.selection.toJSON()) === JSON.stringify(witness.selection)
      && document.querySelectorAll('.spatial-center-tabs [role=tab]').length === witness.tabs;
  })).toBe(true);
});

test("Save of an unnamed draft opens the save dialog and adopts the exact current bytes", async ({ page }) => {
  await page.route("**/api/save-file-dialog", route => route.fulfill({ json: { path: "workflow-saved.md" } }));
  const editor = page.locator('[data-spatial-center=true] .cm-content');
  await editor.click();
  await editor.press("ControlOrMeta+A");
  await editor.pressSequentially("# My saved notebook\n\nA saved draft.");
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "save" })));
  await expect(page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: "workflow-saved.md" })).toHaveAttribute("aria-selected", "true");
  await expect(page.locator('[data-spatial-center=true] .untitled-tab')).toHaveCount(0);
  await expect(page.locator('[data-spatial-center=true] .cm-content')).toContainText("A saved draft.");
});

test("an unnamed cell offers Save through its rail, then becomes runnable", async ({ page }) => {
  await page.route("**/api/save-file-dialog", route => route.fulfill({ json: { path: "workflow-cell.md" } }));
  await page.getByRole("button", { name: /Cell 1.*save this document to enable running/ }).click();
  const note = page.locator('[data-spatial-key="document:workflow-cell.md"]');
  await expect(note).toHaveAttribute("data-spatial-center", "true");
  await note.getByRole("button", { name: /Cell 1.*click to run it/ }).click();
  await expect(note).toContainText("Expected output matches");
});

test("opening a search result from Agent centers the note and Restore restores its command target", async ({ page }) => {
  await page.keyboard.press("ControlOrMeta+Shift+P");
  let input = page.getByRole("textbox", { name: "Commands" });
  await input.fill(">Agent");
  await expect(page.getByRole("option", { name: /Agent — show/ })).toBeVisible();
  await input.press("Enter");
  await expect(page.locator('[data-spatial-center=true]')).toHaveAttribute("data-spatial-key", /chat:/);
  await page.keyboard.press("ControlOrMeta+p");
  input = page.getByRole("textbox", { name: "Search open files and folder" });
  await input.fill("First note");
  await expect(page.getByRole("option", { name: /^note.md / })).toBeVisible();
  await input.press("Enter");
  await expect(page.locator('[data-spatial-key="document:note.md"]')).toHaveAttribute("data-spatial-center", "true");
  await page.locator('[data-spatial-center=true]').getByRole("button", { name: "Restore previous center" }).click();
  await expect(page.locator('[data-spatial-center=true]')).toHaveAttribute("data-spatial-key", /chat:/);
  await page.keyboard.press("ControlOrMeta+Shift+P");
  input = page.getByRole("textbox", { name: "Commands" });
  await input.fill(">Test document");
  await expect(page.getByText("Nothing matches.")).toBeVisible();
});

test("Save follows the selected draft when several unnamed notes are open", async ({ page }) => {
  await page.route("**/api/save-file-dialog", route => route.fulfill({ json: { path: "workflow-first.md" } }));
  const current = () => page.locator('[data-spatial-center=true] .cm-content');
  await current().click();
  await current().press("ControlOrMeta+A");
  await current().pressSequentially("# First draft\n\nSave these bytes.");
  await page.keyboard.press("ControlOrMeta+Shift+P");
  const input = page.getByRole("textbox", { name: "Commands" });
  await input.fill(">New document");
  await expect(page.getByRole("option", { name: /^New document/ })).toBeVisible();
  await input.press("Enter");
  await current().pressSequentially("# Second draft\n\nKeep this unnamed.");
  const tabs = page.getByRole("tablist", { name: "Center views" });
  await tabs.getByRole("tab", { name: "Untitled", exact: true }).click();
  await current().click();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "save" })));
  await expect(tabs.getByRole("tab", { name: "workflow-first.md" })).toHaveAttribute("aria-selected", "true");
  await expect(current()).toContainText("Save these bytes.");
  await tabs.getByRole("tab", { name: "Untitled 2", exact: true }).click();
  await expect(current()).toContainText("Keep this unnamed.");
});
</book:entry>

</book:doc>
