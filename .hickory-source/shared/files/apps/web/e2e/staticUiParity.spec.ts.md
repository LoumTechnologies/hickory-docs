<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/e2e/staticUiParity.spec.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [Browser-local prediction practice uses the shared gate](../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md)
- [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md)

Captured SHA-256: `2d7af96b6b46d06358e2721561afc42ea4a7256d1f886195324afc3eab294b2f`.

<book:entry path="apps/web/e2e/staticUiParity.spec.ts">
import { test, expect, type Page } from "@playwright/test";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
// Guarantees: docs/guarantees/agent/ui-action-dispatch.md
// Real Chrome, origin channel, parser and interpreter; host requests are scripted.
async function connect(page: Page, surface: "site" | "snapshot" | "embed", query = "") {
  const parentOrigin = new URL(process.env.HICKORY_E2E_URL ?? "http://localhost:4178").origin;
  const child = new URL(parentOrigin); child.hostname = child.hostname === "localhost" ? "127.0.0.1" : "localhost";
  const childOrigin = child.origin;
  const engine: string[] = [];
  await page.route(url => url.pathname.startsWith("/api/"), route => { engine.push(route.request().url()); return route.abort(); });
  await page.goto(`${parentOrigin}/snapshot.html`);
  await page.evaluate(({ surface, childOrigin, query }) => {
    const frame = document.createElement("iframe"); frame.id = "static-surface";
    const origin = childOrigin;
    frame.src = `${origin}/${surface}.html?${query ? `${query}&` : ""}parentOrigin=${encodeURIComponent(location.origin)}`;
    Object.assign(window, { staticReady: null });
    window.addEventListener("message", event => {
      if (event.source === frame.contentWindow && event.origin === origin && event.data.channel === "hickory-static-ui" && event.data.event === "ready") Object.assign(window, { staticReady: event.data.data.scope });
    });
    document.body.prepend(frame);
  }, { surface, childOrigin, query });
  await expect.poll(() => page.evaluate(() => (window as any).staticReady)).toBeTruthy();
  const invoke = (method: string, args: Record<string, unknown>, owner?: string) => page.evaluate(({ method, args, owner, childOrigin }) => {
    const frame = document.getElementById("static-surface") as HTMLIFrameElement;
    const origin = childOrigin, scope = (window as any).staticReady;
    return new Promise<any>((resolve, reject) => {
      const id = crypto.randomUUID(), timeout = setTimeout(() => { window.removeEventListener("message", receive); reject(Error("Static UI reply timed out")); }, 5000);
      function receive(event: MessageEvent) {
        if (event.source !== frame.contentWindow || event.origin !== origin || event.data.channel !== "hickory-static-ui" || event.data.id !== id) return;
        clearTimeout(timeout); window.removeEventListener("message", receive); resolve(event.data);
      }
      window.addEventListener("message", receive);
      frame.contentWindow!.postMessage({ channel: "hickory-static-ui", version: 1, op: "ui", id, data: { window: owner ?? scope.window, method, arguments: args } }, origin);
    });
  }, { method, args, owner, childOrigin });
  const actions = async (parent: string) => (await invoke("ui_observe", { parent, limit: 64 })).data.result.actions as any[];
  const act = async (parent: string, label: string, args = {}) => {
    const action = (await actions(parent)).find(action => action.label === label);
    expect(action, label).toBeTruthy();
    const result = (await invoke("ui_act", { handle: action.handle, arguments: args, request_id: crypto.randomUUID() })).data.result;
    expect(result, label).toMatchObject({ status: "result", outcome: { status: "completed" } });
    return result;
  };
  return { engine, invoke, actions, act, frame: page.frameLocator("#static-surface") };
}
test("standalone publication imports through scoped actions without an implicit engine", async ({ page }) => {
  const { invoke, act, frame, engine } = await connect(page, "snapshot");
  const source = "# Actual browser snapshot\n";
  const snapshot = { format: "hickory-viewer-snapshot", version: 1, path: "public.md", source, source_hash: createHash("sha256").update(source).digest("hex"), execution: "Historical fixture", captured_at_ms: 1, cells: [] };
  expect(await act("publication-viewer", "Import exported snapshot JSON", { json: JSON.stringify(snapshot) })).toMatchObject({ status: "result", outcome: { value: { publication_loaded: true, historical: true } } });
  await expect(frame.getByRole("heading", { name: "public.md", exact: true })).toBeVisible();
  await expect(frame.getByRole("group", { name: "public.md" })).toContainText("Actual browser snapshot");
  expect((await invoke("ui_observe", {}, "wrong-window")).event).toBe("error");
  expect(engine).toEqual([]);
});
test("homepage host actions advance only after real browser counterexample and corrected execution", async ({ page }) => {
  const { invoke, act, frame, engine } = await connect(page, "site");
  await expect(frame.getByRole("heading", { name: "Hickory Docs", exact: true })).toBeVisible();
  await act("browser-tour", "Set browser workflow tour visibility", { open: true });
  await act("browser-tour", "Restart browser workflow tour");
  await act("browser-tour", "Continue browser workflow step"); await act("browser-tour", "Continue browser workflow step");
  await act("browser-tour", "Set browser workflow practice objection", { text: "Retries consume the same 100ms budget" });
  await act("browser-tour", "Continue browser workflow step");
  const tour = frame.getByRole("region", { name: "AI engineer tour" });
  await expect(tour.getByRole("button", { name: "Continue", exact: true })).toBeDisabled();
  async function run() {
    const root = (await invoke("ui_observe", { limit: 64 })).data.result;
    const parent = root.groups.find((group: string) => group.startsWith("browser-document:deadline-spec.md:"));
    expect(parent).toBeTruthy();
    await act(parent, "Restart browser document (deadline-spec.md)");
  }
  await run();
  await expect(tour.getByLabel("Program output")).toContainText("160");
  await expect(tour.getByRole("button", { name: "Continue", exact: true })).toBeEnabled();
  await act("browser-tour", "Continue browser workflow step");
  await expect(tour.getByRole("button", { name: "Continue", exact: true })).toBeDisabled();
  await run();
  await expect(tour.getByLabel("Program output")).toContainText("100");
  await expect(tour.getByRole("button", { name: "Continue", exact: true })).toBeEnabled();
  await act("browser-tour", "Continue browser workflow step");
  await expect(tour.getByRole("heading", { name: "Review and retain evidence" })).toBeVisible();
  expect(engine).toEqual([]);
});

