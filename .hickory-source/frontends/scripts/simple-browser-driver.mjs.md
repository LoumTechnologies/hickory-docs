<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Adapt browser gestures to the shared acceptance actions

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The permission probe reads the attached editor’s read-only state and its actual content DOM editability. Dynamically importing CodeMirror to find that view must not query a second module instance’s facet identity: Vite can load the application through its optimized module while the driver imports the raw package. This observation belongs only to verification; application edit authority stays in the source model.

Recovery management gestures below are controlled acceptance automation, including both explicit whole-session confirmation clicks; they do not certify physical input or accessibility.

The multiple-template witness establishes and observes only native editor selections through `editor-selections`. Preview, Apply, linked typing and history still use the existing visible controls and input paths; this hook cannot apply a template or manufacture its provider response.

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
let templateHeldRoute,templateHoldHandler;
const capabilityGates = new Map();
const rememberedEditors = new Map();
const pageErrors = [];
const debugWire=[];
const behaviorRequests=[];
const proposalReports=[];
const lines = createInterface({ input: process.stdin });
async function menu(name, action) {
  await page.locator("nav").getByText(name, { exact: true }).click();
  await page.getByRole("button", { name: action, exact: true }).click();
}
const editor = path => page.locator(".cm-content").and(page.locator(`[aria-label=${JSON.stringify(path)}]`));
const debugRail = () => page.locator(".cm-debug-rail");
const sourceTestAction = name => debugRail().getByRole("button", {
  name: `Inspect source tests: ${name}`, exact: true, includeHidden: true,
});

