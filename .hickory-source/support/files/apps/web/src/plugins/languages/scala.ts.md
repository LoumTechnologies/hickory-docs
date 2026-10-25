<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/scala.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `9215e7e40c156d4fec4a7c2485d008b55890ef1e80f8d09cbd2260008ffc11b8`.

<book:entry path="apps/web/src/plugins/languages/scala.ts">
import { scala } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "scala", grammar: () => StreamLanguage.define(scala), aliases: [], comment: ["// ", ""] };
</book:entry>

</book:doc>