test("embedding host controls use the scoped parent registry and retire stale source handles", async ({ page }) => {
  const { act, actions, invoke, frame, engine } = await connect(page, "embed");
  const editor = frame.getByRole("group", { name: "first.md", exact: true }).locator(".cm-content");
  await expect(editor).toContainText("ordinary note");
  const old = (await actions("embedding-fixture")).find(action => action.label === "Toggle read-only");
  await act("embedding-fixture", "Replace source");
  expect((await invoke("ui_act", { handle: old.handle, arguments: {}, request_id: "stale-fixture-source" })).data.result.status).toBe("stale");
  await act("embedding-fixture", "Save explicitly");
  await expect(frame.getByLabel("Saved document")).toContainText("Host replacement");
  await act("embedding-fixture", "Toggle read-only");
  await expect(editor).toHaveAttribute("contenteditable", "false");
  await act("embedding-fixture", "Mount/unmount"); await expect(editor).toHaveCount(0);
  await act("embedding-fixture", "Mount/unmount"); await expect(editor).toContainText("Host replacement");
  expect(engine).toEqual([]);
});
test("graph controls stay discoverable through the actual parent registry", async ({ page }) => {
  const { invoke, act, frame, engine } = await connect(page, "embed", "graph");
  await expect(frame.getByTestId("graph-node-api")).toBeVisible();
  const groups = (await invoke("ui_observe", { limit: 64 })).data.result.groups;
  const disclosed = await Promise.all(groups.map(async (parent: string) => ({ parent, actions: (await invoke("ui_observe", { parent, limit: 64 })).data.result.actions })));
  const graph = disclosed.find((group: { actions: { label: string }[] }) => group.actions.some(action => action.label === "Move graph node api"));
  expect(graph).toBeTruthy();
  await act(graph.parent, "Move graph node api", { x: 160, y: 80 });
  await expect.poll(async () => JSON.parse(await frame.getByLabel("Graph document source").innerText()).layout.api).toMatchObject({ x: 160, y: 80 });
  await act("embedding-fixture", "Agent move API");
  await expect.poll(async () => JSON.parse(await frame.getByLabel("Graph document source").innerText()).layout.api).toMatchObject({ x: 300, y: 96 });
  expect(engine).toEqual([]);
});
test("profile artifact import and stale replay use parent actions without execution", async ({ page }) => {
  const { invoke, act, frame, engine } = await connect(page, "embed", "profile");
  const record = JSON.parse(await readFile(resolve("src/debug/fixtures/python-profile.json"), "utf8"));
  expect(await act("profile-fixture", "Import retained profile JSON", { json: JSON.stringify(record) })).toMatchObject({ outcome: { value: { loaded: true, executed: false, historical: true } } });
  await expect(frame.getByRole("group", { name: record.sources[0].uri, exact: true })).toBeVisible();
  await act("profile-fixture", "Inspect selected profile context");
  await expect(frame.getByLabel("Selected profile context")).toHaveText("[]");
  await act("profile-fixture", "Edit after captured profile");
  await expect(frame.getByRole("region", { name: "Runtime run history" })).toContainText("source changed (stale)");
  const groups = (await invoke("ui_observe", { limit: 64 })).data.result.groups as string[];
  const history = groups.find(group => group.startsWith("runtime-history:"))!;
  await act(history, "Pin retained run");
  await expect(frame.getByRole("button", { name: "Unpin run", exact: true })).toBeVisible();
  await act(history, "Delete retained runtime run");
  await expect(frame.getByRole("group", { name: record.sources[0].uri, exact: true })).toHaveCount(0);
  expect(engine).toEqual([]);
});

