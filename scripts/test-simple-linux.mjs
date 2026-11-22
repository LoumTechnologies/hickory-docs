import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { access, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { createInterface } from "node:readline";
import { setTimeout as delay } from "node:timers/promises";
import { chromium, expect } from "../apps/web/node_modules/@playwright/test/index.mjs";

const hostOnly = process.argv.includes("--host-only");
const binary = resolve(hostOnly
  ? "target/debug/examples/simple_dev"
  : "target/simple-linux/app/bin/hickory-simple");
await access(binary);
const temporary = await mkdtemp(join(tmpdir(), "hickory-linux-"));
const project = join(temporary, "workspace");
await mkdir(project);
const file = "note + #é.txt";
const source = join(project, file);
await writeFile(source, "first line\n");
await writeFile(join(project, "sibling.txt"), "same workspace\n");
await writeFile(join(project, "author.md"), [
  '<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">',
  '<hick:file path="generated.txt">Generated source',
  "</hick:file>",
  "</hick:doc>",
  "",
].join("\n"));
await writeFile(join(project, "generated.txt"), "Generated source\n");
const visualSource = [
  '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">',
  '# Platform colors', '', '| Product | Quantity |', '| --- | --- |', '| Notebook | 2 |', '',
  '<h:math>x^2 + y^2</h:math>', '',
  '<h:diagram renderer="graph">',
  '{"topology":{"nodes":[{"id":"note"}],"edges":[]},"layout":{"note":{"x":20,"y":20,"w":130,"h":50}}}',
  '</h:diagram>', '</h:doc>', '',
].join("\n");
await writeFile(join(project, "colors.md"), visualSource);

async function sparePort() {
  const server = createServer();
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const port = server.address().port;
  await new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve()));
  return port;
}

async function until(read, description) {
  const deadline = Date.now() + 25000;
  do {
    const value = await read();
    if (value) return value;
    await delay(100);
  } while (Date.now() < deadline);
  throw new Error(`Timed out waiting for ${description}`);
}

const debug = hostOnly ? null : `http://127.0.0.1:${await sparePort()}`;
const settings = {
  ...process.env,
  HICKORY_STATE_DIR: join(temporary, ".engine"),
  XDG_DATA_HOME: join(temporary, ".data"),
  XDG_CACHE_HOME: join(temporary, ".cache"),
  XDG_CONFIG_HOME: join(temporary, ".config"),
  SHELL: "/bin/sh",
};
if (debug) settings.QTWEBENGINE_REMOTE_DEBUGGING = new URL(debug).host;
const args = hostOnly
  ? ["--linux-host", project, resolve("target/simple-linux/web"), "0"]
  : ["--port", "0", source];
const app = spawn(binary, args, { env: settings, stdio: ["pipe", "pipe", "inherit"] });
const stopped = once(app, "exit");
let port;
const output = createInterface({ input: app.stdout });
output.on("line", line => {
  const match = /^HICKORY_SIMPLE_PORT (\d+)$/.exec(line);
  if (match) port = Number(match[1]);
});
let browser;

async function menu(page, group, action) {
  await page.locator("nav").getByText(group, { exact: true }).click();
  await page.getByRole("button", { name: action, exact: true }).click();
}

