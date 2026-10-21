import { test, expect, type Page } from "@playwright/test";
import { createHash } from "node:crypto";
// Guarantees: docs/guarantees/agent/ui-action-dispatch.md
// Real Chrome, origin channel, parser and interpreter; host requests are scripted.
async function connect(page: Page, surface: "site" | "snapshot") {
  const engine: string[] = [];
  await page.route(url => url.pathname.startsWith("/api/"), route => { engine.push(route.request().url()); return route.abort(); });
  await page.goto("http://localhost:4178/snapshot.html");
  await page.evaluate(surface => {
    const frame = document.createElement("iframe"); frame.id = "static-surface";
    const origin = "http://127.0.0.1:4178";
    frame.src = `${origin}/${surface}.html?parentOrigin=${encodeURIComponent(location.origin)}`;
    Object.assign(window, { staticReady: null });
    window.addEventListener("message", event => {
      if (event.source === frame.contentWindow && event.origin === origin && event.data.channel === "hickory-static-ui" && event.data.event === "ready") Object.assign(window, { staticReady: event.data.data.scope });
    });
    document.body.prepend(frame);
  }, surface);
  await expect.poll(() => page.evaluate(() => (window as any).staticReady)).toBeTruthy();
  const invoke = (method: string, args: Record<string, unknown>, owner?: string) => page.evaluate(({ method, args, owner }) => {
    const frame = document.getElementById("static-surface") as HTMLIFrameElement;
    const origin = "http://127.0.0.1:4178", scope = (window as any).staticReady;
    return new Promise<any>((resolve, reject) => {
      const id = crypto.randomUUID(), timeout = setTimeout(() => { window.removeEventListener("message", receive); reject(Error("Static UI reply timed out")); }, 5000);
      function receive(event: MessageEvent) {
        if (event.source !== frame.contentWindow || event.origin !== origin || event.data.channel !== "hickory-static-ui" || event.data.id !== id) return;
        clearTimeout(timeout); window.removeEventListener("message", receive); resolve(event.data);
      }
      window.addEventListener("message", receive);
      frame.contentWindow!.postMessage({ channel: "hickory-static-ui", version: 1, op: "ui", id, data: { window: owner ?? scope.window, method, arguments: args } }, origin);
    });
  }, { method, args, owner });
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
