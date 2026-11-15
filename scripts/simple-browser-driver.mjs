import { chromium } from "../apps/web/node_modules/@playwright/test/index.mjs";
import { createInterface } from "node:readline";
let browser, page, nativeTable;
const rememberedEditors = new Map();
const pageErrors = [];
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
          page.on("pageerror", error => { pageErrors.push(String(error)); if (pageErrors.length > 8) pageErrors.shift(); });
          await page.goto(request.url); break;
        case "browser-surface":
          await page.close(); page = await browser.newPage({ viewport: { width: 1100, height: 750 } }); await page.goto(request.url); break;
        case "mermaid-link": {
          const node = page.locator(".diagram-figure g.node");
          await node.waitFor(); await node.dblclick(); await editor("sample.txt").waitFor();
          value = await editor("sample.txt").innerText(); break;
        }
        case "embedded-wheel": {
          const content = editor(request.path), scroller = content.locator("..");
          const panel = page.getByTestId("table-panel-scroll");
          await panel.waitFor();
          // Engage without moving the mouse, then start the wheel over prose.
          await panel.evaluate(node => node.closest(".engaged-gate").focus({ preventScroll: true }));
          await scroller.evaluate(node => { node.scrollTop = 0; });
          const box = await scroller.boundingBox(), x = box.x + 100, y = box.y + 150;
          await page.mouse.move(x, y); await page.mouse.wheel(0, 60);
          await page.waitForTimeout(120);
          const first = await scroller.evaluate(node => node.scrollTop);
          // Layout moving under a resting pointer must not transfer ownership.
          await panel.evaluate((node, y) => {
            const scroll = node.getRootNode().host.closest(".cm-scroller");
            scroll.scrollTop += node.getBoundingClientRect().top - y + 40;
          }, y);
          await page.waitForTimeout(100);
          const before = await scroller.evaluate(node => node.scrollTop);
          await page.mouse.wheel(0, 60); await page.waitForTimeout(120);
          const after = await scroller.evaluate(node => node.scrollTop);
          const nested = await panel.evaluate(node => node.scrollTop);
          await page.mouse.move(x + 2, y + 2); await page.mouse.wheel(0, 60); await page.waitForTimeout(120);
          value = { first, before, after, nested, moved: await panel.evaluate(node => node.scrollTop) }; break;
        }
        case "embedded-controls": {
          const content = editor(request.path);
          const diagram = page.locator(".rendered-diagram");
          await diagram.locator(".react-flow__node").first().waitFor();
          const height = await diagram.locator(".react-flow").evaluate(node => node.getBoundingClientRect().height);
          await page.getByRole("button", { name: "Edit source text", exact: true }).click();
          await diagram.waitFor({ state: "detached" });
          await page.getByRole("button", { name: "Render embedded content", exact: true }).click();
          await diagram.locator(".react-flow__node").first().waitFor();
          await diagram.locator('[data-testid="graph-node-a"]').dblclick();
          await editor("sample.txt").waitFor();
          value = { height, opened: await editor("sample.txt").innerText() }; break;
        }
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

        case "source-tests-gutter": {
          const rail=page.locator(".cm-debug-rail"),summary=rail.locator(`summary[aria-label="Line actions on line ${request.line}"]`);
          await summary.waitFor();const groups=rail.locator(".cm-debug-rail__group");
          await page.waitForFunction(()=>[...document.querySelectorAll(".cm-debug-rail__group")].every(node=>node.style.top));
          await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
          const before=await groups.evaluateAll(nodes=>nodes.map(node=>({at:node.dataset.sourceOffset,top:node.getBoundingClientRect().top,left:node.getBoundingClientRect().left})));
          await summary.click();
          const after=await groups.evaluateAll(nodes=>nodes.map(node=>({at:node.dataset.sourceOffset,top:node.getBoundingClientRect().top,left:node.getBoundingClientRect().left})));
          await summary.locator("..").getByRole("button",{name:/^Inspect source tests:/}).click();
          const panel=page.getByRole("dialog",{name:"Source tests",exact:true});await panel.waitFor();await panel.getByText(/^Source observation/).waitFor();
          value={text:await panel.innerText(),before,after};break;
        }
        case "coverage-marks": value=await page.locator('.cm-debug-rail button[title^="Coverage "]').count();break;
        case "coverage-gutter": {
          const mark=page.locator('.cm-debug-rail button[title^="Coverage "]').first(),details=mark.locator('xpath=ancestor::details[1]');const groups=page.locator('.cm-debug-rail__group');await page.waitForFunction(()=>[...document.querySelectorAll('.cm-debug-rail__group')].every(node=>node.style.top));await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
          const positions=()=>groups.evaluateAll(nodes=>nodes.map(node=>({at:node.dataset.sourceOffset,top:node.getBoundingClientRect().top,left:node.getBoundingClientRect().left})));
          const before=await positions();if(await details.count())await details.locator('summary').click();const after=await positions();await mark.click();await page.getByRole("dialog",{name:"Tests and code coverage",exact:true}).waitFor();value={before,after};break;
        }
        case "coverage-open": await menu("Edit","Tests and coverage");await page.getByRole("dialog",{name:"Tests and code coverage",exact:true}).waitFor();break;
        case "coverage-close": await page.getByRole("button",{name:"Close coverage",exact:true}).click();break;
        case "coverage-state": value=await page.getByRole("dialog",{name:"Tests and code coverage",exact:true}).innerText();break;
        case "coverage-discover": {
          const dialog=page.getByRole("dialog",{name:"Tests and code coverage",exact:true});await dialog.getByLabel("Project or source file",{exact:true}).fill(".");await dialog.getByRole("button",{name:"Discover runner cases",exact:true}).click();await dialog.getByText(/discovered · coverage_ui_case/).waitFor({timeout:45000});value=await dialog.innerText();break;
        }
        case "coverage-import": {
          const dialog=page.getByRole("dialog",{name:"Tests and code coverage",exact:true});await dialog.locator("summary").filter({hasText:"Import a standard report"}).click();await dialog.getByLabel("Report contents",{exact:true}).fill(request.report);await dialog.getByLabel("Original identity JSON (optional)",{exact:true}).fill(JSON.stringify(request.identity));await dialog.getByRole("button",{name:"Import report",exact:true}).click();await dialog.getByText(/executable lines covered/,{exact:false}).waitFor();value=await dialog.innerText();break;
        }
        case "coverage-source": {
          const dialog=page.getByRole("dialog",{name:"Tests and code coverage",exact:true});await dialog.getByRole("button",{name:request.path,exact:true}).last().click();if(request.stale)await dialog.getByRole("alert").filter({hasText:"Source changed"}).waitFor();else await dialog.waitFor({state:"detached"});break;
        }
        case "source-tests-open": await menu("Edit","Source tests");await page.getByRole("dialog",{name:"Source tests",exact:true}).waitFor();break;
        case "source-tests-read": {
          const panel=page.getByRole("dialog",{name:"Source tests",exact:true});await panel.getByRole("button",{name:"Refresh candidates",exact:true}).click();
          await panel.getByRole("button",{name:"Refresh candidates",exact:true}).waitFor();
          value={text:await panel.innerText(),runEnabled:await panel.getByRole("button",{name:"Run name filter",exact:true}).first().isEnabled()};break;
        }
        case "source-tests-state": {
          const panel=page.getByRole("dialog",{name:"Source tests",exact:true});value={text:await panel.innerText(),runEnabled:await panel.getByRole("button",{name:"Run name filter",exact:true}).first().isEnabled()};break;
        }
        case "source-tests-run": {
          const panel=page.getByRole("dialog",{name:"Source tests",exact:true});await panel.getByRole("button",{name:"Run name filter",exact:true}).first().click();
          await page.waitForFunction(()=>{const panel=document.querySelector('.simple-source-tests');return panel && [...panel.querySelectorAll('button')].some(button=>button.textContent==='Run name filter'&&!button.disabled) && !!panel.querySelector('[role="status"]')?.textContent;});
          value={text:await panel.innerText()};break;
        }
        case "source-tests-refresh-run": {
          const panel=page.getByRole("dialog",{name:"Source tests",exact:true});await panel.getByRole("button",{name:"Refresh this run",exact:true}).click();
          await page.waitForTimeout(100);value={text:await panel.innerText()};break;
        }
        case "source-tests-edit": await editor(request.path).click({position:{x:16,y:8}});await page.keyboard.press(process.platform === "darwin" ? "Meta+ArrowDown" : "Control+End");await page.keyboard.insertText(request.text);break;
        case "source-tests-source": await page.getByRole("dialog",{name:"Source tests",exact:true}).getByRole("button",{name:/^Open authored source/}).first().click();await page.getByRole("dialog",{name:"Source tests",exact:true}).waitFor({state:"detached"});break;
        case "source-tests-revisions": {const panel=page.getByRole("dialog",{name:"Source tests",exact:true});await panel.getByText("Saved revisions checked at admission",{exact:true}).click();value=await panel.innerText();break;}
        case "source-tests-close": await page.getByRole("button",{name:"Close source tests",exact:true}).click();break;
        case "remember-editor": rememberedEditors.set(request.path, await editor(request.path).elementHandle()); break;
        case "editor-is-retained": value = await editor(request.path).evaluate((node, previous) => node === previous, rememberedEditors.get(request.path)); break;
        case "navigate-history":
          await menu("Edit", request.direction === "back" ? "Navigate back" : "Navigate forward");
          await editor(request.path).waitFor(); break;
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
        case "change-open": await menu("Edit", request.command || "Rename symbol"); await page.getByRole("dialog",{name:"Workspace change preview",exact:true}).waitFor(); break;
        case "action-choose": {
          const panel=page.getByRole("dialog",{name:"Workspace change preview",exact:true});await panel.getByRole("button",{name:request.title,exact:true}).click();await panel.getByRole("button",{name:"Apply to source drafts",exact:true}).waitFor();break;
        }
        case "change-prepare": {
          const panel=page.getByRole("dialog",{name:"Workspace change preview",exact:true});
          if (request.name) await panel.getByRole("textbox",{name:"New symbol name",exact:true}).fill(request.name);
          await panel.getByRole("button",{name:request.command || "Prepare rename",exact:true}).click();
          await page.waitForFunction(() => { const panel=[...document.querySelectorAll('[role="dialog"]')].find(node=>node.getAttribute("aria-label")==="Workspace change preview"); return !!panel && !panel.textContent.includes("Preparing complete source changes…"); });
          value=await panel.innerText(); break;
        }
        case "change-state": {
          const panel=page.getByRole("dialog",{name:"Workspace change preview",exact:true});
          value={text:await panel.innerText(),size:await editor(request.path).locator("..").evaluate(node=>({width:node.clientWidth,height:node.clientHeight}))}; break;
        }
        case "change-file": await page.getByRole("combobox",{name:"Preview source file"}).selectOption({label:request.path}); break;
        case "change-apply": await page.getByRole("button",{name:"Apply to source drafts",exact:true}).click(); break;
        case "change-close": await page.getByRole("button",{name:"Close change preview",exact:true}).click(); break;
        case "workspace-search-query": {
          await menu("Edit", "Find in workspace");
          const panel = page.getByRole("dialog", { name: "Find in workspace", exact: true });
          await panel.getByRole("textbox", { name: "Workspace search query" }).fill(request.query);
          await panel.getByRole("checkbox", { name: "Regular expression", exact: true }).setChecked(!!request.regex);
          await panel.getByRole("checkbox", { name: "Match case", exact: true }).setChecked(!!request.matchCase);
          await panel.getByRole("checkbox", { name: "Whole word", exact: true }).setChecked(!!request.wholeWord);
          await panel.getByRole("button", { name: "Search", exact: true }).click();
          await panel.getByText(/\d+ occurrences/).waitFor();
          value = { text: await panel.innerText(), size: await editor(request.path).locator("..").evaluate(node => ({ width: node.clientWidth, height: node.clientHeight })) }; break;
        }
        case "workspace-search-hit": await page.getByRole("dialog", { name: "Find in workspace", exact: true }).getByRole("button", { name: request.label, exact: true }).click(); break;
        case "workspace-search-notice": value = await page.getByRole("dialog", { name: "Find in workspace", exact: true }).innerText(); break;
        case "workspace-search-close": await page.getByRole("button", { name: "Close workspace search", exact: true }).click(); break;
        case "everywhere-open":
          if(request.via==="shortcut")await page.keyboard.press(process.platform==="darwin"?"Meta+p":"Control+p");else await menu("Edit","Search Everywhere");
          await page.getByRole("dialog",{name:"Search Everywhere",exact:true}).waitFor();break;
        case "everywhere-query": {
          const panel=page.getByRole("dialog",{name:"Search Everywhere",exact:true});
          await panel.getByRole("combobox",{name:"Search scope",exact:true}).selectOption(request.kind??"all");
          await panel.getByRole("combobox",{name:"Search files, types, members and actions",exact:true}).fill(request.query);break;
        }
        case "everywhere-state": {
          const panel=page.getByRole("dialog",{name:"Search Everywhere",exact:true}),options=panel.getByRole("listbox").getByRole("option");
          value=await panel.evaluate(node=>{const options=[...node.querySelectorAll('[role="listbox"] [role="option"]')];return {text:node.innerText,options:options.map(item=>item.textContent),disabled:options.filter(item=>item.disabled).length,finished:!node.querySelector('button[aria-label="Read workspace"]').disabled};});
          if(request.path){value.source=await editor(request.path).innerText();value.identity=await editor(request.path).getAttribute("data-simple-acceptance-identity");value.rect=await editor(request.path).locator("..").evaluate(node=>{const r=node.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};});}break;
        }
        case "everywhere-start":await page.getByRole("dialog",{name:"Search Everywhere",exact:true}).getByRole("button",{name:"Read workspace",exact:true}).click();break;
        case "everywhere-close":await page.getByRole("button",{name:"Close Search Everywhere",exact:true}).click();break;
        case "everywhere-choose": {
          const panel=page.getByRole("dialog",{name:"Search Everywhere",exact:true});
          if(request.keyboard)await panel.getByRole("combobox",{name:"Search files, types, members and actions",exact:true}).press("Enter");else await panel.getByRole("listbox").getByRole("option").nth(request.index??0).click();break;
        }
        case "mark-editor":value=await editor(request.path).evaluate(node=>node.dataset.simpleAcceptanceIdentity ||= crypto.randomUUID());break;
        case "editor-rect":value=await editor(request.path).locator("..").evaluate(node=>{const r=node.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};});break;
        case "analysis-open":
          if(request.via==="menu") await menu("Edit","Workspace analysis");
          else await page.getByRole("button",{name:"Workspace analysis status",exact:true}).click();
          await page.getByRole("dialog",{name:"Workspace analysis",exact:true}).waitFor();break;
        case "analysis-start": await page.getByRole("button",{name:"Read workspace",exact:true}).click();break;
        case "analysis-close": await page.getByRole("button",{name:"Close analysis",exact:true}).click();break;
        case "analysis-state": {
          const panel=page.getByRole("dialog",{name:"Workspace analysis",exact:true});
          value={text:await panel.innerText(),finished:await panel.getByRole("button",{name:"Read workspace",exact:true}).isEnabled(),source:await editor(request.path).innerText(),size:await editor(request.path).locator("..").evaluate(node=>({width:node.clientWidth,height:node.clientHeight}))};break;
        }
        case "problem-count": {
          const control = page.getByRole("button", { name: /^Problems: \d+ reported$/ });
          value = Number((await control.getAttribute("aria-label")).match(/\d+/)[0]); break;
        }
        case "problems-open":
          if (request.via === "menu") await menu("Edit", "Problems");
          else await page.getByRole("button", { name: /^Problems: \d+ reported$/ }).click();
          await page.getByRole("dialog", { name: "Workspace Problems" }).waitFor(); break;
        case "problems-state": {
          const panel = page.getByRole("dialog", { name: "Workspace Problems" });
          const text = await panel.innerText();
          value = { text, current: Number(text.match(/(\d+) current locations/)?.[1] ?? 0), size: await editor(request.path).locator("..").evaluate(node => ({ width: node.clientWidth, height: node.clientHeight })) }; break;
        }
        case "problem-next": await menu("Edit", "Next problem"); break;
        case "problems-close": await page.getByRole("button", { name: "Close Problems", exact: true }).click(); break;
        case "semantic-query": {
          const content = editor(request.path);
          await content.focus();
          await page.keyboard.press(process.platform === "darwin" ? "Meta+ArrowUp" : "Control+Home");
          for (let count = 0; count < request.offset; count++) await page.keyboard.press("ArrowRight");
          if (request.query === "Search workspace symbols") page.once("dialog", dialog => dialog.accept(request.term ?? ""));
          await menu("Edit", request.query);
          const panel = page.getByRole("dialog", { name: "Semantic locations" });
          await panel.waitFor();
          await panel.getByRole("status").filter({ hasText: /locations|No results|unavailable|changed/i }).waitFor();
          value = { text: await panel.innerText(), size: await content.locator("..").evaluate(node => ({ width: node.clientWidth, height: node.clientHeight })) };
          break;
        }
        case "semantic-open":
          await page.getByRole("dialog", { name: "Semantic locations" }).getByRole("button", { name: request.label, exact: true }).click();
          await editor(request.path).waitFor(); break;
        case "completion-open": {
          const content=editor(request.path);await content.focus();await page.keyboard.press(process.platform==="darwin"?"Meta+ArrowUp":"Control+Home");
          for(let count=0;count<request.offset;count++)await page.keyboard.press("ArrowRight");await page.keyboard.press("Control+Space");
          const choices=page.locator(".cm-tooltip-autocomplete li");await choices.first().waitFor({timeout:1500}).catch(()=>{});value=await choices.allTextContents();break;
        }
        case "completion-pick":await page.locator(".cm-tooltip-autocomplete li").filter({hasText:request.title}).first().click();break;
        case "completion-cancel":await editor(request.path).press("Escape");await page.waitForFunction(()=>document.querySelector("footer")?.textContent.includes("Completion cancelled"));break;
        case "completion-drain":await page.waitForTimeout(150);value=await page.locator("footer").innerText();break;
        case "completion-choose": {
          await page.locator(".cm-tooltip-autocomplete li").filter({hasText:request.title}).first().click();
          await page.waitForFunction(()=>{const text=document.querySelector("footer")?.textContent??"";return text.includes("Completion")&&!text.includes("Resolving completion");});value=await page.locator("footer").innerText();break;
        }
        case "redo":await editor(request.path).press(process.platform==="darwin"?"Meta+Shift+z":"Control+y");break;
        case "focus-source":await editor(request.path).focus();await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));break;
        case "type-source-selection":await editor(request.path).focus();await page.keyboard.insertText(request.text);break;
        case "construction-open":await menu("View","Construction reading");await page.getByRole("dialog",{name:"Construction reading",exact:true}).waitFor();break;
        case "construction-read":{const panel=page.getByRole("dialog",{name:"Construction reading",exact:true});await panel.getByLabel("Construction entry",{exact:true}).fill(request.entry);await panel.getByLabel("Contributor or output (optional)",{exact:true}).fill(request.focus??"");await panel.getByRole("button",{name:"Read captured drafts",exact:true}).click();await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Construction reading"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});value=await panel.innerText();break;}
        case "construction-side":{const panel=page.getByRole("dialog",{name:"Construction reading",exact:true});await panel.getByRole("button",{name:request.after?"Draft construction":"Saved construction",exact:true}).click();value=await panel.locator("pre").textContent();break;}
        case "construction-close":await page.getByRole("dialog",{name:"Construction reading",exact:true}).getByRole("button",{name:"Close",exact:true}).click();break;
        case "construction-history":await menu("Edit",request.redo?"Redo text edit":"Undo text edit");break;
        case "construction-source":value=await editor(request.path).evaluate(async node=>{const {EditorView}=await import("/node_modules/@codemirror/view/dist/index.js");const view=EditorView.findFromDOM(node);if(!view)throw new Error("Source editor unavailable.");return view.state.doc.toString();});break;
        case "construction-set-source":await editor(request.path).click({position:{x:16,y:8}});await page.keyboard.press(process.platform === "darwin"?"Meta+A":"Control+A");await page.keyboard.insertText(request.source);break;
        case "attribution-open":await menu("View","Saved file attribution");await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Saved file attribution"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});value=await page.getByRole("dialog",{name:"Saved file attribution",exact:true}).innerText();break;
        case "attribution-commit":await page.getByRole("button",{name:`Read commit for saved line ${request.line}`,exact:true}).click();await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Repository history"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});break;
        case "history-state":value=await page.getByRole("dialog",{name:"Repository history",exact:true}).innerText();break;
        case "history-open":await menu("View",request.file?"Saved file history":"Repository history");await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Repository history"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});value=await page.getByRole("dialog",{name:"Repository history",exact:true}).innerText();break;
        case "history-read": {const panel=page.getByRole("dialog",{name:"Repository history",exact:true});await panel.getByRole("button",{name:`Read commit ${request.sha}`,exact:true}).click();await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Repository history"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});value=await panel.innerText();break;}
        case "history-file": {const select=page.getByRole("combobox",{name:"Historical file",exact:true}),valueToSelect=await select.locator("option").evaluateAll((options,path)=>options.find(option=>option.textContent.endsWith(` · ${path}`))?.value,request.path);if(valueToSelect===undefined)throw new Error(`Historical file unavailable: ${request.path}`);await select.selectOption(valueToSelect);break;}
        case "history-side":await page.getByRole("combobox",{name:"Historical side",exact:true}).selectOption(request.side);value=await page.getByLabel("Historical content",{exact:true}).innerText();break;
        case "history-close":await page.getByRole("dialog",{name:"Repository history",exact:true}).getByRole("button",{name:"Close",exact:true}).click();await page.getByRole("dialog",{name:"Repository history",exact:true}).waitFor({state:"detached"});break;
        case "review-open":await menu("View","Repository changes");await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Repository changes"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});value=await page.getByRole("dialog",{name:"Repository changes",exact:true}).innerText();break;
        case "review-select": {const panel=page.getByRole("dialog",{name:"Repository changes",exact:true});await panel.getByRole("button",{name:`${request.staged?'Staged':'Working tree'} ${request.path}`,exact:true}).click();await panel.getByRole("heading",{name:`${request.path} · ${request.staged?'Staged':'Working tree'}`,exact:true}).waitFor();await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Repository changes"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});value=await panel.getByLabel("Git diff",{exact:true}).innerText();break;}
        case "review-stage":case "review-unstage": {const panel=page.getByRole("dialog",{name:"Repository changes",exact:true});await panel.getByRole("button",{name:request.action==="review-stage"?"Stage saved file":"Unstage file",exact:true}).click();await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Repository changes"] [role="status"]')?.textContent??"";return text.startsWith("Saved paths ")||text.startsWith("Index result not confirmed");});value=await panel.getByRole("status").innerText();break;}
        case "review-status":value=await page.getByRole("dialog",{name:"Repository changes",exact:true}).getByRole("status").innerText();break;
        case "review-close":await page.getByRole("dialog",{name:"Repository changes",exact:true}).getByRole("button",{name:"Close",exact:true}).click();break;
        case "template-next":await menu("Edit","Next template field");break;
        case "template-previous":await menu("Edit","Previous template field");break;
        case "template-choices":await menu("Edit","Choose template value");break;
        case "template-finish":await menu("Edit","Finish template editing");break;
        case "selected-source-text": value=await editor(request.path).evaluate(content=>{const selected=window.getSelection();return selected?.anchorNode&&content.contains(selected.anchorNode)?selected.toString():null;});break;
        case "caret-offset":
          value = await editor(request.path).evaluate(content => {
            const selection = window.getSelection(); if (!selection?.anchorNode || !content.contains(selection.anchorNode)) return null;
            // Logical editor lines are block elements: Range.toString omits
            // their separating source newlines. Count those boundaries too.
            const line = (selection.anchorNode.nodeType === Node.ELEMENT_NODE ? selection.anchorNode : selection.anchorNode.parentElement)?.closest(".cm-line");
            if (!line) return null;
            const rows = [...content.querySelectorAll(".cm-line")], index = rows.indexOf(line);
            const range = document.createRange(); range.selectNodeContents(line); range.setEnd(selection.anchorNode, selection.anchorOffset);
            return rows.slice(0, index).reduce((offset, row) => offset + row.textContent.length + 1, 0) + range.toString().length;
          }); break;
        case "semantic-close": await page.getByRole("button", { name: "Close semantic locations", exact: true }).click(); break;
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
          await page.getByRole("button", { name: /Render embedded content|Edit tables? as grids?/ }).click();
          await page.getByTestId("table-panel").waitFor(); break;
        case "resize-gutter": {
          const grip = page.getByRole("separator", { name: "Resize right gutter" });
          await grip.click(); await grip.press("ArrowLeft"); await grip.press("ArrowLeft");
          await page.waitForFunction(() => document.querySelector(".cm-debug-rail")?.clientWidth >= 64);
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
    } catch (error) {
      const visible = page && !page.isClosed() ? await page.locator("body").innerText({ timeout: 1000 }).catch(() => "") : "";
      process.stdout.write(JSON.stringify({ error: [String(error), ...pageErrors, visible.slice(0, 1200)].filter(Boolean).join("\n") }) + "\n");
    }
  }
} finally { await browser?.close(); }
