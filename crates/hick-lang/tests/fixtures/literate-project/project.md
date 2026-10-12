# A complete two-file application

The Markdown closure contains its discovery and inventory rules. New refers to
both completed files, even while inspecting the intentionally broken middle step.

<hick:project sources="library.md" exclude=".git/ .hg/ .hick-cache/ node_modules/ bin/ obj/" />

<hick:file id="app" path="app.ts">
import { greet } from './library';
console.log(greet());
</hick:file>

First change the caller. At this local state the function is still absent.

<hick:patch id="caller" after="app library">
--- a/app.ts
+++ b/app.ts
@@ -1,2 +1,2 @@
-import { greet } from './library';
-console.log(greet());
+import { welcome } from './library';
+console.log(welcome('Hickory'));
</hick:patch>

Complete the whole project by changing the library as well.

<hick:patch id="complete" after="caller">
--- a/library.ts
+++ b/library.ts
@@ -1 +1 @@
-export function greet() { return 'Hello'; }
+export function welcome(name: string) { return 'Hello ' + name; }
</hick:patch>
