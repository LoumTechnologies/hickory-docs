<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/e2e/browserEmbedding.spec.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-31019d3567c7"></a>
## `apps/web/e2e/browserEmbedding.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [GitHub reviews retain reviewed revisions and external identity](../../../../guarantees/authoring/github-reviews-retain-reviewed-revisions/contract.md), [Review threads preserve the questioned target](../../../../guarantees/authoring/review-threads-preserve-the-questioned-target/contract.md), [The engineer tour distinguishes replay from execution](../../../../guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution/contract.md), [Browser debugging runs edited source](../../../../guarantees/debugging/browser-debugging-runs-edited-source/contract.md), [Observations read beside their captured source](../../../../guarantees/debugging/source-anchored-observation-reading/contract.md), [A local save is revision checked](../../../../guarantees/embedding/a-local-save-is-revision-checked/contract.md), [A viewer snapshot keeps recorded results historical](../../../../guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical/contract.md), [An embed has no implicit host](../../../../guarantees/embedding/an-embed-has-no-implicit-host/contract.md), [An iframe validates its host](../../../../guarantees/embedding/an-iframe-validates-its-host/contract.md), [Browser output carries actual runtime source and occurrence evidence](../../../../guarantees/execution/browser-output-has-runtime-source-evidence/contract.md), [Python profiles keep function scope and measurement semantics](../../../../guarantees/execution/python-profile-keeps-function-semantics/contract.md), [Runtime history keeps runs and invocation occurrences separate](../../../../guarantees/execution/runtime-history-keeps-runs-and-occurrences-separate/contract.md), [Execution logs preserve contributor relationships and historical event identity](../../../../guarantees/execution/runtime-logs-preserve-contributor-relationships/contract.md), [The homepage opens with a live, paused debugger](../../../../guarantees/landing/homepage-opens-paused/contract.md), [Authorship display accepts non-Git providers](../../../../guarantees/lineage/authorship-display-accepts-non-git-providers/contract.md).

<a id="f-31019d3567c7-noEngine"></a>
### `noEngine`

<storycode:copy id="f-31019d3567c7-noEngine">
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
  await page.route("**/api/**", (route) => {
    if (!new URL(route.request().url()).pathname.startsWith("/api/")) return route.continue(); // Vite source modules are not engine requests.
    unexpected.push(route.request().url()); return route.abort();
  });
  await page.addInitScript(() => { globalThis.WebSocket = class { constructor() { throw Error("Unexpected WebSocket"); } } as unknown as typeof WebSocket; });
  return unexpected;
}
</storycode:copy>
<a id="f-31019d3567c7-code"></a>
### `code`

<storycode:copy id="f-31019d3567c7-code">
async function code(page: Page, program: string, language = "js") {
  const editor = page.getByRole("group", { name: "fixture.md", exact: true }).locator(".cm-content");
  // Parser loading mounts CodeMirror asynchronously. Replace the host source
  // only after its editor is ready, so WebKit's initial input observation
  // cannot race the replacement and report the previous buffer to the host.
  await expect(editor).toBeVisible();
  await page.getByLabel("Debug source", { exact: true }).fill(`# 🦀 Source\n<hick:file path="main.${language}">\n${program}\n</hick:file>\n`);
  await expect(editor).toContainText(program.split("\n")[0]);
}

</storycode:copy>
<a id="f-31019d3567c7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement">
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

</storycode:copy>
<a id="f-31019d3567c7-for_in_statement"></a>
### `for_in_statement`

