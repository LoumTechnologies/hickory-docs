<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/e2e/python-agent.spec.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c40e3ff7e23b"></a>
## `apps/web/e2e/python-agent.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Conversation edits use the client's review policy](../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md), [Separate model snippets from trusted host operations](../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md).

<a id="f-c40e3ff7e23b-expression_statement"></a>
### `expression_statement`

In a separate disposable confined-worker flow, exercise file selection/removal,
clipboard files and a camera snapshot in a real browser. Chromium supplies its
explicit synthetic camera device; the app still asks for browser permission,
previews the stream and captures real PNG bytes. Check the outgoing payload and
continue through the native host, worker, streamed output and retained model request.
Stop the pending proposal without exercising unrelated review and drawer flows.
A controlled browser speech service supplies interim and final recognition events.
Its status must not move the draft; finalized speech joins ordinary text and files
only after explicit Send. This does not certify a physical microphone or speech
recognition accuracy.
This is transport evidence, not a vision-quality or physical-camera witness.


The browser flow expects the accepted document in retained history after real independent review; equality status is separate from runtime evidence.

<storycode:copy id="f-c40e3ff7e23b-expression_statement">
import { expect, test } from "@playwright/test";

// Guarantees: agent/python-execution-has-two-boundaries.md;
// agent/a-session-is-the-conversation.md; agent/the-agent-pane-is-a-live-document.md.
test("configure the Python agent, review its edit, recover the conversation and stop another proposal", async ({ page }) => {
  test.setTimeout(60_000);
  test.skip(!!process.env.HICKORY_ATTACHMENT_UI_FIXTURE, "Run attachment transport separately in its own disposable fixture.");
  test.skip(!process.env.HICKORY_PYTHON_UI_FIXTURE || !process.env.HICKORY_AGENT_TEST_PYTHON, "Use scripts/test-python-agent-ui.sh with the installed runtime.");
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.goto("/#/settings");
  await page.getByRole("textbox", { name: "Python executable" }).fill(process.env.HICKORY_AGENT_TEST_PYTHON!);
  await page.getByRole("button", { name: "Save Python runtime" }).click();
  await expect(page.getByRole("status").filter({ hasText: "Runtime started in confinement" })).toBeVisible();
  const files = await (await page.request.get("/api/projects/local/docs")).json();
  const note = files.find((doc: { path: string }) => doc.path === "note.md");
  await page.goto(`/#/docs/${note.id}`);
  await expect(page.locator('[data-spatial-key="document:note.md"] .cm-content').filter({ hasText: "Original human input" })).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  const agent = page.getByRole("region", { name: "Agent chat" });
  await agent.getByRole("button", { name: "Agent settings" }).click();
  await page.getByRole("combobox", { name: "Agent", exact: true }).selectOption("python");
  await page.getByRole("combobox", { name: "Agent", exact: true }).press("Escape");
  const response = agent.getByRole("textbox", { name: "Agent conversation and response" });
  await expect(agent.getByRole("region", { name: "Conversation document edits" })).toHaveCount(0);
  await expect(agent.getByRole("button", { name: /Changed agent context/ })).toHaveCount(0);
  await expect(page.locator(".agent-status > summary")).toContainText("Anthropic");
  await page.screenshot({ path: "/tmp/hickory-python-agent-empty.png", fullPage: true });
  await response.click(); await response.press("ControlOrMeta+End"); await response.pressSequentially("Propose in Python");
  const sent = page.waitForRequest(request => request.method() === "POST" && /\/api\/docs\/[^/]+\/agent$/.test(new URL(request.url()).pathname));
  await agent.getByRole("button", { name: "Send", exact: true }).click();
  const turnsPath = `${new URL((await sent).url()).pathname}/turns`;
  const review = page.getByRole("region", { name: "Document change review" });
  await page.getByRole("button", { name: "Show left drawer" }).click();
  await expect(review).toBeVisible();
  await expect(review.locator(".comparison-added")).toContainText("Python agent proposal");
  expect((await (await page.request.get(`/api/docs/${note.id}`)).json()).source).toBe("# Original human input\n");
  await expect(agent.getByRole("checkbox", { name: "Always approve document edits in this conversation" })).toBeVisible();
  await page.screenshot({ path: "/tmp/hickory-python-agent-review.png", fullPage: true });
  await review.getByRole("button", { name: "Accept change" }).click();
  await expect.poll(async () => (await (await page.request.get(`/api/docs/${note.id}`)).json()).source).toBe("# Python agent proposal\n# Original human input\n");
  await expect(agent.getByRole("button", { name: "Stop the agent" })).toHaveCount(0);
  const history = await (await page.request.get(turnsPath)).json();
  expect(history.turns[0].backend).toBe("python");
  expect(history.turns[0].status).toBe("ok");
  expect(history.turns[0].answer).toContain("<hick:doc");
  expect(history.turns[0].answer).toContain("no completed runtime check recorded.");
  await page.reload();
  // Center tabs expose the restored source independently of the review panes.
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: "note.md", exact: true }).click();
  // The menu listener belongs to the asynchronously loaded workspace, not the shell.
  await expect(page.locator('[data-spatial-key="document:note.md"] .cm-content').filter({ hasText: "Original human input" })).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: "Agent", exact: true }).click();
  await agent.getByRole("button", { name: "Agent settings" }).click();
  await expect(page.getByRole("combobox", { name: "Agent", exact: true })).toHaveValue("python");
  await response.click(); await response.press("ControlOrMeta+End"); await response.pressSequentially("/new");
  await agent.getByRole("button", { name: "Send", exact: true }).click();
  await response.click(); await response.press("ControlOrMeta+End"); await response.pressSequentially("Cancel this Python proposal");
  await agent.getByRole("button", { name: "Send", exact: true }).click();
  await page.getByRole("button", { name: "Show left drawer" }).click();
  await expect(review.locator(".comparison-added").filter({ hasText: "This needs approval" })).toBeVisible();
  // Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
  // A completed response must not paint over the next pending review. Trial
  // performs real pointer hit testing without deciding the proposal we cancel.
  await review.getByRole("button", { name: "Reject change" }).click({ trial: true, timeout: 10000 });
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: "Agent", exact: true }).click();
  await agent.getByRole("button", { name: "Stop the agent" }).click();
  await expect.poll(async () => {
    const turns = await (await page.request.get(turnsPath)).json();
    return turns.turns.at(-1).status;
  }).toBe("stopped");
  expect((await page.request.get("/api/file?path=cancelled.md")).ok()).toBe(false);
  expect(errors).toEqual([]);
});

