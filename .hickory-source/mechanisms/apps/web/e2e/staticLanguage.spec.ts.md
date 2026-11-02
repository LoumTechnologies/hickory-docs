<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Drive the static reader through captured language answers

A real browser must see compiled colours, a documentation popup and definition navigation while the document remains read-only. Serve a hash-bound fixture through the ordinary source-publication routes and reject any dependence on backend API traffic. This checks the production reader and shared CodeMirror handlers, not a separate demo renderer. It does not claim that a real LSP supplied this browser fixture; the provider boundary has its own focused witness.

<static:copy id="static-apps-web-e2e-staticLanguage-spec-ts-0">
import { test, expect } from "@playwright/test";
import { createHash } from "node:crypto";
const hash = (text: string) => createHash("sha256").update(text).digest("hex");

test("static source replays colours, hover and definition without a backend", async ({ page }) => {
  const source = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n# A published program\n<h:file path="a.ts">\nexport const count = 1;\ncount;\n</h:file>\n</h:doc>\n';
  const offset = source.lastIndexOf("count;"), declaration = source.indexOf("count =");
  const position = (at: number) => ({ line: source.slice(0, at).split("\n").length - 1, character: at - source.lastIndexOf("\n", at - 1) - 1 });
  const range = { start: position(declaration), end: position(declaration + 5) };
  const intelligence = { version: 1, source_hash: hash(source), state: "captured", spans: [], packed_spans: {
    classes: ["tok-keyword", "tok-variableName"], data: [source.indexOf("const"), 5, 0, offset, 5, 1],
  }, language: { capabilities: {}, queries: {}, diagnostics: [], occurrences: [
    { from: offset, to: offset + 5, position: position(offset), hover: { state: "captured", result: { contents: "```typescript\nconst count: 1\n```\nCaptured documentation" } }, definition: { state: "captured", result: [{ uri: "hick:///project.md", range }] }, references: { state: "captured", result: [{ uri: "hick:///project.md", range }] } },
  ] } };
  const revision = "a".repeat(40), url = `documents/${"b".repeat(64)}.json`;
  const document = { path: "project.md", title: "A program", sha256: hash(source), url, intelligence_sha256: hash(JSON.stringify(intelligence)) };
  const requests: string[] = [];
  page.on("request", request => { if (/^\/api(?:\/|$)|^\/ws(?:\/|$)/.test(new URL(request.url()).pathname)) requests.push(request.url()); });
  await page.route("**/source/index.json", route => route.fulfill({ json: { version: 1, revision, entry: "project.md", documents: [document] } }));
  await page.route(`**/source/${url}`, route => route.fulfill({ json: { path: "project.md", revision, source, intelligence } }));
  await page.goto("/source.html");
  const content = page.locator(".cm-content");
  await expect(content).toHaveAttribute("contenteditable", "false");
  await expect(content.locator(".tok-keyword").filter({ hasText: "const" }).first()).toBeVisible();
  const token = content.locator(".tok-variableName").filter({ hasText: "count" }).last();
  await token.hover();
  await expect(page.locator(".cm-lsp-hover")).toContainText("Captured documentation");
  await token.click({ modifiers: ["Control", "Shift"] });
  await expect(page.getByRole("complementary", { name: "Captured references" })).toContainText("project.md");
  await token.click({ modifiers: ["Control"] });
  await expect(page).toHaveURL(new RegExp(`#L${range.start.line + 1}:${range.start.character}$`));
  expect(requests).toEqual([]);
});
</static:copy>

<static:file path="apps/web/e2e/staticLanguage.spec.ts"><static:paste select="#static-apps-web-e2e-staticLanguage-spec-ts-0"/></static:file>
</static:doc>