test("image reading reaches the scoped parent with real pixels", async ({ page }) => {
  const { invoke, act, frame, engine } = await connect(page, "embed", "picture");
  await expect(frame.getByAltText("red-pixel.png")).toBeVisible();
  const groups = (await invoke("ui_observe", { limit: 64 })).data.result.groups as string[];
  const disclosed = await Promise.all(groups.map(async parent => ({ parent, actions: (await invoke("ui_observe", { parent, limit: 64 })).data.result.actions })));
  const image = disclosed.find(group => group.actions.some((action: { label: string }) => action.label === "Read displayed image red-pixel.png"));
  expect(image).toBeTruthy();
  const result = await act(image!.parent, "Read displayed image red-pixel.png", { max_dimension: 2048 });
  const value = result.outcome.value;
  expect(value.width).toBe(1); expect(value.height).toBe(1);
  expect(value.ui_image.digest).toBe(createHash("sha256").update(Buffer.from(value.ui_image.data, "base64")).digest("hex"));
  expect(engine).toEqual([]);
});
test("review fixture context inspection shares its native handler", async ({ page }) => {
  const { act, frame, engine } = await connect(page, "embed", "review");
  await expect(frame.getByRole("heading", { name: "Review acceptance workspace" })).toBeVisible();
  await act("review-fixture", "Inspect selected GitHub context");
  await expect(frame.getByLabel("Selected GitHub context")).toHaveText("[]");
  expect(engine).toEqual([]);
});

