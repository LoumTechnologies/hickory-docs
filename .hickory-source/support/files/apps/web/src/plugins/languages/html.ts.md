<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/html.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `60a0b428e6f70509db23166ecb878bb0da732a951145df3702b1ab9f278861ce`.

<book:entry path="apps/web/src/plugins/languages/html.ts">
import { html } from "@codemirror/lang-html";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "html", grammar: () => html(), aliases: ["htm"], comment: ["<!-- ", " -->"] };
</book:entry>

</book:doc>