<storycode:copy id="f-31019d3567c7-for_in_statement">
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

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-2">
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

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-3">
test("homepage loads genuinely paused, edits with a visible caret and restarts", async ({ page }) => {
  const unexpected = await noEngine(page); await page.goto("/");
  const demo = page.locator("section", { has: page.getByRole("heading", { name: "Try a note. It’s already paused." }) });
  const pane = demo.getByRole("group", { name: "hello.md" });
  await expect(demo.getByText("Paused on line 6", { exact: true })).toBeVisible();
  await expect(pane.locator(".cm-bp")).toHaveCount(1);
  await expect(pane.locator(".cm-paused-line")).toContainText("return message;");
  await expect(demo.getByLabel("Live variables")).toContainText('message = "Hello, developer"');
  await expect(demo.getByLabel("Call stack")).toContainText("greet");
  await expect(page.getByText("Downloadable software · Runs on your machine")).toBeVisible();
  // The paused editor stays editable, including a contrasting drawn caret.
  const content = pane.locator(".cm-content");
  await content.locator(".cm-line").filter({ hasText: "function greet" }).click({ position: { x: 30, y: 8 } }); await page.keyboard.press("ControlOrMeta+Home");
  await expect(pane.locator(".cm-cursor").first()).toBeVisible();
  expect(await pane.locator(".cm-cursor").first().evaluate((cursor) => getComputedStyle(cursor).borderLeftColor)).toBe("rgb(228, 222, 213)");
  await page.screenshot({ path: test.info().outputPath("homepage-debugger.png"), fullPage: true });
  await page.keyboard.type("A ");
  await expect(demo.getByRole("status").filter({ hasText: "Running previous revision" })).toBeVisible();
  await demo.getByRole("button", { name: "Restart with edits" }).click();
  await expect(demo.getByText("Paused on line 6", { exact: true })).toBeVisible();
  await demo.getByRole("button", { name: "Step out", exact: true }).click();
  await expect(demo.getByLabel("Program output")).toHaveText("Hello, developer\n");
  await page.getByLabel("Debug language").selectOption("js");
  await expect(demo.getByText("Paused on line 6", { exact: true })).toBeVisible();
  await demo.getByRole("button", { name: "Continue", exact: true }).click();
  await expect(demo.getByLabel("Program output")).toHaveText("Hello, developer\n");
  await page.setViewportSize({ width: 390, height: 844 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  expect(unexpected).toEqual([]);
});

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-4">
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

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-5">
test("cross-origin iframe validates sender, saves through its host and disposes", async ({ page }) => {
  const unexpected = await noEngine(page); await page.goto("http://localhost:4178/embed.html");
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
  const ui = (method: string, args: Record<string, unknown>, scope?: string) => page.evaluate(({ method, args, scope }) => {
    const iframe = document.getElementById("external-embed") as HTMLIFrameElement;
    const messages = (window as unknown as { embedMessages: { event: string; data: { ui_scope?: { window: string } } }[] }).embedMessages;
    const owner = scope ?? messages.filter(message => message.event === "loaded").at(-1)!.data.ui_scope!.window;
    return new Promise<{ event: string; data: any }>((resolve, reject) => {
      const id = crypto.randomUUID();
      const timeout = setTimeout(() => { window.removeEventListener("message", receive); reject(Error("Embedded UI response timed out")); }, 5000);
      const receive = (event: MessageEvent) => {
        if (event.source !== iframe.contentWindow || event.origin !== "http://127.0.0.1:4178" || event.data.channel !== "hickory-embed" || event.data.id !== id) return;
        clearTimeout(timeout); window.removeEventListener("message", receive); resolve(event.data);
      };
      window.addEventListener("message", receive);
      iframe.contentWindow!.postMessage({ channel: "hickory-embed", version: 1, op: "ui", id, data: { window: owner, method, arguments: args } }, "http://127.0.0.1:4178");
    });
  }, { method, args, scope });
  const root = await ui("ui_observe", { parent: "root" });
  expect(root.event).toBe("ui-result");
  const originalScope = root.data.scope.window as string;
  const actions = (await Promise.all((root.data.result.groups as string[]).map(parent => ui("ui_observe", { parent, limit: 64 })))).flatMap(reply => reply.data.result.actions);
  const replace = actions.find(action => action.label === "Replace embedded document range");
  expect(replace).toBeTruthy();
  const edited = await ui("ui_act", { handle: replace.handle, arguments: { from: 0, to: 0, text: "# Agent edit\n" }, request_id: "frame-agent-edit" });
  expect(edited.data.result).toMatchObject({ status: "result", outcome: { value: { draft_changed: true, saved: false } } });
  await expect(iframeEditor).toContainText("Agent edit");
  expect((await ui("ui_observe", {}, "retired-window")).event).toBe("error");
  await page.evaluate(() => {
    const iframe = document.getElementById("external-embed") as HTMLIFrameElement;
    iframe.contentWindow!.postMessage({ channel: "hickory-embed", version: 1, op: "save", id: "save-1" }, "http://127.0.0.1:4178");
  });
  await expect.poll(() => page.evaluate(() => (window as unknown as { embedMessages: { event: string; data: unknown }[] }).embedMessages.find((e) => e.event === "save")?.data)).toEqual({ path: "iframe-note.md", source: "# Agent edit\n# From another origin\nCaptured in frame", revision: "base" });
  await page.evaluate(() => {
    const iframe = document.getElementById("external-embed") as HTMLIFrameElement;
    iframe.contentWindow!.postMessage({ channel: "hickory-embed", version: 1, op: "load", data: { path: "replacement.md", source: "# Replacement document\n", revision: "second" } }, "http://127.0.0.1:4178");
  });
  await expect(frame.getByRole("group", { name: "replacement.md" })).toContainText("Replacement document");
  expect((await ui("ui_observe", {}, originalScope)).event).toBe("error");
  await page.evaluate(() => {
    const iframe = document.getElementById("external-embed") as HTMLIFrameElement;
    iframe.contentWindow!.postMessage({ channel: "hickory-embed", version: 1, op: "dispose" }, "http://127.0.0.1:4178");
  });
  await expect(frame.locator(".hickory-browser-debug")).toHaveCount(0);
  expect(unexpected).toEqual([]);
});

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-6">
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

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-7">
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

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-8">
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

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-9">
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
    await expect(page.getByLabel("Program output")).toHaveText("Hello, developer\n");
    expect(unexpected).toEqual([]);
  } finally { server.closeAllConnections(); await new Promise<void>((done) => server.close(() => done())); }
});

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-10">
// Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
test("homepage engineer tour executes the counterexample and correction", async ({ page }) => {
  const unexpected = await noEngine(page);
  await page.goto("/");
  await page.getByRole("button", { name: "AI engineer tour", exact: true }).click();
  const tour = page.getByRole("region", { name: "AI engineer tour" });
  await tour.getByRole("button", { name: "Continue", exact: true }).click();
  await tour.getByRole("button", { name: "Continue", exact: true }).click();
  await tour.getByLabel("Your objection").fill("Each retry receives the full deadline.");
  await tour.getByRole("button", { name: "Continue", exact: true }).click();
  await expect(tour.getByRole("button", { name: "Continue", exact: true })).toBeDisabled();
  await tour.getByRole("button", { name: "Debug document", exact: true }).click();
  await expect(tour.getByRole("button", { name: "Continue", exact: true })).toBeEnabled();
  await tour.getByRole("button", { name: "Continue", exact: true }).click();
  await tour.getByRole("button", { name: "Debug document", exact: true }).click();
  await expect(tour.getByLabel("Program output")).toContainText("R1: all three checks passed");
  await page.screenshot({ path: test.info().outputPath("engineer-tour.png"), fullPage: true });
  await expect(tour.getByRole("button", { name: "Continue", exact: true })).toBeEnabled();
  expect(unexpected).toEqual([]);
});

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-11">
// Guarantee: docs/guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical.md
test("published snapshot reuses document and failed cell views without a backend", async ({ page }) => {
  const unexpected = await noEngine(page);
  await page.goto("/snapshot.html");
  const source = "# Retry deadline\n\nThe budget includes retries.\n";
  const source_hash = await page.evaluate(async text => Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256",new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,"0")).join(""),source);
  const snapshot = {format:"hickory-viewer-snapshot",version:1,path:"spec.md",source,source_hash,captured_at_ms:0,execution:"recorded",cells:[{kind:"exec",id:"deadline-check",container:"shell",command:"node deadline.js",span:[0,1],status:"failed",expect:{match:"exact",body:"100\n"},transcript:[{t:0,kind:"out",data:"160\n"},{t:0,kind:"err",data:"deadline exceeded\n"},{t:1,kind:"exit",code:1}]}]};
  const verification = {status:"failed",revision:"a".repeat(40),source_hash,dependencies:{"Cargo.lock":"b".repeat(64)},assets:{},environment:{executor:"local"},commands:[{command:["hick","test","spec.md"],exit_code:3,stderr:"expected 100; actual 160"}],requirements:[{id:"R1",text:"Total budget across retries",witnesses:[0,9]}],origin:"executed-command; per-cell replay unavailable",scope:"Deterministic model only"};
  Object.assign(snapshot,{verification});
  await page.getByLabel("Open exported snapshot").setInputFiles({name:"snapshot.json",mimeType:"application/json",buffer:Buffer.from(JSON.stringify(snapshot))});
  await expect(page.getByText("historical snapshot",{exact:false})).toBeVisible();
  await expect(page.getByText("deadline-check · failed",{exact:true})).toBeVisible();
  await expect(page.getByRole("heading",{name:"Verification: failed"})).toBeVisible();
  await expect(page.getByRole("link",{name:/Tested revision/})).toHaveAttribute("href",`https://github.com/LoumTechnologies/hickory-docs/commit/${"a".repeat(40)}`);
  await expect(page.getByRole("link",{name:"Witness deadline-check"})).toHaveAttribute("href","#witness-0");
  await expect(page.getByText("Witness 9 unavailable · not verified")).toBeVisible();
  await expect(page.getByText("expected 100; actual 160",{exact:true})).toBeVisible();
  await expect(page.getByTestId("cell-diff")).toContainText("160");
  await expect(page.getByRole("group",{name:"spec.md",exact:true})).toContainText("Retry deadline");
  await page.screenshot({ path: test.info().outputPath("published-snapshot.png"), fullPage: true });
  await page.getByLabel("Open exported snapshot").setInputFiles({name:"tampered.json",mimeType:"application/json",buffer:Buffer.from(JSON.stringify({...snapshot,source:"changed"}))});
  await expect(page.getByRole("alert")).toContainText("hash does not match");
  expect(unexpected).toEqual([]);
});

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-12">
// Guarantees: docs/guarantees/authoring/review-threads-preserve-the-questioned-target.md
// docs/guarantees/lineage/authorship-display-accepts-non-git-providers.md
test("review selection persists its target and non-Git authorship distinguishes recorded roles", async ({ page }) => {
  const events: Record<string, unknown>[] = [];
  await page.route("**/api/reviews",async route => {
    if (route.request().method() === "POST") {
      const body=route.request().postDataJSON();
      const id=`review-${events.length}`;
      const event={id,thread:body.thread ?? id,target:body.target,body:body.body,action:body.action,actor:"user-declared",recorded_at:"1792342800000"};
      events.push(event);
      await route.fulfill({json:event});
    } else await route.fulfill({json:{scope:"workspace",events,targets:events.filter(event=>event.target).map(event=>({thread:event.thread,current_state:"unchanged"}))}});
  });
  await page.goto("/embed.html?review");
  await page.getByRole("button",{name:"Lineage settings",exact:true}).click();
  await page.getByRole("checkbox",{name:"Color credited authors"}).check();
  const editor=page.getByRole("group",{name:"spec.md",exact:true});
  await expect(editor.locator(".cm-origin-human")).toHaveCount(1);
  await expect(editor.locator(".cm-origin-agent")).toHaveCount(1);
  await page.getByRole("button",{name:"Lineage settings",exact:true}).click();
  await editor.locator(".cm-line").nth(1).click();
  await page.keyboard.press("Home"); await page.keyboard.press("Shift+End");
  await page.getByRole("button",{name:"Comment on selection"}).click();
  const panel=page.getByRole("region",{name:"Review threads",exact:true});
  await expect(panel.locator("blockquote")).toContainText("Agent claim about total deadline");
  await panel.getByLabel("Comment or reason").fill("Only each attempt is bounded");
  await panel.getByRole("button",{name:"Save comment",exact:true}).click();
  await expect(panel.getByText("Only each attempt is bounded",{exact:true})).toBeVisible();
  await panel.getByLabel("Comment or reason").fill("Reviewed the correction and its check");
  await panel.getByRole("button",{name:"Resolve after review"}).click();
  await expect(panel.getByText("Resolved by user",{exact:false})).toBeVisible();
  expect(events[0].target).toMatchObject({kind:"claim",path:"spec.md",quote:"Agent claim about total deadline",lines:[2,2]});
  expect(events).toHaveLength(2);
  await page.screenshot({path:test.info().outputPath("review-and-authorship.png"),fullPage:true});
  await page.getByRole("button",{name:"Close review threads",exact:true}).click();
  await editor.locator(".cm-line").nth(1).click();
  await page.keyboard.press("Home"); await page.keyboard.press("Shift+End");
  await page.keyboard.type("Human correction");
  await expect(editor.locator(".cm-origin-agent")).toHaveCount(0);
  await page.getByRole("button",{name:"Lineage settings",exact:true}).click();
  await page.getByRole("checkbox",{name:"Color credited authors"}).uncheck();
  await page.getByRole("checkbox",{name:"Color credited authors"}).check();
  await expect(editor.locator(".cm-origin-human")).toHaveCount(1);
  await expect(editor.locator(".cm-origin-agent")).toHaveCount(0);
});

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-13"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-13">
// Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
test("source observations share exact evidence with expanded context and stay historical after resume", async ({ page }) => {
  await noEngine(page); await page.goto("/embed.html");
  await code(page, 'var n = 4;\nvar result = n + 1;\nconsole.log(result);');
  const pane = page.getByRole("group", { name: "fixture.md", exact: true });
  await pane.locator(".cm-lineNumbers .cm-gutterElement").filter({ hasText: /^4$/ }).scrollIntoViewIfNeeded();
  const number = await pane.locator(".cm-lineNumbers .cm-gutterElement").filter({ hasText: /^4$/ }).boundingBox();
  const gutter = await pane.locator(".cm-breakpoint-gutter").boundingBox();
  if (!number || !gutter) throw Error("Missing source gutter");
  await page.mouse.click(gutter.x + gutter.width / 2, number.y + number.height / 2);
  await expect(pane.locator(".cm-bp")).toHaveCount(1);
  await page.getByRole("button", { name: "Debug document", exact: true }).click();
  const reading = page.locator('.debug-observations[data-observation-kind="debugger"]');
  await expect(reading.locator("summary")).toContainText("captured stop");
  await reading.locator("summary").click();
  await expect(reading).toContainText("PAUSED HERE");
  await expect(reading).toContainText("var result = n + 1;");
  const ids = await reading.locator("[data-evidence]").evaluateAll(elements => elements.map(e => e.getAttribute("data-evidence")));
  expect(new Set(ids).size).toBe(ids.length);
  await page.getByLabel("Watch expression").fill("n + 2");
  await page.getByRole("button", { name: "Watch", exact: true }).click();
  await expect(reading).toContainText('"value":"6"');
  await page.getByRole("button", { name: "Continue", exact: true }).click();
  await expect(reading.locator("summary")).toContainText("historical capture");
  await expect(reading).toContainText('"value":"6"');
});

</storycode:copy>
<a id="f-31019d3567c7-for_in_statement-2"></a>
### `for_in_statement`

<storycode:copy id="f-31019d3567c7-for_in_statement-2">
// Guarantee: docs/guarantees/execution/browser-output-has-runtime-source-evidence.md
for (const language of ["js", "ts"]) {
  test(`source-linked ${language} output preserves invocation and frozen revision`, async ({ page }) => {
    const unexpected = await noEngine(page);
    await page.goto("/embed.html");
    const type = language === "ts" ? ": number" : "";
    await code(page, `function logger(n${type}) {\n console.write('é');\n console.write('🦀\\r\\nsecond\\n');\n if (n > 0) logger(n-1);\n}\nlogger(2);\nconsole.error('Error: plain log text');\nthrow new Error('actual exception');`, language);
    const frozen = await page.getByLabel("Debug source", { exact: true }).inputValue();
    await page.getByRole("button", { name: "Debug document", exact: true }).click();
    const evidence = page.getByLabel("Runtime evidence");
    await expect(evidence).toContainText('"status":"failed"');
    const run = JSON.parse((await evidence.textContent())!);
    const captured = new TextDecoder().decode(new Uint8Array(run.sources[0].bytes));
    expect(captured).toBe(frozen);
    expect(run.sources[0].revision).toMatch(/^[a-f0-9]{64}$/);
    const partial = run.events.filter((event: { observation: { kind: string; bytes: number[] } }) =>
      event.observation.kind === "output" && new TextDecoder().decode(new Uint8Array(event.observation.bytes)) === "é");
    expect(partial).toHaveLength(3);
    expect(new Set(partial.map((event: { invocation: string }) => event.invocation)).size).toBe(3);
    for (const event of partial) {
      expect(event.emission.precision).toBe("exact");
      expect(new TextDecoder().decode(new Uint8Array(run.sources[0].bytes).slice(event.emission.bytes.start))).toMatch(/^console.write/);
      expect(event.callers.length).toBeGreaterThan(0);
    }
    expect(run.events.some((event: { observation: { kind: string; handling: string } }) => event.observation.kind === "exception" && event.observation.handling === "unhandled")).toBe(true);
    await page.getByLabel("Debug source", { exact: true }).fill(frozen.replace("logger(2)", "logger(0)"));
    expect(JSON.parse((await evidence.textContent())!).sources[0].revision).toBe(run.sources[0].revision);
    expect(unexpected).toEqual([]);
  });
}

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-14"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-14">
// Guarantee: docs/guarantees/execution/runtime-history-keeps-runs-and-occurrences-separate.md
test("runtime history selects occurrences, retains pins, marks stale evidence and compares failed runs", async ({ page }) => {
  const unexpected = await noEngine(page); await page.goto("/embed.html");
  await code(page, "function emit(n) { console.log(n); }\nemit(1); emit(2);");
  await page.getByRole("button", { name: "Debug document", exact: true }).click();
  await expect(page.getByLabel("Runtime evidence")).toContainText('"status":"completed"');
  const first = JSON.parse((await page.getByLabel("Runtime evidence").textContent())!);
  const pane = page.getByRole("region", { name: "Runtime run history" });
  await expect(pane.locator("select").first()).toContainText(first.id);
  const invocation = pane.getByLabel("Invocation");
  const firstOccurrence = await invocation.locator("option").nth(1).getAttribute("value");
  await invocation.selectOption(firstOccurrence!);
  await expect(pane.getByText(/Selected occurrence output/)).toContainText(firstOccurrence!);
  const inline = page.locator(".runtime-inline summary").first();
  await inline.focus(); await inline.press("Enter");
  await expect(page.locator(".runtime-inline[open] pre").first()).toContainText(first.events[0].id);
  await pane.getByRole("button", { name: "Pin run", exact: true }).click();
  await expect(pane.getByRole("button", { name: "Unpin run", exact: true })).toBeVisible();
  await page.reload();
  await expect(page.getByRole("button", { name: "Unpin run", exact: true })).toBeVisible();
  await code(page, "var i; for(i=0;i<40;i++) console.log('long stream ' + i);\nthrow new Error('new failure');");
  await page.getByRole("button", { name: "Debug document", exact: true }).click();
  await expect(page.getByLabel("Runtime evidence")).toContainText('"status":"failed"');
  const second = JSON.parse((await page.getByLabel("Runtime evidence").textContent())!);
  await expect(pane.getByText(new RegExp(`^Run ${second.id} · failed`))).toBeVisible();
  await pane.getByRole("combobox", { name: "Runtime run", exact: true }).selectOption(first.id);
  await expect(pane.getByText(/historical · source changed/)).toBeVisible();
  await expect(page.locator(".runtime-inline")).toHaveCount(0);
  await pane.getByLabel("Compare retained run").selectOption(second.id);
  await expect(pane.getByLabel("Separate run comparison")).toContainText("observations remain separate");
  await pane.locator(".debug-observations summary").click();
  await expect(pane.locator(`[data-evidence="${first.events[0].id}"]`)).toBeVisible();
  expect(unexpected).toEqual([]);
});

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-15"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-15">
// Guarantee: docs/guarantees/execution/runtime-logs-preserve-contributor-relationships.md
test("execution logs preserve contributors, exact occurrence navigation and review targets", async ({ page }) => {
  const unexpected = await noEngine(page);
  await page.addInitScript(() => window.addEventListener("hickory:review-target", event => Object.assign(globalThis, { runtimeReview: (event as CustomEvent).detail })));
  await page.goto("/embed.html");
  await code(page, "function logger() {\n console.write('same');\n console.write(' line\\npartial');\n}\nlogger(); logger();");
  await page.getByRole("button", { name: "Debug document", exact: true }).click();
  await expect(page.getByLabel("Runtime evidence")).toContainText('"status":"completed"');
  const run = JSON.parse((await page.getByLabel("Runtime evidence").textContent())!);
  await page.getByText(/Execution log and source relationships/).click();
  const log = page.locator(".runtime-log");
  await expect(log.getByLabel("Execution line contributors")).toContainText("Log line 1 · LF terminated · 2 contributions");
  await expect(log.getByLabel("Execution line contributors")).toContainText("Log line 3 · partial");
  await expect(log.locator(".ribbon-group--execution").first()).toBeVisible();
  const relationship = log.locator(".ribbon-group--execution").first();
  await relationship.focus(); await relationship.press("Alt+Enter");
  await page.waitForFunction(() => !!(globalThis as unknown as { runtimeReview?: unknown }).runtimeReview);
  const target = await page.evaluate(() => (globalThis as unknown as { runtimeReview: { kind: string; source_hash: string; relationship: { run: string; event: { id: string } } } }).runtimeReview);
  expect(target.kind).toBe("runtime-relationship"); expect(target.source_hash).toBe(run.sources[0].revision);
  expect(target.relationship.run).toBe(run.id); expect(run.events.map((event: { id: string }) => event.id)).toContain(target.relationship.event.id);
  const firstLine = log.getByLabel("Execution line contributors").locator(":scope > li").first();
  await firstLine.getByRole("button", { name: "Emission · exact", exact: true }).first().click();
  await expect(page.locator(`.runtime-inline[data-run="${run.id}"][open]`).first()).toBeVisible();
  await page.getByRole("button", { name: `Show ${run.events[2].id} in execution log`, exact: true }).click();
  await expect(log.locator(`[data-runtime-event="${run.events[2].id}"]`).first()).toContainText("Selected");
  await log.getByLabel("Execution connector style").selectOption("bands");
  await expect(log.locator(".ribbon.ribbon-family-execution").first()).toBeVisible();
  await code(page, "console.write('new run\\n');");
  await page.getByRole("button", { name: "Debug document", exact: true }).click();
  await expect(page.getByLabel("Runtime evidence")).not.toContainText(run.id);
  await page.getByRole("combobox", { name: "Runtime run", exact: true }).selectOption(run.id);
  await expect(page.getByText(/historical · source changed/)).toBeVisible();
  await page.getByText(/Execution log and source relationships/).click();
  await expect(log.getByLabel("Captured execution log")).toContainText("same line");
  await expect(log.getByLabel("Captured execution log")).not.toContainText("new run");
  expect(unexpected).toEqual([]);
});

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-16"></a>
### `expression_statement`

Real native capture replay; Python execution is separately checked under confinement.

<storycode:copy id="f-31019d3567c7-expression_statement-16">
// Guarantee: docs/guarantees/execution/python-profile-keeps-function-semantics.md
// Real native capture replay; Python execution is separately checked under confinement.
test("captured Python profile shares function evidence with inline source and selected context", async ({ page }) => {
  const unexpected = await noEngine(page); await page.goto("/embed.html?profile");
  const record = JSON.parse(await readFile(resolve("src/debug/fixtures/python-profile.json"), "utf8"));
  await page.getByLabel("Retained profile record").setInputFiles({ name: "profile.json", mimeType: "application/json", buffer: Buffer.from(JSON.stringify(record)) });
  await expect(page.locator(".runtime-inline").first()).toBeVisible();
  await page.getByRole("button", { name: "Inspect selected profile context" }).click();
  await expect(page.getByLabel("Selected profile context")).toHaveText("[]");
  const profile = page.getByRole("region", { name: "Function profile evidence" });
  // section gets an accessible region from aria-label.
  await profile.locator("summary").click();
  await expect(profile).toContainText("subtotal"); await expect(profile).toContainText("500 / 500");
  await expect(profile).toContainText("10 / 1"); await expect(profile).toContainText("Unresolved/native");
  await page.getByRole("button", { name: "Inspect selected profile context" }).click();
  const projected = JSON.parse((await page.getByLabel("Selected profile context").textContent())!);
  const expected = record.events.filter((event: { observation: { kind: string } }) => event.observation.kind === "profile").map((event: { id: string }) => event.id);
  expect(projected[0].evidence).toEqual(expected);
  expect(projected[0].reading.provider_annotations.every((item: { provider_version: string }) => item.provider_version !== "unknown")).toBe(true);
  const row = profile.locator("tr").filter({ has: page.getByRole("cell", { name: "subtotal", exact: true }) });
  await row.getByRole("button", { name: /^Source/ }).click();
  await expect(page.locator(".runtime-inline[open]").first()).toContainText("subtotal");
  await expect(page.getByRole("table", { name: "Inline function profile" }).filter({ hasText: "subtotal" })).toBeVisible();
  await page.getByRole("button", { name: "Edit after captured profile" }).click();
  await expect(profile).toContainText("source changed"); await expect(row.getByRole("button", { name: /^Source/ })).toBeDisabled();
  await expect(page.locator(".runtime-inline")).toHaveCount(0);
  expect(unexpected).toEqual([]);
});

</storycode:copy>
<a id="f-31019d3567c7-expression_statement-17"></a>
### `expression_statement`

<storycode:copy id="f-31019d3567c7-expression_statement-17">
// Guarantee: docs/guarantees/authoring/github-reviews-retain-reviewed-revisions.md
test("GitHub reviews keep external revisions, selective context, refresh and offline history", async ({ page }) => {
  await noEngine(page);
  await page.route("**/api/reviews", route => route.fulfill({ json: { scope: "local", events: [], targets: [] } }));
  let refreshed = false, offline = false; const requests: string[] = [];
  const bytes = [...Buffer.from("Human requirement\nAgent claim about total deadline\n")];
  const revision = (await import("node:crypto")).createHash("sha256").update(Buffer.from(bytes)).digest("hex");
  const make = () => ({ schema_version: 1, source: { id: "document", uri: "spec.md", revision, bytes }, unsupported: ["outbound synchronization"], annotations: [
    { evidence: "thread-current", provider: "github-pr-review", provider_version: "graphql-v4", payload_schema: "hick.github-review-thread.v1", origin: { kind: "review", repository: "org/repo", pull_request: 7, thread: "thread-current", base_revision: "base", head_revision: "reviewed-head", side: "RIGHT", author: "reviewer", resolved: refreshed, outdated: false }, location: { id: "revision-bound-location", revision, line: 0, location: { source: "document", bytes: { start: 0, end: bytes.length }, precision: "exact" } }, source_precision: "exact", measurement_semantics: null, uncertainty: "exact_revision_content_mapping", payload: { mapping_state: "exact_revision_content_mapping", reviewed_source: { uri: "https://github.com/org/repo/blob/reviewed-head/spec.md" }, thread: { path: "spec.md", startLine: 1, line: 2, comments: [
      { id: "comment-1", body: "Check the requirement <script>window.reviewExecuted=true</script>", author: { login: "reviewer" }, createdAt: "2026-10-01", updatedAt: refreshed ? "2026-10-19" : "2026-10-01", url: "https://github.com/org/repo/pull/7#discussion_r1", diffHunk: "@@ -1,2 +1,2 @@\n Human requirement\n Agent claim" },
      { id: "reply-2", body: refreshed ? "Correction reviewed; externally resolved" : "Please investigate", author: { login: "other" }, createdAt: "2026-10-02", updatedAt: "2026-10-02", url: "https://github.com/org/repo/pull/7#discussion_r2" },
    ] } } },
    { evidence: "thread-old-left", provider: "github-pr-review", provider_version: "graphql-v4", payload_schema: "hick.github-review-thread.v1", origin: { kind: "review", repository: "org/repo", pull_request: 7, thread: "thread-old-left", base_revision: "unknown", head_revision: "old-head", side: "LEFT", author: "reviewer", resolved: false, outdated: true }, location: null, source_precision: "unknown", measurement_semantics: null, uncertainty: "outdated", payload: { mapping_state: "outdated", thread: { originalStartLine: 3, originalLine: 4, path: "deleted.py", comments: [{ id: "old-comment", body: "Historical deleted-side objection", url: "https://github.com/org/repo/pull/7#discussion_r3", diffHunk: "@@ -3,2 +3,0 @@\n-deleted" }] } } },
    { evidence: "pr-conversation", provider: "github-pr-conversation", provider_version: "graphql-v4", payload_schema: "hick.github-pr-conversation.v1", origin: { kind: "review", repository: "org/repo", pull_request: 7, thread: "pr-conversation", base_revision: "base", head_revision: "head", side: "PR_LEVEL", author: "reviewer", resolved: false, outdated: false }, location: null, source_precision: "unknown", measurement_semantics: null, uncertainty: "PR-level comment has no source anchor", payload: { mapping_state: "pr_level", comment: { id: "pr-comment", body: "PR-level discussion remains unanchored", url: "https://github.com/org/repo/pull/7#issuecomment-1" } } },
  ] });
  await page.route("**/api/workspace/github/review-threads?*", route => { requests.push(route.request().method()); expect(new URL(route.request().url()).searchParams.get("repository")).toBe("org/repo"); return route.fulfill(offline ? { status: 503, json: { error: "offline or permission denied" } } : { json: make() }); });
  await page.goto("/embed.html?review"); await page.getByRole("button", { name: "Review threads", exact: true }).click();
  const provider = page.getByRole("region", { name: "GitHub source review annotations" });
  await provider.getByLabel("Review repository").fill("org/repo"); await provider.getByLabel("Review PR number").fill("7");
  await provider.getByRole("button", { name: "Read / refresh selected PR reviews" }).click();
  await expect(provider.locator(":scope > section > details > summary")).toHaveCount(3);
  await expect(page.locator(".provider-inline")).toHaveCount(1);
  await page.getByRole("button", { name: "Inspect selected GitHub context" }).click(); await expect(page.getByLabel("Selected GitHub context")).toHaveText("[]");
  const current = provider.locator('article[data-evidence="thread-current"]').locator("..");
  await current.locator("summary").first().click(); await expect(current).toContainText("Please investigate");
  await expect(current.getByRole("link", { name: /Exact GitHub discussion · comment-1/ })).toHaveAttribute("href", "https://github.com/org/repo/pull/7#discussion_r1");
  await page.getByRole("button", { name: "Inspect selected GitHub context" }).click();
  let selected = JSON.parse((await page.getByLabel("Selected GitHub context").textContent())!);
  expect(selected.map((item: { evidence: string[] }) => item.evidence)).toEqual([["thread-current"]]); expect(selected[0].reading.provider_annotations[0].origin).not.toHaveProperty("run");
  refreshed = true; await provider.getByRole("button", { name: "Read / refresh selected PR reviews" }).click(); await expect(current).toContainText("Correction reviewed; externally resolved"); await expect(current).toContainText("GitHub state: resolved");
  const editor = page.getByRole("group", { name: "spec.md", exact: true }).locator(".cm-content");
  await editor.locator(".cm-line").last().click(); await page.keyboard.press("ControlOrMeta+End"); await page.keyboard.type(" changed after capture");
  await expect(editor).toContainText("changed after capture");
  await expect(page.locator(".provider-inline")).toHaveCount(0); await expect(current.locator("summary").first()).toContainText("source changed");
  await page.getByRole("button", { name: "Inspect selected GitHub context" }).click();
  selected = JSON.parse((await page.getByLabel("Selected GitHub context").textContent())!); expect(selected[0].state).toBe("changed");
  offline = true; await provider.getByRole("button", { name: "Read / refresh selected PR reviews" }).click(); await expect(provider.getByRole("alert")).toContainText("Previous capture remains historical"); await expect(page.locator(".provider-inline")).toHaveCount(0);
  expect(requests).toEqual(["GET", "GET", "GET"]); expect(await page.evaluate(() => (window as unknown as { reviewExecuted?: boolean }).reviewExecuted)).toBeUndefined();
});
</storycode:copy>

### Assemble `apps/web/e2e/browserEmbedding.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/browserEmbedding.spec.ts"><storycode:paste select="#f-31019d3567c7-noEngine"/><storycode:paste select="#f-31019d3567c7-code"/><storycode:paste select="#f-31019d3567c7-expression_statement"/><storycode:paste select="#f-31019d3567c7-for_in_statement"/><storycode:paste select="#f-31019d3567c7-expression_statement-2"/><storycode:paste select="#f-31019d3567c7-expression_statement-3"/><storycode:paste select="#f-31019d3567c7-expression_statement-4"/><storycode:paste select="#f-31019d3567c7-expression_statement-5"/><storycode:paste select="#f-31019d3567c7-expression_statement-6"/><storycode:paste select="#f-31019d3567c7-expression_statement-7"/><storycode:paste select="#f-31019d3567c7-expression_statement-8"/><storycode:paste select="#f-31019d3567c7-expression_statement-9"/><storycode:paste select="#f-31019d3567c7-expression_statement-10"/><storycode:paste select="#f-31019d3567c7-expression_statement-11"/><storycode:paste select="#f-31019d3567c7-expression_statement-12"/><storycode:paste select="#f-31019d3567c7-expression_statement-13"/><storycode:paste select="#f-31019d3567c7-for_in_statement-2"/><storycode:paste select="#f-31019d3567c7-expression_statement-14"/><storycode:paste select="#f-31019d3567c7-expression_statement-15"/><storycode:paste select="#f-31019d3567c7-expression_statement-16"/><storycode:paste select="#f-31019d3567c7-expression_statement-17"/></storycode:file>

</storycode:doc>