test("host-supplied embedded image pixels reach the scoped parent as a real PNG", async ({ page }) => {
  const { invoke, act, frame, engine } = await connect(page, "embed");
  await expect(frame.getByRole("img", { name: "images/mark.svg", exact: true })).toBeVisible();
  const groups = (await invoke("ui_observe", { limit: 64 })).data.result.groups;
  const disclosed = await Promise.all(groups.map(async (parent: string) => ({ parent, actions: (await invoke("ui_observe", { parent, limit: 64 })).data.result.actions })));
  const preview = disclosed.find((group: { actions: { label: string }[] }) => group.actions.some(action => action.label === "Read embedded image images/mark.svg"));
  expect(preview).toBeTruthy();
  const result = await act(preview.parent, "Read embedded image images/mark.svg", { max_dimension: 256 });
  const value = result.outcome.value, bytes = Buffer.from(value.ui_image.data, "base64");
  expect(value).toMatchObject({ path: "images/mark.svg", original_width: 40, original_height: 40, width: 40, height: 40 });
  expect([...bytes.subarray(0, 8)]).toEqual([137, 80, 78, 71, 13, 10, 26, 10]);
  expect(createHash("sha256").update(bytes).digest("hex")).toBe(value.ui_image.digest);
  expect(engine).toEqual([]);
});

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
// Actual interpreter + shared Rust/WASM ledger; no provider or backend.
test("browser prediction practice changes inputs and retains real comparisons without delivering answers", async ({ page }) => {
  const { invoke, act, actions, frame, engine } = await connect(page, "site");
  await act("browser-tour", "Set browser workflow tour visibility", { open: true });
  await act("browser-tour", "Restart browser workflow tour");
  await act("browser-tour", "Continue browser workflow step");
  await act("browser-tour", "Continue browser workflow step");
  await act("browser-tour", "Set browser workflow practice objection", { text: "One shared deadline" });
  await act("browser-tour", "Continue browser workflow step");
  const ordinaryRoot = (await invoke("ui_observe", { limit: 64 })).data.result;
  const ordinaryParent = ordinaryRoot.groups.find((group: string) => group.startsWith("browser-document:deadline-spec.md:"));
  const ordinaryRun = (await actions(ordinaryParent)).find(action => action.label === "Restart browser document (deadline-spec.md)");
  await act("browser-tour", "Set browser prediction practice visibility", { open: true });
  await expect(frame.getByRole("heading", { name: "Predict with your own inputs" })).toBeVisible();
  expect((await invoke("ui_observe", { limit: 64 })).data.result.groups).not.toContain(ordinaryParent);
  expect((await invoke("ui_act", { handle: ordinaryRun.handle, arguments: {}, request_id: crypto.randomUUID() })).data.result).toMatchObject({ status: "unavailable" });

  expect((await actions("browser-prediction")).find(value => value.label === "Run sealed browser prediction").enabled).toBe(false);
  await act("browser-prediction", "Edit browser prediction attempt durations", { value: "[70,65]" });
  await act("browser-prediction", "Edit browser prediction total budget", { value: "90" });
  await act("browser-prediction", "Edit browser prediction expected duration", { value: "90" });
  await act("browser-prediction", "Edit browser prediction rationale", { value: "Both attempts share the original deadline" });
  await act("browser-prediction", "Set browser prediction source disclosure", { open: true });
  await expect(frame.getByText(/target: elapsed\(\[70,65\], 90\)/)).toBeVisible();
  await act("browser-prediction", "Seal browser prediction before execution");
  const fault = await act("browser-prediction", "Run sealed browser prediction");
  expect(fault).toMatchObject({ outcome: { value: { prediction_matched: false, requirement_satisfied: false, independent_checks_accepted: true, values: "withheld", current_certification: false } } });
  expect(JSON.stringify(fault)).not.toContain('"actual":');
  await act("browser-tour", "Continue browser workflow step");
  await expect(frame.getByRole("heading", { name: "Inspect and run the correction" })).toBeVisible();
  expect((await actions("browser-prediction")).find(value => value.label === "Run sealed browser prediction").enabled).toBe(false);
  await act("browser-prediction", "Seal browser prediction before execution");
  const repaired = await act("browser-prediction", "Run sealed browser prediction");
  expect(repaired).toMatchObject({ outcome: { value: { prediction_matched: true, requirement_satisfied: true, independent_checks_accepted: true, values: "withheld", current_certification: false } } });
  const history = await act("browser-prediction", "Read browser prediction history summaries", { offset: 0, limit: 16 });
  expect(history.data).toBeUndefined();
  expect(history.outcome.value.attempts).toHaveLength(2);
  expect(history.outcome.value.attempts[0].original_requirement_satisfied).toBe(false);
  expect(history.outcome.value.attempts[1].original_requirement_satisfied).toBe(true);
  // A different same-origin page holding the runtime lease prevents recovery.
  const other = await page.context().newPage();
  const childOrigin = await frame.getByRole("heading", { name: "Predict with your own inputs" }).evaluate(() => location.origin);
  await other.goto(`${childOrigin}/site.html?predictionPractice=1`);
  await other.evaluate(() => new Promise<boolean>(resolve => { void navigator.locks.request("hickory-browser-prediction-runtime", async () => { resolve(true); await new Promise<void>(() => {}); }); }));
  const recovery = (await actions("browser-prediction")).find(value => value.label === "Recover interrupted browser prediction attempts");
  expect((await invoke("ui_act", { handle: recovery.handle, arguments: {}, request_id: crypto.randomUUID() })).data.result).toMatchObject({ outcome: { status: "unavailable", reason: expect.stringContaining("lease is active") } });
  await other.close();
  await expect.poll(() => frame.getByRole("heading", { name: "Predict with your own inputs" }).evaluate(() => navigator.locks.request("hickory-browser-prediction-runtime", { ifAvailable: true }, lease => !!lease))).toBe(true);
  expect(await act("browser-prediction", "Recover interrupted browser prediction attempts")).toMatchObject({ outcome: { value: { interrupted: 0, execution_replayed: false, current_certification: false } } });
  const promotion = (await actions("browser-prediction")).find(value => value.label === "Request browser prediction promotion");
  expect((await invoke("ui_act", { handle: promotion.handle, arguments: {}, request_id: crypto.randomUUID() })).data.result).toMatchObject({ outcome: { status: "unavailable", reason: expect.stringContaining("Unknown") } });
  const inspection = (await actions("browser-prediction")).find(value => value.label === "Request human inspection of browser prediction");
  expect((await invoke("ui_act", { handle: inspection.handle, arguments: {}, request_id: crypto.randomUUID() })).data.result).toMatchObject({ outcome: { status: "requires_input" } });
  expect(page.context().pages()).toHaveLength(1);
  await expect(frame.getByRole("region", { name: "AI engineer tour" }).getByRole("button", { name: "Continue", exact: true })).toBeEnabled();
  await act("browser-prediction", "Edit browser prediction total budget", { value: "80" });
  await expect(frame.getByRole("region", { name: "AI engineer tour" }).getByRole("button", { name: "Continue", exact: true })).toBeDisabled();
  expect(JSON.stringify(history)).not.toContain('"actual":');
  const visibility = (await actions("browser-tour")).find(value => value.label === "Set browser prediction practice visibility");
  expect((await invoke("ui_act", { handle: visibility.handle, arguments: { open: false }, request_id: crypto.randomUUID() })).data.result).toMatchObject({ outcome: { status: "requires_input" } });
  await expect(frame.getByRole("heading", { name: "Predict with your own inputs" })).toBeVisible();
  expect((await invoke("ui_observe", { limit: 64 })).data.result.groups.every((group: string) => ["browser-tour", "browser-prediction"].includes(group))).toBe(true);
  expect((await invoke("ui_observe", { parent: ordinaryParent })).data.result.actions).toEqual([]);

  expect(engine).toEqual([]);
});

