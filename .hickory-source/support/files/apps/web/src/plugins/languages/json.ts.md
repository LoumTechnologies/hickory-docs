<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/json.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `7ad566a8493a39777fb8c6c53e4b80ed893b3900de501088fa8c942241feac3d`.

<book:entry path="apps/web/src/plugins/languages/json.ts">
import { json } from "@codemirror/lang-json";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "json", grammar: () => json(), aliases: [], comment: ["// ", ""] };
</book:entry>

</book:doc>
