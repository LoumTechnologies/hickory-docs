<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/yaml.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `4eaec42c80e69fb993a4e955ab5802ef40acbd4150d8413a98d8106bdfd07a46`.

<book:entry path="apps/web/src/plugins/languages/yaml.ts">
import { yaml } from "@codemirror/legacy-modes/mode/yaml";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "yaml", grammar: () => StreamLanguage.define(yaml), aliases: ["yml"], comment: ["# ", ""] };
</book:entry>

</book:doc>
