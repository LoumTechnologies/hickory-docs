<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/dart.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `b962125bfed84ea937ab0bba022c03f8d5bdc007494e3353d7f1b44765ca3e30`.

<book:entry path="apps/web/src/plugins/languages/dart.ts">
import { dart } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "dart", grammar: () => StreamLanguage.define(dart), aliases: [], comment: ["// ", ""] };
</book:entry>

</book:doc>
