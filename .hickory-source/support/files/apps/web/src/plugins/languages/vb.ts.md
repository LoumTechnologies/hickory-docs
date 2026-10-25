<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/vb.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `b5563c1adbef9db7165e11bc3471b8b8fd15d90e26ab124736bed09677a21ce3`.

<book:entry path="apps/web/src/plugins/languages/vb.ts">
import { vb } from "@codemirror/legacy-modes/mode/vb";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "vb", grammar: () => StreamLanguage.define(vb), aliases: ["visual-basic", "vbnet"], comment: ["// ", ""] };
</book:entry>

</book:doc>
