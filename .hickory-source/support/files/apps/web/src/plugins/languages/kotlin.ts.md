<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/kotlin.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `a1abb1f8840bd483b4b7c50ec926276e4f14e8a294982fdd26f8aac101ffa47b`.

<book:entry path="apps/web/src/plugins/languages/kotlin.ts">
import { kotlin } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "kotlin", grammar: () => StreamLanguage.define(kotlin), aliases: ["kt"], comment: ["// ", ""] };
</book:entry>

</book:doc>
