<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/fsharp.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `050965a21b2ee6e7788e9e1520e750a536e63321bd81aebb42ec967ad524dee9`.

<book:entry path="apps/web/src/plugins/languages/fsharp.ts">
import { fSharp } from "@codemirror/legacy-modes/mode/mllike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "fsharp", grammar: () => StreamLanguage.define(fSharp), aliases: ["f#"], comment: ["// ", ""] };
</book:entry>

</book:doc>
