import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { expect, test } from "@playwright/test";

// Exercises the real catalog installer in a disposable project. Opt in because
// a fresh machine needs uv and network access to obtain debugpy.
test("opens the requested Hickory file and Files before layout restoration", async ({ page }) => {
  test.skip(!process.env.HICKORY_E2E_URL, "Run against the isolated literate editor engine.");
  const response = await page.request.get("/api/files");
  expect(response.ok()).toBe(true);
  const { root_path } = await response.json() as { root_path: string };
  expect(root_path).toBeTruthy();
  let release!: () => void;
  const layout = new Promise<void>(resolve => { release = resolve; });
  let layoutRequested = false;
  let releaseSelected!: () => void, releaseGuarded!: () => void, releaseParser!: () => void;
  const selected = new Promise<void>(resolve => { releaseSelected = resolve; });
  const guarded = new Promise<void>(resolve => { releaseGuarded = resolve; });
  const parser = new Promise<void>(resolve => { releaseParser = resolve; });
  let selectedRequested = false, guardedRequested = false, parserRequested = false;
  const parserAsset = (url: URL) => /\/hick_lang_bg[^/]*\.wasm$/.test(url.pathname);
  const sourceWrites: string[] = [];
  await page.route(parserAsset, async route => {
    if (route.request().resourceType() === "script") return route.continue();
    parserRequested = true;
    await parser;
    await route.continue();
  });
  await page.route("**/api/open-file", async route => {
    selectedRequested = true;
    await selected;
    await route.continue();
  });
  await page.route("**/api/file", async route => {
    if (route.request().method() === "PUT") sourceWrites.push(route.request().url());
    await route.continue();
  });
  await page.route(url => url.pathname === "/api/file" && url.searchParams.get("path") === "code.py", async route => {
    guardedRequested = true;
    await guarded;
    await route.continue();
  });
  await page.route("**/api/workspace/ui", async route => {
    if (route.request().method() !== "GET") return route.continue();
    layoutRequested = true;
    await layout;
    await route.continue();
  });
  try {
    await page.goto(`/?startupPath=code.py#/file/${encodeURIComponent(`${root_path}/code.py`)}`);
    const editor = page.locator(".cm-content").filter({ hasText: "value = 1" });
    await expect(editor).toBeVisible();
    await expect(editor).toHaveAttribute("contenteditable", "false");
    await expect.poll(() => selectedRequested && guardedRequested && parserRequested).toBe(true);
    const earlyEditor = await editor.elementHandle();
    await editor.press("x");
    await expect(editor).toHaveText("value = 1");
    expect(sourceWrites).toEqual([]);
    releaseSelected();
    await expect(editor).toHaveAttribute("contenteditable", "false");
    releaseGuarded();
    await expect(editor).toHaveAttribute("contenteditable", "true");
    expect(await editor.evaluate((current, original) => current === original, earlyEditor)).toBe(true);
    const files = page.locator("[data-spatial-files=left]");
    await expect(files).toBeVisible();
    await expect(files.locator("[data-tree-path$='code.py']")).toBeVisible();
    await expect.poll(() => layoutRequested).toBe(true);
    const held = await editor.elementHandle();
    await editor.press("ControlOrMeta+Home");
    await editor.press("End");
    await editor.pressSequentially(" # draft");
    await expect(editor).toContainText("value = 1 # draft");
    const layoutResponse = page.waitForResponse(reply => reply.url().endsWith("/api/workspace/ui") && reply.request().method() === "GET");
    release();
    await (await layoutResponse).finished();
    await page.evaluate(() => new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))));
    await expect(files).toBeVisible();
    expect(await editor.evaluate((current, original) => current === original, held)).toBe(true);
    await expect(editor).toContainText("value = 1 # draft");
    await editor.press("Shift+Home");
    const selection = await editor.evaluate(() => window.getSelection()?.toString());
    expect(selection).toBe("value = 1 # draft");
    const parserResponse = page.waitForResponse(reply => parserAsset(new URL(reply.url())) && reply.request().resourceType() !== "script");
    releaseParser();
    await (await parserResponse).finished();
    await page.evaluate(() => new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))));
    expect(await editor.evaluate((current, original) => current === original, held)).toBe(true);
    expect(await editor.evaluate(() => window.getSelection()?.toString())).toBe(selection);
    await editor.press("ControlOrMeta+z");
    await expect(editor).toHaveText("value = 1");
  } finally {
    releaseSelected();
    releaseGuarded();
    releaseParser();
    release();
  }
});

test("unified patch gutter launches and steps the completed captured project", async ({page})=>{
  test.setTimeout(120_000);
  test.skip(!process.env.HICKORY_E2E_DEBUG, "Use just test-unified-debug-ui (requires uv and the Python package index).");
  const errors:string[]=[];
  page.on("pageerror",error=>errors.push(error.message));
  const source=`# Debug a patch
<hick:project/>
<hick:file id="helper" path="helpers.py">
def format_name(name):
    result = name.upper()
    return result
</hick:file>
<hick:file id="main" path="main.py">
from helpers import format_name
name = "developer"
result = format_name(name)
print(result)
</hick:file>
<hick:patch id="change" after="main">
--- a/main.py
+++ b/main.py
@@ -1,4 +1,4 @@
 from helpers import format_name
-name = "developer"
+name = "IDE user"
 result = format_name(name)
 print(result)
</hick:patch>`;
  await page.request.put("/api/workspace/ui",{data:{state:null}});
  const response=await page.request.post("/api/projects/local/docs",{data:{path:"debug-ui.md",source}});
  expect(response.ok()).toBeTruthy(); const note=await response.json();
  await page.goto(`/#/docs/${note.id}`);
  const patch=page.getByRole("region",{name:"Project patch editor"});
  await expect(patch).toBeVisible();
  await expect(patch.locator(".comparison-panes--side-by-side")).toBeVisible();
  await expect(patch.locator(".comparison-deleted")).toContainText('name = "developer"');
  await expect(patch.locator(".comparison-added")).toContainText('name = "IDE user"');
  const launch=patch.getByRole("button",{name:"Debug main.py · After this patch",exact:true});
  await expect(launch).toBeEnabled();
  expect(await launch.evaluate(el=>!!el.closest(".cm-debug-rail") && !el.closest("details"))).toBe(true);
  const after=patch.locator(".comparison-side").last();
  const line=await after.locator(".cm-line").filter({hasText:"result = format_name(name)"}).boundingBox();
  const gutter=await after.locator(".cm-breakpoint-gutter").boundingBox();
  if(!line || !gutter) throw new Error("Expected the captured code's breakpoint gutter");
  await page.mouse.click(gutter.x+gutter.width/2,line.y+line.height/2);
  await expect(after.locator(".cm-bp")).toHaveCount(1);
  await page.screenshot({path:"/tmp/hickory-unified-patch-idle.png",fullPage:true});
  await launch.click();
  const debuggerBar=page.getByRole("toolbar",{name:"Debugger"}).last();
  await expect(debuggerBar.getByRole("button",{name:"Continue",exact:true})).toBeVisible({timeout:90_000});
  expect(await debuggerBar.evaluate(el=>!!el.closest(".cm-debug-rail"))).toBe(true);
  await page.screenshot({path:"/tmp/hickory-unified-patch-paused.png",fullPage:true});
  await debuggerBar.getByRole("button",{name:"Step into",exact:true}).click();
  await page.getByRole("button",{name:"Show generated code",exact:true}).last().click();
  const callee=page.getByRole("region",{name:"Code · helpers.py"});
  await expect(callee).toBeVisible();
  await expect(callee.locator(".cm-editor")).toHaveCount(1);
  await expect(callee.locator(".cm-paused-line")).toBeVisible();
  const calleeBar=callee.getByRole("toolbar",{name:"Debugger"});
  await expect(calleeBar.getByRole("button",{name:"Step over",exact:true})).toBeVisible();
  const returnLine=await callee.locator(".cm-line").filter({hasText:"return result"}).boundingBox();
  const calleeGutter=await callee.locator(".cm-breakpoint-gutter").boundingBox();
  if(!returnLine || !calleeGutter)throw new Error("Expected the callee's live breakpoint gutter");
  await page.mouse.click(calleeGutter.x+calleeGutter.width/2,returnLine.y+returnLine.height/2);
  await expect(callee.locator(".cm-bp:not(.cm-bp-pending)")).toHaveCount(1);
  await calleeBar.getByRole("button",{name:"Continue",exact:true}).click();
  await expect(page.locator(".cm-paused-line").filter({hasText:"return result"}).first()).toBeVisible();
  await page.getByRole("button",{name:"Show generated code",exact:true}).last().click();
  await expect(callee.locator(".cm-paused-line")).toContainText("return result");
  await page.screenshot({path:"/tmp/hickory-unified-callee-paused.png",fullPage:true});
  // Reopen the caller through its frame, preserving this captured identity.
  await calleeBar.getByLabel("Debug options",{exact:true}).click();
  const stack=calleeBar.getByRole("combobox",{name:"Call stack"});
  const callerFrame=await stack.locator("option").filter({hasText:"main.py"}).getAttribute("value");
  await stack.selectOption(callerFrame!);
  await page.getByRole("button",{name:"Show generated code",exact:true}).last().click();
  const caller=page.getByRole("region",{name:"Code · main.py"});
  await expect(caller).toBeVisible();
  await caller.getByRole("button",{name:"Stop",exact:true}).click();
  await expect(caller.getByRole("button",{name:"Debug main.py · main.py",exact:true})).toBeVisible();
  const callerLine=await caller.locator(".cm-line").filter({hasText:"result = format_name(name)"}).boundingBox();
  const callerGutter=await caller.locator(".cm-breakpoint-gutter").boundingBox();
  if(!callerLine || !callerGutter)throw new Error("Expected the caller breakpoint");
  await page.mouse.click(callerGutter.x+callerGutter.width/2,callerLine.y+callerLine.height/2);
  await expect(caller.locator(".cm-bp")).toHaveCount(0);
  // Only the callee breakpoint remains: it must be installed before launch.
  await caller.getByRole("button",{name:"Debug main.py · main.py",exact:true}).click();
  await expect(callee).toBeVisible();
  await expect(callee.locator(".cm-paused-line")).toContainText("return result");
  const clearedLine=await callee.locator(".cm-line").filter({hasText:"return result"}).boundingBox();
  const clearedGutter=await callee.locator(".cm-breakpoint-gutter").boundingBox();
  if(!clearedLine || !clearedGutter)throw new Error("Expected the callee breakpoint to clear");
  await page.mouse.click(clearedGutter.x+clearedGutter.width/2,clearedLine.y+clearedLine.height/2);
  await expect(callee.locator(".cm-bp")).toHaveCount(0);
  await calleeBar.getByRole("button",{name:"Continue",exact:true}).click();
  await expect(calleeBar.getByRole("button",{name:"Continue",exact:true})).toHaveCount(0);
  expect(errors).toEqual([]);
});


