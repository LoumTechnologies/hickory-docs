<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/r.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `eca7fe41fa1e248a0a5c60cf656e827998621522b52d68e9823ae2e750901a5c`.

<book:entry path="apps/web/src/plugins/languages/r.ts">
import { r } from "@codemirror/legacy-modes/mode/r";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "r", grammar: () => StreamLanguage.define(r), aliases: [], comment: ["# ", ""] };
</book:entry>

</book:doc>
