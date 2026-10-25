<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/typescriptreact.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `a83907b062e6b20b1cc3b0ec8a91dc624560e15c8eea151ad1dbded333efec08`.

<book:entry path="apps/web/src/plugins/languages/typescriptreact.ts">
import { javascript } from "@codemirror/lang-javascript";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "typescriptreact", grammar: () => javascript({ typescript: true, jsx: true }), aliases: ["tsx", "typescript-react"], tests: { definition: /^\s*(?:it|test)(?:\.(?:only|skip|each))?\s*\(\s*(["'`])((?:\\.|(?!\1).)*)\1/, group: 2 }, comment: ["// ", ""] };
</book:entry>

</book:doc>
