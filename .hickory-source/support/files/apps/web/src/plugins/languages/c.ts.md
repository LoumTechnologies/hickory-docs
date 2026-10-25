<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/c.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `04a4a28f7b48e929e70b32795d6f54f927261accb1d262240d9466217d537715`.

<book:entry path="apps/web/src/plugins/languages/c.ts">
import { c } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "c", grammar: () => StreamLanguage.define(c), aliases: ["objective-c"], comment: ["// ", ""] };
</book:entry>

</book:doc>
