import { expect, test } from "@playwright/test";

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
  await expect(page.locator(".shell-tabs").first()).toBeVisible();
  await page.evaluate(id=>window.dispatchEvent(new CustomEvent("hickory.open-representation",{detail:{id}})),view.id);
  const pane=page.getByRole("region",{name:"Literate editor"});
  await expect(pane).toBeVisible();
  const line=pane.locator(".cm-line").filter({hasText:"value = 1"});
  await expect(line).toBeVisible(); await line.click();
  const content=pane.locator(".cm-content").first();
  await content.press("Home"); await content.press("Shift+End"); await content.pressSequentially("value = 7");
  await pane.getByRole("button",{name:"Save",exact:true}).click();
  await expect.poll(async()=> (await (await page.request.get("/api/file?path=code.py")).json()).content).toBe("value = 7\n");
  await pane.getByLabel("Comparison base").fill("HEAD");
  await pane.getByRole("button",{name:"Show diff",exact:true}).click();
  await expect(pane.locator(".comparison-removed")).toHaveCount(1);
  await pane.locator(".comparison-removed summary").click();
  await expect(pane.locator(".comparison-removed pre")).toContainText("value = 1");
  await expect(pane.locator(".comparison-added")).toContainText("value = 7");
  const editedLine=pane.locator(".cm-line").filter({hasText:"value = 7"});
  await editedLine.click(); await content.press("Home"); await content.press("Shift+End"); await content.pressSequentially("value = 8");
  await pane.getByRole("button",{name:"Save",exact:true}).click();
  await expect.poll(async()=> (await (await page.request.get("/api/file?path=code.py")).json()).content).toBe("value = 8\n");
  await pane.getByLabel("Comparison target").fill("HEAD"); await pane.getByRole("button",{name:"Show diff",exact:true}).click();
  await expect(content).toHaveAttribute("contenteditable","false");
  await expect(pane).toContainText("historical, read-only");
  expect((await (await page.request.get("/api/file?path=code.py")).json()).content).toBe("value = 8\n");
  await pane.getByRole("button",{name:"Show current state"}).click();
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
  await expect(review.locator(".comparison-removed pre")).toContainText("Original 📝");
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
  const document = page.locator(".debug-block").filter({ has: page.getByRole("toolbar", { name: "Actions for coherence.md", exact: true }) });
  const editor = document.locator(".cm-content");
  await expect(editor).toContainText("Before");
  await editor.click(); await editor.press("ControlOrMeta+a"); await editor.pressSequentially("# After automatic save");
  await expect.poll(async () => (await (await page.request.get(`/api/docs/${note.id}`)).json()).source).toContain("After automatic save");
  await expect(document.locator(".save-state")).toHaveText("Saved to disk");
  await expect(page.getByRole("tab", { name: /coherence.md/ })).not.toContainText("*");
  await page.route(`**/api/docs/${note.id}/run`, route => route.fulfill({ status: 503, contentType: "application/json", body: JSON.stringify({ error: "temporary runner failure" }) }));
  await page.getByRole("button", { name: "Run", exact: true }).click();
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
  const row = page.locator('[data-tree-path="code.py"]');
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
