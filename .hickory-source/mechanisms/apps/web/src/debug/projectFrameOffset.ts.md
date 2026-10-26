<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/projectFrameOffset.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0f2996ef3670"></a>
## `apps/web/src/debug/projectFrameOffset.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-0f2996ef3670-projectFrameOffset"></a>
### `projectFrameOffset`

<storycode:copy id="f-0f2996ef3670-projectFrameOffset">
import type { ProjectEntry } from "../editor/hickLang";
import type { Frame } from "./client";
import { byteToChar } from "../lib/offsets";

/** Resolve only a unique, byte-exact surviving origin from this capture. */
export function projectFrameOffset(prefix: string, entries: readonly ProjectEntry[], document: string, source: string, frame: Frame): number | null {
  if (!frame.source?.startsWith(prefix) || !Number.isInteger(frame.source_line) || frame.source_line! < 0) return null;
  const file = entries.find(e => `${prefix}${e.path}` === frame.source);
  if (!file) return null;
  const text = new TextDecoder("utf-8", {fatal:true}).decode(new Uint8Array(file.bytes));
  const lines = text.split("\n"), line = frame.source_line!;
  if (line >= lines.length) return null;
  const at = new TextEncoder().encode(lines.slice(0, line).map(s => s + "\n").join("")).length;
  const origins = file.origins?.filter(o => o.output[0] <= at && at < o.output[1]);
  if (origins?.length !== 1) return null;
  const origin = origins[0];
  if (origin.document !== document || origin.output[1] - origin.output[0] !== origin.source[1] - origin.source[0]) return null;
  const bytes = new TextEncoder().encode(source);
  if (file.bytes.slice(...origin.output).some((b, i) => bytes[origin.source[0] + i] !== b)) return null;
  return byteToChar(source, origin.source[0] + at - origin.output[0]);
}
</storycode:copy>

### Assemble `apps/web/src/debug/projectFrameOffset.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/projectFrameOffset.ts"><storycode:paste select="#f-0f2996ef3670-projectFrameOffset"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-d5782e065c25"></a>
## `apps/web/src/debug/projectFrameOffset.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d5782e065c25-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d5782e065c25-expression_statement">
import { expect, it } from "vitest";
import { materializeProject } from "../editor/hickLang";
import { projectFrameOffset } from "./projectFrameOffset";

it("steps from a caller patch into a later callee patch using final project provenance", () => {
  const source = `<hick:project/>
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
  const project = materializeProject("project.md", {"project.md": source});
  const frame = {id:1, name:"welcome", source:"hick-project:///test/library.py", source_line:1, line:null, in_document:false};
  expect(projectFrameOffset("hick-project:///test/", project.new, "project.md", source, frame)).toBe(source.indexOf('    return "Hello " + name'));
  expect(projectFrameOffset("hick-project:///other/", project.new, "project.md", source, frame)).toBeNull();
  expect(projectFrameOffset("hick-project:///test/", project.new, "project.md", source.replace('Hello " + name', 'stale " + name'), frame)).toBeNull();
  expect(projectFrameOffset("hick-project:///test/", project.new, "project.md", source, {...frame, source_line:99})).toBeNull();
});
</storycode:copy>

### Assemble `apps/web/src/debug/projectFrameOffset.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/projectFrameOffset.test.ts"><storycode:paste select="#f-d5782e065c25-expression_statement"/></storycode:file>


</storycode:doc>
