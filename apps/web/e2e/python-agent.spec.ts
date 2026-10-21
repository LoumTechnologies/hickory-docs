import { expect, test } from "@playwright/test";

// Guarantees: agent/python-execution-has-two-boundaries.md;
// agent/a-session-is-the-conversation.md; agent/the-agent-pane-is-a-live-document.md.
test("configure the Python agent, review its edit, recover the conversation and stop another proposal", async ({ page }) => {
  test.setTimeout(60_000);
  test.skip(!process.env.HICKORY_PYTHON_UI_FIXTURE || !process.env.HICKORY_AGENT_TEST_PYTHON, "Use scripts/test-python-agent-ui.sh with the installed runtime.");
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.goto("/#/settings");
  await page.getByRole("textbox", { name: "Python executable" }).fill(process.env.HICKORY_AGENT_TEST_PYTHON!);
  await page.getByRole("button", { name: "Save Python runtime" }).click();
  await expect(page.getByRole("status").filter({ hasText: "Runtime started in confinement" })).toBeVisible();
  const files = await (await page.request.get("/api/projects/local/docs")).json();
  const note = files.find((doc: { path: string }) => doc.path === "note.md");
  await page.goto(`/#/docs/${note.id}`);
  await expect(page.locator('[data-spatial-key="document:note.md"] .cm-content').filter({ hasText: "Original human input" })).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  const agent = page.getByRole("region", { name: "Agent chat" });
  await agent.getByRole("button", { name: "Agent settings" }).click();
  await page.getByRole("combobox", { name: "Agent", exact: true }).selectOption("python");
  await page.getByRole("combobox", { name: "Agent", exact: true }).press("Escape");
  const response = agent.getByRole("textbox", { name: "Agent conversation and response" });
  await response.click(); await response.press("ControlOrMeta+End"); await response.pressSequentially("Propose in Python");
  const sent = page.waitForRequest(request => request.method() === "POST" && /\/api\/docs\/[^/]+\/agent$/.test(new URL(request.url()).pathname));
  await agent.getByRole("button", { name: "Send", exact: true }).click();
  const turnsPath = `${new URL((await sent).url()).pathname}/turns`;
  const review = page.getByRole("region", { name: "Document change review" });
  await page.getByRole("button", { name: "Show left drawer" }).click();
  await expect(review).toBeVisible();
  await expect(review.locator(".comparison-added")).toContainText("Python agent proposal");
  expect((await (await page.request.get(`/api/docs/${note.id}`)).json()).source).toBe("# Original human input\n");
  await expect(agent.getByRole("combobox", { name: "Document edits" })).toBeDisabled();
  expect(await agent.getByRole("option", { name: "Auto-accept" }).count()).toBe(0);
  await page.screenshot({ path: "/tmp/hickory-python-agent-review.png", fullPage: true });
  await review.getByRole("button", { name: "Accept change" }).click();
  await expect.poll(async () => (await (await page.request.get(`/api/docs/${note.id}`)).json()).source).toBe("# Python agent proposal\n# Original human input\n");
  await expect(agent.getByRole("button", { name: "Stop the agent" })).toHaveCount(0);
  const history = await (await page.request.get(turnsPath)).json();
  expect(history.turns[0].backend).toBe("python");
  expect(history.turns[0].status).toBe("ok");
  expect(history.turns[0].answer).toContain("Changeset verified (source-output equality)");
  expect(history.turns[0].answer).toContain("runtime checks unrun.");
  await page.reload();
  // Center tabs expose the restored source independently of the review panes.
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: "note.md", exact: true }).click();
  // The menu listener belongs to the asynchronously loaded workspace, not the shell.
  await expect(page.locator('[data-spatial-key="document:note.md"] .cm-content').filter({ hasText: "Original human input" })).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: "Agent", exact: true }).click();
  await agent.getByRole("button", { name: "Agent settings" }).click();
  await expect(page.getByRole("combobox", { name: "Agent", exact: true })).toHaveValue("python");
  await response.click(); await response.press("ControlOrMeta+End"); await response.pressSequentially("/new");
  await agent.getByRole("button", { name: "Send", exact: true }).click();
  await response.click(); await response.press("ControlOrMeta+End"); await response.pressSequentially("Cancel this Python proposal");
  await agent.getByRole("button", { name: "Send", exact: true }).click();
  await page.getByRole("button", { name: "Show left drawer" }).click();
  await expect(review.locator(".comparison-added").filter({ hasText: "This needs approval" })).toBeVisible();
  // Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
  // A completed response must not paint over the next pending review. Trial
  // performs real pointer hit testing without deciding the proposal we cancel.
  await review.getByRole("button", { name: "Reject change" }).click({ trial: true, timeout: 10000 });
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: "Agent", exact: true }).click();
  await agent.getByRole("button", { name: "Stop the agent" }).click();
  await expect.poll(async () => {
    const turns = await (await page.request.get(turnsPath)).json();
    return turns.turns.at(-1).status;
  }).toBe("stopped");
  expect((await page.request.get("/api/file?path=cancelled.md")).ok()).toBe(false);
  expect(errors).toEqual([]);
});
