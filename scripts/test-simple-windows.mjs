import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { once } from "node:events";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { chromium, expect } from "../apps/web/node_modules/@playwright/test/index.mjs";

async function eventually(probe, label, timeout = 45000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    const result = await probe();
    if (result) return result;
    await delay(100);
  }
  throw new Error(`Timed out waiting for ${label}.`);
}

const temp = await mkdtemp(join(tmpdir(), "hickory-windows-"));
const workspace = join(temp, "workspace");
await mkdir(workspace);
const name = "note + #é.txt";
const source = join(workspace, name);
await writeFile(source, "first line\n");
await writeFile(join(workspace, "sibling.txt"), "same workspace\n");
await writeFile(join(workspace, "author.md"), [
  '<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">',
  '<hick:file path="generated.txt">Generated source',
  "</hick:file>",
  "</hick:doc>",
  "",
].join("\n"));
await writeFile(join(workspace, "generated.txt"), "Generated source\n");

const listener = createServer().listen(0, "127.0.0.1");
await once(listener, "listening");
const debug = listener.address().port;
await new Promise((done, fail) => listener.close(error => error ? fail(error) : done()));
const app = spawn(resolve("target/simple-windows/app/HickorySimple.exe"), [source, "--port", "0"], {
  env: {
    ...process.env,
    HICKORY_STATE_DIR: join(temp, "engine"),
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${debug}`,
  },
  stdio: "ignore",
});
const stopped = once(app, "exit");
stopped.catch(() => {});
let browser;

try {
  const debuggerOrigin = `http://127.0.0.1:${debug}`;
  await eventually(() => fetch(`${debuggerOrigin}/json/version`, { signal: AbortSignal.timeout(1000) })
    .then(response => response.ok).catch(() => false), "WebView2");
  browser = await chromium.connectOverCDP(debuggerOrigin, { noDefaults: true });
  const page = await eventually(() => browser.contexts().flatMap(context => context.pages())
    .find(tab => new URL(tab.url()).pathname === "/simple.html"), "the WinForms workspace");
  const editor = page.locator(".cm-content").and(page.locator(`[aria-label=${JSON.stringify(name)}]`));
  const tree = page.locator('[aria-label="Files"]');
  await expect(editor).toHaveText("first line");
  await expect(editor).toHaveAttribute("contenteditable", "true");
  await expect(tree.locator('[title="sibling.txt"]')).toBeVisible();

  const endpoint = new URL("/api/files", page.url());
  const refused = await fetch(endpoint, { headers: { Origin: "https://other.example" } });
  assert.equal(refused.status, 403);

  await editor.press("End");
  await page.keyboard.insertText(" from WinForms");
  assert.equal(await readFile(source, "utf8"), "first line\n");
  await page.locator("nav").getByText("File", { exact: true }).click();
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await eventually(async () => await readFile(source, "utf8") === "first line from WinForms\n", "the guarded save");

  await tree.locator('[title="generated.txt"]').click();
  await expect(page.locator('.cm-content[aria-label="generated.txt"]')).toHaveAttribute("contenteditable", "false");
  assert.equal(await readFile(join(workspace, "generated.txt"), "utf8"), "Generated source\n");

  execFileSync("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command",
    `if (-not (Get-Process -Id ${app.pid}).CloseMainWindow()) { exit 1 }`]);
  const exit = await Promise.race([stopped, delay(15000, null, { ref: false })]);
  assert.equal(exit?.[0], 0, "Closing the last window should stop the app.");
  await eventually(() => fetch(endpoint, { signal: AbortSignal.timeout(1000) })
    .then(() => false).catch(() => true), "the workspace host to detach", 10000);
  console.log("WinForms workspace smoke passed.");
} finally {
  try {
    if (browser) await browser.close();
  } finally {
    if (app.pid && app.exitCode === null) {
      try { execFileSync("taskkill.exe", ["/PID", String(app.pid), "/T", "/F"], { stdio: "ignore" }); }
      catch { if (app.exitCode === null) app.kill(); }
      await Promise.race([stopped, delay(5000, undefined, { ref: false })]);
    }
    await rm(temp, { recursive: true, force: true, maxRetries: 4 });
  }
}
