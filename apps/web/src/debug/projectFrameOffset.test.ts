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