// Exercise captured URI construction through the real comparison gutter, not
// direct debugger RPC. Both percent-distinct names must retain their bytes.
test('captured literal filename uses guarded launch and generated-source navigation',async({page})=>{
  test.setTimeout(120_000);test.skip(!process.env.HICKORY_E2E_DEBUG,'Requires the explicit real debugpy fixture.');
  const {createHash}=await import('node:crypto'),{readFile}=await import('node:fs/promises');const errors:string[]=[],wire:Record<string,any>[]=[];
  page.on('pageerror',error=>errors.push(error.message));page.on('websocket',socket=>{for(const kind of ['framesent','framereceived'] as const)socket.on(kind,({payload})=>{try{if(typeof payload==='string'||payload[0]!==3)return;wire.push(JSON.parse(new TextDecoder().decode(payload.subarray(1))));}catch{}});});
  const primary='%2F #.py',distinct='%252F #.py';
  const source=`# Captured literal debugger
<hick:project/>
<hick:file id="profile" path=".hick-launch.json">
{"version":1,"profiles":[{"id":"literal","name":"Captured literal","program":"${primary}","default":true}]}
</hick:file>
<hick:file id="helper" path="helpers.py">
def answer():
    return 41
</hick:file>
<hick:file id="distinct" path="${distinct}">
marker = 999
</hick:file>
<hick:file id="main" path="${primary}">
from helpers import answer
marker = answer()
print(marker)
</hick:file>
<hick:patch id="change" after="main">
--- a/${primary}
+++ b/${primary}
@@ -1,3 +1,3 @@
 from helpers import answer
-marker = answer()
+marker = answer() + 0
 print(marker)
</hick:patch>`;
  await page.request.put('/api/workspace/ui',{data:{state:null}});const response=await page.request.post('/api/projects/local/docs',{data:{path:'captured-literal.md',source}});expect(response.ok()).toBeTruthy();const note=await response.json();
  await page.goto(`/#/docs/${note.id}`);const patch=page.getByRole('region',{name:'Project patch editor'});await expect(patch.locator('.comparison-panes--side-by-side')).toBeVisible();const after=patch.locator('.comparison-side').last();
  const tree=await (await page.request.get('/api/files')).json(),read=async(path:string)=>readFile(`${tree.root_path}/${path}`).catch(error=>{if(error.code==='ENOENT')return null;throw error;}),before=await read(primary),other=await read(distinct);
  const printLine=after.locator('.cm-line').filter({hasText:/^print\(marker\)$/});await printLine.scrollIntoViewIfNeeded();const line=await printLine.boundingBox(),gutter=await after.locator('.cm-breakpoint-gutter').boundingBox();if(!line||!gutter)throw Error('Captured breakpoint gutter unavailable');await page.mouse.click(gutter.x+gutter.width/2,line.y+line.height/2);await expect(after.locator('.cm-bp')).toHaveCount(1);
  await patch.getByRole('button',{name:`Debug ${primary} · After this patch`,exact:true}).click();const bar=page.getByRole('toolbar',{name:'Debugger'}).last();await expect(bar.getByRole('button',{name:'Continue',exact:true})).toBeVisible({timeout:90_000});
  const start=wire.find(event=>event.op==='start');expect(start?.doc).toMatch(/^hick-project:\/\/\/[^/]+\/%252F%20%23\.py$/);const fileResponse=await page.request.get('/api/project-contexts/file?uri='+encodeURIComponent(start!.doc));expect(fileResponse.ok()).toBeTruthy();const file=await fileResponse.json(),bytes=new Uint8Array(file.bytes);expect(file.path).toBe(primary);expect(new TextDecoder().decode(bytes)).toContain('marker = answer() + 0');expect(file.files.some((entry:{path:string})=>entry.path===distinct)).toBe(true);const hash=createHash('sha256').update(bytes).digest('hex');expect(start?.source_hash).toBe(hash);expect(wire.find(event=>event.event==='started')?.source_hash).toBe(hash);
  await bar.getByLabel('Debug options',{exact:true}).click();await bar.getByRole('button',{name:'+ watch',exact:true}).click();const prompt=page.getByRole('dialog',{name:'Expression to watch:',exact:true});await prompt.getByRole('textbox').fill('marker');await prompt.getByRole('button',{name:'Apply',exact:true}).click();await expect(bar.getByLabel('Value of watch marker',{exact:true})).toHaveText('41');
  await page.getByRole('button',{name:'Show generated code',exact:true}).last().click();const reading=page.getByRole('region',{name:`Code · ${primary}`,exact:true});await expect(reading).toBeVisible();await expect(reading.locator('.cm-content')).toContainText('marker = answer() + 0');await expect(reading.locator('.cm-paused-line')).toContainText('print(marker)');
  const capturedBar=reading.getByRole('toolbar',{name:'Debugger'});await capturedBar.getByRole('button',{name:'Stop',exact:true}).click();await expect(capturedBar.getByRole('button',{name:'Stop',exact:true})).toHaveCount(0);expect(await read(primary)).toEqual(before);expect(await read(distinct)).toEqual(other);expect(errors).toEqual([]);
});
for (const calleeOnly of [false,true]) test(`patch debugger keeps a later welcome definition in the literate document · ${calleeOnly ? "callee breakpoint" : "step into"}`, async ({page}) => {
  test.setTimeout(120_000);
  test.skip(!process.env.HICKORY_E2E_DEBUG, "Use just test-unified-debug-ui.");
  const source = `# Greeting
<hick:project/>
<hick:file id="app" path="app.py">
from library import greet
print(greet())
</hick:file>
<hick:file id="library" path="library.py">
def greet():
    return "Hello"
</hick:file>
<hick:scope id="greeting" after="app library">
<hick:patch id="caller">
--- a/app.py
+++ b/app.py
@@ -1,2 +1,2 @@
-from library import greet
-print(greet())
+from library import welcome
+print(welcome("Hickory"))
</hick:patch>
<hick:patch id="complete" after="caller">
--- a/library.py
+++ b/library.py
@@ -1,2 +1,2 @@
-def greet():
-    return "Hello"
+def welcome(name):
+    return "Hello " + name
</hick:patch>
</hick:scope>`;
  const response = await page.request.post("/api/projects/local/docs", {data:{path:calleeOnly ? "welcome-breakpoint.md" : "welcome-debug.md", source}});
  expect(response.ok()).toBeTruthy();
  const note = await response.json();
  await page.goto(`/#/docs/${note.id}`);
  const patch = page.getByRole("region", {name:"Project patch editor"}).first();
  await expect(patch.getByRole("button", {name:"Debug app.py · After this patch", exact:true})).toBeEnabled();
  const target = calleeOnly ? page.getByRole("region", {name:"Project patch editor"}).nth(1) : patch;
  await expect(target.getByRole("button", {name:/^Debug .* · After this patch$/})).toBeEnabled();
  const after = target.locator(".comparison-side").last();
  const line = await after.locator(".cm-line").filter({hasText:calleeOnly ? 'return "Hello " + name' : 'print(welcome("Hickory"))'}).boundingBox();
  const gutter = await after.locator(".cm-breakpoint-gutter").boundingBox();
  if (!line || !gutter) throw new Error("Expected caller breakpoint gutter");
  await page.mouse.click(gutter.x + gutter.width/2, line.y + line.height/2);
  await expect(after.locator(".cm-bp")).toHaveCount(1);
  await patch.getByRole("button", {name:"Debug app.py · After this patch", exact:true}).click();
  await expect(page.getByRole("button", {name:"Continue",exact:true}).last()).toBeVisible({timeout:90_000});
  await page.screenshot({path:"/tmp/welcome-paused.png",fullPage:true});
  // Pausing and stepping preserve both rendered, highlighted patch panes.
  if (!calleeOnly) {
    await expect(patch.locator(".cm-paused-line")).toContainText('print(welcome("Hickory"))', {timeout:90_000});
    await page.getByRole("button", {name:"Step into", exact:true}).last().click();
  }
  const calleePatch = page.getByRole("region", {name:"Project patch editor"}).nth(1);
  await expect(calleePatch.locator(".cm-paused-line")).toContainText('return "Hello " + name');
  await expect(page.getByRole("region", {name:"Project patch editor"})).toHaveCount(2);
  await expect(calleePatch.locator(".comparison-panes--side-by-side")).toBeVisible();
  await expect(calleePatch.locator(".comparison-added.cm-paused-line")).toContainText('return "Hello " + name');
  await expect(calleePatch.locator(".cm-paused-arrow")).toHaveCount(1);
  await expect(page.getByRole("region", {name:"Code · library.py"})).toHaveCount(0);
  await page.getByRole("button", {name:"Show generated code",exact:true}).last().click();
  const generated = page.getByRole("region", {name:"Code · library.py"});
  await expect(generated.locator(".cm-paused-line")).toContainText('return "Hello " + name');
  await expect(generated.locator(".cm-paused-arrow")).toHaveCount(1);
  await generated.getByRole("button", {name:"Show literate source",exact:true}).click();
  await expect(calleePatch.locator(".comparison-panes--side-by-side")).toBeVisible();
  await expect(calleePatch.locator(".cm-paused-line")).toContainText('return "Hello " + name');
  await expect(calleePatch.locator(".cm-paused-arrow")).toHaveCount(1);
});

