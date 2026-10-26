<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/e2e/liveAgentInvestigation.spec.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3387bba4ed15"></a>
## `apps/web/e2e/liveAgentInvestigation.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-3387bba4ed15-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-3387bba4ed15-expression_statement">
import { test, expect } from "@playwright/test";

// Explicit opt-in only: this makes paid provider calls against a disposable
// native engine. It is ordinary mode, not a #77 gate or packaged-shell witness.
test.use({ video: "on", viewport: { width: 1400, height: 950 } });
</storycode:copy>
<a id="f-3387bba4ed15-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-3387bba4ed15-expression_statement-2">
test("real attached Python agent investigates, proposes a reviewed correction and verifies it", async ({ page }, info) => {
  test.setTimeout(480000);
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
  await response.pressSequentially('Investigate the seeded total-deadline bug before editing. Read deadline-spec.md and its generated code. Use ui_observe to find the document-execution group, invoke its exact current Verify saved document handle ONCE, then read Read latest document run receipt until the actual job is terminal. Explain the two 80ms attempts under one total 100ms budget using that source-bound failed receipt. No source edits, no approvals, no human attestations. Do not call edit_doc, edit_output or create_doc for the explanation: those preview tools wait for human approval. Finish using changeset_status and submit_changeset: decode content[0].text as JSON, use the returned boundary, expected, entry and template, add your explanation to that template while preserving its project construction, and submit sources keyed by entry. Only that completion path may create the explanatory response artifact; that is not runtime verification. Return only the declared raw JSON action objects.');
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

    // The test driver acts as the reviewer of one exact declared patch. This
    // exercises the actual review UI; it is not a claim of human judgment.
    // Opening the native editor can normalize paragraph wrapping. Review the
    // actual saved baseline after investigation rather than the seed layout.
    const original = (await (await page.request.get(`/api/docs/${doc.id}`)).json()).source as string;
    const corrected = original.replace("var allowed = budget;", "var allowed = Math.max(0, budget - total);");
    expect(corrected).not.toBe(original);
    await response.click(); await response.press("ControlOrMeta+End");
    await response.pressSequentially('Now propose exactly one focused source correction in deadline-spec.md: replace var allowed = budget; with var allowed = Math.max(0, budget - total); and change nothing else. Read the current complete document and use edit_doc with fresh snapshot/anchors. The test driver will review that exact preview through the normal review UI; do not approve it yourself or use editor writes to bypass review. After acceptance, invoke the shared Verify saved document action, read its actual current-source terminal run receipt, and report its job ID and scope. Preserve all expectations, short and empty checks. Finish through changeset_status and submit_changeset using the returned template and identities, not another preview tool for your explanation. Return only raw JSON action objects.');
    await chat.getByRole("button", { name: "Send", exact: true }).click();
    let proposal: { id: string; oldText: string; newText: string } | undefined;
    await expect.poll(async () => {
      const state = await (await page.request.get(`${path}/python`)).json();
      proposal = state.edits?.changes.find((change: { status: string }) => change.status === "pending");
      return !!proposal;
    }, { timeout: 160000, intervals: [1000, 2000] }).toBe(true);
    expect(proposal!.oldText).toBe(original);
    expect(proposal!.newText).toBe(corrected);
    await chat.getByRole("button", { name: "Review in document", exact: true }).last().click({ timeout: 15000 });
    const review = page.getByRole("region", { name: "Document change review" });
    await expect(review).toBeVisible();
    await review.getByRole("button", { name: "Accept change" }).click();
    await expect.poll(async () => (await (await page.request.get(`${path}/turns`)).json()).turns.at(-1)?.status ?? "",
      { timeout: 200000, intervals: [1000, 2000] }).toMatch(/^(ok|error|stopped)$/);
    const repairedHistory = await (await page.request.get(`${path}/turns`)).json();
    expect(repairedHistory.turns.at(-1)?.status).toBe("ok");
    expect((await (await page.request.get(`/api/docs/${doc.id}`)).json()).source).toBe(corrected);
    expect(queued.length).toBeGreaterThan(1);
    const repaired = await (await page.request.get(`/api/runs/${queued.at(-1)!.job}`)).json();
    expect(repaired.status).toBe("ok");
    expect(repaired.source_hash.replace(/^sha256:/, "")).toBe(queued.at(-1)!.requested_source_hash);
    expect(repaired.source_hash).not.toBe(actual.source_hash);
    expect(repairedHistory.turns.at(-1)?.answer).toContain(queued.at(-1)!.job);
    await info.attach("ordinary-live-provider-correction", { body: JSON.stringify({ history: repairedHistory, run: repaired, reviewer: "test-driver exact-patch assertion; human judgment not supplied" }, null, 2), contentType: "application/json" });
  } finally {
    await page.request.post(`${path}/stop`).catch(() => {});
  }
});
</storycode:copy>

### Assemble `apps/web/e2e/liveAgentInvestigation.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/liveAgentInvestigation.spec.ts"><storycode:paste select="#f-3387bba4ed15-expression_statement"/><storycode:paste select="#f-3387bba4ed15-expression_statement-2"/></storycode:file>

</storycode:doc>
