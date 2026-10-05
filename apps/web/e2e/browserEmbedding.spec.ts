import { test, expect } from "@playwright/test";
import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import { resolve, extname, sep } from "node:path";
import { once } from "node:events";
import type { Page } from "@playwright/test";
// Guarantees: docs/guarantees/debugging/browser-debugging-runs-edited-source.md
// docs/guarantees/embedding/an-embed-has-no-implicit-host.md
async function noEngine(page: Page) {
  const unexpected: string[] = [];
  await page.route("**/api/**", (route) => { unexpected.push(route.request().url()); return route.abort(); });
  await page.addInitScript(() => { globalThis.WebSocket = class { constructor() { throw Error("Unexpected WebSocket"); } } as unknown as typeof WebSocket; });
  return unexpected;
}
async function code(page: Page, program: string, language = "js") {
  await page.getByLabel("Debug source", { exact: true }).fill(`# 🦀 Source\n<hick:file path="main.${language}">\n${program}\n</hick:file>\n`);
  await expect(page.getByRole("group", { name: "fixture.md", exact: true }).locator(".cm-content")).toContainText(program.split("\n")[0]);
}

test("two embeds, explicit save, replacement, read-only and disposal", async ({ page }) => {
  const unexpected = await noEngine(page);
  await page.goto("/embed.html");
  await expect(page.getByLabel("Browser measurements")).not.toBeEmpty();
  await test.info().attach("portable-core-timings", { body: await page.getByLabel("Browser measurements").innerText(), contentType: "application/json" });
  const left = page.getByRole("group", { name: "first.md", exact: true }).locator(".cm-content");
  const right = page.getByRole("group", { name: "second.md", exact: true }).locator(".cm-content");
  await expect(left).toContainText("An ordinary note.");
  await expect(page.getByAltText("images/mark.svg")).toBeVisible();
  await left.click(); await page.keyboard.press("ControlOrMeta+End"); await page.keyboard.type("First edit");
  await expect(left).toContainText("First edit"); await expect(right).not.toContainText("First edit");
  await expect(page.getByLabel("Saved document")).toBeEmpty();
  await page.getByRole("button", { name: "Save explicitly" }).click();
  await expect(page.getByLabel("Saved document")).toContainText('<hick:unknown untouched="yes">🦀 raw</hick:unknown>');
  await page.getByRole("button", { name: "Toggle read-only" }).click();
  await expect(left).toHaveAttribute("contenteditable", "false");
  await page.getByRole("button", { name: "Replace source" }).click(); await expect(left).toHaveText("# Host replacement");
  await page.getByRole("button", { name: "Mount/unmount" }).click(); await expect(left).toHaveCount(0);
  await page.getByRole("button", { name: "Mount/unmount" }).click(); await expect(left).toContainText("Host replacement");
  expect(unexpected).toEqual([]);
});

