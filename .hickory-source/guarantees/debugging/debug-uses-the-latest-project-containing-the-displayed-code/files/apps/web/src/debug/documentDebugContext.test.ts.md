<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/documentDebugContext.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Debug uses the latest project containing the displayed code](../../../../../contract.md)

Captured SHA-256: `b434b16bda34903c62dfe0ccd9241cea1b77125204bbefa840b61d9cf53d9b01`.

<book:entry path="apps/web/src/debug/documentDebugContext.test.ts">
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
</book:entry>

</book:doc>
