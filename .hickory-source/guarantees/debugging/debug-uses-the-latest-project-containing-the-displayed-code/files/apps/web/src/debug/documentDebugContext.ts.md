<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/documentDebugContext.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Debug uses the latest project containing the displayed code](../../../../../contract.md)

Captured SHA-256: `992a823cf906ae3c4eec7587fa5fce49a874dfafc89f48934816406828bd7a96`.

<book:entry path="apps/web/src/debug/documentDebugContext.ts">
import { parseHickDoc } from "../editor/hickDoc";

/** The clicked construction, never a global project-version preference. */
export function documentDebugContext(path: string, source: string, at?: number): string {
  const doc = parseHickDoc(source);
  if (at === undefined || !doc.blocks.some(b => b.name === "project")) return "new";
  const block = doc.blocks.find(b => b.name === "file" && b.from === at);
  if (!block) throw new Error("This file block changed. Press its Debug icon again.");
  // Parser spans exclude the optional BOM; editor positions include it.
  const byte = new TextEncoder().encode(source.slice(source.startsWith("\uFEFF") ? 1 : 0, at)).length;
  const id = block.attrs.id ?? `${path.split("/").pop()}:${byte}`;
  return `latest-file:${JSON.stringify([id, source])}`;
}
</book:entry>

</book:doc>
