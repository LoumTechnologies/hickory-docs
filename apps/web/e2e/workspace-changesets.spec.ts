// Guarantee: docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md
import { expect, test } from "@playwright/test";

// Real browser + HTTP + ACP subprocess: authoritative untitled bytes and repair.
test("untitled edits complete as a captured changeset without saving", async ({ page }) => {
  test.setTimeout(120_000);
  test.skip(!process.env.HICKORY_CHANGESET_BROWSER, "Use just test-workspace-changesets-browser.");
  const errors: string[] = [];
  page.on("pageerror", e => errors.push(e.message));
  await page.goto("/");
  const editor = page.locator(".untitled-tab .cm-content");
  await expect(editor).toBeVisible({ timeout: 60_000 });
  await editor.click(); await editor.press("ControlOrMeta+a");
  await editor.pressSequentially("Authoritative unsaved A");
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  const agent = page.getByRole("region", { name: "Agent chat" });
  await agent.getByRole("button", { name: "Agent settings" }).click();
  await page.getByRole("combobox", { name: "Agent", exact: true }).selectOption("fixture");
  await page.getByRole("combobox", { name: "Agent", exact: true }).press("Escape");
  const response = agent.getByRole("textbox", { name: "Agent conversation and response" });
  await response.click(); await response.press("ControlOrMeta+End");
  await response.pressSequentially("contract-buffer");
  const sent = page.waitForRequest(r => r.method() === "POST" && /\/api\/docs\/workspace\/agent$/.test(new URL(r.url()).pathname));
  await agent.getByRole("button", { name: "Send", exact: true }).click();
  const request = await sent;
  expect(request.postDataJSON().context.buffers[0].content).toBe("Authoritative unsaved A");
  await agent.getByRole("button", { name: "Accept change", exact: true }).click();
  await expect(agent.getByRole("region", { name: "Final literate changeset" })).toBeVisible({ timeout: 30_000 });
  await expect(agent.getByRole("region", { name: "Final literate changeset" })).toContainText("Edited by the agent");
  await expect(agent.getByRole("button", { name: "Apply to literate source" })).toBeDisabled();
  const turns = await (await page.request.get("/api/docs/workspace/agent/turns")).json();
  const turn = turns.turns.at(-1);
  expect(turn.status).toBe("ok");
  const capture = await (await page.request.get(`/api/docs/workspace/agent/projects/${turn.id}`)).json();
  const values = (snapshot: { entries: Record<string,{bytes:number[]}> }) => Object.entries(snapshot.entries)
    .filter(([path]) => path.startsWith("__hickory_editor_buffers__/"))
    .map(([,entry]) => new TextDecoder().decode(new Uint8Array(entry.bytes)));
  expect(values(capture.starting)).toContain("Authoritative unsaved A");
  expect(values(capture.completed)).toContain("Edited by the agent");
  expect(capture.workspace.root === null).toBe(process.env.HICKORY_CHANGESET_FOLDER === "false");
  expect((await page.request.get("/api/file?path=Untitled.md")).ok()).toBe(false);
  expect(errors).toEqual([]);
});