test("browser prediction refuses corrupt retained history without echoing its private bytes", async ({ page }) => {
  const { invoke, act, actions, frame, engine } = await connect(page, "site");
  await act("browser-tour", "Set browser workflow tour visibility", { open: true });
  await act("browser-tour", "Set browser prediction practice visibility", { open: true });
  await expect(frame.getByRole("heading", { name: "Predict with your own inputs" })).toBeVisible();
  await frame.getByRole("heading", { name: "Predict with your own inputs" }).evaluate(() => localStorage.setItem("hickory.browserPrediction.v1", "private-held-out-corrupt-journal"));
  await act("browser-prediction", "Edit browser prediction rationale", { value: "One total deadline" });
  const seal = (await actions("browser-prediction")).find(value => value.label === "Seal browser prediction before execution");
  const result = (await invoke("ui_act", { handle: seal.handle, arguments: {}, request_id: crypto.randomUUID() })).data.result;
  expect(result).toMatchObject({ outcome: { status: "unavailable" } });
  expect(JSON.stringify(result)).not.toContain("private-held-out-corrupt-journal");
  expect((await actions("browser-prediction")).find(value => value.label === "Run sealed browser prediction").enabled).toBe(false);
  expect(await frame.getByRole("heading", { name: "Predict with your own inputs" }).evaluate(() => localStorage.getItem("hickory.browserPrediction.v1"))).toBe("private-held-out-corrupt-journal");
  expect(engine).toEqual([]);
});