try {
  let page;
  if (hostOnly) {
    await until(() => port, "the workspace host");
    browser = await chromium.launch({ headless: true });
    page = await browser.newPage({ viewport: { width: 1100, height: 750 } });
    const url = new URL(`http://127.0.0.1:${port}/simple.html`);
    url.searchParams.set("path", file);
    await page.goto(url.href);
  } else {
    await until(() => fetch(`${debug}/json/version`, { signal: AbortSignal.timeout(1000) })
      .then(response => response.ok).catch(() => false), "Qt WebEngine");
    browser = await chromium.connectOverCDP(debug, { noDefaults: true });
    page = await until(() => browser.contexts().flatMap(context => context.pages())
      .find(candidate => new URL(candidate.url()).pathname === "/simple.html"), "the workspace window");
  }
  const editor = page.locator(".cm-content").and(page.locator(`[aria-label=${JSON.stringify(file)}]`));
  const tree = page.locator('[aria-label="Files"]');
  await expect(editor).toHaveText("first line");
  await expect(editor).toHaveAttribute("contenteditable", "true");
  await expect(tree.locator('[title="sibling.txt"]')).toBeVisible();

  const endpoint = new URL("/api/files", page.url());
  const refused = await fetch(endpoint, { headers: { Origin: "https://other.example" } });
  assert.equal(refused.status, 403);

  await editor.press("End");
  await page.keyboard.insertText(" from the Linux workspace");
  assert.equal(await readFile(source, "utf8"), "first line\n");
  await menu(page, "File", "Save");
  await until(async () => await readFile(source, "utf8") === "first line from the Linux workspace\n",
    "the guarded save");

  if (!hostOnly) {
    const popup = page.context().waitForEvent("page", { timeout: 10000 });
    await menu(page, "File", "New Window");
    const newWindow = await popup;
    await expect(newWindow.locator(".cm-content").and(
      newWindow.locator(`[aria-label=${JSON.stringify(file)}]`))).toHaveText("first line from the Linux workspace");
    await page.bringToFront();
  }

  await tree.locator('[title="generated.txt"]').click();
  const generated = page.locator('.cm-content[aria-label="generated.txt"]');
  await expect(generated).toHaveText("Generated source");
  await expect(generated).toHaveAttribute("contenteditable", "false");
  await generated.press("End");
  await page.keyboard.insertText(" attempted change");
  await expect(generated).toHaveText("Generated source");
  assert.equal(await readFile(join(project, "generated.txt"), "utf8"), "Generated source\n");

  await menu(page, "View", "Terminal");
  const terminal = page.locator(".terminal-screen .xterm-helper-textarea");
  await terminal.focus();
  await page.keyboard.insertText("printf 'hickory-linux-terminal\\n'");
  await terminal.press("Enter");
  await until(async () => (await page.locator(".terminal-screen .xterm-rows").innerText())
    .split("\n").some(line => line.trim() === "hickory-linux-terminal"), "terminal output");
  await tree.locator('[title="colors.md"]').click();
  const table = page.locator(".table-panel__grid");
  await expect(table).toBeVisible();
  await expect(page.locator('[data-testid="graph-node-note"]')).toBeVisible();
  await expect(page.locator(".katex")).toBeVisible();
  await table.evaluate(element => { element.dataset.retainedThemeWitness = "yes"; });
  await page.locator('.cm-content[aria-label="colors.md"]').focus();
  for (const colorScheme of ["light", "dark", "light"]) {
    await page.emulateMedia({ colorScheme });
    const colors = await table.evaluate(element => {
      const host = element.getRootNode().host;
      const probe = document.createElement("span");
      probe.style.cssText = "color:CanvasText;background:Canvas";
      document.body.append(probe);
      const outer = getComputedStyle(probe), inner = getComputedStyle(host);
      const cell = getComputedStyle(element.querySelector("th"));
      const result = { outer: outer.color, inner: inner.color, scheme: inner.colorScheme,
        outerScheme: getComputedStyle(document.documentElement).colorScheme,
        foreground: cell.color, background: cell.backgroundColor };
      probe.remove(); return result;
    });
    assert.equal(colors.inner, colors.outer);
    assert.equal(colors.scheme, colors.outerScheme);
    const gutter = await page.locator('.editor-cm-host .cm-gutters').first().evaluate(element => getComputedStyle(element).backgroundColor);
    const caret = await page.locator('.editor-cm-host .cm-cursor').first().evaluate(element => getComputedStyle(element).borderLeftColor);
    assert.equal(caret, colors.outer);
    const gutterChannel = Number(gutter.match(/[\d.]+/)[0]) * (gutter.startsWith("color(srgb") ? 255 : 1);
    assert.ok(colorScheme === "dark" ? gutterChannel < 80 : gutterChannel > 180, gutter);
    const luminance = color => {
      const channels = color.match(/[\d.]+/g).slice(0, 3).map(Number).map(value => {
        const channel = color.startsWith("color(srgb") ? value : value / 255;
        return channel <= .04045 ? channel / 12.92 : ((channel + .055) / 1.055) ** 2.4;
      });
      return channels[0] * .2126 + channels[1] * .7152 + channels[2] * .0722;
    };
    const a = luminance(colors.foreground), b = luminance(colors.background);
    assert.ok((Math.max(a, b) + .05) / (Math.min(a, b) + .05) >= 4.5, JSON.stringify(colors));
    await expect(table).toHaveAttribute("data-retained-theme-witness", "yes");
  }
  assert.equal(await readFile(join(project, "colors.md"), "utf8"), visualSource);
  console.log(hostOnly ? "Shared Linux host smoke passed." : "KDE workspace smoke passed.");
} finally {
  try {
    if (browser) await browser.close();
  } finally {
    app.stdin.end();
    if (!hostOnly) app.kill("SIGTERM");
    const exited = await Promise.race([stopped.then(() => true), delay(9000, false, { ref: false })]);
    if (!exited) {
      app.kill("SIGKILL");
      await stopped;
    }
    await rm(temporary, { recursive: true, force: true });
  }
}
