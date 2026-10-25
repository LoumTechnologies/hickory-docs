<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/e2e/workspace-changesets.spec.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Changeset completion is negotiated and checked](../../../../../guarantees/agent/changeset-completion-is-negotiated-and-checked/contract.md)
- [A final changeset revision applies B to C](../../../../../guarantees/agent/final-changeset-revisions-apply-b-to-c/contract.md)

Captured SHA-256: `51f31db0a75834b23c0bffecbeeacc15400d72c58ce3f923ba2465e997991fcc`.

<book:entry path="apps/web/e2e/workspace-changesets.spec.ts">
// Guarantee: docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md
import { expect, test } from "@playwright/test";

// Real browser + HTTP + ACP subprocess: authoritative untitled bytes and repair.
test("untitled edits complete as a captured changeset without saving", async ({ page }) => {
  test.setTimeout(120_000);
  test.skip(!process.env.HICKORY_CHANGESET_BROWSER, "Use just test-workspace-changesets-browser.");
  const errors: string[] = [];
  page.on("pageerror", e => errors.push(e.message));
  await page.goto("/");
  const editor = page.locator(".untitled-tab .cm-content");
  await expect(editor).toBeVisible({ timeout: 60_000 });
  await editor.click(); await editor.press("ControlOrMeta+a");
  await editor.pressSequentially("Authoritative unsaved A");
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  const agent = page.getByRole("region", { name: "Agent chat" });
  await agent.getByRole("button", { name: "Agent settings" }).click();
  await page.getByRole("combobox", { name: "Agent", exact: true }).selectOption("fixture");
  await page.getByRole("combobox", { name: "Agent", exact: true }).press("Escape");
  const response = agent.getByRole("textbox", { name: "Agent conversation and response" });
  await response.click(); await response.press("ControlOrMeta+End");
  await response.pressSequentially("contract-buffer");
  const sent = page.waitForRequest(r => r.method() === "POST" && /\/api\/docs\/workspace\/agent$/.test(new URL(r.url()).pathname));
  await agent.getByRole("button", { name: "Send", exact: true }).click();
  const request = await sent;
  expect(request.postDataJSON().context.buffers[0].content).toBe("Authoritative unsaved A");
  await agent.getByRole("button", { name: "Accept change", exact: true }).click();
  await expect(agent.getByRole("region", { name: "Final literate changeset" })).toBeVisible({ timeout: 30_000 });
  await expect(agent.getByRole("region", { name: "Final literate changeset" })).toContainText("Edited by the agent");
  await expect(agent.getByRole("button", { name: "Apply to literate source" })).toBeDisabled();
  const turns = await (await page.request.get("/api/docs/workspace/agent/turns")).json();
  const turn = turns.turns.at(-1);
  expect(turn.status).toBe("ok");
  let capture = await (await page.request.get(`/api/docs/workspace/agent/projects/${turn.id}`)).json();
  const values = (snapshot: { entries: Record<string,{bytes:number[]}> }) => Object.entries(snapshot.entries)
    .filter(([path]) => path.startsWith("__hickory_editor_buffers__/"))
    .map(([,entry]) => new TextDecoder().decode(new Uint8Array(entry.bytes)));
  expect(values(capture.starting)).toContain("Authoritative unsaved A");
  expect(values(capture.completed)).toContain("Edited by the agent");
  expect(capture.workspace.root === null).toBe(process.env.HICKORY_CHANGESET_FOLDER === "false");
  expect((await page.request.get("/api/file?path=Untitled.md")).ok()).toBe(false);
  if (capture.ambiguous.length) {
    for (const path of capture.ambiguous) await agent.getByLabel(path,{exact:true}).check();
    await agent.getByRole("button",{name:"Verify A-to-B explanation",exact:true}).click();
    await expect(agent.getByRole("button",{name:"Revise New",exact:true})).toBeEnabled();
    capture = await (await page.request.get(`/api/docs/workspace/agent/projects/${turn.id}`)).json();
  }
  const identity = Object.keys(capture.workspace.identities)[0];
  const changedIdentity = await page.request.post(`/api/docs/workspace/agent/projects/${turn.id}/draft/apply`, {data:{sources:capture.artifact.sources,context:{buffers:[{id:identity,name:"Saved elsewhere",path:"invented.md",content:"Edited by the agent"}]}}});
  expect(changedIdentity.status()).toBe(409); // Refuses the changed frozen identity before publishing.
  // Guarantee: docs/guarantees/agent/final-changeset-revisions-apply-b-to-c.md
  await agent.getByRole("button",{name:"Revise New",exact:true}).click();
  const draft = agent.getByRole("region",{name:"Authored final changeset draft"});
  const revisedCode = draft.locator(".comparison-side:last-child .cm-content");
  await revisedCode.click(); await revisedCode.press("ControlOrMeta+a");
  await revisedCode.pressSequentially("Discarded C");
  await expect(draft.getByText("C construction succeeded.",{exact:false})).toBeVisible();
  await expect(editor).toContainText("Edited by the agent");
  await draft.getByRole("button",{name:"Discard draft"}).click();
  await expect(editor).toContainText("Edited by the agent");
  await agent.getByRole("button",{name:"Revise New",exact:true}).click();
  await revisedCode.click(); await revisedCode.press("ControlOrMeta+a");
  await revisedCode.pressSequentially("Authored C");
  await expect(draft.getByText("C construction succeeded.",{exact:false})).toBeVisible();
  await page.getByRole("tab",{name:"Untitled",exact:true}).click();
  await editor.click(); await editor.press("ControlOrMeta+a");
  await editor.pressSequentially("Concurrent editor work");
  await page.getByRole("tab",{name:"Agent",exact:true}).click();
  await draft.getByRole("button",{name:"Apply revision",exact:true}).click();
  await expect(agent.getByRole("alert")).toContainText("Destinations changed");
  await expect(editor).toContainText("Concurrent editor work");
  await page.getByRole("tab",{name:"Untitled",exact:true}).click();
  await editor.click(); await editor.press("ControlOrMeta+a");
  await editor.pressSequentially("Edited by the agent");
  await page.getByRole("tab",{name:"Agent",exact:true}).click();
  await draft.getByRole("button",{name:"Apply revision",exact:true}).click();
  await expect(agent.getByText(/Revision application succeeded/)).toBeVisible();
  await expect(editor).toContainText("Authored C");
  const after = await (await page.request.get(`/api/docs/workspace/agent/projects/${turn.id}`)).json();
  expect(after).toEqual(capture);
  await page.getByRole("tab",{name:"Untitled",exact:true}).click();
  await editor.click(); await editor.press("ControlOrMeta+z");
  await expect(editor).toContainText("Edited by the agent");
  expect((await page.request.get("/api/file?path=Untitled.md")).ok()).toBe(false);
  expect(errors).toEqual([]);
});
</book:entry>

</book:doc>
