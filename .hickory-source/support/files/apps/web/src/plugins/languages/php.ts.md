<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/php.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `d70d651d37d6e2ffa07cfbe9c36806edf6812bdb17730b1c6eb98ba4b2f20ac9`.

<book:entry path="apps/web/src/plugins/languages/php.ts">
import { php } from "@codemirror/lang-php";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "php", grammar: () => php(), aliases: [], comment: ["// ", ""] };
</book:entry>

</book:doc>