// Guarantees: authoring/a-literate-view-writes-through-to-ordinary-source.md;
// editing/a-comparison-keeps-current-code-editable.md.
test("edit ordinary source in the main literate diff, then read history without changing it", async ({page})=>{
  test.setTimeout(45_000);
  test.skip(!process.env.HICKORY_E2E_URL,"Use just test-literate-editor.");
  const errors:string[]=[];
  page.on("pageerror",e=>{errors.push(e.message); console.log("PAGE ERROR:",e.message);});
  page.on("console",m=>{if(m.type()==="error") console.log("BROWSER:",m.text());});
  const response=await page.request.post("/api/representations",{data:{backing:{kind:"files",paths:["code.py"]}}});
  expect(response.ok()).toBeTruthy(); const view=await response.json();
  await page.goto("/");
  await expect(page.getByRole("tablist",{name:"Center views"})).toBeVisible();
  await page.evaluate(id=>window.dispatchEvent(new CustomEvent("hickory.open-representation",{detail:{id}})),view.id);
  await page.getByRole("tab",{name:"Literate view",exact:true}).click();
  const pane=page.getByRole("region",{name:"Literate editor"});
  await expect(pane).toBeVisible();
  const line=pane.locator(".cm-line").filter({hasText:"value = 1"});
  await expect(line).toBeVisible(); await line.click();
  const content=pane.locator(".cm-content").first();
  await content.press("Home"); await content.press("Shift+End"); await content.pressSequentially("value = 7");
  await page.getByRole("button",{name:"Save",exact:true}).click();
  await expect.poll(async()=> (await (await page.request.get("/api/file?path=code.py")).json()).content).toBe("value = 7\n");
  await page.getByText("Compare revisions…", { exact: true }).click();
  await page.getByLabel("Comparison base").fill("HEAD");
  await page.getByRole("button",{name:"Show diff",exact:true}).click();
  await expect(pane.locator(".comparison-removed")).toHaveCount(0);
  await pane.locator(".comparison-deletion").click();
  await expect(pane.locator(".comparison-removed pre")).toContainText("value = 1");
  await expect(pane.locator(".comparison-added")).toContainText("value = 7");
  const editedLine=pane.locator(".cm-line").filter({hasText:"value = 7"});
  await editedLine.click(); await content.press("Home"); await content.press("Shift+End"); await content.pressSequentially("value = 8");
  await page.getByRole("button",{name:"Save",exact:true}).click();
  await expect.poll(async()=> (await (await page.request.get("/api/file?path=code.py")).json()).content).toBe("value = 8\n");
  await page.getByLabel("Comparison target").fill("HEAD"); await page.getByRole("button",{name:"Show diff",exact:true}).click();
  await expect(content).toHaveAttribute("contenteditable","false");
  await expect(page.locator(".representation-versions")).toContainText("historical, read-only");
  expect((await (await page.request.get("/api/file?path=code.py")).json()).content).toBe("value = 8\n");
  await page.getByRole("button",{name:"Return to current code"}).click();
  await expect(content).toHaveAttribute("contenteditable","true");
  expect(errors).toEqual([]);
});

// Guarantee: git/bisect-keeps-candidates-isolated.md.
test("the visual bisect pane shows candidates, verdicts, and Git's first bad result",async({page})=>{
  test.setTimeout(45_000);
  test.skip(!process.env.HICKORY_E2E_URL,"Use just test-literate-editor.");
  await page.goto("/");
  await page.getByRole("button",{name:/^Branch /}).click();
  await page.getByText("Visual bisect",{exact:true}).click();
  await page.getByLabel("Known good commit").fill("HEAD~6");
  await page.getByLabel("Known bad commit").fill("HEAD");
  await page.getByRole("button",{name:"Start bisect",exact:true}).click();
  const pane=page.getByRole("region",{name:"Bisect search"});
  await expect(pane).toBeVisible();
  await expect(pane.getByRole("list",{name:"Bisect commit graph"})).toBeVisible();
  let session=(await (await page.request.get("/api/git/bisect")).json()).sessions[0];
  const id=session.id;
  try {
    for(let i=0;i<10 && !session.outcome;i++) {
      const row=session.graph.find((r:{sha:string})=>r.sha===session.candidate);
      const n=Number(row.subject.split(" ").at(-1));
      await pane.getByRole("button",{name:n>=3?"Bad":"Good",exact:true}).click();
      const previous=`${session.candidate}/${session.outcome}`;
      await expect.poll(async()=> { const s=(await (await page.request.get("/api/git/bisect")).json()).sessions.find((s:{id:string})=>s.id===id); return `${s.candidate}/${s.outcome}`; }).not.toBe(previous);
      session=(await (await page.request.get("/api/git/bisect")).json()).sessions.find((s:{id:string})=>s.id===id);
      if(!session.outcome) await expect(pane).toContainText(session.candidate.slice(0,12));
    }
    expect(session.outcome).toBe("first_bad");
    await expect(pane).toContainText("First bad commit");
    const original=await(await page.request.get("/api/file?path=revision.txt")).json();
    expect(original.content).toBe("6\n");
    await pane.getByRole("button",{name:"End bisect",exact:true}).click();
    await expect(pane).toHaveCount(0);
  } finally {
    await page.request.delete(`/api/git/bisect/${id}`);
  }
});

// Guarantees: lenses/a-commit-reads-as-a-literate-change.md;
// agent/the-agent-pane-is-a-live-document.md; agent/conversation-edits-use-the-client-review-policy.md.
test("review an ACP proposal in the literate editor, then read its commit", async ({ page }) => {
  test.setTimeout(60_000);
  test.skip(!process.env.HICKORY_E2E_URL || !process.env.HICKORY_ACP_FIXTURE, "Use the isolated literate editor harness with its ACP fixture.");
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  const catalogue = await page.request.put("/api/agents", { data: [{ id: "fixture", name: "Fixture", command: process.env.HICKORY_ACP_FIXTURE, args: [] }] });
  expect(catalogue.ok()).toBeTruthy();
  const created = await page.request.post("/api/projects/local/docs", { data: { path: "review-note.md", source: "# Original 📝\n\nKeep this paragraph.\n" } });
  expect(created.ok()).toBeTruthy();
  const note = await created.json();
  await page.goto(`/#/docs/${note.id}`);
  await expect(page.locator(".cm-content").filter({ hasText: "Original 📝" })).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  const agent = page.getByRole("region", { name: "Agent chat" });
  await expect(agent).toBeVisible();
  await agent.getByRole("button", { name: "Agent settings" }).click();
  await page.getByRole("combobox", { name: "Agent", exact: true }).selectOption("fixture");
  await page.getByRole("combobox", { name: "Agent", exact: true }).press("Escape");
  await expect(agent.getByRole("button", { name: "Send", exact: true })).toBeDisabled();
  const response = agent.getByRole("textbox", { name: "Agent conversation and response" });
  await response.click(); await response.press("ControlOrMeta+End"); await response.pressSequentially("buffer-edit");
  // docs/guarantees/agent/the-agent-pane-is-a-live-document.md
  await expect(agent.locator(".cm-line").filter({ hasText: "buffer-edit" }).getByRole("button", { name: "Send", exact: true })).toBeVisible();
  await agent.getByRole("button", { name: "Send", exact: true }).click();
  const review = page.getByRole("region", { name: "Document change review" });
  await expect(review).toBeVisible();
  await expect(review.locator(".comparison-added")).toContainText("Changed by ACP");
  await expect(review.locator(".comparison-deleted")).toContainText("Original 📝");
  expect((await (await page.request.get(`/api/docs/${note.id}`)).json()).source).toContain("Original 📝");
  await expect(agent.getByRole("button", { name: "Stop the agent" })).toBeVisible();
  await page.screenshot({ path: "/tmp/hickory-proposal-reading.png", fullPage: true });
  await review.getByRole("button", { name: "Accept change" }).click();
  await expect.poll(async () => (await (await page.request.get(`/api/docs/${note.id}`)).json()).source).toContain("Changed by ACP");
  await expect(agent.getByRole("button", { name: "Send", exact: true })).toBeVisible();
  // Each recorded message is a separate protected editor.
  await agent.locator(".conversation-card--user").first().scrollIntoViewIfNeeded();
  // docs/guarantees/agent/a-session-is-the-conversation.md
  await expect(agent.locator(".cm-bubble-you").first()).toHaveCSS("text-align", "left");
  await expect(agent.locator(".cm-bubble-you").first()).toHaveCSS("margin-left", "0px");
  await expect(agent.locator(".cm-bubble-agent").first()).toHaveCSS("text-align", "left");
  // Context stays in native editor folds with real caret/selection positions.
  // docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
  await expect(response.locator("details")).toHaveCount(0);
  const context = agent.locator(".cm-line").filter({ hasText: '<hick:context kind="editor-buffers">' });
  await expect(context.locator(".cm-hick-fold-placeholder")).toBeVisible();
  await context.locator(".cm-hick-fold-placeholder").click();
  await expect(agent.locator('.conversation-card--user').first()).toContainText('review-note.md');
  // The completed answer is protected; selecting all and typing cannot erase it.
  await response.press("ControlOrMeta+a"); await response.pressSequentially("Should not replace history");
  await expect(agent.locator(".conversation-card--user").first()).toContainText("buffer-edit");
  await response.press("ControlOrMeta+End"); await response.press("Enter"); await response.pressSequentially("Next response draft");
  await expect(response).toContainText("Next response draft");
  await page.screenshot({ path: "/tmp/hickory-agent-inline-send.png", fullPage: true });
  await page.request.post("/api/git/stage", { data: { paths: ["review-note.md"] } });
  const committed = await page.request.post("/api/git/commit", { data: { message: "# A reviewed note\n\nThe agent changed its heading." } });
  expect(committed.ok()).toBeTruthy();
  const commit = await committed.json();
  await page.evaluate(sha => window.dispatchEvent(new CustomEvent("hickory.open-reading", { detail: { id: `commit:${sha}`, title: "Commit" } })), commit.sha);
  const reading = page.getByRole("article", { name: "Commit reading" });
  await expect(reading).toContainText("A reviewed note");
  await expect(reading.locator(".comparison-added").filter({ hasText: "Changed by ACP" })).toHaveCount(1);
  await page.screenshot({ path: "/tmp/hickory-commit-reading.png", fullPage: true });
  expect(errors).toEqual([]);
});

