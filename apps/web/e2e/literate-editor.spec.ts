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
  await pane.getByRole("button",{name:"Save code and view"}).click();
  await expect.poll(async()=> (await (await page.request.get("/api/file?path=code.py")).json()).content).toBe("value = 7\n");
  await pane.getByLabel("Comparison base").fill("HEAD");
  await pane.getByRole("button",{name:"Show diff",exact:true}).click();
  await expect(pane.locator(".comparison-removed")).toHaveCount(1);
  await pane.locator(".comparison-removed summary").click();
  await expect(pane.locator(".comparison-removed pre")).toContainText("value = 1");
  await expect(pane.locator(".comparison-added")).toContainText("value = 7");
  const editedLine=pane.locator(".cm-line").filter({hasText:"value = 7"});
  await editedLine.click(); await content.press("Home"); await content.press("Shift+End"); await content.pressSequentially("value = 8");
  await pane.getByRole("button",{name:"Save code and view"}).click();
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
  await page.getByRole("button", { name: "Agent settings", exact: true }).click();
  await expect(agent.getByRole("button", { name: "Send", exact: true })).toBeDisabled();
  const response = agent.getByRole("textbox", { name: "Agent conversation and response" });
  await response.click(); await response.press("ControlOrMeta+End"); await response.pressSequentially("buffer-edit");
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
  // The completed answer is protected; selecting all and typing cannot erase it.
  await response.press("ControlOrMeta+a"); await response.pressSequentially("Should not replace history");
  await expect(response).toContainText("buffer-edit");
  await response.press("ControlOrMeta+End"); await response.press("Enter"); await response.pressSequentially("Next response draft");
  await expect(response).toContainText("Next response draft");
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