for (const language of ["js", "ts"]) {
  test(`${language}: real breakpoint, step in/out, watches, edits and offline restart`, async ({ page, context }) => {
    const unexpected = await noEngine(page); await page.goto("/embed.html");
    const type = language === "ts" ? ": number" : "";
    await code(page, `function price(n${type})${type} {\n  var total = n * 3;\n  return total;\n}\nvar input${type} = 4;\nvar result = price(input);\nconsole.log(result);`, language);
    const pane = page.getByRole("group", { name: "fixture.md", exact: true });
    const gutters = pane.locator(".cm-gutter.cm-lineNumbers .cm-gutterElement");
    // The first gutter child is CodeMirror's sizing element, followed by real lines.
    await gutters.filter({ hasText: /^8$/ }).scrollIntoViewIfNeeded();
    const number = await gutters.filter({ hasText: /^8$/ }).boundingBox();
    const gutter = await pane.locator(".cm-breakpoint-gutter").boundingBox();
    if (!number || !gutter) throw Error("Expected live breakpoint gutter");
    await page.mouse.click(gutter.x + gutter.width / 2, number.y + number.height / 2);
    await expect(pane.locator(".cm-bp")).toHaveCount(1);
    await page.getByRole("button", { name: "Debug document", exact: true }).click();
    await expect(page.getByText("Paused on line 8", { exact: true })).toBeVisible();
    await expect(page.getByLabel("Live variables")).toContainText("input = 4");
    await page.getByRole("button", { name: "Step into", exact: true }).click();
    await expect(page.getByText("Paused on line 4", { exact: true })).toBeVisible();
    await expect(page.getByLabel("Call stack")).toContainText("price");
    await expect(page.getByLabel("Live variables")).toContainText("n = 4");
    await page.getByLabel("Watch expression").fill("n * 2"); await page.getByRole("button", { name: "Watch", exact: true }).click();
    await expect(page.getByLabel("Watch values")).toContainText("n * 2 = 8");
    await page.getByRole("button", { name: "Step out", exact: true }).click();
    await expect(page.getByText("Paused on line 9", { exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Continue", exact: true }).click();
    await expect(page.getByLabel("Program output")).toHaveText("12\n");
    // Cache-loaded worker/compiler assets must work without a network.
    await context.setOffline(true);
    await page.getByLabel("Debug source", { exact: true }).fill((await page.getByLabel("Debug source", { exact: true }).inputValue()).replace("= 4;", "= 6;"));
    await expect(pane.locator(".cm-content")).toContainText("= 6;");
    await page.getByRole("button", { name: "Debug document", exact: true }).click();
    await expect(page.getByLabel("Live variables")).toContainText("input = 6");
    await page.getByRole("button", { name: "Continue", exact: true }).click();
    await expect(page.getByLabel("Program output")).toHaveText("18\n");
    expect(unexpected).toEqual([]);
  });
}

test("errors, unsupported syntax, stale source and immediate Stop", async ({ page }) => {
  await noEngine(page); await page.goto("/embed.html");
  await code(page, "throw new Error('broken');"); await page.getByRole("button", { name: "Debug document" }).click();
  await expect(page.getByLabel("Program output")).toContainText("broken");
  await code(page, "async function unavailable() {}"); await page.getByRole("button", { name: "Debug document" }).click();
  await expect(page.getByRole("alert")).toContainText("Unsupported");
  await code(page, "var n = 0;\nwhile (true) { n++; }"); await page.getByRole("button", { name: "Debug document" }).click();
  await page.getByRole("button", { name: "Pause", exact: true }).click();
  await expect(page.locator(".hickory-browser-debug")).toHaveAttribute("data-status", "paused");
  await page.getByLabel("Debug source", { exact: true }).fill("# a completely different revision");
  await expect(page.getByText(/Running previous revision/)).toBeVisible();
  await page.getByRole("button", { name: "Stop", exact: true }).click();
  await expect(page.locator(".hickory-browser-debug")).toHaveAttribute("data-status", "idle");
});

test("homepage debugger uses the real backend without an engine", async ({ page }) => {
  const unexpected = await noEngine(page); await page.goto("/");
  const demo = page.locator("section", { has: page.getByRole("heading", { name: "Edit it. Break on a line. Inspect the real values." }) });
  const pane = demo.getByRole("group", { name: "browser-demo.md" });
  await pane.getByLabel("Set breakpoint on line 13", { exact: true }).click();
  await expect(pane.locator(".cm-bp")).toHaveCount(1);
  await demo.getByRole("button", { name: "Debug document", exact: true }).click();
  await expect(demo.getByText("Paused on line 13", { exact: true })).toBeVisible();
  await expect(demo.getByLabel("Live variables")).toContainText("quantity = 4");
  await demo.getByRole("button", { name: "Step into", exact: true }).click();
  await expect(demo.getByLabel("Call stack")).toContainText("price");
  await expect(demo.getByText("Paused on line 7", { exact: true })).toBeVisible();
  await demo.screenshot({ path: test.info().outputPath("homepage-debugger.png") });
  await demo.getByRole("button", { name: "Continue", exact: true }).click();
  await expect(demo.getByLabel("Program output")).toHaveText("Total: 43\n");
  await page.getByLabel("Debug language").selectOption("js");
  await demo.getByRole("button", { name: "Debug document", exact: true }).click();
  await expect(demo.getByLabel("Program output")).toHaveText("Total: 43\n");
  expect(unexpected).toEqual([]);
});

// Guarantee: docs/guarantees/embedding/a-local-save-is-revision-checked.md
test("local persistence survives reload and a concurrent tab cannot overwrite", async ({ page, context }) => {
  await noEngine(page); await page.goto("/embed.html?workspace");
  const note = page.getByRole("group", { name: "notes/note.md", exact: true }).locator(".cm-content");
  await expect(note).toContainText("A browser note");
  await note.click(); await page.keyboard.press("ControlOrMeta+End"); await page.keyboard.type("Persistent 🦀 note");
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await expect(page.getByRole("status")).toHaveText("Saved in this browser.");
  await page.reload(); await expect(note).toContainText("Persistent 🦀 note");
  const other = await context.newPage(); await noEngine(other); await other.goto("/embed.html?workspace");
  await expect(other.getByRole("group", { name: "notes/note.md", exact: true })).toContainText("Persistent 🦀 note");
  await note.click(); await page.keyboard.press("ControlOrMeta+End"); await page.keyboard.type(" new revision");
  await page.getByRole("button", { name: "Save note", exact: true }).click(); await expect(page.getByRole("status")).toHaveText("Saved in this browser.");
  const draft = other.getByRole("group", { name: "notes/note.md", exact: true }).locator(".cm-content");
  await draft.click(); await other.keyboard.press("ControlOrMeta+End"); await other.keyboard.type(" retained draft");
  await other.getByRole("button", { name: "Save note", exact: true }).click();
  await expect(other.getByRole("status")).toContainText("Save failed. Your draft is still here.");
  await expect(draft).toContainText("retained draft"); await other.close();
});

test("cross-origin iframe validates sender, saves through its host and disposes", async ({ page }) => {
  await noEngine(page); await page.goto("http://localhost:4178/embed.html");
  await page.evaluate(() => {
    const messages: { event: string; data: unknown }[] = [];
    Object.assign(window, { embedMessages: messages });
    const iframe = document.createElement("iframe"); iframe.id = "external-embed";
    iframe.setAttribute("sandbox", "allow-scripts allow-same-origin");
    iframe.src = "http://127.0.0.1:4178/iframe.html?parentOrigin=http%3A%2F%2Flocalhost%3A4178";
    window.addEventListener("message", (event) => {
      if (event.source !== iframe.contentWindow || event.origin !== "http://127.0.0.1:4178" || event.data.channel !== "hickory-embed") return;
      messages.push(event.data);
      if (event.data.event === "ready") iframe.contentWindow!.postMessage({ channel: "hickory-embed", version: 1, op: "load", data: { source: "# From another origin\n", path: "iframe-note.md", revision: "base" } }, "http://127.0.0.1:4178");
    });
    document.body.prepend(iframe);
  });
  const frame = page.frameLocator("#external-embed");
  await expect(frame.getByRole("group", { name: "iframe-note.md" })).toContainText("From another origin");
  // A synthetic event with a different origin/source must not replace it.
  const actual = page.frames().find((f) => f.url().includes("iframe.html"))!;
  await actual.evaluate(() => {
    const data = { channel: "hickory-embed", version: 1, op: "load", data: { path: "wrong.md", source: "wrong", revision: "x" } };
    const wrongOrigin = new MessageEvent("message", { origin: "https://untrusted.example", data });
    Object.defineProperty(wrongOrigin, "source", { value: window.parent });
    window.dispatchEvent(wrongOrigin);
    window.dispatchEvent(new MessageEvent("message", { origin: "http://localhost:4178", source: window, data }));
  });
  await expect(frame.getByRole("group", { name: "iframe-note.md" })).toContainText("From another origin");
  const iframeEditor = frame.getByRole("group", { name: "iframe-note.md" }).locator(".cm-content");
  await iframeEditor.click(); await page.keyboard.press("ControlOrMeta+End"); await page.keyboard.type("Captured in frame");
  await expect(iframeEditor).toContainText("Captured in frame");
  await page.evaluate(() => {
    const iframe = document.getElementById("external-embed") as HTMLIFrameElement;
    iframe.contentWindow!.postMessage({ channel: "hickory-embed", version: 1, op: "save", id: "save-1" }, "http://127.0.0.1:4178");
  });
  await expect.poll(() => page.evaluate(() => (window as unknown as { embedMessages: { event: string; data: unknown }[] }).embedMessages.find((e) => e.event === "save")?.data)).toEqual({ path: "iframe-note.md", source: "# From another origin\nCaptured in frame", revision: "base" });
  await page.evaluate(() => {
    const iframe = document.getElementById("external-embed") as HTMLIFrameElement;
    iframe.contentWindow!.postMessage({ channel: "hickory-embed", version: 1, op: "dispose" }, "http://127.0.0.1:4178");
  });
  await expect(frame.locator(".hickory-browser-debug")).toHaveCount(0);
});

test("running infinite loop stays responsive and sessions stay independent", async ({ page, context }) => {
  await noEngine(page); await page.goto("/embed.html");
  await code(page, "var n = 0;\nwhile (true) { n++; }");
  await page.getByRole("button", { name: "Add/remove second debugger" }).click();
  const primary = page.locator(".hickory-browser-debug").last();
  const secondary = page.locator(".hickory-browser-debug").first();
  await primary.getByRole("button", { name: "Debug document" }).click();
  await expect(primary).toHaveAttribute("data-status", "running");
  await secondary.getByRole("button", { name: "Debug document" }).click();
  await expect(secondary.getByLabel("Program output")).toHaveText("7\n");
  await primary.getByRole("button", { name: "Stop", exact: true }).click();
  await expect(primary).toHaveAttribute("data-status", "idle");
  await context.setOffline(true);
  await code(page, "console.log(9);");
  await primary.getByRole("button", { name: "Debug document" }).click();
  await expect(primary.getByLabel("Program output")).toHaveText("9\n");
});

// Guarantee: docs/guarantees/embedding/a-local-save-is-revision-checked.md
test("workspace transfer, saved-note search and quota errors retain drafts", async ({ page }) => {
  await noEngine(page); await page.goto("/embed.html?workspace");
  const note = page.getByRole("group", { name: "notes/note.md", exact: true }).locator(".cm-content");
  await expect(note).toContainText("A browser note");
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await expect(page.getByRole("status")).toHaveText("Saved in this browser.");
  const bundle = { version: 1, files: [
    { path: "notes/imported.md", base64: Buffer.from("# Imported\nA searchable finding 🦀\n").toString("base64") },
    { path: "notes/assets/image.bin", base64: Buffer.from([0, 255, 7]).toString("base64") },
    { path: ".hick-cache/evidence.json", base64: Buffer.from('{"run":"recorded"}').toString("base64") },
  ] };
  const upload = (value: unknown) => page.getByLabel("Import workspace", { exact: true }).setInputFiles({ name: "workspace.json", mimeType: "application/json", buffer: Buffer.from(JSON.stringify(value)) });
  await upload(bundle);
  await expect(page.getByRole("status")).toContainText("Workspace imported");
  await page.getByLabel("Search saved notes").fill("FINDING");
  await expect(page.getByLabel("Search results")).toContainText("notes/imported.md:2");
  await upload({ version: 1, files: [{ path: "notes/partial.md", base64: "Iw==" }, bundle.files[0]] });
  await expect(page.getByRole("status")).toContainText("no files were imported");
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export saved workspace" }).click();
  const file = await download;
  const exported = JSON.parse(await readFile((await file.path())!, "utf8"));
  expect(exported.files).toEqual(expect.arrayContaining(bundle.files));
  expect(exported.files.some((entry: { path: string }) => entry.path === "notes/partial.md")).toBe(false);
  await page.evaluate(() => {
    const original = IDBObjectStore.prototype.put;
    IDBObjectStore.prototype.put = function () { throw new DOMException("Injected quota failure", "QuotaExceededError"); };
    Object.assign(window, { restoreStoragePut: () => { IDBObjectStore.prototype.put = original; } });
  });
  await note.click(); await page.keyboard.press("ControlOrMeta+End"); await page.keyboard.type(" kept draft");
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await expect(page.getByRole("status")).toContainText("Browser storage is full");
  await expect(note).toContainText("kept draft");
  await page.evaluate(() => (window as unknown as { restoreStoragePut(): void }).restoreStoragePut());
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await expect(page.getByRole("status")).toHaveText("Saved in this browser.");
  await page.reload();
  await page.getByLabel("Browser notes").selectOption("notes/note.md");
  await expect(note).toContainText("kept draft");
});

test("S3 host checks CORS/conditional writes and keeps offline conflicts locally", async ({ page, context }) => {
  const objects = new Map<string, { bytes: Buffer; etag: string }>();
  let serial = 0, offline = false;
  await context.route("http://localhost:4178/bucket/**", async (route) => {
    if (offline) { await route.abort("internetdisconnected"); return; }
    const req = route.request(), key = new URL(req.url()).pathname;
    const headers = { "access-control-allow-origin": "http://127.0.0.1:4178", "access-control-allow-methods": "GET, PUT, OPTIONS", "access-control-allow-headers": "if-match, if-none-match, content-type", "access-control-expose-headers": "ETag" };
    if (req.method() === "OPTIONS") { await route.fulfill({ status: 204, headers }); return; }
    const stored = objects.get(key);
    if (req.method() === "GET") {
      await route.fulfill({ status: stored ? 200 : 404, body: stored?.bytes, headers: { ...headers, ...(stored ? { ETag: stored.etag } : {}) } }); return;
    }
    const conditional = req.headers();
    if (conditional["if-none-match"] === "*" && stored || conditional["if-match"] && conditional["if-match"] !== stored?.etag) { await route.fulfill({ status: 412, headers }); return; }
    const etag = `"opaque-${++serial}"`;
    objects.set(key, { bytes: req.postDataBuffer()!, etag });
    await route.fulfill({ status: 200, headers: { ...headers, ETag: etag } });
  });
  await noEngine(page); await page.goto("/embed.html?workspace&s3&local=one");
  const note = page.getByRole("group", { name: "notes/note.md", exact: true }).locator(".cm-content");
  await expect(note).toContainText("A browser note");
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await expect(page.getByLabel("Remote synchronization")).toContainText("pending");
  await page.getByRole("button", { name: "Sync saved files" }).click();
  await expect(page.getByLabel("Remote synchronization")).toContainText("synced");
  const other = await context.newPage(); await noEngine(other); await other.goto("/embed.html?workspace&s3&local=two");
  const draft = other.getByRole("group", { name: "notes/note.md", exact: true }).locator(".cm-content");
  await expect(draft).toContainText("A browser note");
  offline = true; await context.setOffline(true);
  await draft.click(); await other.keyboard.press("ControlOrMeta+End"); await other.keyboard.type(" offline draft");
  await other.getByRole("button", { name: "Save note", exact: true }).click();
  await expect(other.getByRole("status")).toHaveText("Saved in this browser.");
  await other.getByRole("button", { name: "Sync saved files" }).click();
  await expect(other.getByLabel("Remote synchronization")).toContainText("disconnected");
  offline = false; await context.setOffline(false);
  await note.click(); await page.keyboard.press("ControlOrMeta+End"); await page.keyboard.type(" published elsewhere");
  await page.getByRole("button", { name: "Save note", exact: true }).click();
  await page.getByRole("button", { name: "Sync saved files" }).click();
  await expect(page.getByLabel("Remote synchronization")).toContainText("synced");
  const head = objects.get("/bucket/fixture/head.json")!.etag;
  await other.getByRole("button", { name: "Sync saved files" }).click();
  await expect(other.getByLabel("Remote synchronization")).toContainText("conflict");
  await expect(draft).toContainText("offline draft");
  expect(objects.get("/bucket/fixture/head.json")!.etag).toBe(head);
  await other.reload(); await expect(draft).toContainText("offline draft");
  await expect(other.getByLabel("Remote synchronization")).toContainText("pending");
  await other.getByRole("button", { name: "Sync saved files" }).click();
  await expect(other.getByLabel("Remote synchronization")).toContainText("conflict");
  await other.getByRole("button", { name: "Review sync conflicts" }).click();
  const review = other.getByLabel("Sync conflict notes/note.md", { exact: true });
  await expect(review).toContainText("offline draft"); await expect(review).toContainText("published elsewhere");
  await review.getByRole("button", { name: "Keep local saved version" }).click();
  await expect(other.getByRole("status")).toContainText("Resolution saved locally");
  await other.getByRole("button", { name: "Sync saved files" }).click();
  await expect(other.getByLabel("Remote synchronization")).toContainText("synced");
});

test("static assets and debugger work below a deployment prefix with scoped CSP", async ({ page }) => {
  const root = resolve("dist-site");
  const server = createServer(async (req, res) => {
    const name = new URL(req.url!, "http://fixture").pathname;
    const file = resolve(root, name.slice("/nested/".length));
    if (!name.startsWith("/nested/") || !file.startsWith(root + sep)) { res.writeHead(404).end(); return; }
    try {
      const bytes = await readFile(file);
      const mime: Record<string, string> = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".css": "text/css" };
      res.writeHead(200, { "Content-Type": mime[extname(file)] ?? "application/octet-stream",
        "Content-Security-Policy": "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self' blob: data:; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self'" }).end(bytes);
    } catch { res.writeHead(404).end(); }
  });
  server.listen(0, "127.0.0.1"); await once(server, "listening");
  try {
    const unexpected = await noEngine(page);
    const address = server.address(); if (!address || typeof address === "string") throw Error("Expected static fixture address");
    await page.goto(`http://127.0.0.1:${address.port}/nested/embed.html`);
    await expect(page.getByRole("group", { name: "first.md", exact: true })).toContainText("An ordinary note");
    await page.getByRole("button", { name: "Debug document", exact: true }).click();
    await expect(page.getByLabel("Program output")).toHaveText("Total: 43\n");
    expect(unexpected).toEqual([]);
  } finally { server.closeAllConnections(); await new Promise<void>((done) => server.close(() => done())); }
});
