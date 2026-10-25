<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/cpp.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `b66eb29b3a368eda58f9dec320df8d1f2f35c5e760ba54e7a09b029895f4e873`.

<book:entry path="apps/web/src/plugins/languages/cpp.ts">
import { cpp } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "cpp", grammar: () => StreamLanguage.define(cpp), aliases: ["c++"], comment: ["// ", ""] };
</book:entry>

</book:doc>
