import { chromium } from "../apps/web/node_modules/@playwright/test/index.mjs";
import { createInterface } from "node:readline";
let browser, page;
const lines = createInterface({ input: process.stdin });
async function menu(name, action) {
  await page.locator("nav").getByText(name, { exact: true }).click();
  await page.getByRole("button", { name: action, exact: true }).click();
}
const editor = path => page.locator(".cm-content").and(page.locator(`[aria-label=${JSON.stringify(path)}]`));
try {
  for await (const line of lines) {
    try {
      const request = JSON.parse(line);
      let value = null;
      switch (request.action) {
        case "launch":
          browser = await chromium.launch({ headless: true }); page = await browser.newPage({ viewport: { width: 1100, height: 750 } });
          await page.goto(request.url); break;
        case "open":
          await page.getByRole("button", { name: request.path, exact: true }).click();
          await editor(request.path).waitFor(); break;
        case "append":
          await editor(request.path).click();
          await page.keyboard.press(process.platform === "darwin" ? "Meta+ArrowDown" : "Control+End");
          await page.keyboard.insertText(request.text); break;
        case "content": value = await editor(request.path).innerText(); break;
        case "save": await menu("File", "Save"); break;
        case "split": await menu("View", "Split below"); break;
        case "move": await menu("View", "Move tab to next group"); break;
        case "undo": await editor(request.path).press(process.platform === "darwin" ? "Meta+z" : "Control+z"); break;
        case "close-tab": await menu("File", "Close tab"); break;
        case "alerts": value = await page.getByRole("alert").allTextContents(); break;
        case "tabs": value = await page.getByRole("tab").allTextContents(); break;
        case "terminal": await menu("View", "Terminal"); break;
        case "terminal-draft": await page.getByRole("textbox", { name: "Terminal input", exact: true }).fill(request.text); break;
        case "read-terminal-draft": value = await page.getByRole("textbox", { name: "Terminal input", exact: true }).inputValue(); break;
        case "terminal-send": await page.getByRole("textbox", { name: "Terminal input", exact: true }).press("Enter"); break;
        case "terminal-screen": value = await page.getByLabel("Terminal screen", { exact: true }).innerText(); break;
        case "terminal-eof": await page.getByRole("button", { name: "EOF", exact: true }).click(); break;
        case "agent": await menu("View", "Agent"); break;
        case "draft": await page.getByRole("textbox", { name: "Message", exact: true }).fill(request.text); break;
        case "read-draft": value = await page.getByRole("textbox", { name: "Message", exact: true }).inputValue(); break;
        case "screenshot": await page.screenshot({ path: request.path }); break;
        case "quit": await browser?.close(); browser = undefined; break;
        default: throw new Error(`Unknown acceptance action: ${request.action}`);
      }
      process.stdout.write(JSON.stringify({ value }) + "\n");
    } catch (error) { process.stdout.write(JSON.stringify({ error: String(error) }) + "\n"); }
  }
} finally { await browser?.close(); }
