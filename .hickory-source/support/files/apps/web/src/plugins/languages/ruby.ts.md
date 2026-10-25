<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/ruby.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `7920bad8c61c9a8b4cedb641702ee955016d027a16f1812045812b70d055c455`.

<book:entry path="apps/web/src/plugins/languages/ruby.ts">
import { ruby } from "@codemirror/legacy-modes/mode/ruby";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "ruby", grammar: () => StreamLanguage.define(ruby), aliases: ["rb"], comment: ["# ", ""] };
</book:entry>

</book:doc>