// Guarantee: agent/an-answer-in-the-agent-pane-has-ribbons.md.
test("an ACP final claim about the startup Untitled note has a visible context ribbon", async ({ page }) => {
  test.setTimeout(60_000);
  test.skip(!process.env.HICKORY_E2E_URL || !process.env.HICKORY_ACP_FIXTURE, "Use just test-literate-editor.");
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.request.put("/api/agents", { data: [{ id: "fixture", name: "Fixture", command: process.env.HICKORY_ACP_FIXTURE, args: [] }] });
  await page.addInitScript(() => localStorage.setItem("hickory.filesWindows", "true"));
  await page.goto("/");
  await expect(page.getByRole("tab", { name: "Untitled", exact: true })).toBeVisible({ timeout: 20_000 });
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  await page.getByRole("tab", { name: "Untitled", exact: true }).click();
  await page.getByRole("button", { name: "Pin view in drawer", exact: true }).click();
  await page.getByRole("tab", { name: "Agent", exact: true }).click();
  const agent = page.getByRole("region", { name: "Agent chat" });
  await agent.getByRole("button", { name: "Agent settings" }).click();
  await page.getByRole("combobox", { name: "Agent", exact: true }).selectOption("fixture");
  await page.getByRole("combobox", { name: "Agent", exact: true }).press("Escape");
  await agent.getByRole("button", { name: "Agent settings" }).click();
  await page.getByText("Next context projection", { exact: true }).click();
  await page.getByRole("checkbox", { name: "Untitled", exact: true }).click();
  await expect(page.getByRole("checkbox", { name: "Untitled", exact: true })).toBeChecked();
  await page.getByRole("checkbox", { name: "Pin full text", exact: true }).click();
  await expect(page.getByRole("checkbox", { name: "Pin full text", exact: true })).toBeChecked();
  await page.keyboard.press("Escape");
  const response = agent.getByRole("textbox", { name: "Agent conversation and response" });
  await response.click(); await response.press("ControlOrMeta+End"); await response.pressSequentially("untitled-paragraph");
  await agent.getByRole("button", { name: "Send", exact: true }).click();
  // Each message is a separate native literate buffer; only the composer edits.
  const answer = agent.locator(".conversation-card--assistant").filter({ hasText: "The second paragraph has the most characters." });
  await expect(answer).toBeVisible();
  await expect(response).not.toContainText("The second paragraph has the most characters.");
  await expect(answer.getByRole("textbox", { name: "Recorded message" })).toHaveAttribute("aria-readonly", "true");
  await expect(answer.locator(".cm-content")).toHaveAttribute("contenteditable", "false");
  await response.pressSequentially("## Follow-up\n\nKeep this draft.");
  await expect(response).toContainText("Keep this draft.");
  await agent.getByRole("button", { name: "Message agent" }).click();
  await expect(response).toContainText("Keep this draft.");
  // The response stays available while history grows; review/history start quiet.
  await expect(agent.getByText("Conversation history", { exact: true })).toBeVisible();
  await expect.poll(async () => agent.evaluate(host => {
    const draft = host.querySelector(".conversation-card--composer")!.getBoundingClientRect();
    const log = host.querySelector(".chat-log")!.getBoundingClientRect();
    return draft.top >= log.top && draft.bottom <= log.bottom + 1;
  })).toBe(true);
  await page.screenshot({ path: "/tmp/hickory-agent-pane.png", fullPage: true });
  // Context remains folded; its connection must originate from the answer itself.
  const context = agent.locator(".cm-line").filter({ hasText: '<hick:context kind="editor-buffers">' });
  await expect(context.locator(".cm-hick-fold-placeholder")).toBeVisible();
  await agent.locator(".cm-line").filter({ hasText: "The second paragraph has the most characters." }).scrollIntoViewIfNeeded();
  const ribbon = page.locator(".ribbon-group--context").filter({ has: page.locator("title", { hasText: "selected source lines" }) });
  await expect(ribbon.locator("path.ribbon, path.ribbon-brace-link").last()).toBeVisible();
  // The source brace must be painted over its Agent item, not behind dock chrome.
  const sourceBrace = ribbon.last().locator("path.ribbon-brace:not(.ribbon-brace-link)").first();
  const endpoint = await sourceBrace.evaluate(element => {
    const path = element as SVGPathElement;
    const local = path.getPointAtLength(2);
    const point = new DOMPoint(local.x, local.y).matrixTransform(path.getScreenCTM()!);
    const hit = document.elementFromPoint(point.x, point.y);
    const pane = document.querySelector('.chat-dock--pane')!.getBoundingClientRect();
    return { x: point.x, paneLeft: pane.left, paneRight: pane.right,
      hit: hit?.getAttribute("class"), visible: hit?.classList.contains("ribbon-brace-hit") ?? false };
  });
  expect(endpoint.x).toBeGreaterThan(endpoint.paneLeft);
  expect(endpoint.x).toBeLessThan(endpoint.paneRight);
  expect(endpoint.visible).toBe(true);
  await expect(page.locator("[data-shell-tab-kind=untitled]").first()).toBeVisible();
  await page.screenshot({ path: "/tmp/hickory-untitled-answer-ribbon.png", fullPage: true });
  expect(errors).toEqual([]);
});

// Guarantees: docs/guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file.md;
// docs/guarantees/execution/a-failed-action-keeps-its-editor.md.
test("automatic Save clears dirty state and a failed Run keeps the editable document", async ({ page }) => {
  test.setTimeout(45_000);
  test.skip(!process.env.HICKORY_E2E_URL, "Use just test-literate-editor.");
  const created = await page.request.post("/api/projects/local/docs", { data: { path: "coherence.md", source: "# Before\n" } });
  expect(created.ok()).toBeTruthy();
  const note = await created.json();
  await page.goto(`/#/docs/${note.id}`);
  const document = page.locator("[data-spatial-center=true]").filter({ has: page.getByRole("toolbar", { name: "Actions for coherence.md", exact: true }) });
  const editor = document.locator(".cm-content");
  await expect(editor).toContainText("Before");
  await editor.click(); await editor.press("ControlOrMeta+a"); await editor.pressSequentially("# After automatic save");
  await expect.poll(async () => (await (await page.request.get(`/api/docs/${note.id}`)).json()).source).toContain("After automatic save");
  await expect(document.locator(".save-state")).toHaveText("Saved to disk");
  await expect(page.getByRole("tab", { name: /coherence.md/ })).not.toContainText("*");
  await page.route(`**/api/docs/${note.id}/run`, route => route.fulfill({ status: 503, contentType: "application/json", body: JSON.stringify({ error: "temporary runner failure" }) }));
  await document.getByRole("button", { name: "More actions for this document" }).click();
  await page.getByRole("menuitem", { name: "Run", exact: true }).click();
  const error = document.getByRole("alert");
  await expect(error).toContainText("temporary runner failure");
  await expect(editor).toHaveAttribute("contenteditable", "true");
  await expect(editor).toContainText("After automatic save");
  await page.unroute(`**/api/docs/${note.id}/run`);
  await error.getByRole("button", { name: "Retry Run", exact: true }).click();
  await expect(document).toContainText("Re-woven");
  await expect(editor).toBeVisible();
  await page.screenshot({ path: "/tmp/hickory-design-document.png", fullPage: true });
});

// Guarantee: docs/guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file.md.
test("an inactive literate view prompts before close and writes its code when Save is chosen", async ({ page }) => {
  test.setTimeout(45_000);
  test.skip(!process.env.HICKORY_E2E_URL, "Use just test-literate-editor.");
  const created = await page.request.post("/api/representations", { data: { backing: { kind: "files", paths: ["code.py"] } } });
  const view = await created.json();
  await page.goto("/");
  await page.evaluate(id => window.dispatchEvent(new CustomEvent("hickory.open-representation", { detail: { id } })), view.id);
  const pane = page.getByRole("region", { name: "Literate editor" });
  await expect(pane).toBeVisible();
  const line = pane.locator(".cm-line").filter({ hasText: /value = / });
  await line.click();
  const editor = pane.locator(".cm-content");
  await editor.press("Home"); await editor.press("Shift+End"); await editor.pressSequentially("value = 42");
  await expect(pane).toContainText("Unsaved changes");
  await pane.locator(".change-preview > summary").click();
  await expect(pane.locator(".change-preview")).toContainText("Save writes these edits");
  await page.screenshot({ path: "/tmp/hickory-design-preview.png", fullPage: true });
  // Switching away unmounts the view. Its pending text must still be saveable.
  await page.getByRole("tab", { name: /Untitled/ }).click();
  await page.getByRole("button", { name: "Close Literate view", exact: true }).click();
  const prompt = page.getByRole("dialog").filter({ hasText: "Save changes to Literate view?" });
  await expect(prompt).toContainText("no recovery draft");
  await prompt.getByRole("button", { name: "Save", exact: true }).click();
  await expect.poll(async () => (await (await page.request.get("/api/file?path=code.py")).json()).content).toBe("value = 42\n");
  await expect(page.getByRole("tab", { name: /Literate view/ })).toHaveCount(0);
});

// Guarantee: docs/guarantees/lineage/a-ribbon-crosses-documents.md.
test("a long Files row keeps its ribbon endpoint inside the actual scrollport", async ({ page }) => {
  test.skip(!process.env.HICKORY_E2E_URL, "Use just test-literate-editor.");
  await page.goto("/");
  const row = page.locator('[data-spatial-files=left] [data-tree-path="code.py"]');
  await expect(row).toBeVisible();
  // Force the existing Files editor to overflow just as a long file name does.
  await page.addStyleTag({ content: '.filesystem-editor .cm-line { min-width: 1000px; } .filesystem-editor .cm-scroller::-webkit-scrollbar { width: 15px; height: 15px; }' });
  const measure = () => page.evaluate(async () => {
    const modulePath = "/src/shell/Ribbons.tsx";
    const { findTerminal, attachTerminal } = await import(/* @vite-ignore */ modulePath);
    const row = document.querySelector<HTMLElement>('[data-tree-path="code.py"]')!;
    const scroller = row.closest<HTMLElement>(".cm-scroller")!;
    const scrollBox = scroller.getBoundingClientRect();
    const terminal = findTerminal(document.body, "file", "code.py");
    const attachment = attachTerminal(terminal, new DOMRect(), 1800, { yTop: 100, yBot: 200 }, true, { x: 1790, y: 150 }, -1);
    return { rowRight: row.getBoundingClientRect().right, right: terminal.rect.right,
      left: terminal.rect.left, visibleRight: scrollBox.left + scroller.clientLeft + scroller.clientWidth,
      visibleLeft: scrollBox.left + scroller.clientLeft, connector: attachment.connector };
  });
  const before = await measure();
  expect(before.rowRight).toBeGreaterThan(before.visibleRight + 100);
  expect(before.right).toBeLessThanOrEqual(before.visibleRight);
  expect(before.connector.x + before.connector.width).toBe(before.right);
  await row.evaluate(element => { element.closest(".cm-scroller")!.scrollLeft = 100; });
  const after = await measure();
  expect(after.right).toBe(before.right);
  expect(after.left).toBeGreaterThanOrEqual(after.visibleLeft);
});

