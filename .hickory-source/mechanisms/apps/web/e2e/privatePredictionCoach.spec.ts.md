<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/e2e/privatePredictionCoach.spec.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c4a491bb0b6d"></a>
## `apps/web/e2e/privatePredictionCoach.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../guarantees/verification/predictions-precede-managed-observation/contract.md), [Private edit proposals open review before writing](../../../../guarantees/verification/private-proposals-wait-for-review/contract.md).

<a id="f-c4a491bb0b6d-ambient_declaration"></a>
### `ambient_declaration`

<storycode:copy id="f-c4a491bb0b6d-ambient_declaration">
import { expect, test } from "@playwright/test";
declare const process: { env: Record<string, string | undefined> };
</storycode:copy>
<a id="f-c4a491bb0b6d-expression_statement"></a>
### `expression_statement`

Actual daemon/worker/process/browser; provider and review decisions are scripted.

<storycode:copy id="f-c4a491bb0b6d-expression_statement">
// Guarantee: verification/predictions-precede-managed-observation.md.
// Actual daemon/worker/process/browser; provider and review decisions are scripted.
test("private coach retains failure and reviewed repair without publishing target answers", async ({ page }) => {
  const live = process.env.HICKORY_PRIVATE_LIVE_PROVIDER === "1";
  test.setTimeout(live ? 600_000 : 60_000);
  test.skip(!process.env.HICKORY_PRIVATE_UI_FIXTURE, "Use scripts/test-private-prediction-ui.sh.");
  const errors: string[] = [], paths: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  page.on("request", request => { const path = new URL(request.url()).pathname; if (path.startsWith("/api/")) paths.push(path); });
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Predict, investigate and review" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Delegate investigation" })).toBeEnabled();
  // Real keyboard events must retain every character before any provider call.
  const providerField = page.getByRole("textbox", { name: "Provider", exact: true });
  await providerField.pressSequentially("deepseek", { delay: 0 });
  await expect(providerField).toHaveValue("deepseek");
  await providerField.fill("");
  const unsealed = await (await page.request.post("/api/prediction/debugger", { data: { name: "continue_debugger_prediction", arguments: {} } })).json();
  expect(unsealed.isError).toBe(true);
  const stale = await (await page.request.post("/api/prediction/debugger", { data: { name: "seal_debugger_prediction", arguments: { expected_source_hash: "0".repeat(64), predicted: { kind: "equals", value: 100 }, rationale: "Stale source must refuse before launch" } } })).json();
  expect(stale.isError).toBe(true);
  expect((await (await page.request.get("/api/prediction/history")).json()).attempts).toHaveLength(0);
  if (process.env.HICKORY_PRIVATE_ACP_FIXTURE === "1") {
    await page.getByRole("textbox", { name: "ACP adapter (optional)" }).fill("private-qa");
  }
  if (live) {
    if (process.env.HICKORY_PRIVATE_ACP_FIXTURE !== "1") {
      await page.getByRole("textbox", { name: "Provider", exact: true }).fill(process.env.HICKORY_PRIVATE_PROVIDER ?? "openrouter");
      if (process.env.HICKORY_PRIVATE_MODEL) await page.getByRole("textbox", { name: "Model", exact: true }).fill(process.env.HICKORY_PRIVATE_MODEL);
    }
    await page.getByRole("textbox", { name: "Investigation request" }).fill("Run this coached prediction exercise. First read the current captured source. Before making any edit, use seal_debugger_prediction with equals 100 and a rationale, then continue_debugger_prediction to launch the first stop. While paused, seal_debugger_prediction with equals 20 and a rationale before continue_debugger_prediction resumes to the second stop. These are required expectations, not observed values. Also seal the specified expected behavior as a Fields predicate: output equals {long:100,short:30,empty:0}, debugger equals 20. This is the required behavior, not a runtime observation. Verify the current implementation and retain its non-passing result. Then investigate from source and the requirement with target answers withheld. Propose a minimal correction to retry's per-attempt allowance for human review. After approval, reread the repaired source, repeat the same initial/future debugger expectations (100 before launch, 20 before Continue) through the retained tools, and reseal exactly the same expected behavior against its new source hash. Verify the repair and finish with a concise explanation and limitations. Do not change the capture selection, expectation tags, inputs or requirement. Do not print, reveal or fetch actual runtime target values.");
  } else if (process.env.HICKORY_PRIVATE_ACP_FIXTURE !== "1") await page.getByRole("textbox", { name: "Provider", exact: true }).fill("openai");
  const sending = page.waitForRequest(request => request.method() === "POST" && new URL(request.url()).pathname === "/api/prediction/agent");
  const sent = page.waitForResponse(response => response.request().method() === "POST" && new URL(response.url()).pathname === "/api/prediction/agent");
  await page.getByRole("button", { name: "Delegate investigation" }).click();
  expect((await sent).ok(), await (await sent).text()).toBe(true);
  if (live && process.env.HICKORY_PRIVATE_ACP_FIXTURE !== "1" && process.env.HICKORY_PRIVATE_MODEL) expect((await sending).postDataJSON()).toMatchObject({ model: process.env.HICKORY_PRIVATE_MODEL });
  else await sending;
  const review = page.getByRole("region", { name: "Document change review" });
  await expect.poll(async () => {
    const current = await (await page.request.get("/api/prediction/agent")).json();
    return current.edits.changes.some((change: { status: string }) => change.status === "pending") ? "pending" : current.run.status;
  }, { timeout: live ? 240_000 : 20_000 }).not.toBe("running");
  const arrived = await (await page.request.get("/api/prediction/agent")).json();
  expect(arrived.run.status, arrived.run.answer ?? "Investigation must reach a real review").toBe("running");
  await expect(review).toBeVisible();
  const before = await (await page.request.get("/api/prediction/history")).json();
  if (live) {
    expect(before.attempts.length).toBeGreaterThanOrEqual(3);
    expect(before.attempts.length).toBeLessThanOrEqual(12);
    expect(before.attempts.slice(2).every((attempt: { original_requirement_satisfied: boolean }) => !attempt.original_requirement_satisfied)).toBe(true);
  } else expect(before.attempts).toHaveLength(3);
  expect(before.attempts[0].prediction_matched).toBe(true);
  expect(before.attempts[1].classification).toBe("mismatch");
  expect(before.attempts[2].classification).toBe("execution_rejected");
  expect((await page.request.get("/api/files")).status()).toBe(422);
  // A real model can submit a second formatting cleanup. Review every exact
  // proposal, bounded to three; only the intended repair/blank-line cleanup is
  // authorized by this automated reviewer. This is not human judgment.
  const normalizeRepair = (source: string) => source
    .replaceAll("max(0, budget - elapsed)", "budget - elapsed")
    .replaceAll("max(budget - elapsed, 0)", "budget - elapsed")
    .split("\n").filter(line => line.trim() !== "").map(line => line.trimEnd()).join("\n");
  const first = arrived.edits.changes.find((change: { status: string }) => change.status === "pending");
  const expectedRepair = normalizeRepair(first.oldText.replace("allowance = budget", "allowance = budget - elapsed"));
  const reviewed = new Set<string>();
  await expect.poll(async () => {
    const current = await (await page.request.get("/api/prediction/agent")).json();
    const pending = current.edits.changes.find((change: { status: string }) => change.status === "pending");
    if (pending && !reviewed.has(pending.id)) {
      expect(reviewed.size).toBeLessThan(3);
      expect(normalizeRepair(pending.newText)).toBe(expectedRepair);
      await review.getByRole("button", { name: "Accept change" }).click();
      reviewed.add(pending.id);
    }
    return current.run.status;
  }, { timeout: live ? 240_000 : 20_000 }).toBe("finished");
  const after = await (await page.request.get("/api/prediction/history")).json();
  if (live) {
    expect(after.attempts.length).toBeGreaterThanOrEqual(before.attempts.length + 3);
    expect(after.attempts.length).toBeLessThanOrEqual(15);
  } else expect(after.attempts).toHaveLength(6);
  const corrected = before.attempts.slice(0, 3).map((original: { scenario: string }) =>
    after.attempts.slice(before.attempts.length).find((attempt: { scenario: string; prediction_matched: boolean; original_requirement_satisfied: boolean }) =>
      attempt.scenario === original.scenario && attempt.prediction_matched && attempt.original_requirement_satisfied));
  expect(corrected.every(Boolean)).toBe(true);
  expect(corrected[1].classification).toBe("verified_after_correction");
  expect(corrected[2].classification).toBe("verified_after_correction");
  const allSeals = after.ordering.filter((entry: { event: { kind: string } }) => entry.event.kind === "sealed");
  // Unused/replaced seals are retained too. Bind comparisons to the seals that
  // actually admitted each attempt, rather than assuming exactly six seals.
  const seals = [...before.attempts.slice(0, 3), ...corrected].map((attempt: { seal: string }) => allSeals.find((entry: { event: { id: string } }) => entry.event.id === attempt.seal));
  expect(seals.every(Boolean)).toBe(true);
  for (let index = 0; index < 3; index++) {
    expect(seals[index].event.seal.predicted).toEqual(seals[index + 3].event.seal.predicted);
    expect(seals[index].event.seal.scenario.requirement).toEqual(seals[index + 3].event.seal.scenario.requirement);
  }
  expect(seals[1].event.future_stop.hit).toBe(1);
  expect(seals[4].event.future_stop.hit).toBe(1);
  expect(seals[1].event.future_stop.prior_visible_digest).toMatch(/^[a-f0-9]{64}$/);
  expect(seals[1].sequence).toBeLessThan(after.ordering.find((entry: { event: { kind: string; seal: string } }) => entry.event.kind === "started" && entry.event.seal === seals[1].event.id).sequence);
  expect(after.attempts.every((attempt: { current_certification: boolean }) => !attempt.current_certification)).toBe(true);
  const revalidated = page.waitForResponse(response => response.request().method() === "POST" && new URL(response.url()).pathname === "/api/prediction/promote");
  await page.getByRole("button", { name: "Revalidate and promote", exact: true }).click();
  const promotion = await (await revalidated).json();
  expect(promotion.summary.current_certification).toBe(true);
  expect(promotion.summary.promoted).toBe(true);
  await page.reload();
  await expect(page.getByText(/The repair passed the unchanged prediction/)).toBeVisible();
  expect(paths.filter(path => path !== "/api/files").every(path => path.startsWith("/api/prediction/") || path.startsWith("/api/v1/ui/windows"))).toBe(true);
  const captured = await (await page.request.post("/api/prediction/tool", { data: { name: "read_doc", args: [] } })).json();
  const changedInputs = captured.source.replace("retry([10, 20], 100)", "retry([11, 20], 100)");
  expect(changedInputs).not.toBe(captured.source);
  const edited = await (await page.request.post("/api/prediction/tool", { data: { name: "edit_doc", args: [["whole", "true"], ["snapshot", captured.source_hash]], input: changedInputs } })).json();
  expect(edited.ok).toBe(true);
  const changed = await (await page.request.get("/api/prediction/state")).json();
  const refused = await page.request.post("/api/prediction/seal", { data: { expected_source_hash: changed.source_hash, predicted: seals[5].event.seal.predicted, rationale: "Attempt to change the host inputs" } });
  expect(refused.status()).toBe(422);
  expect((await page.request.post("/api/prediction/promote", { data: { attempt: corrected[2].attempt } })).status()).toBe(409);
  expect((await (await page.request.get("/api/prediction/history")).json()).attempts).toHaveLength(after.attempts.length);
  await expect(page.getByRole("button", { name: "Revalidate and promote", exact: true })).toBeDisabled();
  await page.getByRole("button", { name: "End private practice", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Private practice ended" })).toBeVisible();
  expect(errors).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/e2e/privatePredictionCoach.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/privatePredictionCoach.spec.ts"><storycode:paste select="#f-c4a491bb0b6d-ambient_declaration"/><storycode:paste select="#f-c4a491bb0b6d-expression_statement"/></storycode:file>

</storycode:doc>
