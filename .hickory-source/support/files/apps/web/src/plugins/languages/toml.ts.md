<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/toml.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `e1629902252c04481d36e929bbb49ad1b90815c5f8e07d17289d240c5799bd36`.

<book:entry path="apps/web/src/plugins/languages/toml.ts">
import { toml } from "@codemirror/legacy-modes/mode/toml";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "toml", grammar: () => StreamLanguage.define(toml), aliases: [], comment: ["# ", ""] };
</book:entry>

</book:doc>
