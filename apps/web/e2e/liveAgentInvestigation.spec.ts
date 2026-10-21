import { test, expect } from "@playwright/test";

// Explicit opt-in only: this makes paid provider calls against a disposable
// native engine. It is ordinary mode, not a #77 gate or packaged-shell witness.
test.use({ video: "on", viewport: { width: 1400, height: 950 } });
test("real attached Python agent requests a source-bound check and inspects its actual failure", async ({ page }, info) => {
  test.setTimeout(240000);
  test.skip(!process.env.HICKORY_LIVE_PROVIDER_WITNESS || !process.env.HICKORY_AGENT_TEST_PYTHON, "Explicit live-provider witness; never enabled by ordinary CI.");
  const queued: { job: string; requested_source_hash: string }[] = [];
  page.on("request", request => {
    if (request.method() !== "POST" || !/\/api\/v1\/ui\/windows\/[^/]+\/reply$/.test(new URL(request.url()).pathname)) return;
    const reply = request.postDataJSON()?.result;
    if (reply?.status === "result" && reply.outcome?.status === "queued") queued.push({ job: reply.outcome.job, requested_source_hash: reply.outcome.value?.requested_source_hash });
  });
  await page.addInitScript(() => localStorage.setItem("hickory.showWelcome", "0"));
  await page.goto("/#/settings");
  await page.getByRole("textbox", { name: "Python executable" }).fill(process.env.HICKORY_AGENT_TEST_PYTHON!);
  await page.getByRole("button", { name: "Save Python runtime" }).click();
  await expect(page.getByRole("status").filter({ hasText: "Runtime started in confinement" })).toBeVisible();
  const docs = await (await page.request.get("/api/projects/local/docs")).json();
  const doc = docs.find((entry: { path: string }) => entry.path === "deadline-spec.md"); expect(doc).toBeTruthy();
  await page.goto(`/#/docs/${doc.id}`);
  await expect(page.locator('[data-spatial-key="document:deadline-spec.md"] .cm-content')).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  const chat = page.getByRole("region", { name: "Agent chat" });
  await chat.getByRole("button", { name: "Agent settings" }).click();
  await page.getByRole("combobox", { name: "Agent", exact: true }).selectOption("python");
  await page.getByRole("combobox", { name: "Provider", exact: true }).selectOption("openrouter");
  await page.getByRole("combobox", { name: "Agent", exact: true }).press("Escape");
  const response = chat.getByRole("textbox", { name: "Agent conversation and response" });
  await response.click(); await response.press("ControlOrMeta+End");
  await response.pressSequentially('Investigate the seeded total-deadline bug before editing. Read deadline-spec.md and its generated code. Use ui_observe to find the document-execution group, invoke its exact current Verify saved document handle ONCE, then read Read latest document run receipt until the actual job is terminal. Explain the two 80ms attempts under one total 100ms budget using that source-bound failed receipt. No source edits, no approvals, no human attestations. The completion gate may create only its explanatory response artifact; that is not runtime verification. Return only the declared raw JSON action objects.');
  const sent = page.waitForRequest(request => request.method() === "POST" && /\/api\/docs\/[^/]+\/agent$/.test(new URL(request.url()).pathname));
  await chat.getByRole("button", { name: "Send", exact: true }).click();
  const path = new URL((await sent).url()).pathname;
  try {
    await expect.poll(async () => (await (await page.request.get(`${path}/turns`)).json()).turns.at(-1)?.status ?? "",
      { timeout: 200000, intervals: [1000, 2000] }).toMatch(/^(ok|error|stopped)$/);
    const history = await (await page.request.get(`${path}/turns`)).json();
    await info.attach("ordinary-live-provider-turn", { body: JSON.stringify(history, null, 2), contentType: "application/json" });
    expect(history.turns.at(-1)?.backend).toBe("python");
    expect(history.turns.at(-1)?.status).toBe("ok");
    expect(queued.length, "The real provider must request a shared source-bound document check").toBeGreaterThan(0);
    const actual = await (await page.request.get(`/api/runs/${queued[0].job}`)).json();
    await info.attach("actual-source-bound-run", { body: JSON.stringify(actual, null, 2), contentType: "application/json" });
    expect(actual.status).toBe("failed");
    expect(JSON.stringify(actual)).toContain("160");
    expect(actual.source_hash.replace(/^sha256:/, "")).toBe(queued[0].requested_source_hash);
    expect(history.turns.at(-1)?.answer).toContain(queued[0].job);
  } finally {
    await page.request.post(`${path}/stop`).catch(() => {});
  }
});
