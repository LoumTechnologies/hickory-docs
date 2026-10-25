<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/java.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `411501df91563f99e5a6b8363f7c14e69d77d640c73617607bff808778f80678`.

<book:entry path="apps/web/src/plugins/languages/java.ts">
import { java } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "java", grammar: () => StreamLanguage.define(java), aliases: [], comment: ["// ", ""] };
</book:entry>

</book:doc>
