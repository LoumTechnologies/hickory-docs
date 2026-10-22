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
