<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/typescript.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `9a1e16382902b93a5668acf41fcf81939c11b73bd926af608ec20e1225ceb56a`.

<book:entry path="apps/web/src/plugins/languages/typescript.ts">
import { javascript } from "@codemirror/lang-javascript";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "typescript", grammar: () => javascript({ typescript: true }), aliases: ["ts"], tests: { definition: /^\s*(?:it|test)(?:\.(?:only|skip|each))?\s*\(\s*(["'`])((?:\\.|(?!\1).)*)\1/, group: 2 }, comment: ["// ", ""] };
</book:entry>

</book:doc>
