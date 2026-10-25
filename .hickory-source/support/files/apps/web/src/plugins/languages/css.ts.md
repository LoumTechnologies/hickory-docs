<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/css.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `7d76e0e3a1d94d0e657727abe41cd7821d41b8330ceeface029d70358de12003`.

<book:entry path="apps/web/src/plugins/languages/css.ts">
import { css } from "@codemirror/lang-css";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "css", grammar: () => css(), aliases: [], comment: ["// ", ""] };
</book:entry>

</book:doc>
