<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/markdown.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `9c606de6e6b2c39c94656f902ec6c410bd930b4c0ceb2f83cabf0b062827f7cc`.

<book:entry path="apps/web/src/plugins/languages/markdown.ts">
import { markdown } from "@codemirror/lang-markdown";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "markdown", grammar: () => markdown(), aliases: ["md"], comment: ["<!-- ", " -->"] };
</book:entry>

</book:doc>
