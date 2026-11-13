<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Adapt browser gestures to the shared acceptance actions

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Keep Playwright-specific selectors and gestures in this adapter. The Python acceptance flow below asserts actual disk behavior and document preservation without importing React components.

The file-editing acceptance path opens real files through the File menu, presses Return and two Tabs, saves all three whitespace styles and checks Save As against disk. It runs alone with `--file-editing-only`; browser startup failures are not passing UI evidence.

The editor-navigation witness chooses independent policies through the visible
Edit menu, sends real modifier-arrow keys, observes the browser's selected text,
and reloads to check persistence. It also checks editor geometry and unchanged
disk bytes. This flow uses no editor internals; the platform driver owns only
keyboard spelling and visible control lookup. Run it alone with
`python3 scripts/test-simple-browser.py --editor-navigation-only`.

The search motion probe samples the live editor through twelve rendered frames
in each motion mode. It compares the same DOM editor, text, viewport dimensions
and scroll position while the floating panel is visible. Media emulation changes
the user's preference on the same page, exercising live reduced-motion updates.

<ui:copy id="simple-b622660cef-0">
import { chromium } from "../apps/web/node_modules/@playwright/test/index.mjs";
import { createInterface } from "node:readline";
let browser, page, nativeTable;
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
        case "browser-surface":
          await page.close(); page = await browser.newPage({ viewport: { width: 1100, height: 750 } }); await page.goto(request.url); break;
        case "inline-feature-state": {
          if (request.kind === "diagram") {
            const graph = page.locator(".rendered-diagram");
            await graph.locator(".react-flow__node").nth(2).waitFor();
            value = { nodes: await graph.locator(".react-flow__node").count(), position: await graph.locator(".react-flow__renderer").evaluate(node => getComputedStyle(node).position) };
          } else if (request.kind === "syntax") {
            const content = editor(request.path);
            await content.locator(".tok-keyword").first().waitFor();
            value = await content.evaluate(node => ({ keywords: [...node.querySelectorAll(".tok-keyword")].map(token => token.textContent), colored: getComputedStyle(node.querySelector(".tok-keyword")).color !== getComputedStyle(node).color }));
          } else {
            const patch = page.locator('[aria-label="Project patch editor"]');
            await patch.locator(".comparison-panes .cm-content").first().waitFor();
            value = { editors: await patch.locator(".cm-content").count(), code: await patch.locator(".cm-content").allTextContents(), alerts: await patch.getByRole("alert").allTextContents() };
          }
          break;
        }
        case "enter-setting":
          await page.locator("nav").getByText("Edit", { exact: true }).click();
          await page.getByRole("combobox", { name: "Enter indentation", exact: true }).selectOption(request.mode);
          await page.locator("nav").getByText("Edit", { exact: true }).click(); break;
        case "enter-at-end":
          await editor(request.path).focus();
          await page.keyboard.press(process.platform === "darwin" ? "Meta+ArrowDown" : "Control+End");
          await page.keyboard.press("Enter"); break;
        case "editor-setting":
          await page.locator("nav").getByText("Edit", { exact: true }).click();
          await page.getByRole("combobox", { name: request.modifier + " + Left/Right", exact: true }).selectOption(request.mode);
          await page.locator("nav").getByText("Edit", { exact: true }).click(); break;
        case "editor-word-selection": {
          const content = editor(request.path);
          await content.focus();
          await page.keyboard.press(process.platform === "darwin" ? "Meta+ArrowUp" : "Control+Home");
          for (const key of request.keys) await page.keyboard.press(key);
          value = await page.evaluate(() => window.getSelection()?.toString()); break;
        }
        case "reload-editor-settings":
          await page.reload();
          await page.locator("nav").getByText("Edit", { exact: true }).click();
          value = { control: await page.getByRole("combobox", { name: "Ctrl + Left/Right", exact: true }).inputValue(), alt: await page.getByRole("combobox", { name: "Alt/Option + Left/Right", exact: true }).inputValue() };
          await page.locator("nav").getByText("Edit", { exact: true }).click(); break;
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
        case "table-dataflow-state": {
          const tables = page.getByTestId("table-panel"); await tables.nth(2).waitFor();
          const totals = tables.nth(1).locator('.table-panel__cell[data-column="2"]');
          await totals.nth(1).filter({ hasText: request.tea }).waitFor();
          const receipt = tables.nth(2).locator('.table-panel__cell[data-row="1"][data-column="1"]');
          if (request.total !== undefined) { await receipt.filter({ hasText: request.total }).waitFor(); await page.getByRole("region", { name: "Read-only table" }).getByRole("cell", { name: request.total, exact: true }).waitFor(); }
          value = { tea: await totals.nth(1).textContent(), cake: await totals.nth(2).textContent(), total: await receipt.textContent(), readonly: await page.getByRole("region", { name: "Read-only table" }).count() }; break;
        }
        case "table-dataflow-edit": {
          const cell = page.getByTestId("table-panel").nth(0).locator('.table-panel__cell[data-row="1"][data-column="1"]');
          await cell.dblclick(); const input = page.getByTestId("table-panel").nth(0).locator(".table-panel__input"); await input.fill(request.value); await input.press("Enter"); break;
        }
        case "refresh-recorded": await page.getByRole("button", { name: "Refresh recorded output", exact: true }).click(); break;
        case "native-output": {
          await page.goto(request.url);
          await page.evaluate(source => globalThis.renderNotebookOutput(source), request.source);
          const region = page.getByRole("region", { name: "Read-only table" }); await region.waitFor();
          value = { text: await region.textContent(), editable: await page.locator('.table-panel,[contenteditable="true"]').count() }; break;
        }
        case "native-table": {
          await page.close(); page = await browser.newPage({ viewport: { width: 900, height: 700 } });
          nativeTable = { revision: "native:0", source: request.source, format: request.format, readonly: false, language: "python", header: request.header ?? false, scope: request.scope };
          let revision = 0;
          await page.exposeBinding("tableHostMessage", async (_, json) => {
            const message = JSON.parse(json);
            if (message.kind === "ready") {
              await page.waitForFunction(() => !!globalThis.hickoryTable);
              await page.evaluate(snapshot => globalThis.hickoryTable.update(snapshot), nativeTable);
            } else if (message.kind === "formula") {
              try {
                const response = await fetch(`${request.backend}/api/formula/${message.operation}`, { method: message.operation === "languages" ? "GET" : "POST", headers: { "Content-Type": "application/json" }, ...(message.operation === "languages" ? {} : { body: JSON.stringify({ language: message.language, rows: message.rows, scope: message.scope }) }) });
                const value = await response.json();
                await page.evaluate(({ id, value, error }) => globalThis.hickoryTable.reply(id, value, error), { id: message.id, value, error: response.ok ? undefined : JSON.stringify(value) });
              } catch (error) { await page.evaluate(({ id, error }) => globalThis.hickoryTable.reply(id, null, error), { id: message.id, error: String(error) }); }
            } else if (message.kind === "change" && message.revision === nativeTable.revision && !nativeTable.readonly) {
              nativeTable = { ...nativeTable, source: message.source, revision: `native:${++revision}` };
              await page.evaluate(snapshot => globalThis.hickoryTable.update(snapshot), nativeTable);
            }
          });
          await page.addInitScript(() => { globalThis.HickoryTable = { postMessage: value => { void globalThis.tableHostMessage(value); } }; });
          await page.goto(request.url); await page.getByTestId("table-panel").waitFor(); break;
        }
        case "native-table-source": value = nativeTable.source; break;
        case "spreadsheet-select": {
          const table = page.getByTestId("table-panel");
          const cell = (r, c) => table.locator(`.table-panel__cell[data-row="${r}"][data-column="${c}"]`);
          await cell(request.row, request.column).click();
          if (request.endRow !== undefined) await cell(request.endRow, request.endColumn).click({ modifiers: ["Shift"] });
          value = await table.getByTestId("table-panel-name").textContent(); break;
        }
        case "spreadsheet-touch-range": {
          const table = page.getByTestId("table-panel");
          const cell = (r, c) => table.locator(`.table-panel__cell[data-row="${r}"][data-column="${c}"]`);
          await cell(request.row, request.column).click();
          const a = await cell(request.row, request.column).boundingBox(), b = await cell(request.endRow, request.endColumn).boundingBox();
          const input = await page.context().newCDPSession(page);
          const point = box => ({ x: box.x + box.width / 2, y: box.y + box.height / 2 });
          await input.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [point(a)] });
          await input.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: [point(b)] });
          await input.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
          await input.detach();
          value = await table.getByTestId("table-panel-name").textContent(); break;
        }
        case "spreadsheet-structure": await page.getByTestId("table-panel").getByRole("button", { name: request.button, exact: true }).click(); break;
        case "spreadsheet-formula-point": {
          const table = page.getByTestId("table-panel");
          const cell = (r, c) => table.locator(`.table-panel__cell[data-row="${r}"][data-column="${c}"]`);
          await cell(request.row, request.column).click();
          const bar = table.locator(".table-panel__formula"); await bar.fill(request.prefix);
          const start = await cell(request.fromRow, request.fromColumn).boundingBox();
          const end = await cell(request.toRow, request.toColumn).boundingBox();
          if (request.touch) {
            const input = await page.context().newCDPSession(page);
            await input.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: start.x + start.width / 2, y: start.y + start.height / 2 }] });
            await input.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: [{ x: end.x + end.width / 2, y: end.y + end.height / 2 }] });
            await input.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] }); await input.detach();
          } else {
            await page.mouse.move(start.x + start.width / 2, start.y + start.height / 2); await page.mouse.down();
            await page.mouse.move(end.x + end.width / 2, end.y + end.height / 2, { steps: 8 }); await page.mouse.up();
          }
          value = await bar.inputValue();
          // Return the picked draft before committing an expression in the next action.
          break;
        }
        case "spreadsheet-formula": {
          const bar = page.getByTestId("table-panel").locator(".table-panel__formula");
          await bar.fill(request.value); await bar.press("Enter"); break;
        }
        case "spreadsheet-value": value = await page.getByTestId("table-panel").locator(`.table-panel__cell[data-row="${request.row}"][data-column="${request.column}"]`).textContent(); break;
        case "spreadsheet-language": await page.getByLabel("Formula language (session)").selectOption(request.language); break;
        case "spreadsheet-height": {
          const handle = page.getByLabel("Table height", { exact: true });
          await handle.focus(); await handle.press(request.key);
          value = await page.getByTestId("table-panel-scroll").evaluate(node => node.getBoundingClientRect().height); break;
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
        case "motion-search": {
          await page.emulateMedia({ reducedMotion: request.reduced ? "reduce" : "no-preference" });
          const content = editor(request.path);
          const before = await content.evaluate(node => {
            const scroller = node.closest(".cm-scroller");
            return { text: node.textContent, x: scroller.scrollLeft, y: scroller.scrollTop,
              width: scroller.clientWidth, height: scroller.clientHeight };
          });
          const identity = await content.elementHandle();
          await menu("Edit", "Find and replace");
          const panel = page.locator(".cm-panels-top");
          const duration = await panel.evaluate(node => getComputedStyle(node).animationDuration);
          // Sample throughout the reveal, not just after it has settled.
          const samples = await content.evaluate(async node => {
            const scroller = node.closest(".cm-scroller"); const result = [];
            for (let frame = 0; frame < 12; frame++) {
              await new Promise(resolve => requestAnimationFrame(resolve));
              result.push({ text: node.textContent, x: scroller.scrollLeft, y: scroller.scrollTop,
                width: scroller.clientWidth, height: scroller.clientHeight });
            }
            return result;
          });
          const sameEditor = await content.evaluate((node, original) => node === original, identity);
          await page.getByLabel("close", { exact: true }).click();
          await identity.dispose();
          value = { duration, sameEditor, stable: samples.every(sample => JSON.stringify(sample) === JSON.stringify(before)) };
          break;
        }
        case "search-source":
          // DOM innerText adds a layout newline for an empty final CodeMirror line.
          value = (await editor(request.path).locator(".cm-line").allTextContents()).join("\n"); break;
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
            if (!await marker.isVisible()) continue; // CodeMirror keeps a hidden width spacer with the same glyph.
            const box = await marker.boundingBox();
            if (box && Math.abs(box.y + box.height / 2 - row.y - row.height / 2) < row.height / 2) {
              if (await marker.textContent() !== (request.action === "fold" ? "−" : "+")) throw new Error("Unexpected fold icon");
              const content = await editor(request.path).boundingBox();
              if (!content || box.x + box.width > content.x) throw new Error("Fold icon must be left of text");
              await marker.locator("..").click(); clicked = true; break;
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
        case "terminal-draft": await page.locator(".terminal-screen .xterm-helper-textarea").focus(); await page.keyboard.insertText(request.text); break;
        case "terminal-key": await page.locator(".terminal-screen .xterm-helper-textarea").focus(); await page.keyboard.press(request.key); break;
        case "terminal-send": await page.locator(".terminal-screen .xterm-helper-textarea").press("Enter"); break;
        case "terminal-screen": value = await page.locator(".terminal-screen .xterm-rows").innerText(); break;
        case "terminal-size":
          value = await page.locator(".terminal-screen").evaluate(node => ({ width: node.clientWidth, height: node.clientHeight, separateInputs: node.parentElement.querySelectorAll("form,input,button").length })); break;
        case "terminal-resize": await page.setViewportSize({ width: request.width, height: request.height }); break;
        case "native-terminal-protocol": {
          const native = await browser.newPage({ viewport: { width: 640, height: 360 } });
          try {
            await native.addInitScript(() => { globalThis.messages = []; globalThis.TerminalHost = { postMessage: json => globalThis.messages.push(JSON.parse(json)) }; });
            await native.goto(request.url);
            await native.evaluate(() => globalThis.startInteractiveTerminal());
            await native.waitForFunction(() => globalThis.messages.some(message => message.kind === "resize"));
            await native.locator(".xterm-helper-textarea").focus();
            await native.keyboard.insertText("café 😀"); await native.keyboard.press("ArrowUp"); await native.keyboard.press("Control+c");
            await native.evaluate(() => {
              const bytes = new TextEncoder().encode("\x1b[?1049h\x1b[2J\x1b[H\x1b[31mred café 😀\x1b[0m");
              for (const byte of bytes) globalThis.writeInteractiveTerminal(btoa(String.fromCharCode(byte)));
            });
            await native.waitForFunction(() => interactiveTerminal.buffer.active.getLine(0).translateToString(true) === "red café 😀");
            await native.setViewportSize({width:800,height:480});
            await native.waitForFunction(() => globalThis.messages.filter(message => message.kind === "resize").length > 1);
            value = await native.evaluate(() => ({ input: globalThis.messages.filter(message => message.kind === "input").map(message => message.data).join(""), sizes: globalThis.messages.filter(message => message.kind === "resize"), color: interactiveTerminal.buffer.active.getLine(0).getCell(0).getFgColor(), cursor: interactiveTerminal.options.disableStdin, network: document.querySelector('meta[http-equiv="Content-Security-Policy"]').content }));
          } finally { await native.close(); }
          break;
        }
        case "terminal-eof": await page.locator(".terminal-screen .xterm-helper-textarea").press("Control+d"); break;
        case "agent": await menu("View", "Agent"); break;
        case "agent-documents": {
          await menu("View", "Agent");
          const draft = editor("conversation/draft.md");
          await draft.fill(request.text);
          const history = editor("conversation/agent-history-user.md");
          await history.waitFor();
          const before = await history.textContent();
          await history.click(); await page.keyboard.type("cannot edit");
          const readonly = await history.getAttribute("contenteditable") === "false" && await history.textContent() === before;
          await draft.click(); await page.keyboard.press("End");
          const caret = await draft.evaluate(node => window.getSelection()?.anchorOffset);
          await page.getByLabel("Provider", { exact: true }).selectOption("claude-subscription");
          await page.getByLabel("Model", { exact: true }).fill("unavailable-model");
          await page.getByLabel("Effort", { exact: true }).selectOption("high");
          const stable = await draft.evaluate(node => { window.agentDraft = node; return node.isConnected; });
          const sent = page.waitForRequest(req => req.method() === "POST" && req.url().endsWith("/api/docs/workspace/agent"));
          await page.getByRole("button", { name: "Send ↑", exact: true }).click();
          const payload = (await sent).postDataJSON();
          await page.getByRole("alert").filter({ hasText: "not retried" }).waitFor();
          const retained = await draft.locator(".cm-line").allTextContents().then(lines => lines.join("\n"));
          await menu("View", "Files"); await menu("View", "Agent");
          value = { readonly, payload, retained, stable: stable && await draft.evaluate(node => node === window.agentDraft), history: before };
          break;
        }
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