// Guarantee: docs/guarantees/lineage/a-ribbon-crosses-documents.md.
for (const order of [["editor", "files", "agent"], ["files", "editor", "agent"], ["agent", "files", "editor"]]) {
  test(`a brace opens its file toward the arrow: ${order.join(" | ")}`, async ({ page }) => {
    test.setTimeout(45_000);
    test.skip(!process.env.HICKORY_E2E_URL || !process.env.HICKORY_ACP_FIXTURE, "Use just test-literate-editor.");
    const created = await page.request.post("/api/projects/local/docs", { data: { path: `direction-${order.join("-")}.md`, source: "# Directional opening\n" } });
    expect(created.ok()).toBeTruthy(); const note = await created.json();
    const entries = {
      editor: { id: "editor-tab", kind: "document", target: note.path, docId: note.id, title: "Direction note" },
      files: { id: "files-tab", kind: "tree", target: "folder", title: "Files" },
      agent: { id: "agent-tab", kind: "chat", target: "chat", title: "Agent" },
    };
    const children = order.map(id => ({ type: "pane", id, tabs: [entries[id as keyof typeof entries]], active: 0 }));
    const stored = await page.request.put("/api/workspace/ui", { data: { state: { version: 1, wrap: {}, zoom: {}, tables: {}, layout: {
      root: { type: "split", id: "direction-root", direction: "row", children, sizes: order.map(id => id === "files" ? 0.16 : 0.42) }, focus: "agent",
    } } } });
    expect(stored.ok()).toBeTruthy();
    await page.request.put("/api/agents", { data: [{ id: "fixture", name: "Fixture", command: process.env.HICKORY_ACP_FIXTURE, args: [] }] });
    await page.goto(`/#/docs/${note.id}`);
    const agent = page.getByRole("region", { name: "Agent chat" }); await expect(agent).toBeVisible();
    if (await agent.getByRole("button", { name: "New thread", exact: true }).isVisible())
      await agent.getByRole("button", { name: "New thread", exact: true }).click();
    const editorPane = page.getByRole("tab", { name: "Direction note" }).locator("xpath=ancestor::*[contains(@class, 'shell-pane')][1]");
    const editorId = await editorPane.getAttribute("data-shell-pane-id");
    await agent.getByRole("button", { name: "Agent settings" }).click();
    await page.getByRole("combobox", { name: "Agent", exact: true }).selectOption("fixture");
    await page.getByRole("combobox", { name: "Agent", exact: true }).press("Escape");
    const response = agent.getByRole("textbox", { name: "Agent conversation and response" });
    await response.click(); await response.press("ControlOrMeta+End"); await response.pressSequentially("ribbon-file");
    await agent.getByRole("button", { name: "Send", exact: true }).click();
    await expect(agent.locator(".conversation-card--assistant")).toContainText("See");
    const brace = page.locator(".ribbon-group--declared").filter({ has: page.locator("title", { hasText: "code.py" }) }).last().locator("path.ribbon-brace-link");
    await expect(brace).toBeVisible();
    const point = await brace.evaluate(element => {
      const path = element as SVGPathElement;
      for (const fraction of [0.25, 0.4, 0.6, 0.75]) {
        const local = path.getPointAtLength(path.getTotalLength() * fraction);
        const screen = new DOMPoint(local.x, local.y).matrixTransform(path.getScreenCTM()!);
        const hit = document.elementFromPoint(screen.x, screen.y);
        if (hit?.classList.contains("ribbon-brace-hit") && hit.parentElement === path.parentElement)
          return { x: screen.x, y: screen.y };
      }
      throw new Error("No unobscured click point on the declared connection");
    });
    await page.mouse.click(point.x, point.y);
    const file = page.locator('[data-shell-tab-kind="file"][data-shell-tab-target="code.py"]');
    await expect(file).toBeVisible();
    expect(await file.locator("xpath=ancestor::*[@data-shell-pane-id][1]").getAttribute("data-shell-pane-id")).toBe(editorId);
    await expect(agent).toBeVisible();
  });
}

// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
for (const surface of ["file", "before", "after"] as const) test(`exact-source debug chooses the latest compatible project · ${surface}`, async ({page}) => {
  test.setTimeout(120_000);
  test.skip(!process.env.HICKORY_E2E_DEBUG, "Use just test-exact-source-debug-ui.");
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  const source = `<hick:project/>
<hick:file id="helper" path="helper.py">
value = 1
</hick:file>
<hick:file id="main" after="helper" path="main.py">
from helper import value
marker = "viewed"
total = value
print(marker, total)
</hick:file>
<hick:patch id="updated" after="main">
--- a/helper.py
+++ b/helper.py
@@ -1 +1 @@
-value = 1
+value = 7
</hick:patch>
<hick:patch id="replaced" after="updated">
--- a/main.py
+++ b/main.py
@@ -1,4 +1,4 @@
 from helper import value
-marker = "viewed"
+marker = "replaced"
 total = value
 print(marker, total)
</hick:patch>`;
  await page.request.put("/api/workspace/ui", {data:{state:null}});
  const response = await page.request.post("/api/projects/local/docs", {data:{path:`exact-debug-${surface}.md`,source}});
  expect(response.ok()).toBeTruthy();
  const note = await response.json();
  await page.goto(`/#/docs/${note.id}`);
  await expect(page.locator(".project-context-selector")).toHaveCount(0);
  const patch = page.getByRole("region", {name:"Project patch editor"}).last();
  await expect(patch.getByRole("button", {name:"Debug main.py · Before this patch",exact:true})).toBeEnabled();
  const pane = surface === "file" ? page.locator(".document-editor > .editor-cm-host > .cm-editor").first()
    : patch.locator(".comparison-side").nth(surface === "before" ? 0 : 1);
  const lineNumber = surface === "file" ? source.split("\n").indexOf("print(marker, total)") : 3;
  const ghost = pane.locator(`.cm-bp-ghost[data-breakpoint-line="${lineNumber}"]`).first();
  await ghost.scrollIntoViewIfNeeded();
  await ghost.click();
  await expect(pane.locator(".cm-bp")).toHaveCount(1);
  if (surface === "file") await page.getByRole("button", {name:"Debug main.py",exact:true}).click();
  else await patch.getByRole("button", {name:`Debug main.py · ${surface === "before" ? "Before" : "After"} this patch`,exact:true}).click();
  await expect(page.getByRole("button", {name:"Continue",exact:true}).last()).toBeVisible({timeout:90_000});
  await expect(pane.locator(".cm-paused-line").first()).toContainText("print(marker, total)");
  await expect(pane.locator(".cm-debug-value").filter({hasText:"total = 7"}).first()).toBeVisible();
  await expect(pane.locator(".cm-debug-value").filter({hasText:surface === "after" ? "replaced" : "viewed"}).first()).toBeVisible();
  await expect(page.getByRole("region", {name:"Code · main.py"})).toHaveCount(0);
  await page.screenshot({path:`/tmp/hickory-exact-debug-${surface}.png`,fullPage:true});
  await page.getByRole("button", {name:"Stop",exact:true}).last().click();
  expect(errors).toEqual([]);
});

// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
for(const launch of ["old-caller","new-caller","greet","welcome"] as const) test(`launch profiles reach library breakpoints through the application · ${launch}`,async({page})=>{
  test.setTimeout(120_000);
  test.skip(!process.env.HICKORY_E2E_DEBUG,"Use just test-launch-profiles-ui.");
  const folder=`profiles-${launch}`;
  const library=`<hick:file id="library" path="library.py">
def greet():
    return "Hello"
</hick:file>`;
  const libResponse=await page.request.post("/api/projects/local/docs",{data:{path:`${folder}/library.md`,source:library}});
  expect(libResponse.ok()).toBeTruthy();
  const source=`# Greeting application
<hick:project sources="library.md"/>
<hick:file id="app" path="app.py">
from library import greet
print(greet())
</hick:file>
<hick:scope id="greeting" after="app library">
<hick:patch id="caller">
--- a/app.py
+++ b/app.py
@@ -1,2 +1,2 @@
-from library import greet
-print(greet())
+from library import welcome
+print(welcome("Hickory"))
</hick:patch>
<hick:patch id="complete" after="caller">
--- a/library.py
+++ b/library.py
@@ -1,2 +1,2 @@
-def greet():
-    return "Hello"
+def welcome(name):
+    return "Hello " + name
</hick:patch>
</hick:scope>`;
  await page.request.put("/api/workspace/ui",{data:{state:null}});
  const created=await page.request.post("/api/projects/local/docs",{data:{path:`${folder}/project.md`,source}});
  expect(created.ok()).toBeTruthy();const note=await created.json();
  await page.goto(`/#/docs/${note.id}`);
  const patches=page.getByRole("region",{name:"Project patch editor"});
  const libraryPatch=patches.nth(1);
  const before=libraryPatch.locator(".comparison-side").first(),after=libraryPatch.locator(".comparison-side").last();
  await expect(libraryPatch.getByRole("button",{name:"Debug library.py · Before this patch",exact:true})).toBeEnabled();
  for(const pane of [before,after]) {
    const dot=pane.locator('.cm-bp-ghost[data-breakpoint-line="1"]');await dot.scrollIntoViewIfNeeded();await dot.click();
    await expect(pane.locator(".cm-bp")).toHaveCount(1);
  }
  const old=launch==="old-caller" || launch==="greet";
  const target=launch.endsWith("caller")?patches.first():libraryPatch;
  const path=launch.endsWith("caller")?"app.py":"library.py";
  await target.getByRole("button",{name:`Debug ${path} · ${old?"Before":"After"} this patch`,exact:true}).click();
  const stopped=old?before:after;
  await expect(stopped.locator(".cm-paused-line")).toContainText(old?'return "Hello"':'return "Hello " + name',{timeout:90_000});
  await expect(page.getByRole("region",{name:"Code · library.py"})).toHaveCount(0);
  await page.screenshot({path:`/tmp/hickory-launch-profile-${launch}.png`,fullPage:true});
  await page.getByRole("button",{name:"Stop",exact:true}).last().click();
});

// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
 test("manual launch profiles preserve arguments environment build and source layout",async({page})=>{
  test.setTimeout(60_000);test.skip(!process.env.HICKORY_E2E_URL,"Use just test-launch-profiles-ui.");
  const config={version:1,profiles:[{id:"named",name:"Greeting with Ada",program:"app.py",args:["Ada Lovelace","$(touch unwanted)"],env:{GREETING:"Welcome"},build:["python3","-c","print('profile build')"],default:true}]};
  const source=`<hick:project/>
<hick:file path=".hick-launch.json">${JSON.stringify(config)}</hick:file>
<hick:file path="app.py">
import os, sys
message = os.environ['GREETING'] + ' ' + sys.argv[1]
print(message, sys.argv[2])
</hick:file>`;
  await page.request.put("/api/workspace/ui",{data:{state:null}});
  const created=await page.request.post("/api/projects/local/docs",{data:{path:"manual-profile/project.md",source}});
  expect(created.ok()).toBeTruthy();const note=await created.json();
  await page.goto(`/#/docs/${note.id}`);
  const sourceLine=page.locator(".document-editor > .editor-cm-host > .cm-editor").first().locator(".cm-line").filter({hasText:"message = os.environ"});
  await expect(sourceLine).toBeVisible();const position=await sourceLine.boundingBox();
  await page.getByRole("button",{name:"Launch profiles for app.py",exact:true}).click();
  const chooser=page.getByRole("dialog",{name:"Launch profiles"});await expect(chooser).toBeVisible();
  const after=await sourceLine.boundingBox();expect(after?.y).toBe(position?.y);expect(after?.height).toBe(position?.height);
  await expect(chooser).toContainText(".hick-launch.json");
  await chooser.getByRole("button",{name:"Run Greeting with Ada",exact:true}).click();
  await expect(chooser).toHaveCount(0);
  const session=await expect.poll(async()=>{
    const list=await (await page.request.get("/api/terminals")).json();
    return list.sessions.find((s:{title:string})=>s.title==="Run Greeting with Ada")?.exit_code;
  }).toBe(0);
  void session;
  const list=await (await page.request.get("/api/terminals")).json();
  const terminal=list.sessions.find((s:{title:string})=>s.title==="Run Greeting with Ada");
  const output=await page.evaluate(async id=>{
    return await new Promise<string>((resolve,reject)=>{
      const ws=new WebSocket(`${location.protocol==="https:"?"wss":"ws"}://${location.host}/api/terminals/ws?session=${id}`);
      ws.binaryType="arraybuffer";let text="";
      ws.onmessage=event=>{text+=new TextDecoder().decode(event.data);if(text.includes("Welcome Ada Lovelace")){ws.close();resolve(text);}};
      ws.onerror=()=>reject(new Error("Terminal socket failed"));setTimeout(()=>{ws.close();resolve(text);},5000);
    });
  },terminal.id);
  expect(output).toContain("profile build");expect(output).toContain("Welcome Ada Lovelace $(touch unwanted)");
  await page.screenshot({path:"/tmp/hickory-launch-profile-manual.png",fullPage:true});
});

test("runs and steps inside both recorded message sides in Hickory Docs", async ({ page }) => {
  test.setTimeout(90_000);
  const fixture = process.env.HICKORY_RECORDED_E2E_ROOT;
  test.skip(!fixture || !process.env.HICKORY_E2E_URL, "Use the isolated recorded-fence replay with local debugpy.");
  const recording = join(fixture!, "sessions", "agent.md");
  const original = await readFile(recording);
  const history = await page.request.get("/api/docs/workspace/agent/turns");
  expect(history.ok()).toBe(true);
  const { turns: [turn] } = await history.json() as { turns: { id: string; prompt: string; answer: string }[] };
  expect(turn.id).toBe("main-recorded-debug");

  await page.goto("/");
  await expect(page.getByRole("tab", { name: "Untitled", exact: true })).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  const agent = page.getByRole("region", { name: "Agent chat" });
  const composer = agent.getByRole("textbox", { name: "Agent conversation and response" });
  await composer.press("ControlOrMeta+End");
  await composer.pressSequentially("Retained response draft");
  await expect(composer).toContainText("Retained response draft");

  for (const [role, side, value] of [["user", "prompt", 41], ["assistant", "answer", 52]] as const) {
    const card = agent.locator(`.conversation-card--${role}`).filter({ hasText: `value = ${value}` });
    const editor = card.locator(".cm-content");
    await expect(editor).toBeVisible();
    const held = await editor.elementHandle();
    const sha = createHash("sha256").update(turn[side]).digest("hex");
    const address = { side: role, source_hash: sha, opening_line: 2 };
    const path = `/api/docs/workspace/agent/turns/${turn.id}/fence-run`;
    const terminals = async () => (await (await page.request.get("/api/terminals")).json()).sessions.map((session: {id: string}) => session.id);
    const previous = await terminals();
    const stale = await page.request.post(path, { data: { ...address, source_hash: "0".repeat(64) } });
    expect(stale.status()).toBe(409);
    expect(await terminals()).toEqual(previous);
    const menu = card.getByLabel("Fence actions on line 3", { exact: true });
    await menu.click();
    const [run] = await Promise.all([
      page.waitForResponse(reply => reply.request().method() === "POST" && reply.url().endsWith(path)),
      card.getByRole("button", { name: "Run python fence on line 3", exact: true }).click(),
    ]);
    expect(run.ok(), await run.text()).toBe(true);
    expect(run.request().postDataJSON()).toEqual(address);
    const output = card.getByRole("region", { name: "Recorded fence run" });
    await expect(output).toContainText("Process exit 0");
    await expect(output).toContainText(String(value + 1));
    await expect(output).not.toContainText("wrong");
    expect(await readFile(recording)).toEqual(original);
    await menu.click();
    await card.getByLabel(/^Set breakpoint on line /).first().click();
    await menu.click();
    const [response] = await Promise.all([
      page.waitForResponse(reply => reply.request().method() === "POST" && reply.url().endsWith("/fence-debug"), { timeout: 15_000 }),
      card.getByRole("button", { name: "Debug python fence on line 3", exact: true }).click({ timeout: 10_000 }),
    ]);
    expect(response.ok(), await response.text()).toBe(true);
    expect(response.request().postDataJSON()).toEqual(address);
    const paused = card.locator(".cm-paused-line");
    await expect(paused).toContainText(`value = ${value}`, { timeout: 25_000 });
    await card.getByRole("button", { name: "Step over", exact: true }).click();
    await expect(paused).toContainText("print(value + 1)");
    const options = card.getByLabel("Debug options", { exact: true });
    await options.click();
    page.once("dialog", dialog => dialog.accept("value"));
    await card.getByRole("button", { name: "+ watch", exact: true }).click();
    await expect(card.getByLabel("Value of watch value", { exact: true })).toHaveText(String(value));
    await card.getByRole("button", { name: "Continue", exact: true }).click();
    await expect(card.locator(".debug-strip__status")).toContainText("exit code 0");
    expect(await editor.evaluate((current, originalEditor) => current === originalEditor, held)).toBe(true);
    await expect(composer).toContainText("Retained response draft");
    expect(await readFile(recording)).toEqual(original);
  }
});

test("opens both recorded patch reviews with original workspace authority", async ({ page }) => {
  const fixture = process.env.HICKORY_RECORDED_PATCH_ROOT;
  test.skip(!fixture || !process.env.HICKORY_E2E_URL, "Use the isolated recorded-patch replay.");
  const paths = ["author.md", "review.txt", "generated.py", "sessions/agent.md"];
  const original = await Promise.all(paths.map(path => readFile(join(fixture!, path))));
  const listing = await page.request.get("/api/files");
  expect(listing.ok()).toBe(true);
  const { root_path } = await listing.json() as { root_path: string };
  const guarded = await page.request.get("/api/file?path=review.txt");
  expect(guarded.ok()).toBe(true);
  const { save_guard } = await guarded.json() as { save_guard: string };
  expect(save_guard).toBeTruthy();

  await page.goto(`/#/file/${encodeURIComponent(`${root_path}/review.txt`)}`);
  const file = page.locator(".plain-file-pane .cm-content").filter({ hasText: "saved note" });
  await expect(file).toBeVisible();
  const held = await file.elementHandle();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  const agent = page.getByRole("region", { name: "Agent chat" });
  const composer = agent.getByRole("textbox", { name: "Agent conversation and response" });
  await composer.press("ControlOrMeta+End");
  await composer.pressSequentially("Retained composer");
  await expect(composer).toContainText("Retained composer");
  for (const side of ["user", "assistant"] as const) {
    const card = agent.locator(`.conversation-card--${side}`);
    const review = card.getByRole("button", { name: "Review patch fence on line 3", exact: true });
    const [reply] = await Promise.all([
      page.waitForResponse(response => response.request().method() === "POST" && response.url().endsWith("/fence-patch")),
      review.click(),
    ]);
    expect(reply.ok(), await reply.text()).toBe(true);
    const admitted = reply.request().postDataJSON();
    expect(admitted.fence).toMatchObject({ side, opening_line: 2 });
    expect(admitted.intention.editor_snapshot.buffers).toContainEqual(expect.objectContaining({
      path: "review.txt", save_guard, source_hash: createHash("sha256").update(original[1]).digest("hex"),
    }));
    const panel = page.getByRole("dialog", { name: "Recorded patch review" });
    await expect(panel).toBeVisible();
    await expect(panel).toContainText(`recorded ${side} fence`);
    await expect(panel).toContainText(side === "user" ? "value = 41" : "assistant draft");
    if (side === "user") {
      await panel.getByRole("combobox", { name: "Proposal construction layer" }).selectOption("draft");
      await expect(panel).toContainText("Generated output: generated.py");
    }
    if (side === "user") await expect(panel.getByRole("button", { name: "Apply complete proposal to drafts" })).toHaveCount(0);
    await panel.getByRole("button", { name: "Close" }).click();
    await expect(review).toBeFocused();
    await expect(composer).toContainText("Retained composer");
    expect(await Promise.all(paths.map(path => readFile(join(fixture!, path))))).toEqual(original);
  }
  await page.getByRole("tab", { name: "review.txt", exact: true }).click();
  await expect(file).toBeVisible();
  expect(await file.evaluate((current, originalEditor) => current === originalEditor, held)).toBe(true);
});

