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
  const content=pane.locator(".cm-content");
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