// Synthetic human inputs/review; genuine interpreter and separate human window.
test("isolated browser practice keeps a correctly predicted bug non-passing and human inspection separate", async ({ page }) => {
  const engine: string[] = [];
  await page.route(url => url.pathname.startsWith("/api/"), route => { engine.push(route.request().url()); return route.abort(); });
  await page.setViewportSize({ width: 390, height: 1000 });
  await page.goto("/site.html?predictionPractice=1");
  await expect(page.getByRole("heading", { name: "Predict with your own inputs" })).toBeVisible();
  expect(await page.getByRole("heading", { name: "Hickory Docs", exact: true }).count()).toBe(0);
  await page.getByLabel("Attempt durations (JSON, ms)").fill("[70,65]");
  await page.getByLabel("Total budget (ms)").fill("90");
  // Seeded test-driver prediction, not independent model judgment.
  await page.getByLabel("Predicted elapsed duration (ms)").fill("135");
  await page.getByLabel("Prediction rationale").fill("The faulty recorded example gives each attempt the whole allowance");
  await page.getByRole("button", { name: "Seal browser prediction", exact: true }).click();
  await expect(page.getByRole("button", { name: "Run sealed prediction", exact: true })).toBeEnabled();
  await page.getByRole("button", { name: "Run sealed prediction", exact: true }).click();
  const history = page.getByRole("list", { name: "Retained browser prediction attempts" });
  await expect(history).toContainText("prediction true");
  await expect(history).toContainText("original requirement false");
  await expect(history).toContainText("current certification false");
  await page.getByRole("button", { name: "Request promotion", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Unknown");
  const popup = page.waitForEvent("popup");
  await page.getByRole("button", { name: "Inspect actuals as a human", exact: true }).click();
  const report = await popup;
  await expect(report.locator("body")).toContainText('"actual": 135');
  expect(await report.evaluate(() => window.opener === null)).toBe(true);
  expect(await page.locator("body").textContent()).not.toContain('"actual": 135');
  await report.close();
  await page.getByRole("button", { name: "Inspect prediction source", exact: true }).click();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: test.info().outputPath("isolated-browser-prediction-390.png"), fullPage: true });
  expect(engine).toEqual([]);
});

// Actual comparison may pass after changing an expectation; the coach must not
// call that the original expectation surviving a correction.
test("browser correction cannot advance by replacing its original prediction", async ({ page }) => {
  const { invoke, act, actions, frame, engine } = await connect(page, "site", "predictionPractice=1");
  await expect(frame.getByRole("heading", { name: "Predict with your own inputs" })).toBeVisible();
  await act("browser-prediction", "Edit browser prediction attempt durations", { value: "[70,65]" });
  await act("browser-prediction", "Edit browser prediction total budget", { value: "90" });
  await act("browser-prediction", "Edit browser prediction expected duration", { value: "100" });
  await act("browser-prediction", "Edit browser prediction rationale", { value: "Initial expectation retained even if wrong" });
  await act("browser-prediction", "Seal browser prediction before execution");
  expect(await act("browser-prediction", "Run sealed browser prediction")).toMatchObject({ outcome: { value: { requirement_satisfied: false, original_expectation_unchanged: true } } });
  await act("browser-tour", "Continue browser workflow step");
  await act("browser-prediction", "Edit browser prediction expected duration", { value: "90" });
  await act("browser-prediction", "Seal browser prediction before execution");
  expect(await act("browser-prediction", "Run sealed browser prediction")).toMatchObject({ outcome: { value: { prediction_matched: true, requirement_satisfied: true, original_expectation_unchanged: false, current_certification: false } } });
  await expect(frame.getByRole("region", { name: "AI engineer tour" }).getByRole("button", { name: "Continue", exact: true })).toBeDisabled();
  const promotion = (await actions("browser-prediction")).find(value => value.label === "Request browser prediction promotion");
  expect((await invoke("ui_act", { handle: promotion.handle, arguments: {}, request_id: crypto.randomUUID() })).data.result).toMatchObject({ outcome: { status: "unavailable" } });
  expect(engine).toEqual([]);
});
</book:entry>

</book:doc>
