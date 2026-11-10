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
          if (/\.csv$/i.test(request.path)) await page.getByTestId("table-panel").waitFor();
          else await editor(request.path).waitFor(); break;
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
          const active = table.locator(".table-panel__input");
          if (!await active.count() || await active.inputValue() !== request.before) await table.getByRole("gridcell", { name: request.before, exact: true }).dblclick();
          const input = table.locator(".table-panel__input");
          await input.fill(request.value); await input.press("Enter");
          await table.getByRole("gridcell", { name: request.value, exact: true }).waitFor(); break;
        }
        case "table-keys": {
          const table = page.getByTestId("table-panel");
          const input = table.locator(".table-panel__input");
          if (await input.count() && await input.inputValue() === request.before) await input.focus();
          else await table.getByRole("gridcell", { name: request.before, exact: true }).click();
          for (const key of request.keys) await page.keyboard.press(key);
          value = await table.locator(".table-panel__input").count() ? await table.locator(".table-panel__input").inputValue() : await table.locator(".table-panel__cell--selected").textContent(); break;
        }
        case "table-open-caret": {
          const table = page.getByTestId("table-panel");
          await table.getByRole("gridcell", { name: request.before, exact: true }).dblclick();
          const input = table.locator(".table-panel__input");
          await input.press(process.platform === "darwin" ? "Meta+ArrowLeft" : "Home");
          for (let i = 0; i < request.offset; i++) await input.press("ArrowRight");
          break;
        }
        case "table-caret":
          value = await page.locator(".table-panel__input").evaluate(input => ({ start: input.selectionStart, end: input.selectionEnd })); break;
        case "table-grid":
          await page.getByRole("button", { name: /Edit tables? as grids?/ }).click();
          await page.getByTestId("table-panel").waitFor(); break;
        case "resize-gutter": {
          const grip = page.getByRole("separator", { name: "Resize right gutter" });
          await grip.focus(); await page.keyboard.press("ArrowLeft"); await page.keyboard.press("ArrowLeft");
          value = await page.locator(".cm-debug-rail").evaluate(node => node.clientWidth); break;
        }
        case "table-source":
          await page.getByRole("button", { name: /Edit source( text)?$/, exact: true }).click();
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
        case "indent-tabs":
          await editor(request.path).click();
          await page.keyboard.press(process.platform === "darwin" ? "Meta+ArrowDown" : "Control+End");
          await page.keyboard.press("Enter");
          await page.keyboard.press("Tab"); await page.keyboard.press("Tab");
          break;
        case "file-menu-items":
          await page.locator("nav").getByText("File", { exact: true }).click();
          value = await page.locator("nav details[open] button").allTextContents();
          await page.locator("nav").getByText("File", { exact: true }).click(); break;
        case "file-command":
          if (request.path !== undefined) page.once("dialog", dialog => dialog.accept(request.path));
          await menu("File", request.command); break;
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
