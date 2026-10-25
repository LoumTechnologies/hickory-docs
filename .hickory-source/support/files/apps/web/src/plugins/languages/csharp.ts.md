<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/csharp.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `02a3976654ad3ad54bc43c5a48463300bbffcb9641f72bcb76579c41dd50786b`.

<book:entry path="apps/web/src/plugins/languages/csharp.ts">
import { csharp } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "csharp", grammar: () => StreamLanguage.define(csharp), aliases: ["c#", "cs", "dotnet"], tests: { definition: /^\s*(?:public\s+|private\s+|internal\s+|static\s+|async\s+|virtual\s+|override\s+)*[A-Za-z_<>\[\],. ]+?\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/, group: 1, attribute: /^\s*\[\s*(?:Fact|Theory|Test|TestMethod|TestCase)\b/ }, comment: ["// ", ""] };
</book:entry>

</book:doc>
