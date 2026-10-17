import { expect, it } from "vitest";
import { documentDebugContext } from "./documentDebugContext";
import { parseHickDoc } from "../editor/hickDoc";
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
it("identifies the clicked construction and fences the exact reading, including Unicode and BOM", () => {
  const source = '\uFEFF<hick:project/>\n🦀\n<hick:file path="app.py">print(1)\n</hick:file>';
  const at = source.indexOf('<hick:file');
  const context = documentDebugContext("folder/p.md", source, at);
  const [id, reading] = JSON.parse(context.slice("latest-file:".length));
  expect(id).toBe(`p.md:${new TextEncoder().encode(source.slice(1, at)).length}`);
  expect(reading).toBe(source);
});
it("keeps repeated filenames distinct through construction identities", () => {
  const source = '<hick:project/><hick:file id="first" path="app.py">a</hick:file><hick:file id="second" path="app.py">b</hick:file>';
  const blocks = parseHickDoc(source).blocks.filter(b => b.name === "file");
  expect(JSON.parse(documentDebugContext("p.md", source, blocks[1].from).slice(12))[0]).toBe("second");
});
it("keeps ordinary non-project script debugging", () => {
  expect(documentDebugContext("p.md", '<hick:file path="app.py">print(1)</hick:file>', 0)).toBe("new");
});
