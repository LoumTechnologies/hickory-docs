<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/groovy.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `851b497f2a4c686130c408b176fda693223561bd9bdd84050a112457e35f8df0`.

<book:entry path="apps/web/src/plugins/languages/groovy.ts">
import { groovy } from "@codemirror/legacy-modes/mode/groovy";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "groovy", grammar: () => StreamLanguage.define(groovy), aliases: [], comment: ["// ", ""] };
</book:entry>

</book:doc>
