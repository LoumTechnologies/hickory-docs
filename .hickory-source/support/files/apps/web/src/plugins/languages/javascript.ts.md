<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/javascript.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `fc9b90c151d24eed84c58489d207df84deffcb7290bc57207ac67e85a4326895`.

<book:entry path="apps/web/src/plugins/languages/javascript.ts">
import { javascript } from "@codemirror/lang-javascript";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "javascript", grammar: () => javascript(), aliases: ["js"], tests: { definition: /^\s*(?:it|test)(?:\.(?:only|skip|each))?\s*\(\s*(["'`])((?:\\.|(?!\1).)*)\1/, group: 2 }, comment: ["// ", ""] };
</book:entry>

</book:doc>