async function selectGutterAction(action) {
  await action.and(page.locator("[data-gutter-icon]")).waitFor({ state: "attached" });
  const group = action.locator("xpath=ancestor::div[contains(@class,'cm-debug-rail__group')][1]");
  const offset = await group.getAttribute("data-source-offset");
  const grid = page.getByRole("dialog", { name: "Gutter icons", exact: true });
  const positions = () => debugRail().evaluate((rail, selectedOffset) => {
    const box = element => {
      const { x, y, width, height } = element.getBoundingClientRect();
      return { x, y, width, height };
    };
    const groups = [...rail.querySelectorAll(".cm-debug-rail__group")];
    return {
      groups: groups.map(box),
      following: groups.filter(group => Number(group.dataset.sourceOffset) > Number(selectedOffset)).map(group => {
        const trigger = rail.querySelector(`.cm-gutter-more[data-source-offset="${group.dataset.sourceOffset}"]`);
        const buttons = [...group.querySelectorAll("button")].filter(button => getComputedStyle(button).visibility === "visible");
        if (trigger) buttons.push(trigger);
        return buttons.map(box);
      }),
    };
  }, offset);

  const before = await positions();
  let icons = [];
  if (!await action.isVisible()) {
    await debugRail().locator(`.cm-gutter-more[data-source-offset=${JSON.stringify(offset)}]`).click();
    icons = await grid.evaluate(dialog => dialog.getAttribute("aria-owns").split(" ").map(id => {
      const button = dialog.ownerDocument.getElementById(id);
      const { x, y } = button.getBoundingClientRect();
      return { label: button.getAttribute("aria-label"), x, y };
    }));
  }
  const after = await positions();
  await action.click();
  await grid.waitFor({ state: "detached" });
  return { before, after, icons };
}
try {
  for await (const line of lines) {
    try {
      const request = JSON.parse(line);
      let value = null;
      switch (request.action) {
        case "launch":
          browser = await chromium.launch({ headless: true }); page = await browser.newPage({ viewport: { width: 1100, height: 750 } });
          page.on("pageerror", error => { pageErrors.push(String(error)); if (pageErrors.length > 8) pageErrors.shift(); });
          page.on("websocket",socket=>{for(const direction of ["framesent","framereceived"])socket.on(direction,({payload})=>{try{if(typeof payload==='string'||payload[0]!==3)return;const value=JSON.parse(new TextDecoder().decode(payload.subarray(1)));debugWire.push({direction,op:value.op,event:value.event,session:value.session,doc:value.doc,stop:value.stop,thread:value.thread,frame:value.frame,request_id:value.request_id,reference:value.reference,about:value.about,message:value.message,source_hash:value.source_hash,source_test:value.source_test?{candidate_id:value.source_test.candidate_id,selection:value.source_test.selection}:undefined});if(debugWire.length>512)debugWire.shift();}catch{}});});
          page.on("request",request=>{if(new URL(request.url()).pathname==="/api/agent-proposals/report"&&request.method()==="POST"){try{proposalReports.push(request.postDataJSON());if(proposalReports.length>8)proposalReports.shift();}catch{}}if(new URL(request.url()).pathname!=="/api/behavior/verify"||request.method()!=="POST")return;const body=request.postData();if(body&&body.length<=1048576){try{behaviorRequests.push(JSON.parse(body));if(behaviorRequests.length>16)behaviorRequests.shift();}catch{}}});
          if (request.url) await page.goto(request.url); break;
        case "browser-surface":
          await page.close(); page = await browser.newPage({ viewport: { width: 1100, height: 750 } }); await page.goto(request.url); break;
        case "workspace-welcome": {
          const files = "**/api/files";
          const unavailable = route => route.abort();
          await page.route(files, unavailable);
          await page.goto(request.url);
          const welcome = page.getByRole("heading", { name: "Welcome to Hickory Docs", exact: true });
          const address = page.getByRole("textbox", { name: "Workspace server address" });
          await welcome.waitFor();
          if (await address.count() || await page.getByRole("alert").count())
            throw new Error("A blank launch should show workspace choices without an address field or a failure.");
          await page.getByRole("button", { name: "Connect to a Workspace Server…", exact: true }).click();
          await address.fill("https://user@example.test");
          await page.getByRole("button", { name: "Connect", exact: true }).click();
          await page.getByRole("alert").getByText(/without a path or credentials/).waitFor();
          await page.getByRole("button", { name: "Back", exact: true }).click();
          if (await address.count() || await page.getByRole("alert").count())
            throw new Error("Back should return to the welcome choices.");
          await page.getByRole("button", { name: "Connect to a Workspace Server…", exact: true }).click();
          await address.fill(new URL(request.url).origin);
          await page.getByRole("button", { name: "Connect", exact: true }).click();
          await page.getByRole("alert").getByText(/Could not open the workspace/).waitFor();
          await page.unroute(files, unavailable);
          await page.getByRole("button", { name: "Connect", exact: true }).click();
          await welcome.waitFor({ state: "detached" });
          await page.locator('.file-tree').getByText("notes.txt", { exact: true }).waitFor();
          break;
        }
        case "startup-held": {
          await page.close(); capabilityGates.clear(); rememberedEditors.clear();
          page = await browser.newPage({ viewport: { width: 1100, height: 750 } });
          page.setDefaultTimeout(60_000);
          for (const [name, pattern] of Object.entries(request.gates)) {
            const gate = { route: null, pattern };
            gate.handle = route => {
              if (gate.route || name === "parser" && route.request().resourceType() === "script") return route.continue();
              gate.route = route;
            };
            capabilityGates.set(name, gate);
            await page.route(pattern, gate.handle);
          }
          await page.goto(request.url, { waitUntil: "commit" }); break;
        }
        case "startup-state": {
          await editor(request.path).waitFor();
          const tree = page.locator('[aria-label="Files"]');
          value = { tabs: await page.getByRole("tab").allTextContents(),
            siblingVisible: await tree.locator(`[title=${JSON.stringify(request.sibling)}]`).isVisible(),
            outsidePresent: await tree.locator(`[title=${JSON.stringify(request.outside)}]`).count() > 0,
            readonly: await editor(request.path).getAttribute("contenteditable") === "false" };
          break;
        }
        case "capability-held": value = !!capabilityGates.get(request.name)?.route; break;
        case "workspace-status": value = await page.locator("footer[role=status]").innerText(); break;
        case "agent-render-status": value = await page.locator(".agent-pane-status span[role=status]").innerText(); break;
        case "capability-release": {
          const gate = capabilityGates.get(request.name);
          if (!gate?.route) throw Error(`No held ${request.name} request`);
          const url = gate.route.request().url();
          if (request.fail) { await gate.route.abort(); await page.unroute(gate.pattern, gate.handle); }
          else {
            const response = page.waitForResponse(reply => reply.url() === url);
            await gate.route.continue(); await page.unroute(gate.pattern, gate.handle);
            const reply = await response;
            value = { status: reply.status(), body: request.read ? await reply.json() : undefined };
          }
          break;
        }
        case "capability-ui": {
          const content = editor(request.path);
          if (request.kind === "markdown") value = await content.locator(".cm-md-h1").count() > 0;
          else if (request.kind === "test") value = await sourceTestAction(request.name).count() > 0;
          else {
            const button = page.getByRole("button", { name: "Edit table as grid", exact: true });
            value = { available: await button.isEnabled(), reason: await button.getAttribute("title") };
          }
          break;
        }
        case "visible-document-line": {
          const content = editor(request.path);
          value = await content.evaluate(node => {
            const viewport = node.closest(".cm-scroller").getBoundingClientRect();
            const first = [...node.querySelectorAll(".cm-line")].find(line => line.getBoundingClientRect().bottom > viewport.top + 8);
            return first?.textContent ?? null;
          });
          break;
        }
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
          const reading = await selectGutterAction(sourceTestAction(request.name));
          const panel = page.getByRole("dialog", { name: "Source tests", exact: true });
          await panel.getByText(/^Source observation/).waitFor();
          value = { text: await panel.innerText(), ...reading };
          break;
        }
        case "coverage-marks": value = await debugRail().locator('button[title^="Coverage "]').count(); break;
        case "coverage-gutter": {
          value = await selectGutterAction(debugRail().locator('button[title^="Coverage "]').first());
          await page.getByRole("dialog", { name: "Tests and code coverage", exact: true }).waitFor();
          break;
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
        case "templates-open":{
          if(request.via==='search'){await menu('Edit','Search Everywhere');const search=page.getByRole('dialog',{name:'Search Everywhere',exact:true});await search.getByRole('combobox',{name:'Search scope',exact:true}).selectOption('action');await search.getByRole('combobox',{name:'Search files, types, members and actions',exact:true}).fill('Insert reusable template');await search.getByRole('option').filter({hasText:'Insert reusable template'}).first().click();}else await menu('Edit','Insert reusable template…');
          const panel=page.getByRole('dialog',{name:'Source templates',exact:true});await panel.waitFor();await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));await page.waitForFunction(expectedPath=>{const panel=document.querySelector('[role="dialog"][aria-label="Source templates"]');if(!panel||panel.textContent.includes('Reading template provider…'))return false;const source=panel.querySelector('p')?.textContent,status=panel.querySelector('[role="status"]')?.textContent;if(expectedPath&&source!==expectedPath)return false;return Array.from(panel.querySelectorAll('button')).some(button=>button.textContent.startsWith('Preview '))||!!status&&status!=='Template applied · draft';},request.path??null);value=await panel.innerText();break;
        }
        case "templates-preview":{const panel=page.getByRole('dialog',{name:'Source templates',exact:true});await panel.getByRole('button',{name:'Preview '+request.label,exact:true}).click();await panel.getByRole('region',{name:'Template preview',exact:true}).waitFor();value=await panel.innerText();break;}
        case "templates-hold-apply":{templateHeldRoute=null;let observed;const held=new Promise(resolve=>{observed=resolve;});templateHoldHandler=async route=>{if(templateHeldRoute){await route.continue();return;}templateHeldRoute=route;observed();};await page.route('**/api/templates/plan',templateHoldHandler);await page.getByRole('dialog',{name:'Source templates',exact:true}).getByRole('button',{name:'Apply template to draft',exact:true}).click();await Promise.race([held,page.waitForTimeout(15000).then(()=>{throw Error('The explicit template Apply did not request a fresh provider plan');})]);value=true;break;}
        case "templates-release-apply":{if(!templateHeldRoute)throw Error('No held template plan');const reply=page.waitForResponse(response=>response.url().endsWith('/api/templates/plan')&&response.request().method()==='POST');await templateHeldRoute.continue();const response=await reply;value={status:response.status(),body:await response.text()};await page.unroute('**/api/templates/plan',templateHoldHandler);templateHeldRoute=null;await page.waitForFunction(()=>!document.querySelector('[aria-label="Source templates"]')?.textContent.includes('Reading template provider…'));break;}
        case "templates-read":{const panel=page.getByRole('dialog',{name:'Source templates',exact:true});await panel.getByRole('button',{name:'Read templates',exact:true}).click();await page.waitForFunction(()=>{const panel=document.querySelector('[aria-label="Source templates"]');return !!panel&&!panel.textContent.includes('Reading template provider…')&&Array.from(panel.querySelectorAll('button')).some(button=>button.textContent.startsWith('Preview '));});value=await panel.innerText();break;}
        case "templates-apply":{const panel=page.getByRole('dialog',{name:'Source templates',exact:true});await panel.getByRole('button',{name:'Apply template to draft',exact:true}).click();await panel.waitFor({state:'detached'});value=await editor(request.path).evaluate(async node=>{const {EditorView}=await import('/node_modules/@codemirror/view/dist/index.js');const view=EditorView.findFromDOM(node);return {source:view.state.doc.toString(),anchor:view.state.selection.main.anchor,head:view.state.selection.main.head};});break;}
        case "templates-apply-enabled":{const button=page.getByRole('button',{name:'Apply template to draft',exact:true});value=await button.count()>0&&!await button.isDisabled();break;}
        case "templates-close":await page.getByRole('button',{name:'Close source templates',exact:true}).click();await page.getByRole('dialog',{name:'Source templates',exact:true}).waitFor({state:'detached'});break;
        case "templates-command-state":{await page.locator('nav').getByText('Edit',{exact:true}).click();const button=page.getByRole('button',{name:'Insert reusable template…',exact:true});value={disabled:await button.isDisabled(),reason:await button.getAttribute('title')};await page.locator('nav').getByText('Edit',{exact:true}).click();break;}
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
        case "source-tests-debug": {
          const panel=page.getByRole("dialog",{name:"Source tests",exact:true});
          if(request.filter!==undefined)await panel.getByLabel("Filter source tests",{exact:true}).fill(request.filter);
          const before=Number(/Retained test debug sessions \((\d+)\)/.exec(await panel.innerText())?.[1]??0);
          await panel.getByRole("button",{name:"Debug name filter",exact:true}).first().click();
          await page.waitForFunction(before=>Number(/Retained test debug sessions \((\d+)\)/.exec(document.querySelector('.simple-source-tests')?.textContent??'')?.[1]??0)>before,before);
          value=await panel.innerText();break;
        }
        case "source-tests-debug-link":await page.getByRole("dialog",{name:"Source tests",exact:true}).getByRole("button",{name:"Open this debugger/source",exact:true}).first().click();await page.getByRole("dialog",{name:"Source tests",exact:true}).waitFor({state:"detached"});break;
        case "source-tests-debug-records":value=await page.getByRole("dialog",{name:"Source tests",exact:true}).innerText();break;
        case "source-tests-refresh-run": {
          const panel=page.getByRole("dialog",{name:"Source tests",exact:true});await panel.getByRole("button",{name:"Refresh this run",exact:true}).click();
          await page.waitForTimeout(100);value={text:await panel.innerText()};break;
        }
        case "source-tests-edit": await editor(request.path).click({position:{x:16,y:8}});await page.keyboard.press(process.platform === "darwin" ? "Meta+ArrowDown" : "Control+End");await page.keyboard.insertText(request.text);break;
        case "source-tests-source": await page.getByRole("dialog",{name:"Source tests",exact:true}).getByRole("button",{name:/^Open authored source/}).first().click();await page.getByRole("dialog",{name:"Source tests",exact:true}).waitFor({state:"detached"});break;
        case "source-tests-revisions": {const panel=page.getByRole("dialog",{name:"Source tests",exact:true});await panel.getByText("Saved revisions checked at admission",{exact:true}).click();value=await panel.innerText();break;}
        case "source-tests-close": await page.getByRole("button",{name:"Close source tests",exact:true}).click();break;
        case "remember-editor": rememberedEditors.set(request.path, await editor(request.path).elementHandle()); break;
        case "editor-write-state":value=await editor(request.path).evaluate(async node=>{const {EditorView}=await import("/node_modules/@codemirror/view/dist/index.js");const view=EditorView.findFromDOM(node);if(!view)throw Error('Source editor unavailable');return {readOnly:view.state.readOnly,editable:view.contentDOM.isContentEditable};});break;
        case "source-text-state":value=await editor(request.path).evaluate(async node=>{const {EditorView}=await import("/node_modules/@codemirror/view/dist/index.js"),{undoDepth,redoDepth}=await import("/node_modules/@codemirror/commands/dist/index.js"),source=await import("/src/simple/SourceText.ts"),view=EditorView.findFromDOM(node);if(!view)throw Error('Source editor unavailable');return {source:source.authoredSource(view.state),display:view.state.doc.toString(),selection:source.authoredSelection(view.state),undo:undoDepth(view.state),redo:redoDepth(view.state)};});break;
        case "source-text-select":await editor(request.path).evaluate(async(node,selection)=>{const {EditorView}=await import("/node_modules/@codemirror/view/dist/index.js"),{editorOffset}=await import("/src/simple/SourceText.ts"),view=EditorView.findFromDOM(node);view.dispatch({selection:{anchor:editorOffset(view.state,selection.anchor),head:editorOffset(view.state,selection.head)},scrollIntoView:true,userEvent:'select'});view.focus();},request);break;
        case "source-text-type":await editor(request.path).focus();if(request.key)await page.keyboard.press(request.key);else await page.keyboard.insertText(request.text);break;
        case "source-text-paste":await editor(request.path).evaluate((node,text)=>{const data=new DataTransfer();data.setData('text/plain',text);node.dispatchEvent(new ClipboardEvent('paste',{bubbles:true,cancelable:true,clipboardData:data}));},request.text);break;
        case "editor-selections":await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));value=await editor(request.path).evaluate(async(node,request)=>{const {EditorView}=await import("/node_modules/@codemirror/view/dist/index.js"),{editorOffset,authoredOffset}=await import("/src/simple/SourceText.ts");const view=EditorView.findFromDOM(node),EditorSelection=view.state.selection.constructor;if(request.ranges!==undefined){if(!Array.isArray(request.ranges)||request.ranges.length<1||request.ranges.length>256||!Number.isInteger(request.primary)||request.primary<0||request.primary>=request.ranges.length||request.ranges.some(range=>!Number.isInteger(range.from)||!Number.isInteger(range.to)||range.from<0||range.to<range.from))throw Error("Malformed exact source selection roster");view.dispatch({selection:EditorSelection.create(request.ranges.map(range=>EditorSelection.range(editorOffset(view.state,range.from),editorOffset(view.state,range.to))),request.primary)});}await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));return {ranges:view.state.selection.ranges.map(range=>({from:authoredOffset(view.state,range.from),to:authoredOffset(view.state,range.to)})),primary:view.state.selection.mainIndex};},request);break;
        case "editor-position":await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));value=await editor(request.path).evaluate(async(node,position)=>{const {EditorView}=await import("/node_modules/@codemirror/view/dist/index.js"),{editorOffset,authoredOffset}=await import("/src/simple/SourceText.ts");const view=EditorView.findFromDOM(node);if(position.anchor!==undefined)view.dispatch({selection:{anchor:editorOffset(view.state,position.anchor),...(position.head!==undefined?{head:editorOffset(view.state,position.head)}:{})}});if(position.y!==undefined)view.scrollDOM.scrollTop=position.y;await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));return {anchor:authoredOffset(view.state,view.state.selection.main.anchor),head:authoredOffset(view.state,view.state.selection.main.head),y:view.scrollDOM.scrollTop,x:view.scrollDOM.scrollLeft};},request);break;
        case "editor-is-retained": value = await editor(request.path).evaluate((node, previous) => node === previous, rememberedEditors.get(request.previous??request.path)); break;
        case "navigate-history":
          await menu("Edit", request.direction === "back" ? "Navigate back" : "Navigate forward");
          await editor(request.path).waitFor(); break;
        case "open":
          await page.getByRole("button", { name: request.path, exact: true }).click();
          if (/\.csv$/i.test(request.path)) { await page.getByRole("button", { name: "Edit table as grid", exact: true }).click(); await page.getByTestId("table-panel").waitFor(); }
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
        case "tab-order": value = await page.getByRole("tab", { name: request.path, exact: true }).locator("../..").getByRole("tab").evaluateAll(nodes => nodes.map(node => { const copy=node.cloneNode(true); copy.querySelectorAll('[aria-hidden=true]').forEach(icon=>icon.remove()); return copy.textContent; })); break;
        case "append":
          await editor(request.path).click();
          await page.keyboard.press(process.platform === "darwin" ? "Meta+ArrowDown" : "Control+End");
          await page.keyboard.insertText(request.text); break;
        case "change-open": await menu("Edit", request.command || "Rename symbol"); await page.getByRole("dialog",{name:"Workspace change preview",exact:true}).waitFor(); break;
        case "action-choose": {
          const panel=page.getByRole("dialog",{name:"Workspace change preview",exact:true});await panel.getByRole("button",{name:request.title,exact:true}).click();await panel.getByRole("button",{name:request.resources?"Save reviewed source resources":"Apply to source drafts",exact:true}).waitFor();value=await panel.innerText();break;
        }
        case "change-prepare": {
          const panel=page.getByRole("dialog",{name:"Workspace change preview",exact:true});
          if (request.name) await panel.getByRole("textbox",{name:request.input || "New symbol name",exact:true}).fill(request.name);
          await panel.getByRole("button",{name:request.command || "Prepare rename",exact:true}).click();
          await page.waitForFunction(() => { const panel=[...document.querySelectorAll('[role="dialog"]')].find(node=>node.getAttribute("aria-label")==="Workspace change preview"); return !!panel && !["Preparing complete source changes…","Preparing trailing-whitespace cleanup…","Preparing the main construction move…"].some(message=>panel.textContent.includes(message)); });
          value=await panel.innerText(); break;
        }
        case "change-state": {
          const panel=page.getByRole("dialog",{name:"Workspace change preview",exact:true});
          value={text:await panel.innerText(),size:await editor(request.path).locator("..").evaluate(node=>({width:node.clientWidth,height:node.clientHeight}))}; break;
        }
        case "change-file": await page.getByRole("combobox",{name:"Preview source file"}).selectOption({label:request.path}); break;
        case "change-resource-ready":await page.getByRole("button",{name:"Save reviewed source resources",exact:true}).waitFor();value=await page.getByRole("dialog",{name:"Workspace change preview",exact:true}).innerText();break;
        case "change-resource-save":{const panel=page.getByRole("dialog",{name:"Workspace change preview",exact:true});await panel.getByRole("button",{name:"Save reviewed source resources",exact:true}).click();await page.waitForFunction(()=>!document.querySelector('button[aria-label="Close change preview"]')?.disabled);value=await panel.innerText();break;}
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
        case "semantic-refuse-name": {
          const panel = page.getByRole("dialog", { name: "Semantic locations" });
          await panel.getByRole("button", { name: request.label, exact: true }).click();
          await page.locator("footer").getByText(/The reported symbol has moved or changed/).waitFor();
          value = { sourceVisible: await editor(request.path).isVisible(), targetVisible: await editor(request.target).isVisible() };
          break;
        }
        case "semantic-retire-origin": {
          const read = url => url.pathname === "/api/file" && url.searchParams.get("path") === request.target;
          let release;
          const gate = new Promise(resolve => { release = resolve; });
          const handle = async route => { await gate; await route.continue(); };
          await page.route(read, handle);
          try {
            const requested = page.waitForRequest(item => item.method() === "GET" && read(new URL(item.url())));
            await page.getByRole("dialog", {name: "Semantic locations"}).getByRole("button", {name: request.label, exact: true}).click();
            await requested;
            const origin = editor(request.path);
            await origin.focus();
            await page.keyboard.press("ArrowRight");
            const replied = page.waitForResponse(item => item.request().method() === "GET" && read(new URL(item.url())));
            release();
            const reply = await replied;
            await reply.finished();
            await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
            const originTab = page.getByRole("tab", {name: request.path, exact: true});
            const targetTab = page.getByRole("tab", {name: request.target, exact: true});
            await page.locator("nav").getByText("Edit", {exact: true}).click();
            const back = await page.getByRole("button", {name: "Navigate back", exact: true}).isEnabled();
            await page.locator("nav").getByText("Edit", {exact: true}).click();
            const retained = await origin.count() === 1 && await origin.evaluate((node, old) => node === old, rememberedEditors.get(request.path));
            const caret = retained ? await origin.evaluate(async node => {
              const {EditorView} = await import("/node_modules/@codemirror/view/dist/index.js");
              return EditorView.findFromDOM(node).state.selection.main.head;
            }) : null;
            value = {status: reply.status(), targetTabs: await targetTab.count(), back, caret, retained,
              active: await originTab.getAttribute("aria-selected") === "true"};
          } finally { release(); await page.unroute(read, handle); }
          break;
        }
        case "completion-open": {
          const content=editor(request.path);await content.focus();
          if(request.selection){await content.evaluate(async(node,selection)=>{const {EditorView}=await import('/node_modules/@codemirror/view/dist/index.js'),{editorOffset}=await import('/src/simple/SourceText.ts');const view=EditorView.findFromDOM(node);view.dispatch({selection:{anchor:editorOffset(view.state,selection.anchor),head:editorOffset(view.state,selection.head)},userEvent:'select'});view.focus();},request.selection);}
          else if(!request.keepSelection){await page.keyboard.press(process.platform==="darwin"?"Meta+ArrowUp":"Control+Home");for(let count=0;count<request.offset;count++)await page.keyboard.press("ArrowRight");}
          await page.keyboard.press("Control+Space");
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
        case "proposal-new-note":{await menu("File","New Tab");const node=page.locator('.cm-content[aria-label^="Untitled-"]');await node.waitFor();await node.focus();await page.keyboard.insertText(request.text);const path=await node.getAttribute("aria-label"),label=await node.locator('xpath=ancestor::section[contains(@class,"group")]').getByRole("tab",{selected:true}).innerText();value={path,label};break;}
        case "proposal-activate-note":await page.getByRole("tab",{name:request.label,exact:true}).click();await editor(request.path).waitFor();break;
        case "proposal-editor-context":{
          await menu("View","Agent");const draft=page.locator(".agent-composer .cm-content");await draft.waitFor();await draft.focus();await page.keyboard.press(process.platform==="darwin"?"Meta+A":"Control+A");await page.keyboard.insertText("Capture the owned complete editor snapshot for the scripted proposal witness.");
          let captured;const pattern="**/api/docs/workspace/agent",handler=async route=>{captured=route.request().postDataJSON()?.context;await route.fulfill({status:503,contentType:"application/json",body:JSON.stringify({error:"Owned fixture captured this non-live editor snapshot; the controlled ACP tool is launched separately."})});};
          await page.route(pattern,handler,{times:1});try{await page.getByRole("button",{name:"Send",exact:true}).click();await page.locator(".agent-body [role=alert]").filter({hasText:"Owned fixture captured"}).waitFor();if(!captured||captured.live!==false)throw Error("Actual AgentSession did not send a non-live context snapshot");value=captured;}finally{await page.unroute(pattern,handler);}
          await page.getByRole("button",{name:request.path,exact:true}).click();await editor(request.path).waitFor();break;
        }
        case "source-file-refresh":await menu("File","Refresh");break;
        case "proposal-open":await menu("View","Agent proposal review");await page.waitForFunction(()=>{const notice=document.querySelector('[aria-label="Agent proposal review"] > [role="status"]')?.textContent;return !!notice&&!notice.startsWith("Reading");});break;
        case "recorded-patch-review": {
          const card = page.locator(".agent-message").filter({has: editor(request.path)});
          const review = card.getByRole("button", { name: /^Review patch fence on line [0-9]+$/ });
          await review.click();
          const panel = page.getByRole("dialog", {name:"Agent proposal review", exact:true});
          const refusal = card.getByRole("alert");
          await panel.or(refusal).waitFor();
          if (await refusal.isVisible()) throw Error(await refusal.innerText());
          break;
        }
        case "proposal-select":{const panel=page.getByRole("dialog",{name:"Agent proposal review",exact:true});await panel.getByLabel("Retained agent proposal",{exact:true}).selectOption(request.token);await page.waitForFunction(()=>{const notice=document.querySelector('[aria-label="Agent proposal review"] > [role="status"]')?.textContent;return !!notice&&!notice.startsWith("Verifying");});value=await panel.innerText();break;}
        case "proposal-lose-report":await page.route("**/api/agent-proposals/report",route=>route.abort("failed"),{times:1});break;
        case "proposal-retry-report":{const panel=page.getByRole("dialog",{name:"Agent proposal review",exact:true}),button=panel.getByRole("button",{name:"Retry acknowledgment of recorded transition",exact:true});await button.click();await button.waitFor({state:"hidden"});value=await panel.innerText();break;}
        case "proposal-report-bodies":value=proposalReports;break;
        case "proposal-state":{const panel=page.getByRole("dialog",{name:"Agent proposal review",exact:true}),apply=panel.getByRole("button",{name:/^(Apply complete proposal to drafts|Save reviewed resource namespace)$/});value={text:await panel.innerText(),applyEnabled:await apply.isEnabled()};break;}
        case "proposal-apply":{const panel=page.getByRole("dialog",{name:"Agent proposal review",exact:true});await panel.getByRole("button",{name:request.resources?"Save reviewed resource namespace":"Apply complete proposal to drafts",exact:true}).click();await page.waitForFunction(()=>{const notice=document.querySelector('[aria-label="Agent proposal review"] > [role="status"]')?.textContent;return !!notice&&!notice.startsWith("Revalidating");});value=await panel.innerText();break;}
        case "proposal-close":await page.getByRole("dialog",{name:"Agent proposal review",exact:true}).getByRole("button",{name:"Close",exact:true}).click();break;
        case "session-reload":await page.reload();await page.locator('nav').waitFor();break;
        case "session-new-note":await menu('File','New Tab');{const input=page.locator('.cm-content[contenteditable="true"]').last();await input.focus();await page.keyboard.insertText(request.text);value=await input.getAttribute('aria-label');}break;
        case "session-open":await menu('View','Source recovery');await page.getByRole('dialog',{name:'Source recovery',exact:true}).getByRole('button',{name:'Editor sessions…',exact:true}).click();await page.getByRole('dialog',{name:'Editor session recovery',exact:true}).getByRole('button',{name:'Refresh sessions',exact:true}).waitFor();await page.waitForFunction(()=>document.querySelector('[role="dialog"][aria-label="Editor session recovery"] [role="status"]')?.textContent.includes('Preview a private session'));break;
        case "session-read":{const panel=page.getByRole('dialog',{name:'Editor session recovery',exact:true});if(request.revision!==undefined)await panel.getByLabel('Retained revision '+request.client,{exact:true}).selectOption(String(request.revision));const button=panel.getByRole('button').filter({hasText:request.client}),wanted=request.revision??Number((await button.innerText()).match(/revision (\d+)/)?.[1]);await button.click();await page.waitForFunction(revision=>!!document.querySelector('[role="dialog"][aria-label="Editor session recovery"] select[aria-label="Captured buffer"]')&&Array.from(document.querySelectorAll('[role="dialog"][aria-label="Editor session recovery"] p')).some(node=>node.textContent==='Reading retained revision '+revision),wanted);value=await panel.innerText();break;}
        case "session-more":{const panel=page.getByRole('dialog',{name:'Editor session recovery',exact:true}),before=await panel.locator('[aria-label^="Forget private session "]').count();await panel.getByRole('button',{name:'More private sessions',exact:true}).click();await page.waitForFunction(previous=>{const panel=document.querySelector('[role="dialog"][aria-label="Editor session recovery"]');return panel&&Array.from(panel.querySelectorAll('button')).some(button=>button.textContent==='Refresh sessions'&&!button.disabled)&&(panel.querySelectorAll('[aria-label^="Forget private session "]').length>previous||!Array.from(panel.querySelectorAll('button')).some(button=>button.textContent==='More private sessions'));},before);value=await panel.innerText();break;}
        case "session-refresh":{const panel=page.getByRole('dialog',{name:'Editor session recovery',exact:true});await panel.getByRole('button',{name:'Refresh sessions',exact:true}).click();await page.waitForFunction(()=>document.querySelector('[role="dialog"][aria-label="Editor session recovery"] [role="status"]')?.textContent.includes('Preview a private session'));value=await panel.innerText();break;}
        case "session-forget":{const panel=page.getByRole('dialog',{name:'Editor session recovery',exact:true});await panel.getByRole('button',{name:'Forget private session '+request.client,exact:true}).click();const response=page.waitForResponse(response=>response.url().endsWith('/api/workspace/recovery/forget')&&response.request().method()==='POST');await panel.getByRole('button',{name:'Confirm forget entire private session '+request.client,exact:true}).click();await (await response).finished();await page.waitForFunction(()=>{const panel=document.querySelector('[role="dialog"][aria-label="Editor session recovery"]');return panel&&Array.from(panel.querySelectorAll('button')).some(button=>button.textContent==='Refresh sessions'&&!button.disabled);});value=await panel.innerText();break;}
        case "session-stop-forget":{const panel=page.getByRole('dialog',{name:'Editor session recovery',exact:true});await panel.getByRole('button',{name:'Stop retrying and refresh before another removal',exact:true}).click();await panel.getByRole('button',{name:'Refresh sessions',exact:true}).click();await page.waitForFunction(()=>document.querySelector('[role="dialog"][aria-label="Editor session recovery"] [role="status"]')?.textContent.includes('Preview a private session'));value=await panel.innerText();break;}
        case "session-state":{const panel=page.getByRole('dialog',{name:'Editor session recovery',exact:true});value={text:await panel.innerText(),readCount:await panel.getByRole('button').filter({hasText:/^Read /}).count(),more:await panel.getByRole('button',{name:'More private sessions',exact:true}).count()>0,revisions:await panel.locator('select[aria-label^="Retained revision "]').evaluateAll(nodes=>nodes.map(node=>({label:node.getAttribute('aria-label'),selected:node.value,values:Array.from(node.options).map(option=>option.value)})))};break;}
        case "session-close":await page.getByRole('dialog',{name:'Editor session recovery',exact:true}).getByRole('button',{name:'Close',exact:true}).click();break;
        case "session-restore":{const panel=page.getByRole('dialog',{name:'Editor session recovery',exact:true}),button=panel.getByRole('button',{name:'Recover complete session as draft copies',exact:true});if(request.inspect){value={enabled:await button.isEnabled(),text:await panel.innerText()};break;}if(request.stale&&!await button.isEnabled()){value=(await panel.innerText())+' Host editor lease changed; Restore disabled.';break;}await button.click();if(request.stale){value=await panel.innerText();break;}await panel.getByRole('button',{name:'Close',exact:true}).click();await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));value=await page.locator('.cm-content').evaluateAll(async nodes=>{const {EditorView}=await import('/node_modules/@codemirror/view/dist/index.js');return nodes.map(node=>{const view=EditorView.findFromDOM(node);return {path:node.getAttribute('aria-label'),text:view.state.doc.toString(),anchor:view.state.selection.main.anchor,head:view.state.selection.main.head,y:view.scrollDOM.scrollTop};});});break;}
        case "recovery-open":await menu("View","Source recovery");await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Source recovery"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});break;
        case "recovery-read":{const panel=page.getByRole("dialog",{name:"Source recovery",exact:true});const settled=()=>page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Source recovery"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});if(request.record){await panel.getByLabel("Retained record",{exact:true}).selectOption(request.record);await settled();}if(request.path){await panel.getByLabel("Recorded source",{exact:true}).selectOption(request.path);await settled();}await panel.getByLabel("Snapshot",{exact:true}).selectOption(request.snapshot);await settled();value=await panel.locator("pre").count()?await panel.locator("pre").textContent():null;break;}
        case "recovery-interrupted": {
          const panel=page.getByRole("dialog",{name:"Source recovery",exact:true});
          const section=panel.getByRole("region",{name:"Interrupted Move",exact:true});
          await section.waitFor();
          value={text:await section.innerText(),canComplete:await section.getByRole("button",{name:"Prepare Move completion",exact:true}).count()===1};
          break;
        }
        case "recovery-prepare":{const panel=page.getByRole("dialog",{name:"Source recovery",exact:true});await panel.getByRole("button",{name:request.completion?"Prepare Move completion":request.inverse?"Prepare inverse restoration":"Prepare restoration",exact:true}).click();await page.waitForFunction(()=>!Array.from(document.querySelectorAll('[role="dialog"][aria-label="Source recovery"] button')).find(button=>button.textContent==="Close")?.disabled);value=await panel.locator('[role="status"]').allTextContents();break;}
        case "recovery-publish":{const panel=page.getByRole("dialog",{name:"Source recovery",exact:true});await panel.getByRole("button",{name:request.completion?"Complete reviewed Move":"Restore reviewed saved sources and drafts",exact:true}).click();await page.waitForFunction(()=>!Array.from(document.querySelectorAll('[role="dialog"][aria-label="Source recovery"] button')).find(button=>button.textContent==="Close")?.disabled);value=await panel.locator('[role="status"]').allTextContents();break;}
        case "recovery-close":await page.getByRole("dialog",{name:"Source recovery",exact:true}).getByRole("button",{name:"Close",exact:true}).click();break;
        case "construction-open":await menu("View","Construction reading");await page.getByRole("dialog",{name:"Construction reading",exact:true}).waitFor();break;
        case "construction-read":{const panel=page.getByRole("dialog",{name:"Construction reading",exact:true});await panel.getByLabel("Construction entry",{exact:true}).fill(request.entry);await panel.getByLabel("Contributor or output (optional)",{exact:true}).fill(request.focus??"");await panel.getByRole("button",{name:"Read captured drafts",exact:true}).click();await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Construction reading"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});value=await panel.innerText();break;}
        case "construction-side":{const panel=page.getByRole("dialog",{name:"Construction reading",exact:true});await panel.getByRole("button",{name:request.after?"Draft construction":"Saved construction",exact:true}).click();value=await panel.locator("pre").first().textContent();break;}
        case "behavior-ready":{const panel=page.getByRole("dialog",{name:"Construction reading",exact:true});if(!request.retained)await page.waitForFunction(()=>{const statuses=document.querySelectorAll('[role="dialog"][aria-label="Construction reading"] [role="status"]');const notice=statuses[1]?.textContent;return !!notice&&!notice.startsWith("Reading");});value=await panel.innerText();break;}
        case "behavior-check":{const panel=page.getByRole("dialog",{name:"Construction reading",exact:true});const select=panel.getByLabel("Related check",{exact:true});const options=await select.locator("option").allTextContents(),index=options.findIndex(text=>text.startsWith(request.label+" ·"));if(index<0)throw Error("Related check absent: "+options.join(" / "));await select.selectOption({index});break;}
        case "behavior-check-options":value=await page.getByLabel("Related check",{exact:true}).locator("option").allTextContents();break;
        case "behavior-last-request":value=behaviorRequests.at(-1)??null;break;
        case "behavior-request-count":value=behaviorRequests.length;break;
        case "behavior-run-enabled":value=await page.getByRole("button",{name:"Run selected relevant test",exact:true}).isEnabled();break;
        case "behavior-source":{const panel=page.getByRole("dialog",{name:"Construction reading",exact:true}),select=panel.getByLabel("Related reading",{exact:true});const selected=await select.locator("option").evaluateAll((options,request)=>options.find(option=>option.textContent.startsWith(request.kind+" ·")&&option.textContent.includes(request.label))?.value,request);if(selected===undefined)throw Error("Declared reading absent");await select.selectOption(selected);await panel.getByRole("button",{name:"Read full captured source",exact:true}).click();await page.waitForFunction(()=>{const status=document.querySelector('[role="status"][aria-label="Source reading status"]')?.textContent;return !!status&&!status.startsWith("Reading");});const text=panel.getByLabel("Captured full reading",{exact:true});if(!await text.count())throw Error(await panel.getByRole("status",{name:"Source reading status",exact:true}).innerText());value={text:await text.textContent(),state:await text.locator("..").innerText()};break;}
        case "behavior-evidence":{const panel=page.getByRole("dialog",{name:"Construction reading",exact:true}),summary=panel.locator("summary").filter({hasText:/^Retained evidence ·/});if(!await summary.evaluate(node=>node.parentElement.open))await summary.click();value=await summary.locator("..").innerText();break;}
        case "behavior-refused":{await page.waitForFunction(()=>Array.from(document.querySelectorAll('[role="dialog"][aria-label="Construction reading"] article')).some(article=>article.querySelector('button')?.textContent==='Refresh retained outcome'&&/request-refused|admission-refused/.test(article.textContent??'')));value=await page.getByRole("dialog",{name:"Construction reading",exact:true}).innerText();break;}
        case "behavior-surface":{const panel=page.getByRole("dialog",{name:"Construction reading",exact:true});value=await panel.evaluate(node=>{const rect=node.getBoundingClientRect();return {position:getComputedStyle(node).position,width:rect.width,height:rect.height,bottom:rect.bottom,viewport:innerHeight,scrollable:node.scrollHeight>node.clientHeight};});break;}
        case "behavior-run":await page.getByRole("button",{name:"Run selected relevant test",exact:true}).click();break;
        case "behavior-finished":{const panel=page.getByRole("dialog",{name:"Construction reading",exact:true});await panel.getByText("Retained verification requests",{exact:true}).waitFor();for(let count=0;count<100;count++){const articles=panel.locator("article").filter({has:page.getByRole("button",{name:"Refresh retained outcome",exact:true})});const latest=articles.first();if((await latest.innerText()).includes("process-finished"))break;await latest.getByRole("button",{name:"Refresh retained outcome",exact:true}).click();await page.waitForTimeout(100);}value=await panel.innerText();break;}
        case "behavior-profiles":{const panel=page.getByRole("dialog",{name:"Construction reading",exact:true});await panel.getByRole("button",{name:"Choose profile for this check",exact:true}).click();await page.waitForFunction(()=>{const statuses=document.querySelectorAll('[role="dialog"][aria-label="Construction reading"] [role="status"]');const notice=statuses[1]?.textContent;return !!notice&&!notice.startsWith("Reading");});const choice=panel.getByLabel("Explicit verification profile",{exact:true});if(!await choice.count())throw Error("Profile admission: "+await panel.getByRole("status",{name:"Connected reading status",exact:true}).innerText());await choice.selectOption({label:request.label});break;}
        case "behavior-profile-run":await page.getByRole("button",{name:"Run user-declared profile for selected check",exact:true}).click();break;
        case "construction-close":await page.getByRole("dialog",{name:"Construction reading",exact:true}).getByRole("button",{name:"Close",exact:true}).click();break;
        case "construction-origin":{const panel=page.getByRole("dialog",{name:"Construction reading",exact:true}),button=panel.getByRole("button",{name:`Open ${request.phase} contributor ${request.path}`,exact:true}).first();if(!await button.isVisible())await panel.getByText("Compiler contribution ranges",{exact:true}).click();await button.click();if(request.stale){await panel.getByRole("status").filter({hasText:"Source changed"}).waitFor();value=await panel.innerText();}else await panel.waitFor({state:"detached"});break;}
        case "construction-history":await menu("Edit",request.redo?"Redo text edit":"Undo text edit");break;
        case "construction-source":value=await editor(request.path).evaluate(async (node,selection)=>{const {EditorView}=await import("/node_modules/@codemirror/view/dist/index.js"),{authoredSource,authoredOffset}=await import("/src/simple/SourceText.ts");const view=EditorView.findFromDOM(node);if(!view)throw new Error("Source editor unavailable.");return selection?{source:authoredSource(view.state),anchor:authoredOffset(view.state,view.state.selection.main.anchor),head:authoredOffset(view.state,view.state.selection.main.head)}:authoredSource(view.state);},!!request.selection);break;
        case "construction-set-source":await editor(request.path).click({position:{x:16,y:8}});await page.keyboard.press(process.platform === "darwin"?"Meta+A":"Control+A");if(request.source.length===0)await page.keyboard.press("Backspace");else await editor(request.path).evaluate((node,text)=>{const data=new DataTransfer();data.setData("text/plain",text);node.dispatchEvent(new ClipboardEvent("paste",{bubbles:true,cancelable:true,clipboardData:data}));},request.source);break;
        case "attribution-open":await menu("View","Saved file attribution");await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Saved file attribution"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});value=await page.getByRole("dialog",{name:"Saved file attribution",exact:true}).innerText();break;
        case "attribution-commit":await page.getByRole("button",{name:`Read commit for saved line ${request.line}`,exact:true}).click();await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Repository history"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});break;
        case "workspace-history":await menu("Edit",request.redo?"Redo workspace change":"Undo workspace change");break;
        case "resolution-compose":{const panel=page.locator('section[aria-label="Complete conflict resolution"]');await panel.getByRole('button',{name:'Compose conflict group…',exact:true}).click();await page.waitForFunction(()=>{const panel=document.querySelector('section[aria-label="Complete conflict resolution"]');return panel&&panel.querySelector('[role="status"]')?.textContent&&[...panel.querySelectorAll('button')].some(button=>button.textContent==='Compose conflict group…'&&!button.disabled);});value=await panel.innerText();break;}
        case "resolution-output":{const panel=page.locator('section[aria-label="Complete conflict resolution"]');if(!await panel.getByRole('textbox',{name:'Conflict destination 1',exact:true}).count())await panel.getByRole('button',{name:'Add explicit destination',exact:true}).click();await panel.getByRole('textbox',{name:'Conflict destination 1',exact:true}).fill(request.path);await panel.getByRole('combobox',{name:'Retained identity 1',exact:true}).selectOption(request.identity??request.path);await panel.getByRole('textbox',{name:'Composed resolution 1',exact:true}).fill(request.text);break;}
        case "resolution-preview":{const panel=page.locator('section[aria-label="Complete conflict resolution"]');await panel.getByRole('button',{name:'Preview complete resolution consequences',exact:true}).click();await page.waitForFunction(()=>{const panel=document.querySelector('section[aria-label="Complete conflict resolution"]');return panel&&[...panel.querySelectorAll('button')].some(button=>button.textContent==='Compose conflict group…'&&!button.disabled);});value=await panel.innerText();break;}
        case "resolution-act":{const panel=page.locator('section[aria-label="Complete conflict resolution"]'),name=request.kind==='draft'?'Apply text drafts only':request.kind==='stage'?'Stage complete saved group':'Save complete resolution group';await panel.getByRole('button',{name,exact:true}).click();await page.waitForFunction(()=>{const panel=document.querySelector('section[aria-label="Complete conflict resolution"]');return panel&&[...panel.querySelectorAll('button')].some(button=>['Compose conflict group…','Inspect original resolution request'].includes(button.textContent)&&!button.disabled);});value=await panel.innerText();break;}
        case "resolution-state":{const panel=page.locator('section[aria-label="Complete conflict resolution"]');value={text:await panel.innerText(),stage:await panel.getByRole('button',{name:'Stage complete saved group',exact:true}).count()?await panel.getByRole('button',{name:'Stage complete saved group',exact:true}).isEnabled():false};break;}
        case "local-history-open":await page.locator('nav').getByText('View',{exact:true}).click();await page.getByRole('button',{name:/^Local source history/}).click();await page.getByRole('dialog',{name:'Local source history',exact:true}).waitFor();await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Local source history"] [role="status"]')?.textContent;return text&&!text.startsWith('Reading');});value=await page.getByRole('dialog',{name:'Local source history',exact:true}).innerText();break;
        case "local-history-select":{const panel=page.getByRole('dialog',{name:'Local source history',exact:true}),select=panel.getByRole('combobox',{name:'Retained save',exact:true});const id=request.id??await select.locator('option').evaluateAll(options=>options.find(option=>option.textContent.startsWith('published'))?.value);if(!id)throw Error('No published source history is available');await select.selectOption(id);await panel.getByRole('combobox',{name:'Snapshot',exact:true}).selectOption(request.side??'before');value={id,text:await panel.locator('pre').first().innerText(),draft:await panel.locator('pre').nth(1).innerText(),enabled:await panel.getByRole('button',{name:'Restore selected text to draft',exact:true}).isEnabled()};break;}
        case "local-history-restore":{const panel=page.getByRole('dialog',{name:'Local source history',exact:true});await panel.getByRole('button',{name:'Restore selected text to draft',exact:true}).click();await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Local source history"] [role="status"]')?.textContent;return text&&!text.startsWith('Preview');});value=await panel.innerText();break;}
        case "local-history-close":await page.getByRole('dialog',{name:'Local source history',exact:true}).getByRole('button',{name:'Close',exact:true}).click();break;
        case "saved-status":await page.waitForFunction(()=>document.querySelector('footer')?.textContent?.endsWith(' · Saved'));value=await page.locator('footer').innerText();break;
        case "source-copy-ready":await editor(request.path).waitFor();value={source:await editor(request.path).textContent(),status:await page.locator('footer').innerText()};break;
        case "history-state":value=await page.getByRole("dialog",{name:"Repository history",exact:true}).innerText();break;
        case "history-open":await menu("View",request.file?"Saved file history":"Repository history");await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Repository history"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});value=await page.getByRole("dialog",{name:"Repository history",exact:true}).innerText();break;
        case "history-read": {const panel=page.getByRole("dialog",{name:"Repository history",exact:true});await panel.getByRole("button",{name:`Read commit ${request.sha}`,exact:true}).click();await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Repository history"] [role="status"]')?.textContent;return !!text&&!text.startsWith("Reading");});value=await panel.innerText();break;}
        case "history-file": {const select=page.getByRole("combobox",{name:"Historical file",exact:true}),valueToSelect=await select.locator("option").evaluateAll((options,path)=>options.find(option=>option.textContent.endsWith(` · ${path}`))?.value,request.path);if(valueToSelect===undefined)throw new Error(`Historical file unavailable: ${request.path}`);await select.selectOption(valueToSelect);break;}
        case "history-side":await page.getByRole("combobox",{name:"Historical side",exact:true}).selectOption(request.side);value=await page.getByLabel("Historical content",{exact:true}).innerText();break;
        case "history-close":await page.getByRole("dialog",{name:"Repository history",exact:true}).getByRole("button",{name:"Close",exact:true}).click();await page.getByRole("dialog",{name:"Repository history",exact:true}).waitFor({state:"detached"});break;
        case "review-open":await menu("View","Repository changes");await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Repository changes"] [role="status"][aria-label="Repository change status"]')?.textContent;return !!text&&!text.startsWith("Reading");});value=await page.getByRole("dialog",{name:"Repository changes",exact:true}).innerText();break;
        case "review-select": {const panel=page.getByRole("dialog",{name:"Repository changes",exact:true});await panel.getByRole("button",{name:`${request.staged?'Staged':'Working tree'} ${request.path}`,exact:true}).click();await panel.getByRole("heading",{name:`${request.path} · ${request.staged?'Staged':'Working tree'}`,exact:true}).waitFor();await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Repository changes"] [role="status"][aria-label="Repository change status"]')?.textContent;return !!text&&!text.startsWith("Reading");});value=await panel.getByLabel("Git diff",{exact:true}).innerText();break;}
        case "review-stage":case "review-unstage": {const panel=page.getByRole("dialog",{name:"Repository changes",exact:true});await panel.getByRole("button",{name:request.action==="review-stage"?"Stage saved file":"Unstage file",exact:true}).click();await page.waitForFunction(()=>{const text=document.querySelector('[role="dialog"][aria-label="Repository changes"] [role="status"][aria-label="Repository change status"]')?.textContent??"";return text.startsWith("Saved paths ")||text.startsWith("Index result not confirmed");});value=await panel.getByRole("status",{name:"Repository change status",exact:true}).innerText();break;}
        case "review-conflict-read":{const panel=page.getByRole("region",{name:"Conflict stages",exact:true});await panel.getByRole("button",{name:"Inspect conflict stages",exact:true}).click();await panel.getByRole("status").filter({hasText:"Captured immutable Git stages"}).waitFor();value=await panel.innerText();break;}
        case "review-conflict-preview":{await page.getByRole("region",{name:"Conflict stages",exact:true}).getByRole("button",{name:"Preview this stage as source",exact:true}).click();const panel=page.getByRole("dialog",{name:"Workspace change preview",exact:true});await panel.getByRole("button",{name:"Apply to source drafts",exact:true}).waitFor();await page.getByRole("dialog",{name:"Repository changes",exact:true}).waitFor({state:"detached"});value=await panel.innerText();break;}
        case "review-conflict-stage":{const panel=page.getByRole("region",{name:"Conflict stages",exact:true});await panel.getByRole("button",{name:`Stage ${request.stage}${request.stage===1?' · Base':''}`,exact:true}).click();value=await panel.getByLabel("Conflict stage text",{exact:true}).textContent();break;}
        case "review-status":value=await page.getByRole("dialog",{name:"Repository changes",exact:true}).getByRole("status",{name:"Repository change status",exact:true}).innerText();break;
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
          const tables = page.locator('[data-testid="table-panel"]:not(.notebook-table *)');
          await tables.nth(2).waitFor();
          const totals = tables.nth(1).locator('.table-panel__cell[data-column="2"]');
          const receipt = tables.nth(2).locator('.table-panel__cell[data-row="1"][data-column="1"]');
          if (request.total !== undefined) { await receipt.filter({ hasText: request.total }).waitFor(); await page.getByRole("region", { name: "Read-only table" }).getByRole("cell", { name: request.total, exact: true }).waitFor(); }
          await totals.nth(1).filter({ hasText: new RegExp(`^${request.tea}$`) }).waitFor();
          await totals.nth(2).filter({ hasText: /^5$/ }).waitFor();
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
          value = { text: await region.textContent(), editable: await page.locator('.table-panel__formula:not(:disabled),[contenteditable="true"]').count() }; break;
        }
        case "native-table": {
          await page.close(); page = await browser.newPage({ viewport: { width: 900, height: 700 } });
          nativeTable = { revision: "native:0", source: request.source, format: request.format, readonly: false, language: request.language, header: request.header ?? false, scope: request.scope };
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
        case "spreadsheet-select-header": {
          const table = page.getByTestId("table-panel");
          const headers = request.axis === "row" ? ".table-panel__gutter:not(.table-panel__gutter--lane)" : ".table-panel__head";
          await table.locator(headers).nth(request.index).click();
          value = await table.getByTestId("table-panel-name").textContent(); break;
        }
        case "spreadsheet-fill": {
          const table = page.getByTestId("table-panel");
          const handle = table.getByRole("button", { name: "Fill selected cells" });
          const cell = table.locator(`.table-panel__cell[data-row="${request.row}"][data-column="${request.column}"]`);
          await handle.hover();
          const end = await cell.boundingBox();
          await page.mouse.down();
          try {
            await page.mouse.move(end.x + end.width / 2, end.y + end.height / 2, { steps: 5 });
            await table.locator(".table-fill-range").waitFor({ timeout: 2500 });
          } finally { await page.mouse.up(); }
          break;
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
          if (!request.continuing) await cell(request.row, request.column).click();
          const bar = table.locator(".table-panel__formula"); await bar.fill(request.prefix);
          const start = await cell(request.fromRow, request.fromColumn).boundingBox();
          const end = await cell(request.toRow, request.toColumn).boundingBox();
          const point = box => ({ x: box.x + box.width / 2, y: box.y + box.height / 2 });
          const picked = () => table.locator("[data-formula-reference]").evaluateAll(cells =>
            cells.filter(cell => {
              const content = cell.querySelector(".table-panel__cell, .table-panel__input");
              if (!content) return false;
              const background = getComputedStyle(content).backgroundColor;
              return background !== "transparent" && background !== "rgba(0, 0, 0, 0)";
            }).map(cell => [Number(cell.dataset.fillRow), Number(cell.dataset.fillColumn)]));
          let preview;
          if (request.touch) {
            const input = await page.context().newCDPSession(page);
            await input.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [point(start)] });
            await input.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: [point(end)] });
            preview = await picked();
            await input.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
            await input.detach();
          } else {
            const from = point(start), to = point(end);
            await page.mouse.move(from.x, from.y);
            await page.mouse.down();
            try {
              await page.mouse.move(to.x, to.y, { steps: 8 });
              preview = await picked();
              if (request.releaseOutside) await page.mouse.move(to.x, start.y - start.height * 2, { steps: 1 });
            } finally { await page.mouse.up(); }
            await cell(request.hoverRow, request.hoverColumn).hover();
          }
          value = { draft: await bar.inputValue(), preview, picked: await picked() };
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
        case "debug-threads":{
          const summary=page.getByLabel("Debug options",{exact:true}).last();if(!await summary.evaluate(node=>node.parentElement.open))await summary.click();const panel=summary.locator("..");await panel.getByRole("button",{name:"Read threads",exact:true}).click();
          await page.waitForFunction(()=>{const section=document.querySelector('[aria-label="Debugger threads"]');return !!section&&!section.textContent.includes('Reading adapter threads…');});if(!await summary.evaluate(node=>node.parentElement.open))await summary.click();
          value=await panel.locator('[aria-label="Debugger threads"]').evaluate(node=>{const select=node.querySelector('select');return {message:node.textContent,selected:select?.value??null,threads:select?Array.from(select.options,option=>({id:option.value,name:option.textContent})):[]};});break;
        }
        case "debug-thread-select":{
          const summary=page.getByLabel("Debug options",{exact:true}).last();if(!await summary.evaluate(node=>node.parentElement.open))await summary.click();await page.getByRole("combobox",{name:"Debugger thread",exact:true}).selectOption(String(request.id));
          await page.waitForFunction(id=>{const select=document.querySelector('select[aria-label="Debugger thread"]');return select?.value===id&&!select.disabled;},String(request.id));value=await page.getByLabel("Call stack",{exact:true}).locator('option').allTextContents();break;
        }
        case "adapter-source-request":{const summary=page.getByLabel("Debug options",{exact:true}).last();if(!await summary.evaluate(node=>node.parentElement.open))await summary.click();await summary.locator("..").getByRole("button",{name:"Read adapter source",exact:true}).click();break;}
        case "adapter-source-state":{const region=page.getByRole("region",{name:"Adapter-owned source",exact:true});value=await region.count()?{present:true,text:await region.innerText(),pre:await region.locator("pre").count()?await region.locator("pre").textContent():null,bold:await page.locator("b").count()>0}:{present:false,text:"",pre:null,bold:await page.locator("b").count()>0};break;}
        case "debug-frame-index":{const summary=page.getByLabel("Debug options",{exact:true}).last();if(!await summary.evaluate(node=>node.parentElement.open))await summary.click();const stack=page.getByLabel("Call stack",{exact:true}),options=await stack.locator('option').evaluateAll(options=>options.map(option=>({value:option.value,label:option.textContent})));const selected=options[request.index];if(!selected)throw Error('Requested deep frame absent: '+request.index+' of '+options.length);await stack.selectOption(selected.value);value={count:options.length,index:request.index,...selected};break;}
        case "debug-breakpoint-reading":{const options=page.getByLabel("Debug options",{exact:true}).last();if(!await options.evaluate(node=>node.parentElement.open))await options.click();const summary=page.getByText('Retained breakpoint intentions',{exact:true});if(!await summary.evaluate(node=>node.parentElement.open))await summary.click();if(request.next)await page.getByRole('button',{name:'Next breakpoint intentions',exact:true}).click();value=await page.getByLabel('Retained breakpoint reading',{exact:true}).innerText();break;}
        case "debug-breakpoint-condition":{const marker=editor(request.path).locator('..').locator('.cm-bp').first();await marker.click({button:'right'});const form=page.getByRole('dialog',{name:`Breakpoint on line ${request.line}`,exact:true});await form.getByLabel('Condition',{exact:true}).fill(request.condition);await form.getByLabel('Condition',{exact:true}).press('Enter');await form.waitFor({state:'hidden'});break;}
        case "debug-frame-select":{const summary=page.getByLabel("Debug options",{exact:true}).last();if(!await summary.evaluate(node=>node.parentElement.open))await summary.click();const stack=page.getByLabel("Call stack",{exact:true});const option=await stack.locator('option').evaluateAll((options,name)=>options.find(item=>item.textContent.startsWith(name+' —'))?.value,request.name);if(!option)throw Error('Requested frame absent: '+request.name+'; '+await stack.textContent());const label=await stack.locator('option').evaluateAll((options,value)=>options.find(item=>item.value===value)?.textContent,option);await stack.selectOption(option);value={label,line:Number(/ — line (\d+)$/.exec(label??'')?.[1]??0)};break;}
        case "debug-watch": {
          const surface = request.path ? editor(request.path).locator("xpath=ancestor::section[contains(@class, 'agent-message')][1]") : page;
          const summary = surface.getByLabel("Debug options", {exact:true}).last();
          const open = async () => {
            if (await summary.evaluate(node => node.parentElement.open)) return;
            if (request.path && !await summary.isVisible()) {
              await surface.locator(".cm-debug-rail").getByRole("button", {name:/^Show [0-9]+ more gutter icons$/}).click();
            }
            await summary.click();
          };
          await open();
          page.once('dialog', dialog => dialog.accept(request.expression));
          await summary.locator('..').getByRole('button', {name:'+ watch', exact:true}).click();
          const output = surface.getByLabel('Value of watch ' + request.expression, {exact:true});
          await output.waitFor({state:'attached'});
          await page.waitForFunction(({node, expected}) => !node.textContent.startsWith('No value reported')
            && (expected === undefined || node.textContent === expected), {node: await output.elementHandle(), expected:request.expected});
          await open();
          value = await output.textContent();
          break;
        }
        case "debug-expand":{
          const options=page.getByLabel("Debug options",{exact:true}).last();if(!await options.evaluate(node=>node.parentElement.open))await options.click();
          const names=[request.name,`'${request.name}'`,`"${request.name}"`];let button=null;for(const name of names){const candidate=page.getByRole('button',{name:`Expand value ${name}`,exact:true}).last();if(await candidate.count()){button=candidate;break;}}if(!button)throw Error('No current expandable value '+request.name);
          const node=button.locator('..'),id=await node.getAttribute('data-debug-value-node');await button.click();await page.waitForFunction(id=>{const node=document.querySelector(`[data-debug-value-node="${id}"]`);return !!node&&!node.textContent.includes('Reading children…');},id);value=await page.locator(`[data-debug-value-node="${id}"]`).textContent();break;
        }
        case "debug-move":{
          const options=page.getByLabel('Debug options',{exact:true}).last();if(!await options.evaluate(node=>node.parentElement.open))await options.click();const label=request.kind==='run_to'?'Run to line':'Move instruction pointer to line',button=page.getByRole('button',{name:label,exact:true}).last();if(!await button.count()){value={supported:false};break;}
          await page.getByLabel('Debugger target line',{exact:true}).last().fill(String(request.line));await button.click();await page.waitForFunction(line=>{const stack=document.querySelector('select[aria-label="Call stack"]');const option=stack?.selectedOptions?.[0];return !!option&&option.textContent.endsWith(' — line '+line);},String(request.line));value={supported:true};break;
        }
        case "workbook-link": {
          const link = editor(request.path).locator(".cm-md-link").filter({ hasText: "workbook.md" });
          await link.click();
          await editor(request.path).waitFor();
          await link.click({ modifiers: [process.platform === "darwin" ? "Meta" : "Control"] });
          const workbook = editor("workbook-demo/workbook.md");
          await workbook.waitFor();
          await workbook.locator(".cm-md-heading").first().waitFor();
          break;
        }
        case "architecture-preview": {
          const graph = page.locator(".rendered-diagram");
          const node = graph.getByTestId("graph-node-shop/receipt.ts");
          await node.waitFor({ timeout: 15000 });
          if (request.inspect_source) {
            await page.getByRole("button", { name: "Edit source text", exact: true }).click();
            await graph.waitFor({ state: "detached" });
            const source = await editor(request.path).innerText();
            if (!source.includes("function codeModelQuery()") || !source.includes("function buildDiagram(")) {
              throw new Error("The diagram's source must contain the repository's query and graph builder.");
            }
            await page.getByRole("button", { name: "Render embedded content", exact: true }).click();
            await node.waitFor({ timeout: 15000 });
          }
          const edge = graph.locator(".react-flow__edge").first();
          await edge.waitFor({ state: request.edge_count ? "attached" : "detached", timeout: 15000 });
          break;
        }
        case "architecture-link": {
          const preview = page.locator(".rendered-diagram").filter({ has: page.getByRole("button", { name: "Refresh diagram" }) });
          const node = preview.getByTestId("graph-node-shop/receipt.ts");
          try { await node.click({ timeout: 15000 }); }
          catch (error) {
            const hit = await node.evaluate(node => {
              const rect = node.getBoundingClientRect();
              const hit = node.getRootNode().elementFromPoint(rect.left + rect.width / 2, rect.top + rect.height / 2);
              const details = Array.from([node, node.parentElement, node.closest(".react-flow"), node.closest(".graph-editor")], el => {
                const r = el.getBoundingClientRect(), style = getComputedStyle(el);
                return { class: el.className, rect: [r.x,r.y,r.width,r.height], pointer: style.pointerEvents, position: style.position };
              });
              return { hit: hit?.className, details };
            });
            throw new Error(String(error) + "\n" + JSON.stringify(hit));
          }
          const target = editor(request.path);
          await target.waitFor();
          value = await target.evaluate(async node => {
            const { EditorView } = await import("/node_modules/@codemirror/view/dist/index.js");
            return EditorView.findFromDOM(node).state.selection.main.head;
          });
          break;
        }
        case "workbook-run":
          const group = debugRail().locator(`.cm-debug-rail__group[data-source-offset="${request.at}"]`);
          const profiles = group.getByRole("button", { name: "Launch profiles for python fence", exact: true, includeHidden: true });
          if (await profiles.isVisible()) await profiles.click();
          else await selectGutterAction(profiles);
          await page.getByRole("button", { name: "Run fence", exact: true }).click();
          await page.getByRole("tab", { name: /Terminal/ }).waitFor();
          await page.getByRole("tab", { name: "workbook.md", exact: true }).click();
          await editor(request.path).waitFor();
          break;
        case "workbook-output":
          await page.locator(".cm-run-output").filter({ hasText: request.text }).last().waitFor();
          break;
        case "workbook-debug": {
          const rail = debugRail();
          const group = request.at === undefined ? rail : rail.locator(`.cm-debug-rail__group[data-source-offset="${request.at}"]`);
          const action = group.getByRole("button", { name: `Debug ${request.path}`, exact: true, includeHidden: true });
          await action.waitFor({ state: "attached", timeout: 5000 });
          await group.evaluate(group => {
            const rect = group.getBoundingClientRect();
            const rail = group.closest(".cm-debug-rail").getBoundingClientRect();
            if (rect.top < rail.top || rect.top > rail.bottom) {
              throw Error("Launch gutter outside viewport: " + JSON.stringify({ top: rect.top, rail: [rail.top, rail.bottom], offset: group.dataset.sourceOffset }));
            }
          });
          if (await action.isVisible()) await action.click();
          else await selectGutterAction(action);
          break;
        }
        case "debug-start":
          await page.getByRole("button", { name: `Debug ${request.path}`, exact: true }).click(); break;
        case "debug-continue":await page.getByRole("button",{name:"Continue",exact:true}).click();break;
        case "debug-step":
          await page.getByRole("button", { name: "Step over", exact: true }).click(); break;
        case "debug-stop":
          {const options=page.getByLabel("Debug options",{exact:true}).last();if(await options.count()&&await options.evaluate(node=>node.parentElement.open))await options.click();}
          await page.getByRole("button", { name: "Stop", exact: true }).click(); break;
        case "debug-state": value = await page.locator(".debug-strip__status").allTextContents(); break;
        case "debug-locals":{const locals=page.getByRole("region",{name:/^Selected-frame (locals|values)$/});value=await locals.count()?await locals.textContent():"";break;}
        case "debug-wire":value=debugWire;break;
        case "source-tab":await page.getByRole("tab",{name:request.label,exact:true}).click();break;
        case "debug-diagnostics": value={text:await page.locator("body").innerText(),errors:pageErrors};break;
        case "debug-values": value = await page.locator(".cm-debug-value").allTextContents(); break;
        case "debug-breakpoint-remove": {
          const marks=editor(request.path).locator("..").locator(".cm-bp");
          if(await marks.count()!==1)throw Error("Expected exactly one held breakpoint in the owned editor");
          await marks.first().click();await marks.first().waitFor({state:"detached"});break;
        }
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
        case "tabs": value = await page.getByRole("tab").evaluateAll(nodes => nodes.map(node => { const copy = node.cloneNode(true); copy.querySelectorAll('[aria-hidden=true]').forEach(icon => icon.remove()); return copy.textContent; })); break;
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
        case "recorded-fence-run": {
          const card = editor(request.path).locator("xpath=ancestor::section[contains(@class, 'agent-message')][1]");
          const menu = card.getByLabel(`Fence actions on line ${request.opening}`, { exact: true });
          await menu.click();
          const [response] = await Promise.all([
            page.waitForResponse(reply => reply.request().method() === "POST" && reply.url().endsWith("/fence-run")),
            card.getByRole("button", {name: `Run python fence on line ${request.opening}`, exact: true}).click(),
          ]);
          if (!response.ok()) throw new Error(await response.text());
          const output = card.getByRole("region", { name: "Recorded fence run" });
          await output.getByText("Process exit 0", { exact: true }).waitFor();
          value = { address: response.request().postDataJSON(), output: await output.locator("pre").textContent() };
          await menu.click();
          break;
        }
        case "recorded-fence-debug": {
          const card = editor(request.path).locator("xpath=ancestor::section[contains(@class, 'agent-message')][1]");
          await card.getByLabel(`Set breakpoint on line ${request.breakpoint}`, { exact: true }).click();
          await card.getByLabel(`Fence actions on line ${request.opening}`, { exact: true }).click();
          await card.getByRole("button", {name: `Debug python fence on line ${request.opening}`, exact: true}).click();
          break;
        }
        case "recorded-fence-position": {
          const card = editor(request.path).locator("xpath=ancestor::section[contains(@class, 'agent-message')][1]");
          const line = card.locator(".cm-paused-line");
          value = await line.count() ? await line.evaluate(node => {
            const code = node.cloneNode(true);
            code.querySelectorAll('[contenteditable="false"]').forEach(widget => widget.remove());
            return code.textContent;
          }) : null;
          break;
        }
        case "conversation-tabs": {
          await menu('View', 'Agent History');
          const history = page.getByRole('region', {name:'Agent History'});
          await history.getByRole('button', {name:'First conversation',exact:true}).click();
          const first = page.getByRole('tab', {name:'First conversation',exact:true});
          const draft = editor('conversation/draft.md');
          await draft.fill('Keep first'); await draft.press('End'); await draft.press('Enter'); await page.keyboard.insertText('next line');
          const firstDraft = await draft.locator('.cm-line').allTextContents().then(lines=>lines.join('\n'));
          const firstUsage = await page.locator('.agent-pane-status').innerText();
          await history.getByRole('button', {name:'Second conversation',exact:true}).click();
          await editor('conversation/draft.md').fill('Keep second');
          const secondUsage = await page.locator('.agent-pane-status').innerText();
          await first.click();
          const restored = await editor('conversation/draft.md').locator('.cm-line').allTextContents().then(lines=>lines.join('\n'));
          const answer = page.getByText('First answer', {exact:true});
          const branch = page.getByRole('button', {name:'Branch from assistant message',exact:true});
          const below = await branch.evaluate((node, answer) => !!(node.compareDocumentPosition(answer) & Node.DOCUMENT_POSITION_PRECEDING), await answer.elementHandle());
          await branch.click();
          const branches = await page.getByRole('tab', {name:'First conversation',exact:true}).count();
          const reading = await page.getByRole('log', {name:'Conversation'}).innerText();
          await history.getByRole('button', {name:'New conversation',exact:true}).click();
          const empty = await page.getByText('What shall we work on?', {exact:true}).isVisible();
          const footerSettings = await page.locator('footer').getByRole('button', {name:'Agent settings',exact:true}).count();
          await first.first().click();
          const target = first.first().locator('../..').locator('..').getByRole('tabpanel');
          const bounds = await target.boundingBox();
          await page.getByRole('tab', {name:'Second conversation',exact:true}).locator('..').dragTo(target, {targetPosition:{x:bounds.width - 5,y:bounds.height / 2}});
          const splitUsage = await page.locator('.agent-pane-status').allTextContents();
          value = {firstDraft, restored, firstUsage, secondUsage, branches, below, reading, empty, footerSettings, splitUsage,
            titles: await page.getByRole('tab').allTextContents(), redundantHeading: await page.getByText('Workspace agent', {exact:true}).count()};
          break;
        }
        case "agent-documents": {
          const source = editor("source.md");
          await source.press("Home");
          await source.press("ArrowRight");
          const sourcePosition = await page.getByLabel("Editor position", {exact: true}).innerText();
          await menu("View", "Agent History");
          await page.getByRole('region', {name:'Agent History'}).getByRole('button', {name:'# Previous question',exact:true}).click();
          const draft = editor("conversation/draft.md");
          await draft.fill(request.text);
          const history = editor("conversation/agent-history-user.md");
          await history.waitFor();
          const before = await history.textContent();
          await history.click(); await page.keyboard.type("cannot edit");
          const readonly = await history.getAttribute("contenteditable") === "false" && await history.textContent() === before;
          await draft.click(); await page.keyboard.press("End");
          const caret = await draft.evaluate(node => window.getSelection()?.anchorOffset);
          const settings = page.getByRole("button", {name: "Agent settings", exact: true});
          const dialog = page.getByRole("dialog", {name: "Agent settings", exact: true});
          const quiet = await page.getByLabel("Editor position", {exact: true}).count() === 0
            && await page.getByLabel("Provider", {exact: true}).count() === 0
            && await page.getByRole("button", {name: "Stop", exact: true}).count() === 0
            && await page.locator(".agent-pane-status").getByRole("button", {name: "Agent settings", exact: true}).count() === 1;
          await settings.click();
          await dialog.getByLabel("Provider", { exact: true }).selectOption("claude-subscription");
          const model = dialog.getByLabel("Model", { exact: true });
          await model.selectOption("sonnet");
          const pickedModel = await settings.innerText();
          await model.selectOption("custom");
          await dialog.getByLabel("Custom model", { exact: true }).fill("unavailable-model");
          await dialog.getByLabel("Effort", { exact: true }).selectOption("high");
          await page.keyboard.press("Escape");
          const closed = await dialog.count() === 0;
          const stable = await draft.evaluate(node => { window.agentDraft = node; return node.isConnected; });
          const sent = page.waitForRequest(req => req.method() === "POST" && req.url().endsWith("/api/docs/workspace/agent"));
          await draft.press("Shift+Enter");
          const payload = (await sent).postDataJSON();
          await page.getByRole("alert").filter({ hasText: "not retried" }).waitFor();
          const retained = await draft.locator(".cm-line").allTextContents().then(lines => lines.join("\n"));
          await menu("View", "Files"); await page.getByRole('tab', {name:'# Previous question',exact:true}).click();
          value = { readonly, payload, retained, quiet, closed, pickedModel, sourcePosition, stable: stable && await draft.evaluate(node => node === window.agentDraft), history: before };
          break;
        }
        case "draft": await editor("conversation/draft.md").fill(request.text); break;
        case "read-draft": value = await editor("conversation/draft.md").locator('.cm-line').allTextContents().then(lines => lines.join('\n')); break;
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
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="scripts/simple-browser-driver.mjs"><ui:paste select="#simple-b622660cef-0"/></ui:file>

</ui:doc>