for (const side of ["user", "assistant"] as const) test(`applies one reviewed ${side} patch to managed and plain drafts as a group`, async ({ page }) => {
  const root = process.env.HICKORY_RECORDED_PATCH_ROOT;
  const fixtureSide = process.env.HICKORY_RECORDED_PATCH_MULTI === "1" ? "user" : process.env.HICKORY_RECORDED_PATCH_MULTI;
  test.skip(!root || fixtureSide !== side, "Use the isolated two-target recorded patch replay for this message side.");
  const note = `${side} draft`;
  const value = `value = ${side === "user" ? 41 : 53}`;
  const paths = ["author.md", "review.txt", "generated.py", "sessions/agent.md"];
  const original = await Promise.all(paths.map(path => readFile(join(root!, path))));
  const response = await page.request.get("/api/files");
  expect(response.ok()).toBe(true);
  const { root_path, tree } = await response.json() as {root_path: string; tree: {path: string; doc_id?: string}[]};
  const documentId = tree.find(file => file.path === "author.md")?.doc_id;
  expect(documentId).toBeTruthy();
  const writes: string[] = [];
  let roomUpdates = 0;
  page.on("request", request => {
    const path = new URL(request.url()).pathname;
    if (request.method() === "PUT" && /^\/api\/(file|docs\/|source-history\/)/.test(path)) writes.push(path);
  });
  await page.routeWebSocket(url => url.pathname === "/api/ws" && url.searchParams.get("doc") === `doc:${documentId}`, room => {
    const server = room.connectToServer();
    room.onMessage(message => {
      if (typeof message !== "string" && message[0] === 0 && message[1] === 0 && message[2] === 2) roomUpdates++;
      server.send(message);
    });
    server.onMessage(message => room.send(message));
  });
  await page.goto(`/?startupPath=review.txt#/file/${encodeURIComponent(join(root_path, "review.txt"))}`);
  const plain = page.locator(".plain-file-pane .cm-content");
  await expect(plain).toContainText("saved note");
  await expect(plain).toHaveAttribute("contenteditable", "true");
  const plainEditor = await plain.elementHandle();
  await page.getByRole("treeitem", { name: "author.md", exact: true }).click();
  const managed = page.locator('[data-shell-target="author.md"] .cm-content, [data-shell-target$="/author.md"] .cm-content');
  await expect(managed).toBeVisible();
  const managedEditor = await managed.elementHandle();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  const review = page.getByRole("region", { name: "Agent chat" }).locator(`.conversation-card--${side}`)
    .getByRole("button", { name: "Review patch fence on line 3", exact: true });
  const [reviewed] = await Promise.all([
    page.waitForResponse(reply => reply.request().method() === "POST" && reply.url().endsWith("/fence-patch")),
    review.click(),
  ]);
  expect(reviewed.ok(), await reviewed.text()).toBe(true);
  expect(reviewed.request().postDataJSON().fence).toMatchObject({ side, opening_line: 2 });
  const panel = page.getByRole("dialog", { name: "Recorded patch review" });
  await panel.getByRole("button", { name: "Apply complete proposal to drafts" }).click();
  await expect(panel).toContainText("Complete proposal applied to drafts");
  await expect(plain).toContainText(note);
  await expect(managed).toContainText(value);
  await panel.getByRole("button", { name: "Undo workspace draft Apply" }).click();
  await expect(plain).toContainText("saved note");
  await expect(managed).toContainText("value = 1");
  await panel.getByRole("button", { name: "Redo workspace draft Apply" }).click();
  await expect(plain).toContainText(note);
  await expect(managed).toContainText(value);
  expect(await plain.evaluate((current, originalEditor) => current === originalEditor, plainEditor)).toBe(true);
  expect(await managed.evaluate((current, originalEditor) => current === originalEditor, managedEditor)).toBe(true);
  // Give the ordinary plain publication debounce time to prove it stayed off.
  await page.waitForTimeout(800);
  expect(writes).toEqual([]);
  expect(roomUpdates).toBe(0);
  expect(await Promise.all(paths.map(path => readFile(join(root!, path))))).toEqual(original);

  await panel.getByRole("button", { name: "Close" }).click();
  await page.locator(".spatial-center-tabs").getByRole("tab", { name: "author.md", exact: true }).click();
  await page.evaluate(() => window.addEventListener("hickory:flush-saves", event => {
    document.body.dataset.draftSaveBroadcast = String((event as CustomEvent).detail);
  }));
  const menu = (action: string) => page.evaluate(detail => window.dispatchEvent(new CustomEvent("hickory-menu", {detail})), action);
  const [savedDocument, focusedSaveReceipt] = await Promise.all([
    page.waitForResponse(reply => reply.request().method() === "PUT" && reply.url().endsWith(`/api/source-history/docs/${documentId}`)),
    page.waitForResponse(reply => {
      const request = reply.request();
      if (request.method() !== "POST" || !/\/api\/v1\/ui\/windows\/[^/]+\/receipts$/.test(reply.url())) return false;
      const receipt = request.postDataJSON();
      return receipt.stage === "result" && receipt.action?.label === "Save focused buffer";
    }),
    menu("save"),
  ]);
  expect(savedDocument.ok(), await savedDocument.text()).toBe(true);
  expect(focusedSaveReceipt.ok(), await focusedSaveReceipt.text()).toBe(true);
  expect(focusedSaveReceipt.request().postDataJSON().result).toMatchObject({status: "result", outcome: {status: "completed"}});
  await expect(page.locator("body")).toHaveAttribute("data-draft-save-broadcast", "false");
  expect(await readFile(join(root!, "review.txt"))).toEqual(original[1]);
  expect(writes.some(path => path.endsWith("/source-history/file"))).toBe(false);
  const [savedFile] = await Promise.all([
    page.waitForResponse(reply => reply.request().method() === "PUT" && reply.url().includes("/api/source-history/file")),
    menu("save-all"),
  ]);
  expect(savedFile.ok(), await savedFile.text()).toBe(true);
  expect(await readFile(join(root!, "author.md"), "utf8")).toContain(value);
  expect(await readFile(join(root!, "review.txt"), "utf8")).toBe(`${note}\n`);
  expect(await readFile(join(root!, "generated.py"))).toEqual(original[2]);
  expect(await readFile(join(root!, "sessions/agent.md"))).toEqual(original[3]);
});

