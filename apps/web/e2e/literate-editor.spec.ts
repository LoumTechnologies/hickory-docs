import { expect, test } from "@playwright/test";

// Exercises the real catalog installer in a disposable project. Opt in because
// a fresh machine needs uv and network access to obtain debugpy.
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
  // CodeMirror virtualizes offscreen history: reveal the opening turn before inspecting it.
  await response.press("ControlOrMeta+Home");
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
  await expect(response).toContainText('review-note.md');
  // The completed answer is protected; selecting all and typing cannot erase it.
  await response.press("ControlOrMeta+a"); await response.pressSequentially("Should not replace history");
  await expect(response).toContainText("buffer-edit");
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
  await page.goto("/");
  await expect(page.locator(".untitled-tab .cm-editor")).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  const agent = page.getByRole("region", { name: "Agent chat" });
  await agent.getByRole("button", { name: "Agent settings" }).click();
  await page.getByRole("combobox", { name: "Agent", exact: true }).selectOption("fixture");
  await page.getByRole("combobox", { name: "Agent", exact: true }).press("Escape");
  const response = agent.getByRole("textbox", { name: "Agent conversation and response" });
  await response.click(); await response.press("ControlOrMeta+End"); await response.pressSequentially("untitled-paragraph");
  await agent.getByRole("button", { name: "Send", exact: true }).click();
  await expect(response).toContainText("The second paragraph has the most characters.");
  // Context remains folded; its connection must originate from the answer itself.
  const context = agent.locator(".cm-line").filter({ hasText: '<hick:context kind="editor-buffers">' });
  await expect(context.locator(".cm-hick-fold-placeholder")).toBeVisible();
  await agent.locator(".cm-line").filter({ hasText: "The second paragraph has the most characters." }).scrollIntoViewIfNeeded();
  const ribbon = page.locator(".ribbon-group--context").filter({ has: page.locator("title", { hasText: "editor snapshot of Untitled" }) });
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
  await expect(page.locator(".untitled-tab")).toContainText("Hickory");
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
    await expect(response).toContainText("See");
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