test.use({ launchOptions: { args: process.env.HICKORY_ATTACHMENT_UI_FIXTURE ? ["--use-fake-device-for-media-stream", "--use-fake-ui-for-media-stream"] : [] } });

test.describe("chat attachment transport", () => {
test("chat attachments and dictation reach the confined host only after Send", async ({ page }) => {
  test.setTimeout(60_000);
  test.skip(!process.env.HICKORY_ATTACHMENT_UI_FIXTURE || !process.env.HICKORY_AGENT_TEST_PYTHON, "Use scripts/test-python-agent-ui.sh with the installed runtime.");
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  // Deterministic recognition events exercise draft timing and transport, not
  // vendor speech accuracy. No external speech service is called in this fixture.
  await page.addInitScript(() => {
    class Recognition {
      continuous = false; interimResults = false; lang = "";
      onstart: (() => void) | null = null;
      onend: (() => void) | null = null;
      onerror: ((event: { error: string }) => void) | null = null;
      onresult: ((event: { results: { isFinal: boolean; 0: { transcript: string } }[] }) => void) | null = null;
      result = (event: Event) => {
        const detail = (event as CustomEvent<{ text: string; final: boolean }>).detail;
        this.onresult?.({ results: [{ isFinal: detail.final, 0: { transcript: detail.text } }] });
      };
      end = () => { this.abort(); this.onend?.(); };
      start() { window.addEventListener("fixture-dictation-result", this.result); window.addEventListener("fixture-dictation-end", this.end); this.onstart?.(); }
      stop() { /* Final events are delivered explicitly after the user stops. */ }
      abort() { window.removeEventListener("fixture-dictation-result", this.result); window.removeEventListener("fixture-dictation-end", this.end); }
    }
    Object.defineProperty(window, "SpeechRecognition", { configurable: true, value: Recognition });
  });
  let agentRequests = 0;
  page.on("request", request => { if (request.method() === "POST" && /\/api\/docs\/[^/]+\/agent$/.test(new URL(request.url()).pathname)) agentRequests++; });
  await page.goto("/#/settings");
  await page.getByRole("textbox", { name: "Python executable" }).fill(process.env.HICKORY_AGENT_TEST_PYTHON!);
  await page.getByRole("button", { name: "Save Python runtime" }).click();
  await expect(page.getByRole("status").filter({ hasText: "Runtime started in confinement" })).toBeVisible();
  const files = await (await page.request.get("/api/projects/local/docs")).json();
  const note = files.find((doc: { path: string }) => doc.path === "note.md");
  await page.goto(`/#/docs/${note.id}`);
  await expect(page.locator('[data-spatial-key="document:note.md"] .cm-content').filter({ hasText: "Original human input" })).toBeVisible();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("hickory-menu", { detail: "show-agent" })));
  const agent = page.getByRole("region", { name: "Agent chat" });
  await agent.getByRole("button", { name: "Agent settings" }).click();
  await page.getByRole("combobox", { name: "Agent", exact: true }).selectOption("python");
  await page.getByRole("combobox", { name: "Agent", exact: true }).press("Escape");
  const response = agent.getByRole("textbox", { name: "Agent conversation and response" });
  await expect(agent.getByRole("region", { name: "Conversation document edits" })).toHaveCount(0);
  await expect(agent.getByRole("button", { name: /Changed agent context/ })).toHaveCount(0);
  await expect(page.locator(".agent-status > summary")).toContainText("Anthropic");
  // Real clipboard and camera bytes must reach the native host with the same
  // request as the text. These controls must not require a provider call.
  await agent.getByLabel("Attach files", { exact: true }).setInputFiles({ name: "selected.txt", mimeType: "text/plain", buffer: Buffer.from("selected file") });
  await expect(agent.getByRole("button", { name: "Remove selected.txt" })).toBeVisible();
  await agent.getByRole("button", { name: "Remove selected.txt" }).click();
  await response.evaluate(element => {
    const clipboard = new DataTransfer();
    clipboard.items.add(new File(["clipboard bytes"], "pasted.txt", { type: "text/plain" }));
    const png = Uint8Array.from(atob("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg=="), character => character.charCodeAt(0));
    clipboard.items.add(new File([png], "pasted.png", { type: "image/png" }));
    element.dispatchEvent(new ClipboardEvent("paste", { clipboardData: clipboard, bubbles: true, cancelable: true }));
  });
  await expect(agent.getByRole("button", { name: "Remove pasted.txt" })).toBeVisible();
  await agent.getByRole("button", { name: "Take picture" }).click();
  const camera = page.getByRole("dialog", { name: "Camera" });
  await expect(camera.locator("video")).toBeVisible();
  await expect.poll(() => camera.locator("video").evaluate(video => (video as HTMLVideoElement).videoWidth)).toBeGreaterThan(0);
  const track = await camera.locator("video").evaluateHandle(video => (video as HTMLVideoElement).srcObject instanceof MediaStream ? ((video as HTMLVideoElement).srcObject as MediaStream).getVideoTracks()[0] : null);
  await camera.getByRole("button", { name: "Use photo" }).click();
  await expect.poll(() => track.evaluate(value => value?.readyState)).toBe("ended");
  await expect(camera).toHaveCount(0);
  await expect(agent.getByRole("button", { name: "Remove camera-photo.png" })).toBeVisible();
  await page.screenshot({ path: "/tmp/hickory-python-agent-empty.png", fullPage: true });
  const draftBounds = await response.boundingBox();
  await agent.getByRole("button", { name: "Dictate", exact: true }).click();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("fixture-dictation-result", { detail: { text: "interim words", final: false } })));
  await expect(agent.getByLabel("Dictation status")).toContainText("interim words");
  await expect(response).not.toContainText("interim words");
  expect(await response.boundingBox()).toEqual(draftBounds);
  await expect(agent.getByRole("button", { name: "Send", exact: true })).toBeDisabled();
  await agent.getByRole("button", { name: "Stop dictation" }).click();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent("fixture-dictation-result", { detail: { text: "Dictated request.", final: true } })));
  await expect(response).toContainText("Dictated request.");
  await expect(agent.getByRole("button", { name: "Send", exact: true })).toBeDisabled();
  await page.evaluate(() => window.dispatchEvent(new Event("fixture-dictation-end")));
  await expect(agent.getByRole("button", { name: "Send", exact: true })).toBeEnabled();
  expect(agentRequests).toBe(0);
  await response.click(); await response.press("ControlOrMeta+End"); await response.pressSequentially(" Propose in Python");
  const sent = page.waitForRequest(request => request.method() === "POST" && /\/api\/docs\/[^/]+\/agent$/.test(new URL(request.url()).pathname));
  await agent.getByRole("button", { name: "Send", exact: true }).click();
  const request = await sent;
  expect(request.postDataJSON().prompt).toContain("Dictated request.");
  const attached = request.postDataJSON().context.attachments;
  expect(attached.map((file: { name: string }) => file.name)).toEqual(["pasted.txt", "pasted.png", "camera-photo.png"]);
  expect(Buffer.from(attached[0].data, "base64").toString()).toBe("clipboard bytes");
  expect(attached[1].media_type).toBe("image/png");
  const turnsPath = `${new URL(request.url()).pathname}/turns`;
  await expect(agent).toContainText("read_doc");
  await expect.poll(async () => {
    const history = await (await page.request.get(turnsPath)).json();
    if (!history.turns.length) return false;
    const session = await (await page.request.get(`/api/sessions/view?path=${encodeURIComponent(history.turns[0].session)}`)).json();
    return session.source.includes('"images":[{') && session.source.includes("clipboard bytes");
  }).toBe(true);
  await agent.getByRole("button", { name: "Stop the agent" }).click();
  await expect.poll(async () => (await (await page.request.get(turnsPath)).json()).turns.at(-1)?.status).toBe("stopped");
  expect(errors).toEqual([]);
});
});
</storycode:copy>

### Assemble `apps/web/e2e/python-agent.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/python-agent.spec.ts"><storycode:paste select="#f-c40e3ff7e23b-expression_statement"/></storycode:file>

</storycode:doc>