for (const simple of [false, true]) test(`inspects the local transition of a recorded declared patch in ${simple ? "Simple Browser" : "Hickory Docs"}`, async ({ page }) => {
  const root = process.env.HICKORY_RECORDED_CONSTRUCTION_ROOT;
  test.skip(!root || !process.env.HICKORY_E2E_URL, "Use the isolated recorded construction replay.");
  const side = process.env.HICKORY_RECORDED_CONSTRUCTION_E2E === "user" ? "user" : "assistant";
  const document = side === "user" ? "request.md" : "response.md";
  const kept = [join(root!, "sessions/agent.md"), join(root!, "materialized.txt")];
  const original = await Promise.all(kept.map(path => readFile(path)));
  const history = await page.request.get("/api/docs/workspace/agent/turns");
  expect(history.ok()).toBe(true);
  const { turns: [turn] } = await history.json() as { turns: { id: string; prompt: string; answer: string }[] };
  const [source, adjacent] = side === "user" ? [turn.prompt, turn.answer] : [turn.answer, turn.prompt];
  expect(source).toContain('<hick:patch id="selected">');
  expect(adjacent).not.toContain('<hick:patch id="selected">');

  await page.goto(simple ? "/simple.html" : "/");
  if (simple) {
    await page.locator("nav").getByText("View", { exact: true }).click();
    await page.getByRole("button", { name: "Agent", exact: true }).click();
  } else {
    await expect(page.getByRole("tab", { name: "Untitled", exact: true })).toBeVisible();
    await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  }
  const messages = page.getByRole("log", { name: "Conversation" }).locator(".agent-message");
  const card = simple ? messages.nth(side === "user" ? 0 : 1)
    : page.getByRole("region", { name: "Agent chat" }).locator(`.conversation-card--${side}`);
  const summary = card.getByLabel("Review construction patch on line 3", { exact: true });
  const [result] = await Promise.all([
    page.waitForResponse(response => response.url().endsWith("/fence-construction") && response.request().method() === "POST"),
    summary.click(),
  ]);
  expect(result.ok(), await result.text()).toBe(true);
  const address = { side, opening_line: 2, source_hash: createHash("sha256").update(source).digest("hex") };
  expect(result.request().postDataJSON()).toEqual(address);
  expect(await result.json()).toMatchObject({ ...address, turn: turn.id, document, operation_id: "selected", read_only: true });
  const reading = card.getByRole("region", { name: "Recorded construction patch" });
  await reading.getByText("materialized.txt", { exact: true }).click();
  await expect(reading).toContainText("old");
  await expect(reading).toContainText("middle");
  await expect(reading).not.toContainText("final");
  await expect(reading.getByRole("button", { name: /apply|save|run/i })).toHaveCount(0);
  expect(await Promise.all(kept.map(path => readFile(path)))).toEqual(original);
});
test("keeps the managed startup editor through guarded open and first collaboration sync", async ({ page }) => {
  test.skip(!process.env.HICKORY_E2E_URL, "Run against the isolated literate editor engine.");
  const path = "managed startup #é.md", text = "Managed startup text";
  const files = await (await page.request.get("/api/files")).json() as {root_path: string; tree: {path: string; doc_id?: string}[]};
  const docId = files.tree.find(file => file.path === path)?.doc_id;
  expect(docId).toBeTruthy();

  let releaseOpen!: () => void, releaseFile!: () => void, releaseParser!: () => void;
  const waitingOpen = new Promise<void>(resolve => {releaseOpen = resolve;});
  const waitingFile = new Promise<void>(resolve => {releaseFile = resolve;});
  const waitingParser = new Promise<void>(resolve => {releaseParser = resolve;});
  let askedOpen = false, askedFile = false, askedRoom = false, roomSyncHeld = false, askedParser = false, allowSync = false;
  let clientSourceUpdates = 0;
  const heldRoom: (() => void)[] = [];
  await page.addInitScript(() => localStorage.setItem("hickory.debug", "1"));
  await page.route(url => /\/hick_lang_bg[^/]*\.wasm$/.test(url.pathname), async request => {
    if (request.request().resourceType() === "script") return request.continue();
    askedParser = true;
    await waitingParser;
    await request.continue();
  });
  await page.route("**/api/open-file", async request => {
    askedOpen = true;
    await waitingOpen;
    await request.continue();
  });
  await page.route(`**/api/docs/${docId}`, async request => {
    if (request.request().method() !== "GET") return request.continue();
    askedFile = true;
    await waitingFile;
    await request.continue();
  });
  await page.routeWebSocket(url => url.pathname === "/api/ws" && url.searchParams.get("doc") === `doc:${docId}`, ws => {
    askedRoom = true;
    const server = ws.connectToServer();
    ws.onMessage(message => {
      if (typeof message !== "string" && message[0] === 0 && message[1] === 0 && message[2] === 2) clientSourceUpdates++;
      server.send(message);
    });
    server.onMessage(message => {
      const isSourceSync = typeof message !== "string" && message[0] === 0 && message[1] === 0;
      if (!allowSync && isSourceSync) {
        if (message[2] === 1) roomSyncHeld = true;
        heldRoom.push(() => ws.send(message));
      } else ws.send(message);
    });
  });

  function releaseRoom() {
    allowSync = true;
    for (const send of heldRoom.splice(0)) send();
  }
  try {
    await page.goto(`/?startupPath=${encodeURIComponent(path)}#/file/${encodeURIComponent(join(files.root_path, path))}`);
    const editor = page.locator(".document-editor .cm-content");
    await expect(editor).toBeVisible();
    await expect(editor).toContainText(text);
    await expect(editor).toHaveAttribute("contenteditable", "false");
    await expect.poll(() => askedOpen && askedParser).toBe(true);
    expect(askedFile || askedRoom).toBe(false);
    const held = await editor.elementHandle();
    const initialSelection = await page.evaluate(() => {
      const view = (window as unknown as {__hickoryView: {dispatch: (value: unknown) => void;
        state: {doc: {line: (line: number) => {from: number}}}; scrollDOM: HTMLElement}}).__hickoryView;
      const anchor = view.state.doc.line(82).from + 2, head = anchor + 8;
      view.dispatch({selection: {anchor, head}});
      view.scrollDOM.scrollTop = 1200;
      return [anchor, head];
    });
    const viewport = () => editor.evaluate(node => {
      const scroller = node.closest(".cm-scroller")!;
      const top = scroller.getBoundingClientRect().top;
      const line = [...node.querySelectorAll(".cm-line")].find(row => row.getBoundingClientRect().bottom > top + 8);
      return {scroll: scroller.scrollTop, line: line?.textContent};
    });
    await expect.poll(async () => (await viewport()).scroll).toBeGreaterThan(500);
    const initialViewport = await viewport();
    expect(initialViewport.line).toContain("Managed startup row");
    releaseOpen();
    await expect.poll(() => askedFile).toBe(true);
    await expect(editor).toHaveAttribute("contenteditable", "false");
    expect(askedRoom).toBe(false);
    releaseFile();
    await expect.poll(() => roomSyncHeld).toBe(true);
    await expect(editor).toHaveAttribute("contenteditable", "false");
    expect(clientSourceUpdates).toBe(0);
    releaseRoom();
    await expect(page.getByText("Waiting for the document room…", {exact: true})).toHaveCount(0);
    await expect(editor).toHaveAttribute("contenteditable", "false");
    expect(clientSourceUpdates).toBe(0);
    releaseParser();
    await expect(editor).toHaveAttribute("contenteditable", "true");
    expect(await editor.evaluate((current, original) => current === original, held)).toBe(true);
    const selection = await page.evaluate(() => {
      const view = (window as unknown as {__hickoryView: {state: {selection: {main: {anchor: number; head: number}}}}}).__hickoryView;
      return [view.state.selection.main.anchor, view.state.selection.main.head];
    });
    expect(selection).toEqual(initialSelection);
    await expect.poll(viewport).toEqual(initialViewport);
    expect(clientSourceUpdates).toBe(0);
    const editedLine = () => page.evaluate(() => {
      const view = (window as unknown as {__hickoryView: {state: {doc: {line: (line: number) => {text: string}}}}}).__hickoryView;
      return view.state.doc.line(82).text;
    });
    const before = await editedLine();
    await editor.press("End");
    await editor.pressSequentially(" draft");
    await expect.poll(editedLine).toBe(before + " draft");
    await expect.poll(() => clientSourceUpdates).toBeGreaterThan(0);
    await editor.press("ControlOrMeta+z");
    await expect.poll(editedLine).toBe(before);
  } finally {
    releaseOpen();
    releaseFile();
    releaseRoom();
    releaseParser();
  }
});
test("Simple Browser keeps a newer tab choice during a slow file open and search reveal", async ({page}) => {
  test.skip(!process.env.HICKORY_E2E_URL, "Run against the isolated literate editor engine.");
  const listing = await page.request.get("/api/files");
  expect(listing.ok()).toBe(true);
  const {tree} = await listing.json() as {tree: {path: string; doc_id?: string}[]};
  const selectionTarget = "managed startup #é.md";
  const targetId = tree.find(file => file.path === selectionTarget)?.doc_id;
  expect(targetId).toBeTruthy();

  await page.goto("/simple.html");
  const treeView = page.getByLabel("Files", {exact: true});
  const editor = (path: string) => page.locator(".cm-content").and(page.getByLabel(path, {exact: true}));
  const original = editor("code.py");
  const originalTab = page.getByRole("tab", {name: /^code\.py/});
  const readState = async (path: string) => editor(path).evaluate(async node => {
    const viewModule = "/node_modules/@codemirror/view/dist/index.js";
    const sourceModule = "/src/simple/SourceText.ts";
    const [{EditorView}, {authoredSource, authoredSelection}] = await Promise.all([
      import(viewModule), import(sourceModule),
    ]);
    const view = EditorView.findFromDOM(node);
    return {source: authoredSource(view.state), selection: authoredSelection(view.state)[0]};
  });

  await treeView.getByRole("button", {name: "code.py", exact: true}).click();
  await expect(original).toBeVisible();
  const originalNode = await original.elementHandle();
  await original.press("End");
  await page.keyboard.insertText(" # retained draft");
  await expect(original).toContainText("retained draft");
  await original.press("Shift+Home");
  const before = await readState("code.py");
  expect(before.selection.anchor).not.toBe(before.selection.head);

  async function reselectDuringRead(pathMatches: (url: URL) => boolean, activate: () => Promise<void>, unopened: string) {
    let release!: () => void;
    const gate = new Promise<void>(resolve => {release = resolve;});
    const read = (url: URL) => pathMatches(url);
    await page.route(read, async route => {
      await gate;
      await route.continue();
    });
    try {
      const requested = page.waitForRequest(request => request.method() === "GET" && read(new URL(request.url())));
      await activate();
      await requested;
      await expect(originalTab).toHaveAttribute("aria-selected", "true");
      await originalTab.click();
      const response = page.waitForResponse(reply => reply.request().method() === "GET" && read(new URL(reply.url())));
      release();
      const reply = await response;
      expect(reply.ok()).toBe(true);
      await reply.finished();
      await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
      await expect(page.getByRole("tab", {name: unopened, exact: true})).toHaveCount(0);
      await expect(originalTab).toHaveAttribute("aria-selected", "true");
      expect(await original.evaluate((node, retained) => node === retained, originalNode)).toBe(true);
      expect(await readState("code.py")).toEqual(before);
    } finally {
      release();
      await page.unroute(read);
    }
  }

  await reselectDuringRead(
    url => url.pathname === "/api/file" && url.searchParams.get("path") === "revision.txt",
    () => treeView.getByRole("button", {name: "revision.txt", exact: true}).click(), "revision.txt",
  );

  await page.locator("nav").getByText("Edit", {exact: true}).click();
  await expect(page.getByRole("button", {name: "Undo text edit", exact: true})).toBeEnabled();
  await expect(page.getByRole("button", {name: "Navigate back", exact: true})).toBeDisabled();
  await page.getByRole("button", {name: "Find in workspace", exact: true}).click();
  const search = page.getByRole("dialog", {name: "Find in workspace"});
  await search.getByRole("textbox", {name: "Workspace search query"}).fill("Managed startup row 123");
  await search.getByRole("button", {name: "Search", exact: true}).click();
  const match = search.getByRole("button", {name: /managed startup #é\.md:\d+ · Managed startup row 123/});
  await expect(match).toBeVisible();
  await reselectDuringRead(
    url => decodeURIComponent(url.pathname) === `/api/docs/${targetId}`,
    () => match.click(), selectionTarget,
  );
  await expect(search).toBeVisible();
  await search.getByRole("button", {name: "Close workspace search"}).click();

  await treeView.getByRole("button", {name: selectionTarget, exact: true}).click();
  await expect(editor(selectionTarget)).toBeVisible();
  const fresh = await readState(selectionTarget);
  expect(fresh.selection).toMatchObject({anchor: 0, head: 0});
  await originalTab.click();
  await page.evaluate(({path, source}) => {
    window.dispatchEvent(new CustomEvent("hickory:open-location", {
      detail: {path, line: 124, character: 4, expectedSource: source},
    }));
  }, {path: selectionTarget, source: fresh.source});
  await expect(editor(selectionTarget)).toBeVisible();
  const expected = fresh.source.indexOf("Managed startup row 123") + 4;
  expect(expected).toBeGreaterThan(4);
  await expect.poll(async () => (await readState(selectionTarget)).selection).toMatchObject({anchor: expected, head: expected});
  await originalTab.click();
  await original.press("ControlOrMeta+z");
  await expect(original).not.toContainText("retained draft");
  const saved = await page.request.get("/api/file?path=code.py");
  expect((await saved.json()).content).toBe("value = 1\n");
});
