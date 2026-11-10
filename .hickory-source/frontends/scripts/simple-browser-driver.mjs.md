<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Adapt browser gestures to the shared acceptance actions

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Keep Playwright-specific selectors and gestures in this adapter. The Python acceptance flow below asserts actual disk behavior and document preservation without importing React components.

<ui:copy id="simple-b622660cef-0">
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
        case "dock-tab": {
          const source = page.getByRole("tab", { name: request.path, exact: true }).locator("..");
          const pane = page.getByRole("tab", { name: request.target, exact: true }).locator("../..").locator("..").getByRole("tabpanel");
          const box = await pane.boundingBox();
          if (!box) throw new Error("Target pane has no visible bounds");
          const position = { x: request.edge === "left" ? 5 : request.edge === "right" ? box.width - 5 : box.width / 2,
            y: request.edge === "top" ? 5 : request.edge === "bottom" ? box.height - 5 : box.height / 2 };
          await source.dragTo(pane, { targetPosition: position }); break;
        }
        case "pane-count": value = await page.getByRole("tabpanel").count(); break;
        case "reorder-tab": {
          const source = page.getByRole("tab", { name: request.path, exact: true }).locator("..");
          const target = page.getByRole("tab", { name: request.before, exact: true }).locator("..");
          await source.dragTo(target); break;
        }
        case "tab-order": value = await page.getByRole("tab", { name: request.path, exact: true }).locator("../..").getByRole("tab").allTextContents(); break;
        case "append":
          await editor(request.path).click();
          await page.keyboard.press(process.platform === "darwin" ? "Meta+ArrowDown" : "Control+End");
          await page.keyboard.insertText(request.text); break;
        case "semantic-colors": value = await editor(request.path).locator('[class*="cm-st-"]').allTextContents(); break;
        case "append-with-cached-colors": {
          const content = editor(request.path);
          await content.click();
          await page.keyboard.press(process.platform === "darwin" ? "Meta+ArrowDown" : "Control+End");
          await page.keyboard.insertText(request.text);
          value = await content.locator('[class*="cm-st-"]').allTextContents();
          break;
        }
        case "table-edit": {
          const table = page.getByTestId("table-panel");
          await table.waitFor();
          await table.getByRole("gridcell", { name: request.before, exact: true }).dblclick();
          const input = table.getByRole("textbox", { name: "Cell contents" });
          await input.fill(request.value); await input.press("Enter");
          await table.getByRole("gridcell", { name: request.value, exact: true }).waitFor(); break;
        }
        case "table-source":
          await page.getByRole("button", { name: "Edit source", exact: true }).click();
          await page.getByTestId("table-panel").waitFor({ state: "detached" }); break;
        case "content": value = await editor(request.path).innerText(); break;
        case "markdown-formatting": {
          const content = editor(request.path);
          await content.locator(".cm-md-h1").waitFor();
          value = await content.evaluate(element => {
            const style = selector => {
              const node = element.querySelector(selector);
              if (!node) return null;
              const css = getComputedStyle(node);
              return { text: node.textContent, weight: css.fontWeight, italic: css.fontStyle, size: parseFloat(css.fontSize) };
            };
            return { heading: style(".cm-md-h1"), bold: style(".cm-md-strong"), italic: style(".cm-md-em"), size: parseFloat(getComputedStyle(element).fontSize) };
          }); break;
        }
        case "search-open": await menu("Edit", "Find and replace"); break;
        case "search-query":
          await page.getByLabel("Find", { exact: true }).fill(request.query);
          await page.getByLabel("Find", { exact: true }).dispatchEvent("change");
          await page.getByLabel("regexp", { exact: true }).setChecked(!!request.regex);
          await page.getByLabel("Replace", { exact: true }).fill(request.replacement);
          await page.getByLabel("Replace", { exact: true }).dispatchEvent("change"); break;
        case "search-replace-all": await page.getByRole("button", { name: "replace all", exact: true }).click(); break;
        case "search-close": await page.getByLabel("close", { exact: true }).click(); break;
        case "editor-size": value = await editor(request.path).locator("..").evaluate(node => ({ width: node.clientWidth, height: node.clientHeight })); break;
        case "fold":
        case "unfold": {
          const pane = editor(request.path).locator("xpath=ancestor::div[contains(@class, 'editor-row')][1]");
          const number = pane.locator(".cm-lineNumbers .cm-gutterElement").getByText(String(request.line), { exact: true });
          const row = await number.boundingBox();
          if (!row) throw new Error("Fold header is not visible");
          const markers = pane.locator(`.cm-foldGutter span[title="${request.action === "fold" ? "Fold" : "Unfold"} line"]`);
          let clicked = false;
          for (const marker of await markers.all()) {
            const box = await marker.boundingBox();
            if (box && Math.abs(box.y + box.height / 2 - row.y - row.height / 2) < row.height / 2) {
              if (await marker.textContent() !== (request.action === "fold" ? "−" : "+")) throw new Error("Unexpected fold icon");
              const content = await editor(request.path).locator(".cm-content").boundingBox();
              if (!content || box.x + box.width > content.x) throw new Error("Fold icon must be left of text");
              await marker.click(); clicked = true; break;
            }
          }
          if (!clicked) throw new Error("No fold icon on requested line");
          break;
        }
        case "fold-count": value = await editor(request.path).locator(".cm-foldPlaceholder").count(); break;
        case "minimap": {
          const map = page.getByRole("navigation", { name: "Document minimap", exact: true }).last();
          await map.focus(); await map.press("End");
          value = await map.evaluate(element => element.getBoundingClientRect().width);
          break;
        }
        case "debug-start":
          await page.getByRole("button", { name: `Debug ${request.path}`, exact: true }).click(); break;
        case "debug-step":
          await page.getByRole("button", { name: "Step over", exact: true }).click(); break;
        case "debug-stop":
          await page.getByRole("button", { name: "Stop", exact: true }).click(); break;
        case "debug-state": value = await page.locator(".debug-strip__status").allTextContents(); break;
        case "debug-values": value = await page.locator(".cm-debug-value").allTextContents(); break;
        case "debug-breakpoint":
          await page.getByLabel(`Set breakpoint on line ${request.line}`, { exact: true }).click(); break;
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
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="scripts/simple-browser-driver.mjs"><ui:paste select="#simple-b622660cef-0"/></ui:file>

</ui:doc>
