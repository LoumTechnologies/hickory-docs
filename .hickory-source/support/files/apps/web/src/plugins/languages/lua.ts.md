<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/lua.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `9bd4e6a4e5a1d818c81637634c614a4493432c7a424592090f7d8fc6e6fc4f19`.

<book:entry path="apps/web/src/plugins/languages/lua.ts">
import { lua } from "@codemirror/legacy-modes/mode/lua";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "lua", grammar: () => StreamLanguage.define(lua), aliases: [], comment: ["// ", ""] };
</book:entry>

</book:doc>
