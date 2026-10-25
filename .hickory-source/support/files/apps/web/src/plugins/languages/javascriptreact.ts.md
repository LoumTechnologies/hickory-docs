<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/javascriptreact.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3bf1840e3fbf43bc564164a05aaec71112d3c1bf20d886c7a3427673361fabff`.

<book:entry path="apps/web/src/plugins/languages/javascriptreact.ts">
import { javascript } from "@codemirror/lang-javascript";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "javascriptreact", grammar: () => javascript({ jsx: true }), aliases: ["jsx", "javascript-react"], tests: { definition: /^\s*(?:it|test)(?:\.(?:only|skip|each))?\s*\(\s*(["'`])((?:\\.|(?!\1).)*)\1/, group: 2 }, comment: ["// ", ""] };
</book:entry>

</book:doc>
