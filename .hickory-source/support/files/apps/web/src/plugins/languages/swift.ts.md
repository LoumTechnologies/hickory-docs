<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/swift.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ba9eeec514880c69b195d10a2d65f6683c33e56c738bac5c6b22ab6e9c51502b`.

<book:entry path="apps/web/src/plugins/languages/swift.ts">
import { swift } from "@codemirror/legacy-modes/mode/swift";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "swift", grammar: () => StreamLanguage.define(swift), aliases: [], comment: ["// ", ""] };
</book:entry>

</book:doc>
