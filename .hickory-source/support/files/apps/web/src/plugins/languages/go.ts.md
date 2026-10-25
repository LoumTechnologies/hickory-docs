<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/go.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `1605d464cd2dae80da0ab77a35e4fba865de07bbc3af107a69f45c0b17078437`.

<book:entry path="apps/web/src/plugins/languages/go.ts">
import { go } from "@codemirror/legacy-modes/mode/go";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "go", grammar: () => StreamLanguage.define(go), aliases: ["golang"], tests: { definition: /^\s*func\s+(Test[A-Za-z0-9_]*)\s*\(\s*\w+\s+\*testing\.T\s*\)/, group: 1 }, comment: ["// ", ""] };
</book:entry>

</book:doc>
